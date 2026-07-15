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
    input::keyboard::Key,
    prelude::*,
    render::view::window::screenshot::{save_to_disk, Screenshot},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    chamber::{boot::MainMenuUi, ChamberState},
    modes::{
        game_mode::GameMode,
        oracle_riddle::{OracleState, TriggerOracleRiddle},
    },
    services::{
        ledger::append_to_ledger,
        llm::{ollama_chat, ollama_model},
        paths::app_data_root,
    },
    theme::Archetype,
};

const SOURCE_CANON: &str = "C:\\mecha\\aura-mechanician\\frontend\\src";
const SELECTOR_CANVAS_WIDTH: f32 = 2816.0;
const SELECTOR_CANVAS_HEIGHT: f32 = 1536.0;
const SWITCHING_SECS: f32 = 0.85;
const MAX_DRAFT_CHARS: usize = 1200;
const HISTORY_RENDER_LIMIT: usize = 10;

pub struct StandardMechaPlugin;

impl Plugin for StandardMechaPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<StandardMechaState>()
            .init_resource::<StandardMechaSession>()
            .init_resource::<ChatBridge>()
            .add_systems(
                Update,
                check_trigger.run_if(in_state(StandardMechaState::Inactive)),
            )
            .add_systems(OnEnter(StandardMechaState::Selector), spawn_selector_ui)
            .add_systems(
                Update,
                (
                    update_selector_nodes,
                    activate_selector_node,
                    render_selector_details,
                )
                    .chain()
                    .run_if(in_state(StandardMechaState::Selector)),
            )
            .add_systems(OnExit(StandardMechaState::Selector), despawn_standard_mecha_ui)
            .add_systems(OnEnter(StandardMechaState::Switching), spawn_switching_ui)
            .add_systems(
                Update,
                advance_switching.run_if(in_state(StandardMechaState::Switching)),
            )
            .add_systems(OnExit(StandardMechaState::Switching), despawn_standard_mecha_ui)
            .add_systems(OnEnter(StandardMechaState::Chat), enter_chat)
            .add_systems(
                Update,
                (
                    handle_chat_keyboard,
                    activate_chat_switcher,
                    poll_chat_response,
                    render_chat_ui,
                )
                    .chain()
                    .run_if(in_state(StandardMechaState::Chat)),
            )
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
    pub selector_x: f32,
    pub selector_y: f32,
    pub selector_color: Rgb,
    pub css_primary: Rgb,
    pub css_secondary: Rgb,
    pub css_bg_void: Rgb,
    pub css_bg_primary: Rgb,
    pub css_bg_elevated: Rgb,
    pub portrait_path: &'static str,
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
        selector_color: Rgb::new(0xdc, 0x26, 0x26),
        css_primary: Rgb::new(0xdc, 0x26, 0x26),
        css_secondary: Rgb::new(0x99, 0x1b, 0x1b),
        css_bg_void: Rgb::new(0x0f, 0x05, 0x05),
        css_bg_primary: Rgb::new(0x1a, 0x0a, 0x0a),
        css_bg_elevated: Rgb::new(0x2d, 0x15, 0x15),
        portrait_path: "mecha/sentinel.png",
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
        selector_color: Rgb::new(0x3b, 0x82, 0xf6),
        css_primary: Rgb::new(0x3b, 0x82, 0xf6),
        css_secondary: Rgb::new(0x60, 0xa5, 0xfa),
        css_bg_void: Rgb::new(0x0a, 0x0e, 0x14),
        css_bg_primary: Rgb::new(0x11, 0x16, 0x21),
        css_bg_elevated: Rgb::new(0x1a, 0x23, 0x32),
        portrait_path: "mecha/architect.png",
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
        selector_color: Rgb::new(0x8b, 0x5c, 0xf6),
        css_primary: Rgb::new(0x05, 0x96, 0x69),
        css_secondary: Rgb::new(0x10, 0xb9, 0x81),
        css_bg_void: Rgb::new(0x05, 0x12, 0x0a),
        css_bg_primary: Rgb::new(0x0a, 0x18, 0x10),
        css_bg_elevated: Rgb::new(0x15, 0x28, 0x22),
        portrait_path: "mecha/mentor.png",
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
        selector_color: Rgb::new(0xec, 0x48, 0x99),
        css_primary: Rgb::new(0xec, 0x48, 0x99),
        css_secondary: Rgb::new(0xf4, 0x72, 0xb6),
        css_bg_void: Rgb::new(0x0f, 0x05, 0x12),
        css_bg_primary: Rgb::new(0x1a, 0x0f, 0x1f),
        css_bg_elevated: Rgb::new(0x2d, 0x1a, 0x33),
        portrait_path: "mecha/empath.png",
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
        selector_color: Rgb::new(0x10, 0xb9, 0x81),
        css_primary: Rgb::new(0x8b, 0x5c, 0xf6),
        css_secondary: Rgb::new(0xa7, 0x8b, 0xfa),
        css_bg_void: Rgb::new(0x0a, 0x05, 0x0f),
        css_bg_primary: Rgb::new(0x14, 0x0a, 0x1f),
        css_bg_elevated: Rgb::new(0x1f, 0x13, 0x33),
        portrait_path: "mecha/oracle.png",
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
        selector_color: Rgb::new(0xf5, 0x9e, 0x0b),
        css_primary: Rgb::new(0xf5, 0x9e, 0x0b),
        css_secondary: Rgb::new(0xd9, 0x77, 0x06),
        css_bg_void: Rgb::new(0x0f, 0x08, 0x05),
        css_bg_primary: Rgb::new(0x1a, 0x11, 0x08),
        css_bg_elevated: Rgb::new(0x2d, 0x1f, 0x10),
        portrait_path: "mecha/explorer.png",
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
        selector_color: Rgb::new(0xa8, 0x55, 0xf7),
        css_primary: Rgb::new(0x10, 0xb9, 0x81),
        css_secondary: Rgb::new(0x34, 0xd3, 0x99),
        css_bg_void: Rgb::new(0x05, 0x0f, 0x0a),
        css_bg_primary: Rgb::new(0x0a, 0x1a, 0x10),
        css_bg_elevated: Rgb::new(0x13, 0x29, 0x1d),
        portrait_path: "mecha/jester.png",
        icon_path: "mecha/jester-icon.png",
        solid: "tetrahedron",
    },
];

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
    receiver: Mutex<Option<mpsc::Receiver<Result<String, String>>>>,
    waiting: bool,
}

