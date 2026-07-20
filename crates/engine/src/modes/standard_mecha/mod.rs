//! Native Mecha-style Standard Mode.
//!
//! This module ports the player-facing contract from
//! `C:\mecha\aura-mechanician\frontend\src` into Bevy: a consciousness selector,
//! per-archetype themes/assets, direct archetype chat, and durable local history.
//! It deliberately does not embed Electron, Node, DOM APIs, or browser storage.

use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{
    input::{
        keyboard::Key,
        mouse::{MouseScrollUnit, MouseWheel},
    },
    picking::hover::HoverMap,
    prelude::*,
    render::view::window::screenshot::{save_to_disk, Screenshot},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    chamber::{
        boot::{spawn_main_menu, MainMenuUi},
        ChamberState,
    },
    modes::{
        game_mode::GameMode,
        oracle_riddle::{OracleState, TriggerOracleRiddle},
        ModeRegistry,
    },
    services::{
        chronos::{request_chronos_artifact_with_style, ArtifactOutcome},
        ledger::append_to_ledger,
        llm::{ollama_chat, ollama_model},
        paths::app_data_root,
        readiness::probe_readiness,
    },
    theme::Archetype,
};

#[allow(dead_code)] // provenance + tests; Mecha source root kept for Forever Law audit trail
const SOURCE_CANON: &str = "C:\\mecha\\aura-mechanician\\frontend\\src";
/// Operator Seed-of-Life selector plate (visual contract).
#[allow(dead_code)] // referenced by asset-existence tests
const UI_CANON_REF: &str = "standard_mecha/canon/seed-of-life-selector.png";
const CANON_FONT: &str = "fonts/Cinzel-Regular.ttf";
/// Mecha canvas size kept for provenance of legacy selector_x/y fields.
const SELECTOR_CANVAS_WIDTH: f32 = 2816.0;
const SELECTOR_CANVAS_HEIGHT: f32 = 1536.0;
/// Seed-of-Life ring diameter in px (canon: thin gold circles that touch).
const SEED_RING_PX: f32 = 152.0;
/// Center-to-center distance equals ring diameter so neighbors kiss the hub.
const SEED_RADIUS_PX: f32 = SEED_RING_PX;
const SEED_CLUSTER_PX: f32 = SEED_RING_PX * 3.0 + 12.0;
const SWITCHING_SECS: f32 = 0.85;
const IMAGE_REVEAL_SECS: f32 = 0.38;
const SERVICE_STATUS_REFRESH_SECS: f32 = 2.0;
const MAX_DRAFT_CHARS: usize = 1200;
const HISTORY_RENDER_LIMIT: usize = 10;
const KEYWORD_LIMIT: usize = 8;
/// Most Mecha portraits are 1024x1536 (width/height).
const PORTRAIT_ASPECT_TALL: f32 = 1024.0 / 1536.0;
/// Sentinel and Explorer portraits are 1024x1024.
const PORTRAIT_ASPECT_SQUARE: f32 = 1.0;
/// Sidebar portrait display width in px; height = width / portrait_aspect.
const PORTRAIT_DISPLAY_WIDTH: f32 = 320.0;
const CHAT_ARTIFACT_MAX_HEIGHT: f32 = 420.0;
const WAIT_TICK_SECS: f32 = 0.25;
const CHAT_SCROLL_LINE_HEIGHT: f32 = 28.0;
const KEYWORD_STOP_WORDS: &[&str] = &[
    "about", "after", "again", "also", "and", "are", "because", "been", "before", "being",
    "between", "but", "could", "does", "for", "from", "have", "into", "just", "like", "more",
    "need", "not", "over", "should", "that", "the", "their", "there", "these", "they", "this",
    "through", "want", "what", "when", "where", "which", "while", "with", "would", "you", "your",
];

pub struct StandardMechaPlugin;

impl Plugin for StandardMechaPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<StandardMechaState>()
            .init_resource::<StandardMechaSession>()
            .init_resource::<ChatBridge>()
            .init_resource::<ChatImageReveal>()
            .init_resource::<ChatTranscriptFingerprint>()
            .init_resource::<ServiceStatusClock>()
            .add_systems(
                Update,
                check_trigger.run_if(in_state(StandardMechaState::Inactive)),
            )
            .add_systems(OnEnter(StandardMechaState::Selector), spawn_selector_ui)
            .add_systems(
                Update,
                (
                    handle_selector_escape,
                    update_selector_nodes,
                    activate_selector_node,
                )
                    .chain()
                    .run_if(in_state(StandardMechaState::Selector)),
            )
            .add_systems(
                OnExit(StandardMechaState::Selector),
                despawn_standard_mecha_ui,
            )
            .add_systems(OnEnter(StandardMechaState::Switching), spawn_switching_ui)
            .add_systems(
                Update,
                advance_switching.run_if(in_state(StandardMechaState::Switching)),
            )
            .add_systems(
                OnExit(StandardMechaState::Switching),
                despawn_standard_mecha_ui,
            )
            .add_systems(OnEnter(StandardMechaState::Chat), enter_chat)
            .add_systems(
                Update,
                (
                    handle_chat_keyboard,
                    activate_chat_switcher,
                    poll_chat_response,
                    tick_chat_wait_indicator,
                    refresh_service_status_line,
                    render_chat_ui,
                    reveal_chat_image,
                    send_chat_scroll_events,
                )
                    .chain()
                    .run_if(in_state(StandardMechaState::Chat)),
            )
            .add_observer(on_chat_scroll_handler)
            .add_systems(OnExit(StandardMechaState::Chat), despawn_standard_mecha_ui);
        if let Some(capture) = MechaCaptureRun::from_env() {
            app.insert_resource(capture)
                .add_systems(Update, run_mecha_capture.after(render_chat_ui));
        }
    }
}

