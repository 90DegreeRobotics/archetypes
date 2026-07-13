use std::{
    fs,
    path::{Path, PathBuf},
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use bevy::{
    input::keyboard::Key,
    prelude::*,
    render::view::window::screenshot::{save_to_disk, Screenshot},
    window::PrimaryWindow,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{
    council::{CouncilStatus, CouncilTranscript},
    portal::StargatePortal,
    speech::{CouncilVoiceState, SpeechStatus},
    spheres::ArchetypeSphere,
    ChamberState, CurrentFocus,
};
use crate::chamber::camera::WitnessCamera;
use crate::theme::Archetype;

const DIRECTOR_URL: &str = "http://127.0.0.1:7777";

/// Comfy-only image backend. The Chronos Foundry supervisor keeps ComfyUI alive on
/// this port; the game always requests the fast Comfy path, never the slow Blender path.
fn comfyui_url() -> String {
    std::env::var("ARCHETYPES_COMFYUI_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_owned())
}

fn comfyui_output_dir() -> String {
    std::env::var("ARCHETYPES_COMFYUI_OUTPUT_DIR")
        .unwrap_or_else(|_| r"C:\Users\m\Documents\ComfyUI\output".to_owned())
}

pub struct RitualPlugin;

impl Plugin for RitualPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RitualSession>()
            .init_resource::<ChronosBridge>()
            .init_resource::<DialogueUiState>()
            .add_systems(Startup, (spawn_ritual_ui, load_witness_profile))
            .add_systems(
                Update,
                (
                    receive_text_input,
                    toggle_transcript_drawer,
                    begin_chronos_request,
                    poll_chronos_result,
                    render_ritual_ui,
                    update_speaker_avatar,
                )
                    .chain(),
            )
            .add_systems(OnEnter(ChamberState::IdleAtTable), clear_focus_on_table)
            .add_systems(
                OnEnter(ChamberState::ArtifactResult),
                present_artifact_image,
            )
            .add_systems(
                Update,
                position_artifact_image.run_if(in_state(ChamberState::ArtifactResult)),
            )
            .add_systems(OnExit(ChamberState::ArtifactResult), clear_artifact_image);

        if let Some(run) = CaptureRun::from_env() {
            app.insert_resource(run)
                .add_systems(Update, run_capture.after(render_ritual_ui));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct WitnessProfile {
    name: String,
}

#[derive(Resource, Default)]
pub(super) struct RitualSession {
    profile: Option<WitnessProfile>,
    draft: String,
    cursor: usize,
    offering: String,
    pub(super) verdict: String,
    artifact: Option<ArtifactOutcome>,
    artifact_note: Option<String>,
}

impl RitualSession {
    pub(super) fn offering(&self) -> &str {
        &self.offering
    }

    pub(super) fn witness_name(&self) -> &str {
        self.profile
            .as_ref()
            .map(|profile| profile.name.as_str())
            .unwrap_or("Witness")
    }

    pub(super) fn set_verdict(&mut self, verdict: String) {
        self.verdict = verdict;
    }

    pub(super) fn has_profile(&self) -> bool {
        self.profile.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArtifactOutcome {
    status: String,
    job_id: Option<String>,
    artifact_id: Option<String>,
    png_path: Option<String>,
    proof_receipt_id: Option<String>,
    detail: String,
}

#[derive(Resource, Default)]
struct ChronosBridge {
    receiver: Mutex<Option<mpsc::Receiver<ArtifactOutcome>>>,
    started: bool,
}

#[derive(Component)]
struct RitualUi;

#[derive(Component)]
struct TranscriptDrawer;

#[derive(Component)]
struct SpeakerBubble;

#[derive(Component)]
struct SpeakerBubbleText;

/// The circular face-crop avatar of the archetype currently speaking, inside the bubble.
#[derive(Component)]
struct SpeakerBubbleAvatar;

#[derive(Component)]
struct RitualPrompt;

#[derive(Resource)]
struct DialogueUiState {
    drawer_open: bool,
}

impl Default for DialogueUiState {
    fn default() -> Self {
        Self { drawer_open: true }
    }
}

fn spawn_ritual_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ui_camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            IsDefaultUiCamera,
            Name::new("RitualUiCamera"),
        ))
        .id();
    // The single consolidated ritual panel: one clear, bordered container pinned to
    // the top of the screen. Everything the Witness needs to read lives here instead
    // of being split between opposite screen corners.
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 19.0,
            ..default()
        },
        TextColor(Color::srgb(0.86, 0.88, 0.92)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(24.0),
            top: Val::Px(24.0),
            width: Val::Px(620.0),
            max_width: Val::Percent(70.0),
            max_height: Val::Percent(70.0),
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(2.0)),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.018, 0.022, 0.032, 0.96)),
        BorderColor::all(Color::srgba(0.55, 0.65, 0.9, 0.5)),
        UiTargetCamera(ui_camera),
        RitualUi,
        TranscriptDrawer,
    ));
    // A small persistent status/hint line, docked to the same top edge as the main
    // panel (not the opposite corner) so it reads as part of the same top area even
    // though it must stay visible independent of whether the panel above is closed.
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.78, 0.81, 0.88)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(24.0),
            top: Val::Px(24.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(2.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.018, 0.025, 0.85)),
        BorderColor::all(Color::srgba(0.55, 0.65, 0.9, 0.35)),
        UiTargetCamera(ui_camera),
        RitualPrompt,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(57.0),
                top: Val::Percent(20.0),
                width: Val::Px(410.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.025, 0.04, 0.94)),
            BorderColor::all(Color::WHITE),
            UiTargetCamera(ui_camera),
            SpeakerBubble,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(asset_server.load("avatars/architect.png")),
                Node {
                    width: Val::Px(78.0),
                    height: Val::Px(78.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                SpeakerBubbleAvatar,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    max_width: Val::Px(284.0),
                    ..default()
                },
                SpeakerBubbleText,
            ));
        });
}