impl Default for ChatBridge {
    fn default() -> Self {
        Self {
            receiver: Mutex::new(None),
            waiting: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChatRecord {
    timestamp: u64,
    archetype: String,
    role: String,
    content: String,
}

#[derive(Component)]
struct StandardMechaUi;

#[derive(Component)]
struct SelectorNodeButton {
    index: usize,
}

#[derive(Component)]
struct SelectorDetailsText;

#[derive(Component)]
struct SwitchingText;

#[derive(Component)]
struct ChatTranscriptText;

#[derive(Component)]
struct ChatInputText;

#[derive(Component)]
struct ChatStatusText;

#[derive(Component)]
struct ChatTitleText;

#[derive(Component)]
struct ChatSubtitleText;

#[derive(Component)]
struct ChatElementText;

#[derive(Component)]
struct ChatPortrait;

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

fn spawn_selector_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<StandardMechaSession>,
) {
    session.hovered_index = None;
    session.status = "Select one consciousness. Ctrl+Space returns here from chat.".to_owned();
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
            GlobalZIndex(920),
            StandardMechaUi,
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(asset_server.load("mecha/uxbacklayer.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(-5.0),
                    top: Val::Percent(-5.0),
                    width: Val::Percent(110.0),
                    height: Val::Percent(110.0),
                    ..default()
                },
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.42)),
            ));
            root.spawn((
                ImageNode::new(asset_server.load("mecha/logo.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(28.0),
                    top: Val::Px(24.0),
                    width: Val::Px(74.0),
                    height: Val::Px(74.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("AURA CONSCIOUSNESS SELECTOR"),
                TextFont {
                    font_size: 25.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.93, 1.0)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(118.0),
                    top: Val::Px(36.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("QSIC ONLINE"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.06, 0.73, 0.51)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent((150.0 / SELECTOR_CANVAS_WIDTH) * 100.0),
                    top: Val::Percent((150.0 / SELECTOR_CANVAS_HEIGHT) * 100.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new(selector_detail_text(session.active())),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.83, 0.86, 0.92)),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(34.0),
                    bottom: Val::Px(34.0),
                    width: Val::Px(380.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.04, 0.05, 0.80)),
                BorderColor::all(Color::srgba(0.45, 0.65, 0.9, 0.40)),
                SelectorDetailsText,
            ));

            for (index, archetype) in MECHA_ARCHETYPES.iter().copied().enumerate() {
                root.spawn((
                    Button,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(archetype.selector_left_percent()),
                        top: Val::Percent(archetype.selector_top_percent()),
                        width: Val::Px(146.0),
                        height: Val::Px(132.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(9.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(archetype.css_bg_primary.alpha(0.80)),
                    BorderColor::all(archetype.selector_color.alpha(0.72)),
                    SelectorNodeButton { index },
                ))
                .with_children(|node| {
                    node.spawn((
                        ImageNode::new(asset_server.load(archetype.icon_path)),
                        Node {
                            width: Val::Px(62.0),
                            height: Val::Px(62.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(archetype.selector_color.alpha(0.85)),
                    ));
                    node.spawn((
                        Text::new(archetype.display_name.to_uppercase()),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    node.spawn((
                        Text::new(archetype.selector_role),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(archetype.selector_color.color()),
                    ));
                });
            }
        });
}

fn selector_detail_text(archetype: MechaArchetype) -> String {
    let color_note = if archetype.selector_color_matches_css() {
        "selector and CSS theme colors agree"
    } else {
        "selector and CSS theme colors intentionally differ"
    };
    format!(
        "{}\n{}\nElement: {}\nRole: {}\nSolid: {}\n{}\nSource: {}",
        archetype.display_name.to_uppercase(),
        archetype.subtitle,
        archetype.element,
        archetype.selector_role,
        archetype.solid,
        color_note,
        SOURCE_CANON
    )
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
        let archetype = MECHA_ARCHETYPES[button.index];
        match *interaction {
            Interaction::Pressed => {
                session.hovered_index = Some(button.index);
                *background = BackgroundColor(archetype.selector_color.alpha(0.42));
                *border = BorderColor::all(Color::WHITE);
            }
            Interaction::Hovered => {
                session.hovered_index = Some(button.index);
                *background = BackgroundColor(archetype.selector_color.alpha(0.28));
                *border = BorderColor::all(archetype.selector_color.color());
            }
            Interaction::None => {
                if session.hovered_index == Some(button.index) {
                    session.hovered_index = None;
                }
                *background = BackgroundColor(archetype.css_bg_primary.alpha(0.80));
                *border = BorderColor::all(archetype.selector_color.alpha(0.72));
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

fn render_selector_details(
    session: Res<StandardMechaSession>,
    mut details: Query<&mut Text, With<SelectorDetailsText>>,
) {
    if !session.is_changed() {
        return;
    }
    let index = session.hovered_index.unwrap_or(session.active_index);
    if let Ok(mut text) = details.single_mut() {
        text.0 = selector_detail_text(MECHA_ARCHETYPES[index]);
    }
}

fn spawn_switching_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut session: ResMut<StandardMechaSession>,
) {
    session.switch_elapsed = 0.0;
    let archetype = session.active();
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
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(archetype.css_bg_void.alpha(0.94)),
            GlobalZIndex(930),
            StandardMechaUi,
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(asset_server.load(archetype.icon_path)),
                Node {
                    width: Val::Px(132.0),
                    height: Val::Px(132.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(archetype.selector_color.color()),
            ));
            root.spawn((
                Text::new(format!("MANIFESTING {}", archetype.display_name.to_uppercase())),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(archetype.selector_color.color()),
                SwitchingText,
            ));
            root.spawn((
                Text::new(format!("{} | {}", archetype.subtitle, archetype.element)),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.81, 0.88)),
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
) {
    let archetype = session.active();
    session.history = read_history(archetype.id).unwrap_or_else(|error| {
        session.status = format!("History unavailable for {}: {error}", archetype.display_name);
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
    let archetype = session.active();
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
            BackgroundColor(archetype.css_bg_void.color()),
            GlobalZIndex(925),
            StandardMechaUi,
        ))
        .with_children(|root| {
            root.spawn((
                ImageNode::new(asset_server.load("mecha/uxbacklayer.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(archetype.css_bg_void.alpha(0.82)),
            ));
            root.spawn((
                ImageNode::new(asset_server.load("mecha/logo.png")),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    top: Val::Px(20.0),
                    width: Val::Px(58.0),
                    height: Val::Px(58.0),
                    ..default()
                },
            ));
            spawn_chat_switcher(root, asset_server, session);
            spawn_chat_panel(root, session);
            spawn_portrait_panel(root, asset_server, session);
        });
}

fn spawn_chat_switcher(
    root: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    session: &StandardMechaSession,
) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(98.0),
            top: Val::Px(20.0),
            height: Val::Px(58.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
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
                    width: Val::Px(if active { 144.0 } else { 48.0 }),
                    height: Val::Px(48.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(7.0),
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(if active {
                    archetype.selector_color.alpha(0.32)
                } else {
                    archetype.css_bg_primary.alpha(0.72)
                }),
                BorderColor::all(if active {
                    archetype.selector_color.color()
                } else {
                    archetype.selector_color.alpha(0.36)
                }),
                ChatSwitchButton { index },
            ))
            .with_children(|button| {
                button.spawn((
                    ImageNode::new(asset_server.load(archetype.icon_path)),
                    Node {
                        width: Val::Px(34.0),
                        height: Val::Px(34.0),
                        ..default()
                    },
                ));
                if active {
                    button.spawn((
                        Text::new(archetype.display_name.to_uppercase()),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                }
            });
        }
    });
}

fn spawn_chat_panel(root: &mut ChildSpawnerCommands, session: &StandardMechaSession) {
    let archetype = session.active();
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(30.0),
            top: Val::Px(96.0),
            width: Val::Percent(57.0),
            height: Val::Percent(82.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(1.0)),
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(archetype.css_bg_primary.alpha(0.88)),
        BorderColor::all(archetype.selector_color.alpha(0.58)),
    ))
    .with_children(|panel| {
        panel.spawn((
            Text::new(format!("{} CHANNEL", archetype.display_name.to_uppercase())),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(archetype.selector_color.color()),
            ChatTitleText,
        ));
        panel.spawn((
            Text::new(render_history(&session.history, false)),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.88, 0.92)),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(68.0),
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.34)),
            BorderColor::all(archetype.selector_color.alpha(0.28)),
            ChatTranscriptText,
        ));
        panel.spawn((
            Text::new(draft_with_caret(session)),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(58.0),
                padding: UiRect::all(Val::Px(14.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.46)),
            BorderColor::all(archetype.selector_color.alpha(0.64)),
            ChatInputText,
        ));
        panel.spawn((
            Text::new(session.status.clone()),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.68, 0.72, 0.80)),
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
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(34.0),
            top: Val::Px(96.0),
            width: Val::Percent(33.0),
            height: Val::Percent(82.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(1.0)),
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(archetype.css_bg_elevated.alpha(0.86)),
        BorderColor::all(archetype.selector_color.alpha(0.70)),
    ))
    .with_children(|panel| {
        panel.spawn((
            ImageNode::new(asset_server.load(archetype.portrait_path)),
            Node {
                width: Val::Percent(86.0),
                height: Val::Percent(70.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(archetype.selector_color.color()),
            ChatPortrait,
        ));
        panel.spawn((
            Text::new(archetype.display_name.to_uppercase()),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(archetype.selector_color.color()),
            ChatTitleText,
        ));
        panel.spawn((
            Text::new(archetype.subtitle),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.88, 0.92)),
            ChatSubtitleText,
        ));
        panel.spawn((
            Text::new(format!("Element: {}", archetype.element)),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(archetype.css_secondary.color()),
            ChatElementText,
        ));
        panel.spawn((
            Text::new("LLM: local Ollama route\nGPU/VRAM/RAM/CPU: unavailable in native monitor"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.62, 0.66, 0.74)),
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
                session.status = "A response is still forming; wait before switching archetypes.".to_owned();
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
            session.status = "A response is still forming; selector return is blocked until it lands.".to_owned();
        } else {
            next_state.set(StandardMechaState::Selector);
        }
        return;
    }

    if keycodes.just_pressed(KeyCode::Escape) {
        if bridge.waiting {
            session.status = "A response is still forming; selector return is blocked until it lands.".to_owned();
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
    };
    if let Err(error) = append_history_record(archetype.id, &user_record) {
        session.status = format!("Chat blocked: history could not be written ({error}).");
        return;
    }
    session.history.push(user_record);
    let _ = append_to_ledger(
        GameMode::Standard,
        "mecha_chat_user",
        json!({ "archetype": archetype.id, "chars": content.chars().count() }),
    );

    session.draft.clear();
    session.cursor = 0;
    session.status = format!(
        "Awaiting {} through local Ollama ({})...",
        archetype.display_name,
        ollama_model()
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
    thread::spawn(move || {
        let _ = sender.send(ollama_chat(&model, &system, &prompt));
    });
}

fn poll_chat_response(mut session: ResMut<StandardMechaSession>, mut bridge: ResMut<ChatBridge>) {
    if !bridge.waiting {
        return;
    }
    let result = {
        let guard = bridge
            .receiver
            .lock()
            .expect("standard chat receiver lock");
        guard.as_ref().and_then(|receiver| receiver.try_recv().ok())
    };
    let Some(result) = result else { return };
    bridge.waiting = false;
    let archetype = session.active();
    match result {
        Ok(content) => {
            let record = ChatRecord {
                timestamp: now_millis(),
                archetype: archetype.id.to_owned(),
                role: archetype.id.to_owned(),
                content: content.clone(),
            };
            if let Err(error) = append_history_record(archetype.id, &record) {
                session.status = format!(
                    "{} answered, but history persistence failed: {error}",
                    archetype.display_name
                );
            } else {
                session.status = format!("{} answered. History persisted.", archetype.display_name);
            }
            session.history.push(record);
            let _ = append_to_ledger(
                GameMode::Standard,
                "mecha_chat_assistant",
                json!({ "archetype": archetype.id, "chars": content.chars().count() }),
            );
        }
        Err(error) => {
            let visible = format!("LOCAL LLM FAILURE: {error}");
            let record = ChatRecord {
                timestamp: now_millis(),
                archetype: archetype.id.to_owned(),
                role: "system".to_owned(),
                content: visible.clone(),
            };
            let _ = append_history_record(archetype.id, &record);
            session.history.push(record);
            session.status = visible;
            let _ = append_to_ledger(
                GameMode::Standard,
                "mecha_chat_failed",
                json!({ "archetype": archetype.id, "error": error }),
            );
        }
    }
}

fn render_chat_ui(
    session: Res<StandardMechaSession>,
    bridge: Res<ChatBridge>,
    asset_server: Res<AssetServer>,
    mut transcript: Query<&mut Text, With<ChatTranscriptText>>,
    mut input: Query<&mut Text, (With<ChatInputText>, Without<ChatTranscriptText>)>,
    mut status: Query<
        &mut Text,
        (
            With<ChatStatusText>,
            Without<ChatTranscriptText>,
            Without<ChatInputText>,
        ),
    >,
    mut portraits: Query<&mut ImageNode, With<ChatPortrait>>,
) {
    if !session.is_changed() && !bridge.is_changed() {
        return;
    }
    let archetype = session.active();
    if let Ok(mut text) = transcript.single_mut() {
        text.0 = render_history(&session.history, bridge.waiting);
    }
    if let Ok(mut text) = input.single_mut() {
        text.0 = draft_with_caret(&session);
    }
    if let Ok(mut text) = status.single_mut() {
        text.0 = session.status.clone();
    }
    if let Ok(mut portrait) = portraits.single_mut() {
        portrait.image = asset_server.load(archetype.portrait_path);
    }
}

fn render_history(history: &[ChatRecord], waiting: bool) -> String {
    if history.is_empty() && !waiting {
        return "No prior messages for this archetype yet. Type and press Enter to send through local Ollama.".to_owned();
    }
    let start = history.len().saturating_sub(HISTORY_RENDER_LIMIT);
    let mut lines: Vec<String> = history[start..]
        .iter()
        .map(|record| {
            let label = match record.role.as_str() {
                "witness" => "Witness",
                "system" => "System",
                other => MECHA_ARCHETYPES
                    .iter()
                    .find(|archetype| archetype.id == other)
                    .map(|archetype| archetype.display_name)
                    .unwrap_or(other),
            };
            format!("{label}: {}", compact(&record.content, 700))
        })
        .collect();
    if waiting {
        lines.push("System: local Ollama response is forming...".to_owned());
    }
    lines.join("\n\n")
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

fn despawn_standard_mecha_ui(
    mut commands: Commands,
    query: Query<Entity, With<StandardMechaUi>>,
) {
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
        assert_eq!(
            mismatched,
            HashSet::from(["mentor", "oracle", "jester"])
        );
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
        };
        let second = ChatRecord {
            timestamp: 2,
            archetype: "architect".to_owned(),
            role: "architect".to_owned(),
            content: "Name the load-bearing span.".to_owned(),
        };
        append_history_record_to(&path, &first).unwrap();
        append_history_record_to(&path, &second).unwrap();
        assert_eq!(read_history_from(&path).unwrap(), vec![first, second]);
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

    fn workspace_assets_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
    }
}