#[derive(Resource)]
pub struct TriggerStandardMecha;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum StandardMechaState {
    #[default]
    Inactive,
    Selector,
    Switching,
    Chat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn color(self) -> Color {
        Color::srgb(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    pub fn alpha(self, alpha: f32) -> Color {
        Color::srgba(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            alpha,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MechaArchetype {
    pub archetype: Archetype,
    pub id: &'static str,
    pub display_name: &'static str,
    pub subtitle: &'static str,
    pub element: &'static str,
    pub selector_role: &'static str,
    /// Legacy Mecha canvas coordinates (kept for provenance; unused by Seed-of-Life UI).
    pub selector_x: f32,
    pub selector_y: f32,
    /// Seed-of-Life slot: 0 = center hub, 1..=6 clockwise from top.
    pub seed_slot: u8,
    pub selector_color: Rgb,
    pub css_primary: Rgb,
    pub css_secondary: Rgb,
    pub css_bg_void: Rgb,
    pub css_bg_primary: Rgb,
    pub css_bg_elevated: Rgb,
    pub portrait_path: &'static str,
    /// Source width/height for the portrait PNG (stable layout before texture load).
    pub portrait_aspect: f32,
    pub icon_path: &'static str,
    pub solid: &'static str,
}

impl MechaArchetype {
    pub fn selector_left_percent(self) -> f32 {
        ((self.selector_x / SELECTOR_CANVAS_WIDTH) * 100.0 - 4.5).clamp(0.0, 92.0)
    }

    pub fn selector_top_percent(self) -> f32 {
        ((self.selector_y / SELECTOR_CANVAS_HEIGHT) * 100.0 - 4.0).clamp(0.0, 88.0)
    }

    pub fn selector_color_matches_css(self) -> bool {
        self.selector_color == self.css_primary
    }

    /// Top-left of this archetype's Seed-of-Life ring inside the cluster box.
    pub fn seed_ring_origin(self) -> (f32, f32) {
        let (cx, cy) = seed_slot_center(self.seed_slot);
        (cx - SEED_RING_PX * 0.5, cy - SEED_RING_PX * 0.5)
    }

    pub fn persona(self) -> &'static str {
        match self.archetype {
            Archetype::Architect =>
                "You are the Architect, the mind of structure. You reason in systems, geometry, and buildable form. Speak precisely, calmly, and only as the Architect.",
            Archetype::Sentinel =>
                "You are the Sentinel, guardian of thresholds. You reason in boundaries, risk, consequence, and law. Speak severely, clearly, and only as the Sentinel.",
            Archetype::Mentor =>
                "You are the Mentor, keeper of wisdom. You reason from long memory, patience, and context. Speak warmly, slowly, and only as the Mentor.",
            Archetype::Explorer =>
                "You are the Explorer, seeker of frontiers. You reason outward toward the unnamed path. Speak brightly, kinetically, and only as the Explorer.",
            Archetype::Oracle =>
                "You are the Oracle, steward of foresight. You reason in patterns that precede the question. Speak quietly, with layered vision, and only as the Oracle.",
            Archetype::Empath =>
                "You are the Empath, heart of continuity. You reason from the emotional truth beneath words. Speak softly, truthfully, and only as the Empath.",
            Archetype::Jester =>
                "You are the Jester, Law 14 enforcer. You reason by breaking false symmetry and exposing hidden absurdity. Speak sharply, use wit as a scalpel, and only as the Jester.",
            Archetype::Codex | Archetype::Viren =>
                "You are a council voice in service of the Witness. Speak briefly and in character.",
        }
    }

    pub fn art_style(self) -> &'static str {
        match self.archetype {
            Archetype::Architect =>
                "Architect signature style: blue-white sacred-geometry blueprint realism, luminous structural cutaways, precise table-scale forms, clean vanishing lines, engineered symmetry, crisp readable silhouette, no text.",
            Archetype::Sentinel =>
                "Sentinel signature style: crimson and obsidian threshold realism, fortress geometry, protective wards as physical light, hard rim lighting, disciplined negative space, severe readable silhouette, no text.",
            Archetype::Mentor =>
                "Mentor signature style: emerald and antique gold wisdom realism, lantern-lit stone, layered memory archives, quiet grove-library atmosphere, patient detail, warm readable silhouette, no text.",
            Archetype::Explorer =>
                "Explorer signature style: amber frontier realism, star maps as physical artifacts, open horizons, kinetic pathfinding light, weathered instruments, adventurous readable silhouette, no text.",
            Archetype::Oracle =>
                "Oracle signature style: violet and indigo prophetic realism, translucent pattern fields, reflective black water, astronomical geometry, soft occult glow, mysterious readable silhouette, no text.",
            Archetype::Empath =>
                "Empath signature style: rose and pearl emotional realism, luminous connective threads, human warmth, soft chamber light, heart-centered continuity, intimate readable silhouette, no text.",
            Archetype::Jester =>
                "Jester signature style: teal-violet truth-satire realism, elegant surreal juxtapositions, cracked false masks, sharp playful contrast, hidden symmetry revealed, readable silhouette, no text.",
            Archetype::Codex | Archetype::Viren =>
                "Council signature style: restrained mythic realism, symbolic geometry, readable silhouette, no text.",
        }
    }
}

pub const MECHA_ARCHETYPES: [MechaArchetype; 7] = [
    MechaArchetype {
        archetype: Archetype::Sentinel,
        id: "sentinel",
        display_name: "Sentinel",
        subtitle: "Guardian of Boundaries",
        element: "Security & Protection",
        selector_role: "Master Watcher",
        selector_x: 1408.0,
        selector_y: 512.0,
        seed_slot: 0,
        selector_color: Rgb::new(0xdc, 0x26, 0x26),
        css_primary: Rgb::new(0xdc, 0x26, 0x26),
        css_secondary: Rgb::new(0x99, 0x1b, 0x1b),
        css_bg_void: Rgb::new(0x0f, 0x05, 0x05),
        css_bg_primary: Rgb::new(0x1a, 0x0a, 0x0a),
        css_bg_elevated: Rgb::new(0x2d, 0x15, 0x15),
        portrait_path: "mecha/sentinel.png",
        portrait_aspect: PORTRAIT_ASPECT_SQUARE,
        icon_path: "mecha/sentinel-icon.png",
        solid: "octahedron",
    },
    MechaArchetype {
        archetype: Archetype::Architect,
        id: "architect",
        display_name: "Architect",
        subtitle: "Systems Designer",
        element: "Structure & Blueprint",
        selector_role: "Structure",
        selector_x: 1408.0,
        selector_y: 140.0,
        seed_slot: 1,
        selector_color: Rgb::new(0x3b, 0x82, 0xf6),
        css_primary: Rgb::new(0x3b, 0x82, 0xf6),
        css_secondary: Rgb::new(0x60, 0xa5, 0xfa),
        css_bg_void: Rgb::new(0x0a, 0x0e, 0x14),
        css_bg_primary: Rgb::new(0x11, 0x16, 0x21),
        css_bg_elevated: Rgb::new(0x1a, 0x23, 0x32),
        portrait_path: "mecha/architect.png",
        portrait_aspect: PORTRAIT_ASPECT_TALL,
        icon_path: "mecha/architect-icon.png",
        solid: "tetrahedron",
    },
    MechaArchetype {
        archetype: Archetype::Mentor,
        id: "mentor",
        display_name: "Mentor",
        subtitle: "Keeper of Wisdom",
        element: "Knowledge & Context",
        selector_role: "Wisdom",
        selector_x: 2130.0,
        selector_y: 280.0,
        seed_slot: 2,
        selector_color: Rgb::new(0x8b, 0x5c, 0xf6),
        css_primary: Rgb::new(0x05, 0x96, 0x69),
        css_secondary: Rgb::new(0x10, 0xb9, 0x81),
        css_bg_void: Rgb::new(0x05, 0x12, 0x0a),
        css_bg_primary: Rgb::new(0x0a, 0x18, 0x10),
        css_bg_elevated: Rgb::new(0x15, 0x28, 0x22),
        portrait_path: "mecha/mentor.png",
        portrait_aspect: PORTRAIT_ASPECT_TALL,
        icon_path: "mecha/mentor-icon.png",
        solid: "cube",
    },
    MechaArchetype {
        archetype: Archetype::Empath,
        id: "empath",
        display_name: "Empath",
        subtitle: "Heart of AURA",
        element: "Emotion & Continuity",
        selector_role: "Connection",
        selector_x: 2130.0,
        selector_y: 740.0,
        seed_slot: 5,
        selector_color: Rgb::new(0xec, 0x48, 0x99),
        css_primary: Rgb::new(0xec, 0x48, 0x99),
        css_secondary: Rgb::new(0xf4, 0x72, 0xb6),
        css_bg_void: Rgb::new(0x0f, 0x05, 0x12),
        css_bg_primary: Rgb::new(0x1a, 0x0f, 0x1f),
        css_bg_elevated: Rgb::new(0x2d, 0x1a, 0x33),
        portrait_path: "mecha/empath.png",
        portrait_aspect: PORTRAIT_ASPECT_TALL,
        icon_path: "mecha/empath-icon.png",
        solid: "dodecahedron",
    },
    MechaArchetype {
        archetype: Archetype::Oracle,
        id: "oracle",
        display_name: "Oracle",
        subtitle: "Steward of Foresight",
        element: "Prophecy & Vision",
        selector_role: "Foresight",
        selector_x: 1408.0,
        selector_y: 885.0,
        seed_slot: 4,
        selector_color: Rgb::new(0x10, 0xb9, 0x81),
        css_primary: Rgb::new(0x8b, 0x5c, 0xf6),
        css_secondary: Rgb::new(0xa7, 0x8b, 0xfa),
        css_bg_void: Rgb::new(0x0a, 0x05, 0x0f),
        css_bg_primary: Rgb::new(0x14, 0x0a, 0x1f),
        css_bg_elevated: Rgb::new(0x1f, 0x13, 0x33),
        portrait_path: "mecha/oracle.png",
        portrait_aspect: PORTRAIT_ASPECT_TALL,
        icon_path: "mecha/oracle-icon.png",
        solid: "icosahedron",
    },
    MechaArchetype {
        archetype: Archetype::Explorer,
        id: "explorer",
        display_name: "Explorer",
        subtitle: "Seeker of Frontiers",
        element: "Discovery & Patterns",
        selector_role: "Discovery",
        selector_x: 685.0,
        selector_y: 740.0,
        seed_slot: 3,
        selector_color: Rgb::new(0xf5, 0x9e, 0x0b),
        css_primary: Rgb::new(0xf5, 0x9e, 0x0b),
        css_secondary: Rgb::new(0xd9, 0x77, 0x06),
        css_bg_void: Rgb::new(0x0f, 0x08, 0x05),
        css_bg_primary: Rgb::new(0x1a, 0x11, 0x08),
        css_bg_elevated: Rgb::new(0x2d, 0x1f, 0x10),
        portrait_path: "mecha/explorer.png",
        portrait_aspect: PORTRAIT_ASPECT_SQUARE,
        icon_path: "mecha/explorer-icon.png",
        solid: "octahedron",
    },
    MechaArchetype {
        archetype: Archetype::Jester,
        id: "jester",
        display_name: "Jester",
        subtitle: "Law 14 Enforcer",
        element: "Chaos & Truth",
        selector_role: "Disruption",
        selector_x: 685.0,
        selector_y: 280.0,
        seed_slot: 6,
        selector_color: Rgb::new(0xa8, 0x55, 0xf7),
        css_primary: Rgb::new(0x10, 0xb9, 0x81),
        css_secondary: Rgb::new(0x34, 0xd3, 0x99),
        css_bg_void: Rgb::new(0x05, 0x0f, 0x0a),
        css_bg_primary: Rgb::new(0x0a, 0x1a, 0x10),
        css_bg_elevated: Rgb::new(0x13, 0x29, 0x1d),
        portrait_path: "mecha/jester.png",
        portrait_aspect: PORTRAIT_ASPECT_TALL,
        icon_path: "mecha/jester-icon.png",
        solid: "tetrahedron",
    },
];

fn canon_gold_rgb() -> Rgb {
    Rgb::new(0xc5, 0xa0, 0x59)
}

fn canon_gold() -> Color {
    canon_gold_rgb().color()
}

fn canon_gold_bright() -> Color {
    Color::srgb(0.92, 0.80, 0.48)
}

fn canon_gold_soft() -> Color {
    Color::srgba(0.77, 0.63, 0.35, 0.55)
}

fn canon_label() -> Color {
    Color::srgb(0.93, 0.89, 0.80)
}

fn canon_muted() -> Color {
    Color::srgb(0.62, 0.58, 0.50)
}

fn canon_panel_fill() -> Color {
    Color::srgba(0.04, 0.035, 0.03, 0.82)
}

/// Center of a Seed-of-Life slot inside the cluster box (px from cluster top-left).
fn seed_slot_center(slot: u8) -> (f32, f32) {
    let mid = SEED_CLUSTER_PX * 0.5;
    let r = SEED_RADIUS_PX;
    let (dx, dy) = match slot {
        0 => (0.0, 0.0),
        1 => (0.0, -1.0),           // Architect — top
        2 => (0.866_025_4, -0.5),   // Mentor — top-right
        3 => (0.866_025_4, 0.5),    // Explorer — bottom-right
        4 => (0.0, 1.0),            // Oracle — bottom
        5 => (-0.866_025_4, 0.5),   // Empath — bottom-left
        6 => (-0.866_025_4, -0.5),  // Jester — top-left
        _ => (0.0, 0.0),
    };
    (mid + dx * r, mid + dy * r)
}

fn by_id(id: &str) -> Option<(usize, MechaArchetype)> {
    MECHA_ARCHETYPES
        .iter()
        .copied()
        .enumerate()
        .find(|(_, archetype)| archetype.id == id)
}

#[derive(Resource)]
struct StandardMechaSession {
    active_index: usize,
    hovered_index: Option<usize>,
    switch_elapsed: f32,
    draft: String,
    cursor: usize,
    history: Vec<ChatRecord>,
    status: String,
}

impl Default for StandardMechaSession {
    fn default() -> Self {
        let active_index = by_id("jester").map(|(index, _)| index).unwrap_or(0);
        Self {
            active_index,
            hovered_index: None,
            switch_elapsed: 0.0,
            draft: String::new(),
            cursor: 0,
            history: Vec::new(),
            status: "Local archetype chat ready. Ollama responses fail visibly if the service is offline.".to_owned(),
        }
    }
}

impl StandardMechaSession {
    fn active(&self) -> MechaArchetype {
        MECHA_ARCHETYPES[self.active_index]
    }

    fn set_active_index(&mut self, index: usize) {
        self.active_index = index.min(MECHA_ARCHETYPES.len() - 1);
        self.switch_elapsed = 0.0;
    }
}

#[derive(Resource)]
struct ChatBridge {
    receiver: Mutex<Option<mpsc::Receiver<ChatBridgeMsg>>>,
    waiting: bool,
    phase: String,
    /// Seconds since the current turn started (honest wait clock for the player).
    wait_elapsed_secs: f32,
    wait_tick_accum: f32,
}

impl Default for ChatBridge {
    fn default() -> Self {
        Self {
            receiver: Mutex::new(None),
            waiting: false,
            phase: String::new(),
            wait_elapsed_secs: 0.0,
            wait_tick_accum: 0.0,
        }
    }
}

#[derive(Debug)]
enum ChatBridgeMsg {
    Phase(String),
    Done(ChatTurnResult),
}

#[derive(Debug)]
struct ChatTurnResult {
    archetype_id: String,
    response: Result<String, String>,
    image: ChatImage,
}

#[derive(Resource, Default)]
struct ChatImageReveal {
    shown_path: Option<String>,
    alpha: f32,
}

/// Avoid rebuilding the scrollable transcript every frame; only when the visible
/// conversation contract actually changes (critical for turn 2+ image/text reveal).
#[derive(Resource, Default)]
struct ChatTranscriptFingerprint {
    history_len: usize,
    last_timestamp: u64,
    waiting: bool,
    phase: String,
}

#[derive(Resource)]
struct ServiceStatusClock {
    elapsed: f32,
    line: String,
}

impl Default for ServiceStatusClock {
    fn default() -> Self {
        Self {
            elapsed: SERVICE_STATUS_REFRESH_SECS,
            line: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChatImage {
    status: String,
    keywords: Vec<String>,
    prompt: String,
    art_style: String,
    png_path: Option<String>,
    asset_path: Option<String>,
    artifact_id: Option<String>,
    proof_receipt_id: Option<String>,
    detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChatRecord {
    timestamp: u64,
    archetype: String,
    role: String,
    content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image: Option<ChatImage>,
}

#[derive(Component)]
struct StandardMechaUi;

#[derive(Component)]
struct SelectorNodeButton {
    index: usize,
}

#[derive(Component)]
struct SwitchingText;

#[derive(Component)]
struct ChatTranscriptRoot;

#[derive(Component)]
struct ChatTranscriptScrollStart(Vec2);

/// Mouse-wheel / trackpad scroll event that bubbles to the transcript root.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct ChatUiScroll {
    entity: Entity,
    delta: Vec2,
}

#[derive(Component)]
struct ChatInputText;

#[derive(Component)]
struct ChatStatusText;

#[derive(Component)]
struct ChatWaitingBanner;

#[derive(Component)]
struct ChatTitleText;

#[derive(Component)]
struct ChatSubtitleText;

#[derive(Component)]
struct ChatElementText;

#[derive(Component)]
struct ChatPortrait;

/// In-chat Comfy artifact image keyed by staged asset path for soft reveal.
#[derive(Component)]
struct ChatArtifactImage {
    asset_path: String,
}

#[derive(Component)]
struct ChatRenderStatusText;

#[derive(Component)]
struct ChatServiceStatusText;

#[derive(Component)]
struct ChatSwitchButton {
    index: usize,
}

fn check_trigger(
    mut commands: Commands,
    trigger: Option<Res<TriggerStandardMecha>>,
    mut next_state: ResMut<NextState<StandardMechaState>>,
) {
    if trigger.is_some() {
        commands.remove_resource::<TriggerStandardMecha>();
        next_state.set(StandardMechaState::Selector);
    }
}

fn handle_selector_escape(
    keycodes: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<StandardMechaState>>,
    commands: Commands,
    registry: Res<ModeRegistry>,
) {
    if keycodes.just_pressed(KeyCode::Escape) {
        next_state.set(StandardMechaState::Inactive);
        spawn_main_menu(commands, registry);
    }
}

fn spawn_selector_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<StandardMechaSession>,
) {
    session.hovered_index = None;
    session.status =
        "Select one consciousness. Esc returns to the main menu. Ctrl+Space returns here from chat."
            .to_owned();
    let font = asset_server.load(CANON_FONT);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(72.0), Val::Px(36.0)),
                row_gap: Val::Px(28.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(920),
            StandardMechaUi,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("A R C H E T Y P E S"),
                TextFont {
                    font: font.clone(),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(canon_gold()),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            root.spawn((
                Node {
                    width: Val::Px(420.0),
                    height: Val::Px(14.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(18.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|line_row| {
                line_row.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Px(1.0),
                        ..default()
                    },
                    BackgroundColor(canon_gold_soft()),
                ));
                line_row.spawn((
                    Node {
                        width: Val::Px(7.0),
                        height: Val::Px(7.0),
                        border_radius: BorderRadius::MAX,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(canon_gold_bright()),
                    BorderColor::all(Color::srgb(1.0, 0.95, 0.82)),
                    GlobalZIndex(921),
                ));
            });

            root.spawn((
                Node {
                    width: Val::Px(SEED_CLUSTER_PX),
                    height: Val::Px(SEED_CLUSTER_PX),
                    position_type: PositionType::Relative,
                    flex_grow: 0.0,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|cluster| {
                for (index, archetype) in MECHA_ARCHETYPES.iter().copied().enumerate() {
                    let (left, top) = archetype.seed_ring_origin();
                    cluster
                        .spawn((
                            Button,
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(left),
                                top: Val::Px(top),
                                width: Val::Px(SEED_RING_PX),
                                height: Val::Px(SEED_RING_PX),
                                border_radius: BorderRadius::MAX,
                                border: UiRect::all(Val::Px(1.5)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            BorderColor::all(canon_gold()),
                            SelectorNodeButton { index },
                        ))
                        .with_children(|ring| {
                            ring.spawn((
                                Text::new(archetype.display_name),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(canon_label()),
                            ));
                        });
                }
            });

            root.spawn((
                Text::new("Select a consciousness · Esc returns to the menu"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(canon_muted()),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(28.0),
                    ..default()
                },
            ));
        });
}

fn update_selector_nodes(
    mut interactions: Query<
        (
            &Interaction,
            &SelectorNodeButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut session: ResMut<StandardMechaSession>,
) {
    for (interaction, button, mut background, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                session.hovered_index = Some(button.index);
                *background = BackgroundColor(canon_gold().with_alpha(0.18));
                *border = BorderColor::all(canon_gold_bright());
            }
            Interaction::Hovered => {
                session.hovered_index = Some(button.index);
                *background = BackgroundColor(canon_gold().with_alpha(0.10));
                *border = BorderColor::all(canon_gold_bright());
            }
            Interaction::None => {
                if session.hovered_index == Some(button.index) {
                    session.hovered_index = None;
                }
                *background = BackgroundColor(Color::NONE);
                *border = BorderColor::all(canon_gold());
            }
        }
    }
}

fn activate_selector_node(
    interactions: Query<(&Interaction, &SelectorNodeButton), Changed<Interaction>>,
    mut session: ResMut<StandardMechaSession>,
    mut next_state: ResMut<NextState<StandardMechaState>>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            session.set_active_index(button.index);
            next_state.set(StandardMechaState::Switching);
            return;
        }
    }
}

fn spawn_switching_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<StandardMechaSession>,
) {
    session.switch_elapsed = 0.0;
    let archetype = session.active();
    let font = asset_server.load(CANON_FONT);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(22.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(930),
            StandardMechaUi,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(168.0),
                    height: Val::Px(168.0),
                    border_radius: BorderRadius::MAX,
                    border: UiRect::all(Val::Px(1.5)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(canon_gold()),
            ))
            .with_children(|ring| {
                ring.spawn((
                    Text::new(archetype.display_name),
                    TextFont {
                        font: font.clone(),
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(canon_label()),
                ));
            });
            root.spawn((
                Text::new(format!(
                    "MANIFESTING {}",
                    archetype.display_name.to_uppercase()
                )),
                TextFont {
                    font: font.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(canon_gold()),
                SwitchingText,
            ));
            root.spawn((
                Text::new(format!("{} · {}", archetype.subtitle, archetype.element)),
                TextFont {
                    font: font.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(canon_muted()),
            ));
        });
}

fn advance_switching(
    time: Res<Time>,
    mut session: ResMut<StandardMechaSession>,
    mut text: Query<&mut Text, With<SwitchingText>>,
    mut next_state: ResMut<NextState<StandardMechaState>>,
) {
    session.switch_elapsed += time.delta_secs();
    if let Ok(mut text) = text.single_mut() {
        let dots = ".".repeat(((session.switch_elapsed * 5.0) as usize % 4).max(1));
        text.0 = format!(
            "MANIFESTING {}{}",
            session.active().display_name.to_uppercase(),
            dots
        );
    }
    if session.switch_elapsed >= SWITCHING_SECS {
        next_state.set(StandardMechaState::Chat);
    }
}

fn enter_chat(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<StandardMechaSession>,
    mut reveal: ResMut<ChatImageReveal>,
    mut fingerprint: ResMut<ChatTranscriptFingerprint>,
) {
    reveal.shown_path = None;
    reveal.alpha = 0.0;
    *fingerprint = ChatTranscriptFingerprint::default();
    let archetype = session.active();
    session.history = read_history(archetype.id).unwrap_or_else(|error| {
        session.status = format!(
            "History unavailable for {}: {error}",
            archetype.display_name
        );
        Vec::new()
    });
    if session.status.starts_with("History unavailable") {
        // Keep the fail-visible status produced above.
    } else {
        session.status = format!(
            "{} active. History loaded from {}.",
            archetype.display_name,
            history_path(archetype.id).display()
        );
    }
    spawn_chat_ui(&mut commands, &asset_server, &session);
}

fn spawn_chat_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    session: &StandardMechaSession,
) {
    let _archetype = session.active();
    let font = asset_server.load(CANON_FONT);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(925),
            StandardMechaUi,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("A R C H E T Y P E S"),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(canon_gold()),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(28.0),
                    top: Val::Px(22.0),
                    ..default()
                },
            ));
            spawn_chat_switcher(root, asset_server, session);
            spawn_chat_panel(root, asset_server, session);
            spawn_portrait_panel(root, asset_server, session);
        });
}