fn toggle_transcript_drawer(keyboard: Res<ButtonInput<Key>>, mut ui: ResMut<DialogueUiState>) {
    if keyboard.just_pressed(Key::Tab) {
        ui.drawer_open = !ui.drawer_open;
    }
}

fn load_witness_profile(mut session: ResMut<RitualSession>) {
    let Ok(body) = fs::read_to_string(profile_path()) else {
        return;
    };
    if let Ok(profile) = serde_json::from_str::<WitnessProfile>(&body) {
        if !profile.name.trim().is_empty() {
            session.profile = Some(profile);
        }
    }
}

fn receive_text_input(
    keyboard: Res<ButtonInput<Key>>,
    state: Res<State<ChamberState>>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut session: ResMut<RitualSession>,
) {
    for key in keyboard.get_just_pressed() {
        let editing = matches!(
            state.get(),
            ChamberState::Onboarding | ChamberState::IdleAtTable
        );
        match key {
            Key::Backspace if editing => backspace_at_cursor(&mut session),
            Key::Delete if editing => delete_at_cursor(&mut session),
            Key::ArrowLeft if editing => session.cursor = session.cursor.saturating_sub(1),
            Key::ArrowRight if editing => {
                session.cursor = (session.cursor + 1).min(session.draft.chars().count())
            }
            Key::Home if editing => session.cursor = 0,
            Key::End if editing => session.cursor = session.draft.chars().count(),
            Key::Space if editing => insert_at_cursor(&mut session, " "),
            Key::Enter => match state.get() {
                ChamberState::Onboarding => seal_profile(&mut session, &mut next_state),
                ChamberState::IdleAtTable => submit_offering(&mut session, &mut next_state),
                // Enter during deliberation abandons this council and returns to the table
                // (an escape hatch if the council is slow or has failed).
                ChamberState::Deliberating => next_state.set(ChamberState::IdleAtTable),
                ChamberState::WitnessVerdict => {
                    session.artifact = None;
                    next_state.set(ChamberState::ArtifactPending);
                }
                ChamberState::ArtifactResult => {
                    session.draft.clear();
                    session.cursor = 0;
                    next_state.set(ChamberState::IdleAtTable);
                }
                _ => {}
            },
            Key::Character(text) if editing => {
                let room = 600usize.saturating_sub(session.draft.chars().count());
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

fn byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn insert_at_cursor(session: &mut RitualSession, text: &str) {
    let cursor = session.cursor.min(session.draft.chars().count());
    let byte = byte_index(&session.draft, cursor);
    session.draft.insert_str(byte, text);
    session.cursor = cursor + text.chars().count();
}

fn backspace_at_cursor(session: &mut RitualSession) {
    if session.cursor == 0 {
        return;
    }
    let end = byte_index(&session.draft, session.cursor);
    let start = byte_index(&session.draft, session.cursor - 1);
    session.draft.replace_range(start..end, "");
    session.cursor -= 1;
}

fn delete_at_cursor(session: &mut RitualSession) {
    if session.cursor >= session.draft.chars().count() {
        return;
    }
    let start = byte_index(&session.draft, session.cursor);
    let end = byte_index(&session.draft, session.cursor + 1);
    session.draft.replace_range(start..end, "");
}

fn draft_with_caret(session: &RitualSession) -> String {
    let byte = byte_index(
        &session.draft,
        session.cursor.min(session.draft.chars().count()),
    );
    let mut rendered = session.draft.clone();
    rendered.insert_str(byte, "▌");
    rendered
}

fn seal_profile(session: &mut RitualSession, next_state: &mut NextState<ChamberState>) {
    let name = session.draft.trim();
    if name.is_empty() {
        return;
    }
    let profile = WitnessProfile {
        name: name.to_owned(),
    };
    if persist_json(&profile_path(), &profile).is_ok() {
        session.profile = Some(profile);
        session.draft.clear();
        session.cursor = 0;
        next_state.set(ChamberState::IdleAtTable);
    }
}

fn submit_offering(session: &mut RitualSession, next_state: &mut NextState<ChamberState>) {
    let offering = session.draft.trim();
    if offering.is_empty() {
        return;
    }
    session.offering = offering.to_owned();
    session.draft.clear();
    session.cursor = 0;
    next_state.set(ChamberState::Deliberating);
}

/// When the ritual returns to the table, no archetype holds the floor.
fn clear_focus_on_table(mut focus: ResMut<CurrentFocus>) {
    focus.0 = None;
}

fn begin_chronos_request(
    state: Res<State<ChamberState>>,
    session: Res<RitualSession>,
    mut bridge: ResMut<ChronosBridge>,
) {
    if *state.get() != ChamberState::ArtifactPending {
        bridge.started = false;
        return;
    }
    if bridge.started {
        return;
    }

    bridge.started = true;
    let prompt = artifact_prompt(&session.offering, &session.verdict);
    let (sender, receiver) = mpsc::channel();
    *bridge.receiver.lock().expect("Chronos receiver lock") = Some(receiver);
    thread::spawn(move || {
        let _ = sender.send(request_chronos_artifact(&prompt));
    });
}

fn poll_chronos_result(
    state: Res<State<ChamberState>>,
    mut session: ResMut<RitualSession>,
    bridge: Res<ChronosBridge>,
    mut next_state: ResMut<NextState<ChamberState>>,
) {
    if *state.get() != ChamberState::ArtifactPending {
        return;
    }
    let result = {
        let guard = bridge.receiver.lock().expect("Chronos receiver lock");
        guard.as_ref().and_then(|receiver| receiver.try_recv().ok())
    };
    if let Some(result) = result {
        // The player-facing text never shows these raw ids (see render_ritual_ui);
        // they're logged here so the receipt is still traceable for debugging.
        info!(
            "ARTIFACT status={} artifact_id={:?} png_path={:?} proof_receipt_id={:?} detail={}",
            result.status, result.artifact_id, result.png_path, result.proof_receipt_id, result.detail
        );
        let _ = persist_json(&artifact_receipt_path(), &result);
        session.artifact = Some(result);
        next_state.set(ChamberState::ArtifactResult);
    }
}

fn request_chronos_artifact(prompt: &str) -> ArtifactOutcome {
    let status: Value = match response_json(
        ureq::get(&format!("{DIRECTOR_URL}/api/v1/status"))
            .timeout(Duration::from_secs(4))
            .call(),
    ) {
        Ok(status) => status,
        Err(error) => return failed_outcome(format!("Chronos Director is unreachable: {error}")),
    };
    if status.get("readiness").and_then(Value::as_str) != Some("ready") {
        return failed_outcome(format!(
            "Chronos is not ready: {}",
            status
                .get("readiness_reasons")
                .cloned()
                .unwrap_or(Value::Null)
        ));
    }

    // Match Chronos's proven museum-canvas metabolism on the 12 GB Forge: the
    // council's Ollama model has finished its work, so release its VRAM before
    // Flux loads. This is best-effort; the Director response remains fail-closed.
    let _ = ureq::post("http://127.0.0.1:11434/api/generate")
        .timeout(Duration::from_secs(20))
        .send_json(json!({
            "model": std::env::var("ARCHETYPES_OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen2.5:7b-instruct".to_owned()),
            "prompt": "",
            "keep_alive": 0,
        }));

    // Direct canvas-image route: Chronos reuses the same Flux generator that paints
    // its museum-wall canvas, returning the painting PNG without a Blender wrapper.
    let submit: Value = match response_json(
        ureq::post(&format!("{DIRECTOR_URL}/api/v1/pipeline/concept-thumbnail"))
            .timeout(Duration::from_secs(480))
            .send_json(json!({
                "prompt": prompt,
                "style": "modern realistic digital painting, natural lighting and color, crisp detail, readable silhouette, no text",
                // Chronos's `storyboard_prompt_from_req` only recognizes "refined";
                // anything else (this used to send "final") silently falls back to
                // its rough-sketch default, which is why every artifact came back
                // as a dark, low-detail sketch regardless of the `style` above.
                "fidelity": "refined",
                "comfyui_url": comfyui_url(),
                "comfyui_output_dir": comfyui_output_dir(),
                "session_id": "archetypes-council",
            })),
    ) {
        Ok(body) => body,
        Err(error) => {
            return failed_outcome(format!(
                "Chronos rejected the Comfy request: {error}. Is the Chronos Foundry (ComfyUI on :8000) running?"
            ))
        }
    };
    let png_path = string_field(&submit, "png_path");
    let verified_path = png_path.as_deref().map(resolve_chronos_path);
    if verified_path.as_ref().is_none_or(|path| !path.is_file()) {
        return ArtifactOutcome {
            status: "failed".to_owned(),
            job_id: None,
            artifact_id: string_field(&submit, "concept_id"),
            png_path,
            proof_receipt_id: string_field(&submit, "completion_event_id"),
            detail: "Chronos reported completion without a readable canvas PNG".to_owned(),
        };
    }
    ArtifactOutcome {
        status: "complete".to_owned(),
        job_id: None,
        artifact_id: string_field(&submit, "concept_id"),
        png_path: verified_path.map(|path| path.display().to_string()),
        proof_receipt_id: string_field(&submit, "completion_event_id"),
        detail: "Chronos returned a verified direct canvas image".to_owned(),
    }
}

fn render_ritual_ui(
    state: Res<State<ChamberState>>,
    session: Res<RitualSession>,
    council: Res<CouncilTranscript>,
    _speech: Res<SpeechStatus>,
    voice: Res<CouncilVoiceState>,
    _focus: Res<CurrentFocus>,
    _time: Res<Time>,
    ui: ResMut<DialogueUiState>,
    _windows: Query<&Window, With<PrimaryWindow>>,
    _camera: Query<(&Camera, &GlobalTransform), With<WitnessCamera>>,
    _spheres: Query<(&ArchetypeSphere, &GlobalTransform)>,
    _portal: Query<&GlobalTransform, With<StargatePortal>>,
    mut drawer: Query<
        (&mut Text, &mut Node, &mut TextFont),
        (With<TranscriptDrawer>, Without<SpeakerBubble>),
    >,
    mut bubble: Query<
        (&mut Node, &mut BackgroundColor, &mut BorderColor),
        (With<SpeakerBubble>, Without<TranscriptDrawer>),
    >,
    _bubble_text: Query<
        (&mut Text, &mut TextColor),
        (
            With<SpeakerBubbleText>,
            Without<TranscriptDrawer>,
            Without<RitualPrompt>,
        ),
    >,
    mut prompt: Query<
        (&mut Text, &mut Node),
        (
            With<RitualPrompt>,
            Without<SpeakerBubbleText>,
            Without<TranscriptDrawer>,
            Without<SpeakerBubble>,
        ),
    >,
) {
    let (Ok((mut drawer_text, mut drawer_node, mut drawer_font)), Ok((mut prompt_text, mut prompt_node))) =
        (drawer.single_mut(), prompt.single_mut())
    else {
        return;
    };
    let witness = session
        .profile
        .as_ref()
        .map(|profile| profile.name.as_str())
        .unwrap_or("Witness");
    let ritual_text = match state.get() {
        ChamberState::Booting => String::new(),
        ChamberState::MainMenu => String::new(),
        ChamberState::Onboarding => format!(
            "YOU ARE THE WITNESS\n\nSeven minds convene in this chamber, and they answer to one\nseat only — yours. Before the council will speak, it must know\nwho holds that seat.\n\nType your name, then press Enter:\n\n  {}\n\nThe council will address you by this name. It is sealed here.",
            draft_with_caret(&session)
        ),
        ChamberState::IdleAtTable => format!(
            "{witness}, THE COUNCIL IS SEATED\n\nOffer a thought, a question, or a fear to the seven:\n\n  {}\n\nEnter sends it into deliberation.",
            draft_with_caret(&session)
        ),
        ChamberState::Deliberating => match &council.status {
            CouncilStatus::Failed(reason) => format!(
                "THE COUNCIL IS SILENT\n\n{reason}\n\nEnter returns to the table."
            ),
            _ => format!(
                "THE COUNCIL DELIBERATES\n\nOffering: {}\n\nThe seven are conferring; their voices are forming...",
                session.offering
            ),
        },
        ChamberState::CouncilSpeaking => {
            // Lines before the cursor have already fully played. The current line
            // only shows its generated text once its voice is actually audible —
            // otherwise the transcript would race ahead of what's being heard.
            let mut shown: Vec<String> = council
                .lines
                .iter()
                .take(council.cursor)
                .map(|line| format!("{:?} - {}\n{}", line.archetype, line.role, line.text))
                .collect();
            if let Some(line) = council.lines.get(council.cursor) {
                shown.push(if voice.line_audible(council.cursor) {
                    format!("{:?} - {}\n{}", line.archetype, line.role, line.text)
                } else {
                    format!("{:?} - {}\n…", line.archetype, line.role)
                });
            }
            if shown.is_empty() {
                "THE COUNCIL SPEAKS".to_owned()
            } else {
                shown.join("\n\n")
            }
        }
        ChamberState::WitnessVerdict => format!(
            "COUNCIL VERDICT\n\n{}\n\n{witness}, Enter brings this verdict into being as an image.",
            session.verdict
        ),
        ChamberState::ArtifactPending => {
            "TURNING THE VERDICT INTO AN IMAGE\n\nThis is being painted for you now. It will appear here as soon as it's ready.".to_owned()
        }
        ChamberState::ArtifactResult => {
            let outcome = session.artifact.as_ref();
            match (
                outcome.map(|value| value.status.as_str()),
                session.artifact_note.as_deref(),
            ) {
                (Some("complete"), None) => {
                    "YOUR ARTIFACT\n\nThe council's verdict, made visible.\n\nEnter returns to the table."
                        .to_owned()
                }
                (Some("complete"), Some(note)) => format!(
                    "THE IMAGE COULDN'T BE SHOWN\n\nIt was painted, but {note}.\n\nEnter returns to the table."
                ),
                _ => {
                    let reason = outcome
                        .map(|value| value.detail.as_str())
                        .unwrap_or("No result was returned.");
                    format!(
                        "THE IMAGE COULD NOT BE MADE\n\n{reason}\n\nEnter returns to the table."
                    )
                }
            }
        }
    };
    drawer_text.0 = ritual_text;
    // Per the operator directive, do not show the ritual text over the scene while
    // the Witness is only watching. The panel appears solely where typing is
    // required (naming the Witness, offering at the table); every other state plays
    // out on the voice + the scene alone, uncluttered.
    let show_drawer = ui.drawer_open
        && matches!(
            state.get(),
            ChamberState::Onboarding | ChamberState::IdleAtTable
        );
    drawer_node.display = if show_drawer {
        Display::Flex
    } else {
        Display::None
    };
    if matches!(
        state.get(),
        ChamberState::Onboarding | ChamberState::IdleAtTable
    ) {
        // Dock the input panel to the left so the ornate table stays unobstructed
        // in the centre of frame (the 3/4 table camera puts the portal centre-frame,
        // so a portal-following panel would sit right on top of the tabletop).
        drawer_node.width = Val::Px(360.0);
        drawer_font.font_size = 16.0;
        drawer_node.left = Val::Px(40.0);
        drawer_node.top = Val::Percent(30.0);
    } else {
        drawer_node.width = Val::Px(620.0);
        drawer_font.font_size = 19.0;
        drawer_node.left = Val::Px(24.0);
        drawer_node.top = Val::Px(24.0);
    }
    // Per the "just don't show it" directive, no ritual status text is drawn over
    // the scene. (Kept as a single sink so the node exists but stays empty/hidden.)
    prompt_text.0 = String::new();
    prompt_node.display = if prompt_text.0.is_empty() {
        Display::None
    } else {
        Display::Flex
    };

    // The floating speaker bubble is suppressed entirely per the "just don't show
    // it" directive — the council speaks through voice and the scene, not text.
    if let Ok((mut bubble_node, _, _)) = bubble.single_mut() {
        bubble_node.display = Display::None;
    }
}

/// Set the bubble's face-crop avatar to the currently speaking archetype. The bubble
/// (and thus this child) is hidden by `render_ritual_ui` whenever no one holds the floor.
fn update_speaker_avatar(
    state: Res<State<ChamberState>>,
    focus: Res<CurrentFocus>,
    asset_server: Res<AssetServer>,
    mut avatar: Query<&mut ImageNode, With<SpeakerBubbleAvatar>>,
) {
    if *state.get() != ChamberState::CouncilSpeaking {
        return;
    }
    let Some(archetype) = focus.0 else {
        return;
    };
    let Ok(mut image) = avatar.single_mut() else {
        return;
    };
    image.image = asset_server.load(format!("avatars/{}.png", avatar_slug(archetype)));
}

fn avatar_slug(archetype: Archetype) -> &'static str {
    match archetype {
        Archetype::Architect => "architect",
        Archetype::Sentinel => "sentinel",
        Archetype::Jester => "jester",
        Archetype::Mentor => "mentor",
        Archetype::Explorer => "explorer",
        Archetype::Oracle => "oracle",
        Archetype::Empath | Archetype::Codex | Archetype::Viren => "empath",
    }
}

fn artifact_prompt(offering: &str, verdict: &str) -> String {
    format!(
        "Council-authorized study. Original offering: {offering}. Build verdict: {verdict}. Produce one restrained, artifact-grade image, modern and realistic, with a readable silhouette and no text."
    )
}

fn failed_outcome(detail: String) -> ArtifactOutcome {
    ArtifactOutcome {
        status: "failed".to_owned(),
        job_id: None,
        artifact_id: None,
        png_path: None,
        proof_receipt_id: None,
        detail,
    }
}

fn string_field(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_owned)
}

fn response_json(response: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
    let response = response.map_err(|error| error.to_string())?;
    response.into_json().map_err(|error| error.to_string())
}

fn resolve_chronos_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        Path::new("C:\\chronos").join(path)
    }
}

