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
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{speech::SpeechStatus, ChamberState, CurrentFocus};
use crate::theme::Archetype;

const DIRECTOR_URL: &str = "http://127.0.0.1:7777";

pub struct RitualPlugin;

impl Plugin for RitualPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RitualSession>()
            .init_resource::<RitualClock>()
            .init_resource::<ChronosBridge>()
            .add_systems(Startup, (spawn_ritual_ui, load_witness_profile))
            .add_systems(
                Update,
                (
                    receive_text_input,
                    advance_ritual_clock,
                    begin_chronos_request,
                    poll_chronos_result,
                    render_ritual_ui,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(ChamberState::ArtifactResult),
                present_artifact_image,
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
    offering: String,
    pub(super) verdict: String,
    artifact: Option<ArtifactOutcome>,
    artifact_note: Option<String>,
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
struct RitualClock {
    last_state: Option<ChamberState>,
    elapsed: f32,
}

#[derive(Resource, Default)]
struct ChronosBridge {
    receiver: Mutex<Option<mpsc::Receiver<ArtifactOutcome>>>,
    started: bool,
}

#[derive(Component)]
struct RitualUi;

fn spawn_ritual_ui(mut commands: Commands) {
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
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 27.0,
            ..default()
        },
        TextColor(Color::srgb(0.93, 0.93, 0.93)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(36.0),
            right: Val::Px(36.0),
            bottom: Val::Px(32.0),
            padding: UiRect::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.018, 0.025, 0.88)),
        UiTargetCamera(ui_camera),
        RitualUi,
    ));
}