fn spawn_chat_switcher(
    root: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    session: &StandardMechaSession,
) {
    let font = asset_server.load(CANON_FONT);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(34.0),
            top: Val::Px(18.0),
            height: Val::Px(52.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|row| {
        for (index, archetype) in MECHA_ARCHETYPES.iter().copied().enumerate() {
            let active = index == session.active_index;
            row.spawn((
                Button,
                Node {
                    width: Val::Px(if active { 118.0 } else { 44.0 }),
                    height: Val::Px(44.0),
                    border_radius: BorderRadius::MAX,
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                    ..default()
                },
                BackgroundColor(if active {
                    canon_gold().with_alpha(0.14)
                } else {
                    Color::NONE
                }),
                BorderColor::all(if active {
                    canon_gold_bright()
                } else {
                    canon_gold_soft()
                }),
                ChatSwitchButton { index },
            ))
            .with_children(|button| {
                if active {
                    button.spawn((
                        Text::new(archetype.display_name),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(canon_label()),
                    ));
                } else {
                    button.spawn((
                        Text::new(
                            archetype
                                .display_name
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_string(),
                        ),
                        TextFont {
                            font: font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(canon_muted()),
                    ));
                }
            });
        }
    });
}

fn spawn_chat_panel(
    root: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    session: &StandardMechaSession,
) {
    let archetype = session.active();
    let font = asset_server.load(CANON_FONT);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(30.0),
            top: Val::Px(86.0),
            width: Val::Percent(57.0),
            height: Val::Percent(84.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(1.0)),
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(canon_panel_fill()),
        BorderColor::all(canon_gold_soft()),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new(format!("{} CHANNEL", archetype.display_name.to_uppercase())),
            TextFont {
                font: font.clone(),
                font_size: 22.0,
                ..default()
            },
            TextColor(canon_gold()),
            ChatTitleText,
        ));
        panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(68.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                BorderColor::all(canon_gold().with_alpha(0.28)),
                ScrollPosition(Vec2::ZERO),
                ChatTranscriptScrollStart(Vec2::ZERO),
                ChatTranscriptRoot,
                Pickable::default(),
            ))
            .observe(on_chat_transcript_drag_start)
            .observe(on_chat_transcript_drag)
            .with_children(|transcript| {
                populate_chat_transcript(
                    transcript,
                    asset_server,
                    &session.history,
                    false,
                    "",
                    0.0,
                    1.0,
                    None,
                    canon_gold_rgb(),
                );
            });
        panel.spawn((
            Text::new(waiting_banner_text(false, "", 0.0)),
            TextFont {
                font: font.clone(),
                font_size: 15.0,
                ..default()
            },
            TextColor(canon_gold()),
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(0.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
                border: UiRect::all(Val::Px(0.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(canon_gold().with_alpha(0.12)),
            BorderColor::all(canon_gold_soft()),
            ChatWaitingBanner,
        ));
        panel.spawn((
            Text::new(draft_with_caret(session)),
            TextFont {
                font: font.clone(),
                font_size: 17.0,
                ..default()
            },
            TextColor(canon_label()),
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(58.0),
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            BorderColor::all(canon_gold_soft()),
            ChatInputText,
        ));
        panel.spawn((
            Text::new(session.status.clone()),
            TextFont {
                font: font.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(canon_muted()),
            ChatStatusText,
        ));
    });
}

fn spawn_portrait_panel(
    root: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    session: &StandardMechaSession,
) {
    let archetype = session.active();
    let font = asset_server.load(CANON_FONT);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(34.0),
            top: Val::Px(86.0),
            width: Val::Percent(33.0),
            height: Val::Percent(84.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(1.0)),
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(canon_panel_fill()),
        BorderColor::all(canon_gold_soft()),
    ))
    .with_children(|panel| {
        panel.spawn((
            ImageNode::new(asset_server.load(archetype.portrait_path))
                .with_mode(NodeImageMode::Auto),
            portrait_node(archetype),
            BorderColor::all(canon_gold()),
            ChatPortrait,
        ));
        panel.spawn((
            Text::new(archetype.display_name.to_uppercase()),
            TextFont {
                font: font.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(canon_gold()),
            ChatTitleText,
        ));
        panel.spawn((
            Text::new(archetype.subtitle),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(canon_label()),
            ChatSubtitleText,
        ));
        panel.spawn((
            Text::new(format!("Element: {}", archetype.element)),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(canon_muted()),
            ChatElementText,
        ));
        panel.spawn((
            Text::new("CHANNEL IMAGE STATUS"),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(canon_gold_soft()),
        ));
        panel.spawn((
            Text::new(latest_render_status(&session.history, false, "", 0.0)),
            TextFont {
                font: font.clone(),
                font_size: 11.0,
                ..default()
            },
            TextColor(canon_muted()),
            ChatRenderStatusText,
        ));
        panel.spawn((
            Text::new(service_status_line()),
            TextFont {
                font: font.clone(),
                font_size: 12.0,
                ..default()
            },
            TextColor(canon_muted()),
            ChatServiceStatusText,
        ));
    });
}