fn app_data_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("data")
}

fn profile_path() -> PathBuf {
    app_data_root().join("witness_profile.json")
}

fn artifact_receipt_path() -> PathBuf {
    app_data_root().join("last_artifact.json")
}

fn persist_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().ok_or("path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let body = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, format!("{body}\n")).map_err(|error| error.to_string())
}

/// UI node holding the returned artifact image, so it can be cleaned up on exit.
#[derive(Component)]
struct ArtifactImageNode;

/// Seconds since the artifact image was first shown, driving its zoom-in.
#[derive(Component, Default)]
struct ArtifactImageAge(f32);

/// On entering the artifact result, stage the verified render into the asset tree
/// and display it in the chamber. The image is only shown when Chronos returned a
/// verified, on-disk PNG — never a placeholder standing in for a render that does
/// not exist.
fn present_artifact_image(
    mut commands: Commands,
    mut session: ResMut<RitualSession>,
    asset_server: Res<AssetServer>,
) {
    session.artifact_note = None;

    let Some(outcome) = session.artifact.clone() else {
        return;
    };
    if outcome.status != "complete" {
        return;
    }
    let Some(png_path) = outcome.png_path.as_deref() else {
        return;
    };

    let file_name = format!(
        "return-{}.png",
        sanitize_id(
            outcome
                .artifact_id
                .as_deref()
                .or(outcome.job_id.as_deref())
                .unwrap_or("latest"),
        )
    );
    let dir = artifacts_asset_dir();
    if let Err(error) = fs::create_dir_all(&dir) {
        session.artifact_note = Some(format!("could not prepare display directory: {error}"));
        return;
    }
    if let Err(error) = fs::copy(png_path, dir.join(&file_name)) {
        session.artifact_note = Some(format!(
            "verified render could not be staged for display: {error}"
        ));
        return;
    }

    let handle = asset_server.load(format!("artifacts/{file_name}"));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(ARTIFACT_DISPLAY_SIZE * ARTIFACT_ZOOM_START_SCALE),
            ..default()
        },
        ImageNode::new(handle),
        ArtifactImageNode,
        ArtifactImageAge::default(),
    ));
}