fn load_witness_profile(
    mut session: ResMut<RitualSession>,
    mut next_state: ResMut<NextState<ChamberState>>,
) {
    let Ok(body) = fs::read_to_string(profile_path()) else {
        return;
    };
    if let Ok(profile) = serde_json::from_str::<WitnessProfile>(&body) {
        if !profile.name.trim().is_empty() {
            session.profile = Some(profile);
            next_state.set(ChamberState::IdleAtTable);
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
        match key {
            Key::Backspace
                if matches!(
                    state.get(),
                    ChamberState::Onboarding | ChamberState::IdleAtTable
                ) =>
            {
                session.draft.pop();
            }
            Key::Enter => match state.get() {
                ChamberState::Onboarding => seal_profile(&mut session, &mut next_state),
                ChamberState::IdleAtTable => submit_offering(&mut session, &mut next_state),
                ChamberState::ArchitectInterior => {
                    session.verdict = architect_verdict(&session.offering);
                    next_state.set(ChamberState::WitnessVerdict);
                }
                ChamberState::WitnessVerdict => {
                    session.artifact = None;
                    next_state.set(ChamberState::ArtifactPending);
                }
                ChamberState::ArtifactResult => {
                    session.draft.clear();
                    next_state.set(ChamberState::IdleAtTable);
                }
                _ => {}
            },
            Key::Character(text)
                if matches!(
                    state.get(),
                    ChamberState::Onboarding | ChamberState::IdleAtTable
                ) =>
            {
                let room = 600usize.saturating_sub(session.draft.chars().count());
                session.draft.extend(
                    text.chars()
                        .filter(|character| !character.is_control())
                        .take(room),
                );
            }
            _ => {}
        }
    }
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
    next_state.set(ChamberState::Deliberating);
}

fn advance_ritual_clock(
    time: Res<Time>,
    state: Res<State<ChamberState>>,
    mut clock: ResMut<RitualClock>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut focus: ResMut<CurrentFocus>,
) {
    let current = state.get().clone();
    if clock.last_state.as_ref() != Some(&current) {
        clock.last_state = Some(current.clone());
        clock.elapsed = 0.0;
    } else {
        clock.elapsed += time.delta_secs();
    }

    match current {
        ChamberState::Deliberating if clock.elapsed >= 2.0 => {
            focus.0 = Some(Archetype::Architect);
            next_state.set(ChamberState::FocusArchetype);
        }
        ChamberState::FocusArchetype if clock.elapsed >= 2.0 => {
            next_state.set(ChamberState::ArchitectInterior);
        }
        ChamberState::IdleAtTable => focus.0 = None,
        _ => {}
    }
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

    let submit: Value = match response_json(
        ureq::post(&format!("{DIRECTOR_URL}/api/v1/pipeline/render-still"))
            .timeout(Duration::from_secs(10))
            .send_json(json!({ "prompt": prompt, "quick": true, "use_dsl": false })),
    ) {
        Ok(body) => body,
        Err(error) => {
            return failed_outcome(format!("Chronos rejected the artifact request: {error}"))
        }
    };
    let Some(job_id) = submit
        .get("job_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return failed_outcome("Chronos returned no job id".to_owned());
    };

    for _ in 0..300 {
        thread::sleep(Duration::from_secs(1));
        let job: Value = match response_json(
            ureq::get(&format!("{DIRECTOR_URL}/api/v1/pipeline/job/{job_id}"))
                .timeout(Duration::from_secs(4))
                .call(),
        ) {
            Ok(job) => job,
            Err(error) => {
                return failed_outcome(format!("Chronos job polling failed: {error}"));
            }
        };
        match job.get("status").and_then(Value::as_str) {
            Some("done") => {
                let png_path = job
                    .get("png_path")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                let verified_path = png_path.as_deref().map(resolve_chronos_path);
                if verified_path.as_ref().is_none_or(|path| !path.is_file()) {
                    return ArtifactOutcome {
                        status: "failed".to_owned(),
                        job_id: Some(job_id),
                        artifact_id: string_field(&job, "artifact_id"),
                        png_path,
                        proof_receipt_id: string_field(&job, "proof_receipt_id"),
                        detail: "Chronos reported success without a readable PNG artifact"
                            .to_owned(),
                    };
                }
                return ArtifactOutcome {
                    status: "complete".to_owned(),
                    job_id: Some(job_id),
                    artifact_id: string_field(&job, "artifact_id"),
                    png_path: verified_path.map(|path| path.display().to_string()),
                    proof_receipt_id: string_field(&job, "proof_receipt_id"),
                    detail: "Chronos returned a verified local render artifact".to_owned(),
                };
            }
            Some("failed") => {
                return ArtifactOutcome {
                    status: "failed".to_owned(),
                    job_id: Some(job_id),
                    artifact_id: string_field(&job, "artifact_id"),
                    png_path: string_field(&job, "png_path"),
                    proof_receipt_id: string_field(&job, "proof_receipt_id"),
                    detail: job
                        .get("output")
                        .and_then(Value::as_str)
                        .unwrap_or("Chronos render failed without output")
                        .chars()
                        .take(500)
                        .collect(),
                };
            }
            _ => {}
        }
    }

    ArtifactOutcome {
        status: "failed".to_owned(),
        job_id: Some(job_id),
        artifact_id: None,
        png_path: None,
        proof_receipt_id: None,
        detail: "Chronos render exceeded the 300 second council timeout".to_owned(),
    }
}

fn render_ritual_ui(
    state: Res<State<ChamberState>>,
    session: Res<RitualSession>,
    speech: Res<SpeechStatus>,
    mut query: Query<&mut Text, With<RitualUi>>,
) {
    let Ok(mut text) = query.single_mut() else {
        return;
    };
    let witness = session
        .profile
        .as_ref()
        .map(|profile| profile.name.as_str())
        .unwrap_or("Witness");
    let ritual_text = match state.get() {
        ChamberState::Onboarding => format!(
            "THE WITNESS\n\nName the sovereign center:\n{}▌\n\nEnter seals the profile.",
            session.draft
        ),
        ChamberState::IdleAtTable => format!(
            "{witness} — THE TABLE\n\nOffer a thought to the council:\n{}▌\n\nEnter begins deliberation.",
            session.draft
        ),
        ChamberState::Deliberating => format!(
            "THE COUNCIL AWAKENS\n\nOffering: {}\n\nSeven laws of mind are taking position.",
            session.offering
        ),
        ChamberState::FocusArchetype => {
            "THE ARCHITECT TAKES THE CENTER\n\nStructure emerges from contention.".to_owned()
        }
        ChamberState::ArchitectInterior => format!(
            "LUMINOUS BLUEPRINT\n\nThe Architect frames the offering:\n{}\n\nEnter requests a buildable verdict.",
            session.offering
        ),
        ChamberState::WitnessVerdict => format!(
            "COUNCIL VERDICT\n\n{}\n\n{witness}, Enter authorizes this becoming through Chronos.",
            session.verdict
        ),
        ChamberState::ArtifactPending => {
            "CHRONOS ARTIFACT ORGAN\n\nThe authorized brief is rendering locally.\nNo artifact will be claimed until a real PNG path is verified.".to_owned()
        }
        ChamberState::ArtifactResult => {
            let outcome = session.artifact.as_ref();
            let display_line = match (
                outcome.map(|value| value.status.as_str()),
                session.artifact_note.as_deref(),
            ) {
                (Some("complete"), None) => "Display: manifested above.".to_owned(),
                (Some("complete"), Some(note)) => format!("Display: {note}"),
                _ => "Display: none — no verified image to show.".to_owned(),
            };
            format!(
                "ARTIFACT RETURN\n\nStatus: {}\n{}\n{}\nArtifact: {}\nPNG: {}\nProof: {}\n\nEnter returns to the table.",
                outcome.map(|value| value.status.as_str()).unwrap_or("unknown"),
                outcome.map(|value| value.detail.as_str()).unwrap_or("No result"),
                display_line,
                outcome.and_then(|value| value.artifact_id.as_deref()).unwrap_or("none"),
                outcome.and_then(|value| value.png_path.as_deref()).unwrap_or("none"),
                outcome.and_then(|value| value.proof_receipt_id.as_deref()).unwrap_or("none"),
            )
        }
    };
    text.0 = if speech.line.is_empty() {
        ritual_text
    } else {
        format!("{ritual_text}\n\n{}", speech.line)
    };
}

fn architect_verdict(offering: &str) -> String {
    format!(
        "Build one coherent visual artifact from this seed: ‘{}’. Preserve a clear central structure, restrained sacred geometry, material legibility, and one intentional point of asymmetry. The Witness retains acceptance authority.",
        offering.trim()
    )
}

fn artifact_prompt(offering: &str, verdict: &str) -> String {
    format!(
        "Council-authorized Architect study. Original offering: {offering}. Build verdict: {verdict}. Produce one restrained, artifact-grade 3D still with a readable silhouette, dark ceremonial environment, and no text."
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
            top: Val::Percent(7.0),
            left: Val::Percent(28.0),
            width: Val::Percent(44.0),
            ..default()
        },
        ImageNode::new(handle),
        ArtifactImageNode,
    ));
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
    /// Submit the offering, handing off to the deliberation clock.
    Submit,
    /// Resolve the Architect's build verdict.
    Verdict,
    /// Authorize the becoming, triggering the real Chronos request.
    Authorize,
    /// Save a screenshot of the current frame under this stem.
    Shot(&'static str),
}