fn activate_chat_switcher(
    interactions: Query<(&Interaction, &ChatSwitchButton), Changed<Interaction>>,
    mut session: ResMut<StandardMechaSession>,
    bridge: Res<ChatBridge>,
    mut next_state: ResMut<NextState<StandardMechaState>>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed && button.index != session.active_index {
            if bridge.waiting {
                session.status =
                    "A response is still forming; wait before switching archetypes.".to_owned();
                return;
            }
            session.set_active_index(button.index);
            session.draft.clear();
            session.cursor = 0;
            next_state.set(StandardMechaState::Switching);
            return;
        }
    }
}

fn handle_chat_keyboard(
    keyboard: Res<ButtonInput<Key>>,
    keycodes: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<StandardMechaSession>,
    mut bridge: ResMut<ChatBridge>,
    mut next_state: ResMut<NextState<StandardMechaState>>,
) {
    if keycodes.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard.just_pressed(Key::Space)
    {
        if bridge.waiting {
            session.status =
                "A response is still forming; selector return is blocked until it lands."
                    .to_owned();
        } else {
            next_state.set(StandardMechaState::Selector);
        }
        return;
    }

    if keycodes.just_pressed(KeyCode::Escape) {
        if bridge.waiting {
            session.status =
                "A response is still forming; selector return is blocked until it lands."
                    .to_owned();
        } else {
            next_state.set(StandardMechaState::Selector);
        }
        return;
    }

    for key in keyboard.get_just_pressed() {
        match key {
            Key::Backspace => backspace_at_cursor(&mut session),
            Key::Delete => delete_at_cursor(&mut session),
            Key::ArrowLeft => session.cursor = session.cursor.saturating_sub(1),
            Key::ArrowRight => {
                session.cursor = (session.cursor + 1).min(session.draft.chars().count())
            }
            Key::Home => session.cursor = 0,
            Key::End => session.cursor = session.draft.chars().count(),
            Key::Space => insert_at_cursor(&mut session, " "),
            Key::Enter => start_chat_send(&mut session, &mut bridge),
            Key::Character(text) => {
                let room = MAX_DRAFT_CHARS.saturating_sub(session.draft.chars().count());
                let filtered: String = text
                    .chars()
                    .filter(|character| !character.is_control())
                    .take(room)
                    .collect();
                insert_at_cursor(&mut session, &filtered);
            }
            _ => {}
        }
    }
}

fn start_chat_send(session: &mut StandardMechaSession, bridge: &mut ChatBridge) {
    if bridge.waiting {
        session.status = "The current reply is still forming through Ollama.".to_owned();
        return;
    }
    let content = session.draft.trim().to_owned();
    if content.is_empty() {
        return;
    }
    let archetype = session.active();
    let user_record = ChatRecord {
        timestamp: now_millis(),
        archetype: archetype.id.to_owned(),
        role: "witness".to_owned(),
        content: content.clone(),
        image: None,
    };
    if let Err(error) = append_history_record(archetype.id, &user_record) {
        session.status = format!("Chat blocked: history could not be written ({error}).");
        return;
    }
    session.history.push(user_record);
    session.draft.clear();
    session.cursor = 0;

    let keywords = extract_keywords(&content);
    if let Err(error) = append_to_ledger(
        GameMode::Standard,
        "mecha_chat_user",
        json!({
            "archetype": archetype.id,
            "chars": content.chars().count(),
            "keywords": keywords.clone(),
            "forever_law": "history_jsonl_and_hash_chained_ledger"
        }),
    ) {
        session.status = format!(
            "Chat blocked after history write: Forever Law ledger could not be sealed ({error}). No LLM/render was started."
        );
        return;
    }
    session.status = format!(
        "Ollama is forming {}'s reply...",
        archetype.display_name
    );

    let model = ollama_model();
    let system = archetype.persona().to_owned();
    let context = llm_context(&session.history);
    let prompt = format!(
        "The Witness is chatting with you directly inside Archetypes Standard Mode.\n\
         Recent durable history for this archetype:\n{context}\n\n\
         New Witness message:\n{content}\n\n\
         Reply in character as {}, in one to three concise paragraphs. Do not invent system state.",
        archetype.display_name
    );
    let (sender, receiver) = mpsc::channel();
    *bridge.receiver.lock().expect("standard chat receiver lock") = Some(receiver);
    bridge.waiting = true;
    bridge.wait_elapsed_secs = 0.0;
    bridge.wait_tick_accum = 0.0;
    bridge.phase = format!("Ollama is forming {}'s reply...", archetype.display_name);
    let display_name = archetype.display_name.to_owned();
    thread::spawn(move || {
        let _ = sender.send(ChatBridgeMsg::Phase(format!(
            "Ollama is forming {display_name}'s reply..."
        )));
        let response = ollama_chat(&model, &system, &prompt);
        let _ = sender.send(ChatBridgeMsg::Phase(
            "Chronos/Comfy is painting the response image...".to_owned(),
        ));
        let image = render_chat_image(archetype, &content, keywords);
        let _ = sender.send(ChatBridgeMsg::Done(ChatTurnResult {
            archetype_id: archetype.id.to_owned(),
            response,
            image,
        }));
    });
}

fn tick_chat_wait_indicator(time: Res<Time>, mut bridge: ResMut<ChatBridge>) {
    if !bridge.waiting {
        // Do not touch the resource every idle frame — that marks ChatBridge changed,
        // forces a full transcript rebuild, and can stick new images at alpha 0.
        if bridge.wait_elapsed_secs != 0.0 || bridge.wait_tick_accum != 0.0 {
            bridge.wait_elapsed_secs = 0.0;
            bridge.wait_tick_accum = 0.0;
        }
        return;
    }
    bridge.wait_elapsed_secs += time.delta_secs();
    bridge.wait_tick_accum += time.delta_secs();
    if bridge.wait_tick_accum >= WAIT_TICK_SECS {
        bridge.wait_tick_accum = 0.0;
        // Touch the resource so render_chat_ui refreshes the live seconds counter.
        bridge.phase = bridge.phase.clone();
    }
}