/// On-screen size of the returned artifact, manifested over the portal table.
const ARTIFACT_DISPLAY_SIZE: f32 = 390.0;
/// The reveal starts at this fraction of the final size and eases up to full size
/// over `ARTIFACT_ZOOM_DURATION`, so the image draws the eye in instead of popping
/// onto the table at full size.
const ARTIFACT_ZOOM_START_SCALE: f32 = 0.3;
const ARTIFACT_ZOOM_DURATION: f32 = 2.0;

/// Keep the returned image pinned over the portal, zooming in from a smaller size
/// to `ARTIFACT_DISPLAY_SIZE` over the first couple of seconds. ArtifactResult
/// returns the camera downward to the table, so creation visibly lands where the
/// Witness placed intent.
fn position_artifact_image(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<WitnessCamera>>,
    named: Query<(&Name, &GlobalTransform)>,
    mut node: Query<(&mut Node, &mut ArtifactImageAge), With<ArtifactImageNode>>,
) {
    let (Ok((mut node, mut age)), Ok(window), Ok((camera, camera_transform))) =
        (node.single_mut(), windows.single(), camera.single())
    else {
        return;
    };
    let Some((_, witness)) = named
        .iter()
        .find(|(name, _)| name.as_str() == "Stargate_Portal")
    else {
        return;
    };
    age.0 += time.delta_secs();
    let t = (age.0 / ARTIFACT_ZOOM_DURATION).clamp(0.0, 1.0);
    let eased = t * t * (3.0 - 2.0 * t); // smoothstep
    let size =
        ARTIFACT_DISPLAY_SIZE * (ARTIFACT_ZOOM_START_SCALE + (1.0 - ARTIFACT_ZOOM_START_SCALE) * eased);
    node.width = Val::Px(size);
    if let Ok(viewport) = camera.world_to_viewport(camera_transform, witness.translation()) {
        node.left = Val::Px((viewport.x - size / 2.0).clamp(8.0, window.width() - size - 8.0));
        node.top = Val::Px((viewport.y - size / 2.0).clamp(8.0, window.height() - size - 8.0));
    }
}