#[derive(Resource)]
struct CaptureRun {
    dir: PathBuf,
    steps: Vec<(f32, CaptureKind)>,
    next: usize,
    artifact_seen_at: Option<f32>,
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
            // (seconds since app start, action). Ordered. Timings leave room for
            // the GLB scene to load and for the deliberation clock (2s + 2s) to
            // carry Deliberating -> FocusArchetype -> ArchitectInterior on its own.
            steps: vec![
                (2.0, CaptureKind::Onboard),
                (3.5, CaptureKind::Shot("01_onboarding")),
                // Second onboarding frame: the camera is fixed and every object is
                // static, so this must be pixel-identical to 01 unless something is
                // still moving. A hash comparison of the two is the static check.
                (7.0, CaptureKind::Shot("01b_onboarding_static_check")),
                (7.3, CaptureKind::Seal),
                (8.2, CaptureKind::Shot("02_table")),
                (8.5, CaptureKind::Submit),
                (9.6, CaptureKind::Shot("03_deliberating")),
                (11.2, CaptureKind::Shot("04_architect_focus")),
                (13.8, CaptureKind::Shot("05_architect_interior")),
                (14.1, CaptureKind::Verdict),
                (15.0, CaptureKind::Shot("06_witness_verdict")),
                (15.3, CaptureKind::Authorize),
                (17.0, CaptureKind::Shot("07_artifact_pending")),
            ],
            next: 0,
            artifact_seen_at: None,
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
                next_state.set(ChamberState::Onboarding);
            }
            CaptureKind::Seal => {
                session.profile = Some(WitnessProfile {
                    name: CAPTURE_NAME.to_owned(),
                });
                session.draft = CAPTURE_OFFERING.to_owned();
                next_state.set(ChamberState::IdleAtTable);
            }
            CaptureKind::Submit => {
                session.offering = CAPTURE_OFFERING.to_owned();
                session.draft.clear();
                next_state.set(ChamberState::Deliberating);
            }
            CaptureKind::Verdict => {
                session.verdict = architect_verdict(&session.offering);
                next_state.set(ChamberState::WitnessVerdict);
            }
            CaptureKind::Authorize => {
                session.artifact = None;
                next_state.set(ChamberState::ArtifactPending);
            }
            CaptureKind::Shot(stem) => capture.shot(&mut commands, stem),
        }
        capture.next += 1;
    }

    // The Chronos render finishes on its own schedule, so the result frame is
    // captured on arrival rather than on a fixed timeline.
    if *state.get() == ChamberState::ArtifactResult {
        match capture.artifact_seen_at {
            None => capture.artifact_seen_at = Some(now),
            Some(seen) if !capture.artifact_shot && now - seen >= 2.5 => {
                capture.shot(&mut commands, "08_artifact_result");
                capture.artifact_shot = true;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architect_verdict_preserves_witness_authority() {
        let verdict = architect_verdict("a city grown around a memory");
        assert!(verdict.contains("a city grown around a memory"));
        assert!(verdict.contains("Witness retains acceptance authority"));
    }

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
}