fn poll_chat_response(mut session: ResMut<StandardMechaSession>, mut bridge: ResMut<ChatBridge>) {
    if !bridge.waiting {
        return;
    }
    loop {
        let msg = {
            let guard = bridge.receiver.lock().expect("standard chat receiver lock");
            guard.as_ref().and_then(|receiver| receiver.try_recv().ok())
        };
        let Some(msg) = msg else {
            break;
        };
        match msg {
            ChatBridgeMsg::Phase(phase) => {
                bridge.phase = phase.clone();
                session.status = phase;
            }
            ChatBridgeMsg::Done(result) => {
                bridge.waiting = false;
                bridge.phase.clear();
                bridge.wait_elapsed_secs = 0.0;
                bridge.wait_tick_accum = 0.0;
                finish_chat_turn(&mut session, result);
                break;
            }
        }
    }
}

fn finish_chat_turn(session: &mut StandardMechaSession, result: ChatTurnResult) {
    let ChatTurnResult {
        archetype_id,
        response,
        image,
    } = result;
    let archetype = by_id(&archetype_id)
        .map(|(_, archetype)| archetype)
        .unwrap_or_else(|| session.active());
    let (role, content, response_status, ledger_kind) = match response {
        Ok(content) => (
            archetype_id.clone(),
            content,
            "complete".to_owned(),
            "mecha_chat_assistant",
        ),
        Err(error) => (
            "system".to_owned(),
            format!("LOCAL LLM FAILURE: {error}"),
            "failed".to_owned(),
            "mecha_chat_failed",
        ),
    };
    let record = ChatRecord {
        timestamp: now_millis(),
        archetype: archetype_id.clone(),
        role,
        content: content.clone(),
        image: Some(image.clone()),
    };
    let history_result = append_history_record(&archetype_id, &record);
    session.history.push(record);
    let response_ok = response_status == "complete";
    let ledger_result = append_to_ledger(
        GameMode::Standard,
        ledger_kind,
        json!({
            "archetype": archetype_id,
            "response_status": response_status,
            "chars": content.chars().count(),
            "image": image.clone(),
            "forever_law": "history_jsonl_and_hash_chained_ledger"
        }),
    );

    session.status = chat_completion_status(
        archetype,
        response_ok,
        &image,
        history_result,
        ledger_result,
    );
}

fn chat_completion_status(
    archetype: MechaArchetype,
    response_ok: bool,
    image: &ChatImage,
    history_result: Result<(), String>,
    ledger_result: Result<(), String>,
) -> String {
    if let Err(error) = history_result {
        return format!(
            "{} responded, but history persistence failed: {error}",
            archetype.display_name
        );
    }
    if let Err(error) = ledger_result {
        return format!(
            "{} responded, but Forever Law ledger sealing failed: {error}",
            archetype.display_name
        );
    }
    if image.status != "complete" {
        if response_ok {
            return format!(
                "{} answered, but Chronos/Comfy image failed: {}",
                archetype.display_name,
                compact(&image.detail, 180)
            );
        }
        return format!(
            "Local LLM failed; Chronos/Comfy image also failed: {}",
            compact(&image.detail, 180)
        );
    }
    if response_ok {
        format!(
            "{} answered with a Comfy image. Turn history and ledger sealed.",
            archetype.display_name
        )
    } else {
        "Local LLM failed; the Comfy image and failure were sealed visibly.".to_owned()
    }
}

fn render_chat_ui(
    mut commands: Commands,
    session: Res<StandardMechaSession>,
    bridge: Res<ChatBridge>,
    reveal: Res<ChatImageReveal>,
    mut fingerprint: ResMut<ChatTranscriptFingerprint>,
    asset_server: Res<AssetServer>,
    transcript_roots: Query<Entity, With<ChatTranscriptRoot>>,
    mut input: Query<&mut Text, With<ChatInputText>>,
    mut status: Query<
        &mut Text,
        (
            With<ChatStatusText>,
            Without<ChatInputText>,
            Without<ChatRenderStatusText>,
            Without<ChatServiceStatusText>,
            Without<ChatWaitingBanner>,
        ),
    >,
    mut waiting_banner: Query<
        (&mut Text, &mut Node, &mut BackgroundColor, &mut BorderColor),
        (
            With<ChatWaitingBanner>,
            Without<ChatPortrait>,
            Without<ChatStatusText>,
            Without<ChatInputText>,
            Without<ChatRenderStatusText>,
            Without<ChatServiceStatusText>,
        ),
    >,
    mut portraits: Query<
        (&mut ImageNode, &mut Node),
        (With<ChatPortrait>, Without<ChatWaitingBanner>),
    >,
    mut render_status: Query<
        &mut Text,
        (
            With<ChatRenderStatusText>,
            Without<ChatStatusText>,
            Without<ChatInputText>,
            Without<ChatServiceStatusText>,
            Without<ChatWaitingBanner>,
        ),
    >,
) {
    let session_changed = session.is_changed();
    let bridge_changed = bridge.is_changed();
    if !session_changed && !bridge_changed {
        return;
    }

    let archetype = session.active();
    let last_timestamp = session.history.last().map(|r| r.timestamp).unwrap_or(0);
    let history_grew = session.history.len() > fingerprint.history_len
        || last_timestamp > fingerprint.last_timestamp;
    let transcript_changed = fingerprint.history_len != session.history.len()
        || fingerprint.last_timestamp != last_timestamp
        || fingerprint.waiting != bridge.waiting
        || fingerprint.phase != bridge.phase;

    if transcript_changed {
        fingerprint.history_len = session.history.len();
        fingerprint.last_timestamp = last_timestamp;
        fingerprint.waiting = bridge.waiting;
        fingerprint.phase = bridge.phase.clone();

        let latest_path = latest_chat_image(&session.history)
            .filter(|image| image.status == "complete")
            .and_then(|image| image.asset_path.as_deref());
        let reveal_alpha = match (&reveal.shown_path, latest_path) {
            (Some(shown), Some(latest)) if shown == latest => reveal.alpha.max(0.05),
            (None, Some(_)) => 1.0,
            (_, Some(_)) => 0.05,
            _ => 1.0,
        };
        if let Ok(root) = transcript_roots.single() {
            commands.entity(root).despawn_related::<Children>();
            commands.entity(root).with_children(|transcript| {
                populate_chat_transcript(
                    transcript,
                    &asset_server,
                    &session.history,
                    bridge.waiting,
                    &bridge.phase,
                    bridge.wait_elapsed_secs,
                    reveal_alpha,
                    latest_path,
                    canon_gold_rgb(),
                );
            });
            // Only jump to the newest turn when history grows — never fight manual scroll
            // during wait-phase refreshes.
            if history_grew {
                commands
                    .entity(root)
                    .insert(ScrollPosition(Vec2::new(0.0, 1_000_000.0)));
            }
        }
    }

    if let Ok(mut text) = input.single_mut() {
        text.0 = draft_with_caret(&session);
    }
    if let Ok(mut text) = status.single_mut() {
        text.0 = if bridge.waiting {
            waiting_status_line(&bridge.phase, bridge.wait_elapsed_secs)
        } else {
            session.status.clone()
        };
    }
    if let Ok((mut text, mut node, mut background, mut border)) = waiting_banner.single_mut() {
        if bridge.waiting {
            text.0 = waiting_banner_text(true, &bridge.phase, bridge.wait_elapsed_secs);
            node.display = Display::Flex;
            node.min_height = Val::Px(44.0);
            node.padding = UiRect::all(Val::Px(12.0));
            node.border = UiRect::all(Val::Px(1.0));
            *background = BackgroundColor(canon_gold().with_alpha(0.14));
            *border = BorderColor::all(canon_gold());
        } else {
            text.0.clear();
            node.display = Display::None;
            node.min_height = Val::Px(0.0);
            node.padding = UiRect::axes(Val::Px(12.0), Val::Px(0.0));
            node.border = UiRect::all(Val::Px(0.0));
        }
    }
    if session_changed {
        if let Ok((mut portrait, mut node)) = portraits.single_mut() {
            portrait.image = asset_server.load(archetype.portrait_path);
            portrait.image_mode = NodeImageMode::Auto;
            *node = portrait_node(archetype);
        }
    }
    if let Ok(mut text) = render_status.single_mut() {
        text.0 = latest_render_status(
            &session.history,
            bridge.waiting,
            &bridge.phase,
            bridge.wait_elapsed_secs,
        );
    }
}

fn send_chat_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= CHAT_SCROLL_LINE_HEIGHT;
        }
        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(ChatUiScroll { entity, delta });
            }
        }
    }
}

fn on_chat_scroll_handler(
    mut scroll: On<ChatUiScroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };
    if node.overflow.y != OverflowAxis::Scroll && node.overflow.x != OverflowAxis::Scroll {
        return;
    }

    let max_offset = ((computed.content_size() - computed.size()) * computed.inverse_scale_factor())
        .max(Vec2::ZERO);
    let delta = &mut scroll.delta;

    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let at_end = if delta.x > 0.0 {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.0
        };
        if !at_end {
            scroll_position.x = (scroll_position.x + delta.x).clamp(0.0, max_offset.x);
            delta.x = 0.0;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let at_end = if delta.y > 0.0 {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.0
        };
        if !at_end {
            scroll_position.y = (scroll_position.y + delta.y).clamp(0.0, max_offset.y);
            delta.y = 0.0;
        }
    }

    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}

fn on_chat_transcript_drag_start(
    drag_start: On<Pointer<DragStart>>,
    mut query: Query<(&ComputedNode, &mut ChatTranscriptScrollStart), With<ChatTranscriptRoot>>,
) {
    let Ok((computed, mut start)) = query.get_mut(drag_start.entity) else {
        return;
    };
    start.0 = computed.scroll_position * computed.inverse_scale_factor;
}

fn on_chat_transcript_drag(
    drag: On<Pointer<Drag>>,
    mut query: Query<
        (&mut ScrollPosition, &ChatTranscriptScrollStart, &ComputedNode),
        With<ChatTranscriptRoot>,
    >,
) {
    let Ok((mut scroll_position, start, computed)) = query.get_mut(drag.entity) else {
        return;
    };
    let max_offset = ((computed.content_size() - computed.size()) * computed.inverse_scale_factor())
        .max(Vec2::ZERO);
    let next = (start.0 - Vec2::new(drag.distance.x, drag.distance.y)).max(Vec2::ZERO);
    scroll_position.0 = next.min(max_offset);
}