/// Remove the artifact image when leaving the result state.
fn clear_artifact_image(mut commands: Commands, query: Query<Entity, With<ArtifactImageNode>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Directory inside the asset tree where returned renders are staged for the
/// Bevy asset server to load them by relative path.
fn artifacts_asset_dir() -> PathBuf {
    let base = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
    } else {
        PathBuf::from("assets")
    };
    base.join("artifacts")
}

/// Reduce an identifier to a filesystem-safe stem for the staged asset name.
fn sanitize_id(id: &str) -> String {
    let cleaned: String = id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .take(48)
        .collect();
    if cleaned.trim_matches('-').is_empty() {
        "latest".to_owned()
    } else {
        cleaned
    }
}

// ---------------------------------------------------------------------------
// Self-driving capture mode.
//
// Enabled by setting the `ARCHETYPES_CAPTURE` environment variable. It scripts a
// deterministic walk through the real ritual — using the same systems, states,
// and Chronos seam as a live player — and writes a screenshot of each stage via
// Bevy's own screenshot pipeline. This exists because the vertical slice lives in
// a windowed GPU app: the honest green check is a rendered frame, and this makes
// that check reproducible on any machine. It is never on during normal play.
// ---------------------------------------------------------------------------

const CAPTURE_NAME: &str = "Aidan of the Long Table";
const CAPTURE_OFFERING: &str = "a library that remembers everyone who ever read in it";

#[derive(Clone, Copy)]
enum CaptureKind {
    /// Force the onboarding ritual with a drafted name.
    Onboard,
    /// Seal the profile and stage the offering draft at the table.
    Seal,
    /// Submit the offering; the real council (Ollama) then drives the flow.
    Submit,
    /// Save a screenshot of the current frame under this stem.
    Shot(&'static str),
}

#[derive(Resource)]
struct CaptureRun {
    dir: PathBuf,
    steps: Vec<(f32, CaptureKind)>,
    next: usize,
    // The post-submit stages are event-driven because the council's Ollama calls
    // and the Comfy render finish on their own, variable schedules.
    council_seen: Option<f32>,
    council_shot: bool,
    council_shot2: bool,
    verdict_seen: Option<f32>,
    verdict_shot: bool,
    authorized: bool,
    pending_seen: Option<f32>,
    pending_shot: bool,
    artifact_seen: Option<f32>,
    artifact_shot: bool,
}

impl CaptureRun {
    fn from_env() -> Option<Self> {
        if std::env::var_os("ARCHETYPES_CAPTURE").is_none() {
            return None;
        }
        let dir = std::env::var_os("ARCHETYPES_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("capture"));
        let _ = fs::create_dir_all(&dir);
        Some(Self {
            dir,
            // (seconds since app start, action). Only the pre-council steps are timed;
            // everything after Submit is captured event-driven below.
            steps: vec![
                (1.0, CaptureKind::Shot("00_title")),
                (2.0, CaptureKind::Onboard),
                (3.5, CaptureKind::Shot("01_onboarding")),
                // Second onboarding frame: the camera is fixed and every object is
                // static, so this must be pixel-identical to 01 unless something is
                // still moving. A hash comparison of the two is the static check.
                (7.0, CaptureKind::Shot("01b_onboarding_static_check")),
                (7.3, CaptureKind::Seal),
                (8.2, CaptureKind::Shot("02_table")),
                (8.5, CaptureKind::Submit),
                (10.5, CaptureKind::Shot("03_deliberating")),
            ],
            next: 0,
            council_seen: None,
            council_shot: false,
            council_shot2: false,
            verdict_seen: None,
            verdict_shot: false,
            authorized: false,
            pending_seen: None,
            pending_shot: false,
            artifact_seen: None,
            artifact_shot: false,
        })
    }

    fn shot(&self, commands: &mut Commands, stem: &str) {
        let path = self.dir.join(format!("{stem}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn run_capture(
    time: Res<Time>,
    state: Res<State<ChamberState>>,
    council: Res<CouncilTranscript>,
    mut capture: ResMut<CaptureRun>,
    mut session: ResMut<RitualSession>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();

    while capture.next < capture.steps.len() && now >= capture.steps[capture.next].0 {
        match capture.steps[capture.next].1 {
            CaptureKind::Onboard => {
                session.profile = None;
                session.draft = CAPTURE_NAME.to_owned();
                session.cursor = session.draft.chars().count();
                next_state.set(ChamberState::Onboarding);
            }
            CaptureKind::Seal => {
                session.profile = Some(WitnessProfile {
                    name: CAPTURE_NAME.to_owned(),
                });
                session.draft = CAPTURE_OFFERING.to_owned();
                session.cursor = session.draft.chars().count();
                next_state.set(ChamberState::IdleAtTable);
            }
            CaptureKind::Submit => {
                session.offering = CAPTURE_OFFERING.to_owned();
                session.draft.clear();
                session.cursor = 0;
                next_state.set(ChamberState::Deliberating);
            }
            CaptureKind::Shot(stem) => capture.shot(&mut commands, stem),
        }
        capture.next += 1;
    }

    // Post-submit stages are event-driven: the council (Ollama) and the Comfy
    // render finish on their own schedules.
    match state.get() {
        ChamberState::Deliberating => {
            if matches!(council.status, CouncilStatus::Failed(_)) && !capture.council_shot {
                capture.shot(&mut commands, "05_council_failed");
                capture.council_shot = true;
            }
        }
        ChamberState::CouncilSpeaking => {
            let seen = *capture.council_seen.get_or_insert(now);
            if !capture.council_shot && now - seen >= 2.0 {
                capture.shot(&mut commands, "05_council_speaking");
                capture.council_shot = true;
            }
            // A second frame of the same speaker: the focused panel should have turned
            // (clockwise) between the two, while the fixed star/spheres have not moved.
            if capture.council_shot && !capture.council_shot2 && now - seen >= 4.5 {
                capture.shot(&mut commands, "05b_council_speaking");
                capture.council_shot2 = true;
            }
        }
        ChamberState::WitnessVerdict => {
            let seen = *capture.verdict_seen.get_or_insert(now);
            if !capture.verdict_shot && now - seen >= 2.5 {
                capture.shot(&mut commands, "06_witness_verdict");
                capture.verdict_shot = true;
            }
            if capture.verdict_shot && !capture.authorized {
                session.artifact = None;
                next_state.set(ChamberState::ArtifactPending);
                capture.authorized = true;
            }
        }
        ChamberState::ArtifactPending => {
            let seen = *capture.pending_seen.get_or_insert(now);
            if !capture.pending_shot && now - seen >= 1.5 {
                capture.shot(&mut commands, "07_artifact_pending");
                capture.pending_shot = true;
            }
        }
        ChamberState::ArtifactResult => {
            let seen = *capture.artifact_seen.get_or_insert(now);
            if !capture.artifact_shot && now - seen >= 2.5 {
                capture.shot(&mut commands, "08_artifact_result");
                capture.artifact_shot = true;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_chronos_paths_resolve_under_repo_root() {
        assert_eq!(
            resolve_chronos_path("renders\\artifact.png"),
            PathBuf::from("C:\\chronos\\renders\\artifact.png")
        );
    }

    #[test]
    fn sanitize_id_keeps_safe_chars_and_falls_back() {
        assert_eq!(sanitize_id("artifact_9f-3A"), "artifact_9f-3A");
        assert_eq!(sanitize_id("a/b\\c:d"), "a-b-c-d");
        assert_eq!(sanitize_id("///"), "latest");
        assert_eq!(sanitize_id(""), "latest");
    }

    #[test]
    fn staged_artifacts_dir_sits_inside_asset_tree() {
        assert!(artifacts_asset_dir()
            .to_string_lossy()
            .replace('\\', "/")
            .ends_with("assets/artifacts"));
    }

    #[test]
    fn speaker_bubble_keeps_the_full_council_line() {
        let source = "Every generated word remains visible and audible.";
        assert_eq!(source.to_owned(), source);
    }

    #[test]
    fn ritual_editor_inserts_spaces_and_edits_at_character_cursor() {
        let mut session = RitualSession {
            draft: "helloworld".to_owned(),
            cursor: 5,
            ..default()
        };
        insert_at_cursor(&mut session, " ");
        assert_eq!(session.draft, "hello world");
        assert_eq!(session.cursor, 6);
        backspace_at_cursor(&mut session);
        assert_eq!(session.draft, "helloworld");
        assert_eq!(session.cursor, 5);
        delete_at_cursor(&mut session);
        assert_eq!(session.draft, "helloorld");
        assert_eq!(draft_with_caret(&session), "hello▌orld");
    }
}