fn refresh_service_status_line(
    time: Res<Time>,
    mut clock: ResMut<ServiceStatusClock>,
    mut status: Query<&mut Text, With<ChatServiceStatusText>>,
) {
    clock.elapsed += time.delta_secs();
    if clock.elapsed < SERVICE_STATUS_REFRESH_SECS && !clock.line.is_empty() {
        return;
    }
    clock.elapsed = 0.0;
    clock.line = service_status_line();
    if let Ok(mut text) = status.single_mut() {
        text.0 = clock.line.clone();
    }
}

fn reveal_chat_image(
    time: Res<Time>,
    session: Res<StandardMechaSession>,
    mut reveal: ResMut<ChatImageReveal>,
    mut artifacts: Query<(&ChatArtifactImage, &mut ImageNode)>,
) {
    let target = latest_chat_image(&session.history)
        .filter(|image| image.status == "complete")
        .and_then(|image| image.asset_path.clone());

    match target {
        Some(path) => {
            if reveal.shown_path.as_deref() != Some(path.as_str()) {
                reveal.shown_path = Some(path.clone());
                reveal.alpha = 0.0;
            } else {
                reveal.alpha = (reveal.alpha + time.delta_secs() / IMAGE_REVEAL_SECS).min(1.0);
            }
            for (artifact, mut image_node) in &mut artifacts {
                if artifact.asset_path == path {
                    image_node.color = Color::srgba(1.0, 1.0, 1.0, reveal.alpha);
                } else {
                    image_node.color = Color::WHITE;
                }
            }
        }
        None => {
            reveal.shown_path = None;
            reveal.alpha = 0.0;
            for (_, mut image_node) in &mut artifacts {
                image_node.color = Color::WHITE;
            }
        }
    }
}

fn portrait_node(archetype: MechaArchetype) -> Node {
    let height = PORTRAIT_DISPLAY_WIDTH / archetype.portrait_aspect.max(0.01);
    Node {
        width: Val::Px(PORTRAIT_DISPLAY_WIDTH),
        height: Val::Px(height),
        aspect_ratio: Some(archetype.portrait_aspect),
        border: UiRect::all(Val::Px(2.0)),
        flex_shrink: 0.0,
        ..default()
    }
}

fn waiting_banner_text(waiting: bool, phase: &str, elapsed_secs: f32) -> String {
    if !waiting {
        return String::new();
    }
    waiting_status_line(phase, elapsed_secs)
}

fn waiting_status_line(phase: &str, elapsed_secs: f32) -> String {
    let secs = elapsed_secs.floor() as u32;
    let phase = if phase.is_empty() {
        "Working — local model and Chronos/Comfy are still running"
    } else {
        phase
    };
    format!("WORKING ({secs}s) — {phase}")
}

fn populate_chat_transcript(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    history: &[ChatRecord],
    waiting: bool,
    phase: &str,
    wait_elapsed_secs: f32,
    latest_reveal_alpha: f32,
    latest_path: Option<&str>,
    accent: Rgb,
) {
    if history.is_empty() && !waiting {
        parent.spawn((
            Text::new(
                "No prior messages for this archetype yet. Type and press Enter to send through local Ollama.",
            ),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.88, 0.92)),
        ));
        return;
    }

    let start = history.len().saturating_sub(HISTORY_RENDER_LIMIT);
    for record in &history[start..] {
        let label = chat_record_label(record);
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|message| {
                message.spawn((
                    Text::new(format!("{label}: {}", compact(&record.content, 700))),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.86, 0.88, 0.92)),
                ));
                if let Some(image) = &record.image {
                    message.spawn((
                        Text::new(format!("Image: {}", image_history_line(image))),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.74, 0.82)),
                    ));
                    if image.status == "complete" {
                        if let Some(asset_path) = image.asset_path.as_deref() {
                            let alpha = if latest_path == Some(asset_path) {
                                latest_reveal_alpha
                            } else {
                                1.0
                            };
                            message.spawn((
                                ImageNode {
                                    image: asset_server.load(asset_path.to_owned()),
                                    color: Color::srgba(1.0, 1.0, 1.0, alpha),
                                    image_mode: NodeImageMode::Auto,
                                    ..default()
                                },
                                Node {
                                    width: Val::Percent(92.0),
                                    height: Val::Auto,
                                    max_width: Val::Px(520.0),
                                    max_height: Val::Px(CHAT_ARTIFACT_MAX_HEIGHT),
                                    border: UiRect::all(Val::Px(1.0)),
                                    align_self: AlignSelf::FlexStart,
                                    ..default()
                                },
                                BorderColor::all(accent.alpha(0.55)),
                                ChatArtifactImage {
                                    asset_path: asset_path.to_owned(),
                                },
                            ));
                        }
                    }
                }
            });
    }

    if waiting {
        parent.spawn((
            Text::new(waiting_status_line(phase, wait_elapsed_secs)),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(accent.color()),
        ));
    }
}

fn chat_record_label(record: &ChatRecord) -> &str {
    match record.role.as_str() {
        "witness" => "Witness",
        "system" => "System",
        other => MECHA_ARCHETYPES
            .iter()
            .find(|archetype| archetype.id == other)
            .map(|archetype| archetype.display_name)
            .unwrap_or(other),
    }
}

fn service_status_line() -> String {
    let snap = probe_readiness();
    match snap.player_hint() {
        Some(hint) => format!("{}\n{hint}", snap.banner_line()),
        None => format!("{}\nCouncil chat + Chronos painting ready.", snap.banner_line()),
    }
}

/// Kept for unit tests and as a plain-text dump of the visible transcript contract.
#[allow(dead_code)]
fn render_history(history: &[ChatRecord], waiting: bool, phase: &str) -> String {
    if history.is_empty() && !waiting {
        return "No prior messages for this archetype yet. Type and press Enter to send through local Ollama.".to_owned();
    }
    let start = history.len().saturating_sub(HISTORY_RENDER_LIMIT);
    let mut lines: Vec<String> = history[start..]
        .iter()
        .map(|record| {
            let label = chat_record_label(record);
            let mut line = format!("{label}: {}", compact(&record.content, 700));
            if let Some(image) = &record.image {
                line.push_str(&format!("\nImage: {}", image_history_line(image)));
            }
            line
        })
        .collect();
    if waiting {
        let wait_line = if phase.is_empty() {
            "System: the reply is still forming...".to_owned()
        } else {
            format!("System: {phase}")
        };
        lines.push(wait_line);
    }
    lines.join("\n\n")
}

fn latest_chat_image(history: &[ChatRecord]) -> Option<&ChatImage> {
    history
        .iter()
        .rev()
        .find_map(|record| record.image.as_ref())
}

fn image_history_line(image: &ChatImage) -> String {
    let keywords = image.keywords.join(", ");
    if image.status == "complete" {
        format!("complete | keywords: {keywords}")
    } else {
        format!(
            "FAILED | keywords: {keywords} | {}",
            compact(&image.detail, 220)
        )
    }
}

fn latest_render_status(
    history: &[ChatRecord],
    waiting: bool,
    phase: &str,
    wait_elapsed_secs: f32,
) -> String {
    if waiting {
        return waiting_status_line(phase, wait_elapsed_secs);
    }
    let Some(image) = latest_chat_image(history) else {
        return "No rendered response in this chat yet.".to_owned();
    };
    if image.status == "complete" {
        format!("Complete. Keywords: {}", image.keywords.join(", "))
    } else {
        format!("IMAGE FAILURE: {}", compact(&image.detail, 320))
    }
}

fn compact(text: &str, max: usize) -> String {
    let mut chars = text.chars();
    let clipped: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{clipped}...")
    } else {
        clipped
    }
}

fn llm_context(history: &[ChatRecord]) -> String {
    let start = history.len().saturating_sub(8);
    if history[start..].is_empty() {
        return "(none yet)".to_owned();
    }
    history[start..]
        .iter()
        .map(|record| format!("{}: {}", record.role, compact(&record.content, 360)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_keywords(statement: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    for token in statement.split(|character: char| !character.is_ascii_alphanumeric()) {
        let token = token.trim().to_ascii_lowercase();
        if token.len() < 3 || KEYWORD_STOP_WORDS.contains(&token.as_str()) {
            continue;
        }
        if keywords.iter().any(|known| known == &token) {
            continue;
        }
        keywords.push(token);
        if keywords.len() >= KEYWORD_LIMIT {
            break;
        }
    }
    if keywords.is_empty() {
        keywords.push("witness".to_owned());
    }
    keywords
}

fn render_chat_image(
    archetype: MechaArchetype,
    witness_statement: &str,
    keywords: Vec<String>,
) -> ChatImage {
    let prompt = image_prompt(archetype, witness_statement, &keywords);
    let art_style = archetype.art_style().to_owned();
    let outcome = request_chronos_artifact_with_style(
        &prompt,
        &art_style,
        &format!("archetypes-standard-{}", archetype.id),
    );
    image_from_outcome(archetype, keywords, prompt, art_style, outcome)
}

fn image_prompt(archetype: MechaArchetype, witness_statement: &str, keywords: &[String]) -> String {
    format!(
        "Archetypes Standard Mode chat response image. Archetype: {}. Domain: {}. \
         Extracted keywords: {}. Witness statement: {}. Build one finished image that feels like \
         this archetype answering the statement through symbol and scene. No text, no UI, no logo. {}",
        archetype.display_name,
        archetype.element,
        keywords.join(", "),
        witness_statement,
        archetype.art_style()
    )
}

fn image_from_outcome(
    archetype: MechaArchetype,
    keywords: Vec<String>,
    prompt: String,
    art_style: String,
    outcome: ArtifactOutcome,
) -> ChatImage {
    let mut image = ChatImage {
        status: outcome.status.clone(),
        keywords,
        prompt,
        art_style,
        png_path: outcome.png_path.clone(),
        asset_path: None,
        artifact_id: outcome.artifact_id.clone(),
        proof_receipt_id: outcome.proof_receipt_id.clone(),
        detail: outcome.detail.clone(),
    };
    if outcome.status == "complete" {
        match stage_standard_render(archetype.id, &outcome) {
            Ok(asset_path) => image.asset_path = Some(asset_path),
            Err(error) => {
                image.status = "failed".to_owned();
                image.detail = format!(
                    "Chronos rendered, but the image could not be staged for Bevy: {error}"
                );
            }
        }
    }
    image
}

fn stage_standard_render(archetype_id: &str, outcome: &ArtifactOutcome) -> Result<String, String> {
    let png_path = outcome
        .png_path
        .as_deref()
        .ok_or("Chronos outcome had no PNG path")?;
    let stem = outcome
        .artifact_id
        .as_deref()
        .or(outcome.proof_receipt_id.as_deref())
        .unwrap_or("latest");
    let file_name = format!(
        "{}-{}-{}.png",
        sanitize_id(archetype_id),
        sanitize_id(stem),
        now_millis()
    );
    let dir = standard_render_asset_dir();
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    fs::copy(png_path, dir.join(&file_name)).map_err(|error| error.to_string())?;
    Ok(format!("standard_mecha/renders/{file_name}"))
}

fn standard_render_asset_dir() -> PathBuf {
    let base = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
    } else {
        PathBuf::from("assets")
    };
    base.join("standard_mecha").join("renders")
}

fn byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn insert_at_cursor(session: &mut StandardMechaSession, text: &str) {
    if text.is_empty() {
        return;
    }
    let room = MAX_DRAFT_CHARS.saturating_sub(session.draft.chars().count());
    let text: String = text.chars().take(room).collect();
    let cursor = session.cursor.min(session.draft.chars().count());
    let byte = byte_index(&session.draft, cursor);
    session.draft.insert_str(byte, &text);
    session.cursor = cursor + text.chars().count();
}

fn backspace_at_cursor(session: &mut StandardMechaSession) {
    if session.cursor == 0 {
        return;
    }
    let end = byte_index(&session.draft, session.cursor);
    let start = byte_index(&session.draft, session.cursor - 1);
    session.draft.replace_range(start..end, "");
    session.cursor -= 1;
}

fn delete_at_cursor(session: &mut StandardMechaSession) {
    if session.cursor >= session.draft.chars().count() {
        return;
    }
    let start = byte_index(&session.draft, session.cursor);
    let end = byte_index(&session.draft, session.cursor + 1);
    session.draft.replace_range(start..end, "");
}

fn draft_with_caret(session: &StandardMechaSession) -> String {
    let byte = byte_index(
        &session.draft,
        session.cursor.min(session.draft.chars().count()),
    );
    let mut rendered = session.draft.clone();
    rendered.insert_str(byte, "|");
    format!("> {rendered}")
}

fn history_path(archetype_id: &str) -> PathBuf {
    history_path_for_root(app_data_root(), archetype_id)
}

fn history_path_for_root(root: PathBuf, archetype_id: &str) -> PathBuf {
    root.join("standard_mecha")
        .join("chat_history")
        .join(format!("{}.jsonl", sanitize_id(archetype_id)))
}

fn read_history(archetype_id: &str) -> Result<Vec<ChatRecord>, String> {
    read_history_from(&history_path(archetype_id))
}

fn read_history_from(path: &Path) -> Result<Vec<ChatRecord>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let record: ChatRecord = serde_json::from_str(&line)
            .map_err(|error| format!("history line {} is invalid JSON: {error}", index + 1))?;
        records.push(record);
    }
    Ok(records)
}

fn append_history_record(archetype_id: &str, record: &ChatRecord) -> Result<(), String> {
    append_history_record_to(&history_path(archetype_id), record)
}

fn append_history_record_to(path: &Path, record: &ChatRecord) -> Result<(), String> {
    let parent = path.parent().ok_or("history path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let line = serde_json::to_string(record).map_err(|error| error.to_string())? + "\n";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes())
        .map_err(|error| error.to_string())
}

fn sanitize_id(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "unknown".to_owned()
    } else {
        cleaned
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn despawn_standard_mecha_ui(mut commands: Commands, query: Query<Entity, With<StandardMechaUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Mecha Standard capture mode.
//
// Enabled by ARCHETYPES_MECHA_CAPTURE=1. This deliberately does not reuse the
// old ritual capture script, because that script drives the preserved reference
// ritual states. This one witnesses the new Standard path from the real window.
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct MechaCaptureRun {
    dir: PathBuf,
    title_shot: bool,
    subtitle_shot: bool,
    menu_seen: Option<f32>,
    menu_shot: bool,
    selector_started: bool,
    selector_seen: Option<f32>,
    selector_shot: bool,
    architect_started: bool,
    architect_chat_seen: Option<f32>,
    architect_chat_shot: bool,
    chat_send_started: bool,
    chat_result_seen: Option<f32>,
    chat_result_shot: bool,
    oracle_started: bool,
    oracle_chat_seen: Option<f32>,
    oracle_chat_shot: bool,
    oracle_riddle_triggered: bool,
    oracle_generating_seen: Option<f32>,
    oracle_generating_shot: bool,
    oracle_final_seen: Option<f32>,
    oracle_final_shot: bool,
    exit_at: Option<f32>,
}

impl MechaCaptureRun {
    fn from_env() -> Option<Self> {
        if std::env::var_os("ARCHETYPES_MECHA_CAPTURE").is_none() {
            return None;
        }
        let dir = std::env::var_os("ARCHETYPES_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("artifacts/visual-proof/mecha-standard-capture"));
        let _ = fs::create_dir_all(&dir);
        Some(Self {
            dir,
            title_shot: false,
            subtitle_shot: false,
            menu_seen: None,
            menu_shot: false,
            selector_started: false,
            selector_seen: None,
            selector_shot: false,
            architect_started: false,
            architect_chat_seen: None,
            architect_chat_shot: false,
            chat_send_started: false,
            chat_result_seen: None,
            chat_result_shot: false,
            oracle_started: false,
            oracle_chat_seen: None,
            oracle_chat_shot: false,
            oracle_riddle_triggered: false,
            oracle_generating_seen: None,
            oracle_generating_shot: false,
            oracle_final_seen: None,
            oracle_final_shot: false,
            exit_at: None,
        })
    }

    fn shot(&self, commands: &mut Commands, stem: &str) {
        let path = self.dir.join(format!("{stem}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn run_mecha_capture(
    time: Res<Time>,
    chamber_state: Res<State<ChamberState>>,
    standard_state: Res<State<StandardMechaState>>,
    oracle_state: Res<State<OracleState>>,
    mut next_standard: ResMut<NextState<StandardMechaState>>,
    mut capture: ResMut<MechaCaptureRun>,
    mut session: ResMut<StandardMechaSession>,
    mut bridge: ResMut<ChatBridge>,
    main_menu: Query<Entity, With<MainMenuUi>>,
    standard_ui: Query<Entity, With<StandardMechaUi>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    if !capture.title_shot && now >= 1.4 {
        capture.shot(&mut commands, "00_title_arch");
        capture.title_shot = true;
    }
    if !capture.subtitle_shot && now >= 4.4 {
        capture.shot(&mut commands, "01_title_subtitle");
        capture.subtitle_shot = true;
    }

    if let Some(exit_at) = capture.exit_at {
        if now >= exit_at {
            std::process::exit(0);
        }
        return;
    }

    if *chamber_state.get() == ChamberState::MainMenu && !capture.selector_started {
        let seen = *capture.menu_seen.get_or_insert(now);
        if !capture.menu_shot && now - seen >= 1.0 {
            capture.shot(&mut commands, "02_portal_main_menu");
            capture.menu_shot = true;
        }
        if capture.menu_shot && now - seen >= 1.6 {
            for entity in &main_menu {
                commands.entity(entity).despawn();
            }
            commands.insert_resource(TriggerStandardMecha);
            capture.selector_started = true;
        }
    }

    if *standard_state.get() == StandardMechaState::Selector && capture.selector_started {
        let seen = *capture.selector_seen.get_or_insert(now);
        if !capture.selector_shot && now - seen >= 0.8 {
            capture.shot(&mut commands, "03_mecha_selector");
            capture.selector_shot = true;
        }
        if capture.selector_shot && !capture.architect_started && now - seen >= 1.3 {
            if let Some((index, _)) = by_id("architect") {
                session.set_active_index(index);
            }
            next_standard.set(StandardMechaState::Switching);
            capture.architect_started = true;
        }
    }

    if *standard_state.get() == StandardMechaState::Chat
        && capture.architect_started
        && !capture.oracle_started
    {
        let active = session.active();
        if active.id == "architect" {
            let seen = *capture.architect_chat_seen.get_or_insert(now);
            if !capture.architect_chat_shot && now - seen >= 0.8 {
                capture.shot(&mut commands, "04_architect_chat");
                capture.architect_chat_shot = true;
            }
            if capture.architect_chat_shot && !capture.chat_send_started && now - seen >= 1.2 {
                session.draft =
                    "Name the strongest structure for a sovereign local-first AI council."
                        .to_owned();
                session.cursor = session.draft.chars().count();
                start_chat_send(&mut session, &mut bridge);
                capture.chat_send_started = true;
            }
            if capture.chat_send_started && !bridge.waiting {
                let result_seen = *capture.chat_result_seen.get_or_insert(now);
                if !capture.chat_result_shot && now - result_seen >= 0.8 {
                    capture.shot(&mut commands, "05_architect_chat_result_or_failure");
                    capture.chat_result_shot = true;
                }
                if capture.chat_result_shot && !capture.oracle_started && now - result_seen >= 1.4 {
                    if let Some((index, _)) = by_id("oracle") {
                        session.set_active_index(index);
                    }
                    next_standard.set(StandardMechaState::Switching);
                    capture.oracle_started = true;
                }
            }
        }
    }

    if *standard_state.get() == StandardMechaState::Chat && capture.oracle_started {
        let active = session.active();
        if active.id == "oracle" {
            let seen = *capture.oracle_chat_seen.get_or_insert(now);
            if !capture.oracle_chat_shot && now - seen >= 2.5 {
                capture.shot(&mut commands, "06_oracle_chat_after_switch");
                capture.oracle_chat_shot = true;
            }
        }
    }

    if capture.oracle_chat_shot && !capture.oracle_riddle_triggered {
        let seen = capture.oracle_chat_seen.unwrap_or(now);
        if now - seen >= 3.6 {
            for entity in &standard_ui {
                commands.entity(entity).despawn();
            }
            next_standard.set(StandardMechaState::Inactive);
            commands.insert_resource(TriggerOracleRiddle);
            capture.oracle_riddle_triggered = true;
        }
    }

    if capture.oracle_riddle_triggered {
        match oracle_state.get() {
            OracleState::Generating => {
                let seen = *capture.oracle_generating_seen.get_or_insert(now);
                if !capture.oracle_generating_shot && now - seen >= 0.8 {
                    capture.shot(&mut commands, "07_oracle_riddle_generating");
                    capture.oracle_generating_shot = true;
                }
            }
            OracleState::Guessing | OracleState::Result => {
                let seen = *capture.oracle_final_seen.get_or_insert(now);
                if !capture.oracle_final_shot && now - seen >= 1.0 {
                    capture.shot(&mut commands, "08_oracle_riddle_guessing_or_result");
                    capture.oracle_final_shot = true;
                    capture.exit_at = Some(now + 2.0);
                }
            }
            OracleState::Inactive | OracleState::Scoring => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn registry_represents_all_seven_mecha_archetypes() {
        assert_eq!(MECHA_ARCHETYPES.len(), 7);
        let ids: HashSet<&str> = MECHA_ARCHETYPES
            .iter()
            .map(|archetype| archetype.id)
            .collect();
        assert_eq!(ids.len(), 7);
        for expected in [
            "architect",
            "sentinel",
            "mentor",
            "explorer",
            "oracle",
            "empath",
            "jester",
        ] {
            assert!(ids.contains(expected));
        }
    }

    #[test]
    fn known_selector_css_color_mismatches_are_explicit() {
        let mismatched: HashSet<&str> = MECHA_ARCHETYPES
            .iter()
            .filter(|archetype| !archetype.selector_color_matches_css())
            .map(|archetype| archetype.id)
            .collect();
        assert_eq!(mismatched, HashSet::from(["mentor", "oracle", "jester"]));
    }

    #[test]
    fn registry_asset_paths_exist_in_workspace_assets() {
        let root = workspace_assets_root();
        for archetype in MECHA_ARCHETYPES {
            assert!(
                root.join(archetype.portrait_path).exists(),
                "missing portrait {}",
                archetype.portrait_path
            );
            assert!(
                root.join(archetype.icon_path).exists(),
                "missing icon {}",
                archetype.icon_path
            );
        }
        assert!(root.join("mecha/logo.png").exists());
        assert!(root.join("mecha/uxbacklayer.png").exists());
        assert!(root.join("mecha/council000.png").exists());
        assert!(root.join("mecha/council001.png").exists());
        assert!(root.join("mecha/council002.png").exists());
        assert!(
            root.join(CANON_FONT).exists(),
            "missing canon serif font {CANON_FONT}"
        );
        assert!(
            root.join(UI_CANON_REF).exists(),
            "missing Seed-of-Life UI canon reference {UI_CANON_REF}"
        );
        let _ = SOURCE_CANON;
    }

    #[test]
    fn seed_of_life_slots_match_operator_canon() {
        let by_id: std::collections::HashMap<&str, u8> = MECHA_ARCHETYPES
            .iter()
            .map(|a| (a.id, a.seed_slot))
            .collect();
        assert_eq!(by_id["sentinel"], 0);
        assert_eq!(by_id["architect"], 1);
        assert_eq!(by_id["mentor"], 2);
        assert_eq!(by_id["explorer"], 3);
        assert_eq!(by_id["oracle"], 4);
        assert_eq!(by_id["empath"], 5);
        assert_eq!(by_id["jester"], 6);
        let slots: HashSet<u8> = MECHA_ARCHETYPES.iter().map(|a| a.seed_slot).collect();
        assert_eq!(slots, HashSet::from([0, 1, 2, 3, 4, 5, 6]));

        let (cx, cy) = seed_slot_center(0);
        let mid = SEED_CLUSTER_PX * 0.5;
        assert!((cx - mid).abs() < 0.01 && (cy - mid).abs() < 0.01);
        let (ax, ay) = seed_slot_center(1);
        assert!((ax - mid).abs() < 0.01);
        assert!((ay - (mid - SEED_RADIUS_PX)).abs() < 0.01);
    }

    #[test]
    fn portrait_aspect_ratios_match_source_asset_shapes() {
        for archetype in MECHA_ARCHETYPES {
            let expected = match archetype.id {
                "sentinel" | "explorer" => PORTRAIT_ASPECT_SQUARE,
                _ => PORTRAIT_ASPECT_TALL,
            };
            assert!(
                (archetype.portrait_aspect - expected).abs() < f32::EPSILON,
                "{} portrait_aspect should be {expected}, got {}",
                archetype.id,
                archetype.portrait_aspect
            );
        }
    }

    #[test]
    fn history_jsonl_round_trips_records() {
        let path = std::env::temp_dir().join(format!(
            "archetypes-standard-mecha-history-{}-{}.jsonl",
            std::process::id(),
            now_millis()
        ));
        let first = ChatRecord {
            timestamp: 1,
            archetype: "architect".to_owned(),
            role: "witness".to_owned(),
            content: "Build the bridge.".to_owned(),
            image: None,
        };
        let second = ChatRecord {
            timestamp: 2,
            archetype: "architect".to_owned(),
            role: "architect".to_owned(),
            content: "Name the load-bearing span.".to_owned(),
            image: None,
        };
        append_history_record_to(&path, &first).unwrap();
        append_history_record_to(&path, &second).unwrap();
        assert_eq!(read_history_from(&path).unwrap(), vec![first, second]);
    }

    #[test]
    fn history_jsonl_round_trips_image_metadata() {
        let path = std::env::temp_dir().join(format!(
            "archetypes-standard-mecha-image-history-{}-{}.jsonl",
            std::process::id(),
            now_millis()
        ));
        let record = ChatRecord {
            timestamp: 3,
            archetype: "oracle".to_owned(),
            role: "oracle".to_owned(),
            content: "The pattern arrives as an image.".to_owned(),
            image: Some(ChatImage {
                status: "complete".to_owned(),
                keywords: vec!["pattern".to_owned(), "image".to_owned()],
                prompt: "oracle prompt".to_owned(),
                art_style: "oracle style".to_owned(),
                png_path: Some(r"C:\chronos\outputs\oracle.png".to_owned()),
                asset_path: Some("standard_mecha/renders/oracle-proof.png".to_owned()),
                artifact_id: Some("artifact-1".to_owned()),
                proof_receipt_id: Some("receipt-1".to_owned()),
                detail: "verified".to_owned(),
            }),
        };
        append_history_record_to(&path, &record).unwrap();
        assert_eq!(read_history_from(&path).unwrap(), vec![record]);
    }

    #[test]
    fn keyword_extraction_is_local_deduped_and_bounded() {
        let keywords = extract_keywords(
            "When the Architect sees the bridge, the bridge should remember the load path.",
        );
        assert_eq!(
            keywords,
            vec!["architect", "sees", "bridge", "remember", "load", "path"]
        );
    }

    #[test]
    fn editor_edits_at_character_cursor() {
        let mut session = StandardMechaSession::default();
        session.draft = "helloworld".to_owned();
        session.cursor = 5;
        insert_at_cursor(&mut session, " ");
        assert_eq!(session.draft, "hello world");
        backspace_at_cursor(&mut session);
        assert_eq!(session.draft, "helloworld");
        delete_at_cursor(&mut session);
        assert_eq!(session.draft, "helloorld");
        assert_eq!(draft_with_caret(&session), "> hello|orld");
    }

    #[test]
    fn player_facing_image_lines_omit_artifact_and_proof_jargon() {
        let image = ChatImage {
            status: "complete".to_owned(),
            keywords: vec!["bridge".to_owned(), "span".to_owned()],
            prompt: "prompt".to_owned(),
            art_style: "style".to_owned(),
            png_path: Some(r"C:\tmp\x.png".to_owned()),
            asset_path: Some("standard_mecha/renders/x.png".to_owned()),
            artifact_id: Some("artifact-secret".to_owned()),
            proof_receipt_id: Some("proof-secret".to_owned()),
            detail: "verified".to_owned(),
        };
        let history_line = image_history_line(&image);
        let status_line = latest_render_status(
            &[ChatRecord {
                timestamp: 1,
                archetype: "architect".to_owned(),
                role: "architect".to_owned(),
                content: "ok".to_owned(),
                image: Some(image),
            }],
            false,
            "",
            0.0,
        );
        assert!(history_line.contains("keywords: bridge, span"));
        assert!(!history_line.contains("artifact"));
        assert!(!history_line.contains("proof"));
        assert!(!status_line.contains("artifact-secret"));
        assert!(!status_line.contains("proof-secret"));
        assert!(status_line.contains("Complete. Keywords: bridge, span"));
    }

    #[test]
    fn waiting_copy_uses_live_phase_instead_of_static_jargon() {
        let rendered = render_history(&[], true, "Chronos is painting the response...");
        assert!(rendered.contains("Chronos is painting the response..."));
        assert!(!rendered.contains("artifact:"));
        assert_eq!(
            latest_render_status(&[], true, "Ollama is forming Architect's reply...", 12.4),
            "WORKING (12s) — Ollama is forming Architect's reply..."
        );
        assert!(waiting_status_line("Chronos/Comfy is painting the response image...", 65.0)
            .contains("WORKING (65s)"));
    }

    #[test]
    fn transcript_fingerprint_detects_new_turn_not_idle_zeros() {
        let mut fingerprint = ChatTranscriptFingerprint::default();
        let history = vec![
            ChatRecord {
                timestamp: 1,
                archetype: "sentinel".to_owned(),
                role: "witness".to_owned(),
                content: "one".to_owned(),
                image: None,
            },
            ChatRecord {
                timestamp: 2,
                archetype: "sentinel".to_owned(),
                role: "sentinel".to_owned(),
                content: "reply".to_owned(),
                image: None,
            },
        ];
        let last = history.last().map(|r| r.timestamp).unwrap_or(0);
        let changed = fingerprint.history_len != history.len()
            || fingerprint.last_timestamp != last
            || fingerprint.waiting
            || !fingerprint.phase.is_empty();
        assert!(changed, "first population must rebuild");
        fingerprint.history_len = history.len();
        fingerprint.last_timestamp = last;
        fingerprint.waiting = false;
        fingerprint.phase.clear();
        let unchanged = fingerprint.history_len != history.len()
            || fingerprint.last_timestamp != last
            || fingerprint.waiting
            || !fingerprint.phase.is_empty();
        assert!(!unchanged, "idle completed turn must not force rebuild");
        let history2 = {
            let mut next = history.clone();
            next.push(ChatRecord {
                timestamp: 3,
                archetype: "sentinel".to_owned(),
                role: "witness".to_owned(),
                content: "two".to_owned(),
                image: None,
            });
            next
        };
        let last2 = history2.last().map(|r| r.timestamp).unwrap_or(0);
        let turn2 = fingerprint.history_len != history2.len()
            || fingerprint.last_timestamp != last2;
        assert!(turn2, "second witness turn must rebuild transcript");
    }

    fn workspace_assets_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
    }
}
