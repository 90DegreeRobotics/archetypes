use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{audio::AudioSource, prelude::*};

use super::{ritual::RitualSession, ChamberState, CurrentFocus};
use crate::theme::Archetype;

pub struct SpeechPlugin;

impl Plugin for SpeechPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpeechBridge>()
            .init_resource::<SpeechStatus>()
            .add_systems(Update, (play_focused_signature, poll_speech_result))
            .add_systems(OnEnter(ChamberState::WitnessVerdict), request_verdict_voice);
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct SpeechStatus {
    pub line: String,
}

#[derive(Resource, Default)]
struct SpeechBridge {
    receiver: Mutex<Option<mpsc::Receiver<Result<SpeechResult, String>>>>,
}

struct SpeechResult {
    archetype: Archetype,
    wav: Vec<u8>,
}

#[derive(Component)]
struct ArchetypeVoice;

fn play_focused_signature(
    mut commands: Commands,
    focus: Res<CurrentFocus>,
    asset_server: Res<AssetServer>,
    mut status: ResMut<SpeechStatus>,
    playing: Query<Entity, With<ArchetypeVoice>>,
) {
    // Each time the council yields the floor to a new speaker, `CurrentFocus`
    // changes; play that archetype's signature voice on the change only.
    if !focus.is_changed() {
        return;
    }
    let Some(archetype) = focus.0 else {
        status.line = String::new();
        return;
    };
    for entity in &playing {
        commands.entity(entity).despawn();
    }
    let filename = format!("audio/archetypes/{}.wav", archetype_slug(archetype));
    commands.spawn((
        AudioPlayer::new(asset_server.load(filename)),
        PlaybackSettings::DESPAWN,
        ArchetypeVoice,
        Name::new(format!("{:?}SignatureVoice", archetype)),
    ));
    status.line = format!("Voice: {:?} speaking", archetype);
}

fn request_verdict_voice(
    focus: Res<CurrentFocus>,
    session: Res<RitualSession>,
    bridge: Res<SpeechBridge>,
    mut status: ResMut<SpeechStatus>,
) {
    let Some(archetype) = focus.0 else { return };
    if session.verdict.trim().is_empty() {
        return;
    }
    let request = VoiceRequest::for_archetype(archetype);
    let request = VoiceRequest {
        text: session.verdict.clone(),
        ..request
    };
    let (sender, receiver) = mpsc::channel();
    *bridge.receiver.lock().expect("speech receiver lock") = Some(receiver);
    status.line = format!("Voice: {} is forming speech...", request.name);
    thread::spawn(move || {
        let result = synthesize(request).map(|wav| SpeechResult { archetype, wav });
        let _ = sender.send(result);
    });
}

fn poll_speech_result(
    mut commands: Commands,
    bridge: Res<SpeechBridge>,
    mut status: ResMut<SpeechStatus>,
    mut audio_assets: ResMut<Assets<AudioSource>>,
    playing: Query<Entity, With<ArchetypeVoice>>,
) {
    let result = {
        let guard = bridge.receiver.lock().expect("speech receiver lock");
        guard.as_ref().and_then(|receiver| receiver.try_recv().ok())
    };
    let Some(result) = result else { return };
    match result {
        Ok(result) => {
            for entity in &playing {
                commands.entity(entity).despawn();
            }
            let bytes: std::sync::Arc<[u8]> = result.wav.into();
            let handle = audio_assets.add(AudioSource { bytes });
            commands.spawn((
                AudioPlayer::new(handle),
                PlaybackSettings::DESPAWN,
                ArchetypeVoice,
                Name::new(format!("{:?}Voice", result.archetype)),
            ));
            status.line = format!("Voice: {:?} speaking", result.archetype);
        }
        Err(error) => status.line = format!("Voice unavailable: {error}"),
    }
}

#[derive(Debug, Clone)]
struct VoiceRequest {
    name: &'static str,
    speaker_id: u8,
    text: String,
}

impl VoiceRequest {
    fn for_archetype(archetype: Archetype) -> Self {
        match archetype {
            Archetype::Architect => Self {
                name: "Architect",
                speaker_id: 9,
                text: "Structure is not a cage. It is the geometry that lets becoming endure."
                    .to_owned(),
            },
            Archetype::Sentinel => Self {
                name: "Sentinel",
                speaker_id: 10,
                text: "Hold the threshold. Nothing crosses until the Witness gives authority."
                    .to_owned(),
            },
            Archetype::Mentor => Self {
                name: "Mentor",
                speaker_id: 5,
                text: "We do not rush the answer. We make enough room for wisdom to arrive."
                    .to_owned(),
            },
            Archetype::Explorer => Self {
                name: "Explorer",
                speaker_id: 4,
                text: "There is another path beyond the one we have already named. Let us find it."
                    .to_owned(),
            },
            Archetype::Oracle => Self {
                name: "Oracle",
                speaker_id: 7,
                text: "The pattern was present before the question. Listen for what repeats."
                    .to_owned(),
            },
            Archetype::Empath => Self {
                name: "Empath",
                speaker_id: 3,
                text: "I hear the weight beneath your words. You do not have to carry it alone."
                    .to_owned(),
            },
            Archetype::Jester => Self {
                name: "Jester",
                speaker_id: 2,
                text: "A perfect plan? Wonderful. Now let us find the door it forgot to lock."
                    .to_owned(),
            },
            Archetype::Codex | Archetype::Viren => Self {
                name: "Council",
                speaker_id: 6,
                text: "The council is listening.".to_owned(),
            },
        }
    }
}

fn archetype_slug(archetype: Archetype) -> &'static str {
    match archetype {
        Archetype::Architect => "architect",
        Archetype::Sentinel => "sentinel",
        Archetype::Mentor => "mentor",
        Archetype::Explorer => "explorer",
        Archetype::Oracle => "oracle",
        Archetype::Empath => "empath",
        Archetype::Jester => "jester",
        Archetype::Codex | Archetype::Viren => "architect",
    }
}

struct SpeechPaths {
    executable: PathBuf,
    model_dir: PathBuf,
    cache_dir: PathBuf,
}

impl SpeechPaths {
    fn resolve() -> Result<Self, String> {
        let (executable, model_dir) = match (
            std::env::var_os("ARCHETYPES_TTS_EXE"),
            std::env::var_os("ARCHETYPES_TTS_MODEL_DIR"),
        ) {
            (Some(exe), Some(model)) => (PathBuf::from(exe), PathBuf::from(model)),
            (None, None) => {
                let root = default_speech_root().ok_or("no installed speech runtime found")?;
                (
                    root.join("sherpa-onnx-v1.13.4-win-x64-shared-MD-Release")
                        .join("bin")
                        .join("sherpa-onnx-offline-tts.exe"),
                    root.join("kokoro-en-v0_19"),
                )
            }
            _ => {
                return Err(
                    "both ARCHETYPES_TTS_EXE and ARCHETYPES_TTS_MODEL_DIR must be set".to_owned(),
                )
            }
        };
        require_file(&executable, "sherpa-onnx executable")?;
        for file in ["model.onnx", "voices.bin", "tokens.txt"] {
            require_file(&model_dir.join(file), file)?;
        }
        let cache_dir = app_data_root().join("audio_cache");
        fs::create_dir_all(&cache_dir).map_err(|error| error.to_string())?;
        Ok(Self {
            executable,
            model_dir,
            cache_dir,
        })
    }
}

fn default_speech_root() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("ARCHETYPES_SPEECH_ROOT") {
        return Some(PathBuf::from(explicit));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let portable = dir.join("speech");
            if portable.is_dir() {
                return Some(portable);
            }
        }
    }
    std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .map(|root| root.join("Archetypes").join("speech"))
}

fn synthesize(request: VoiceRequest) -> Result<Vec<u8>, String> {
    let paths = SpeechPaths::resolve()?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let output = paths
        .cache_dir
        .join(format!("{}-{stamp}.wav", request.name.to_ascii_lowercase()));
    let mut command = Command::new(&paths.executable);
    command.args([
        format!(
            "--kokoro-model={}",
            paths.model_dir.join("model.onnx").display()
        ),
        format!(
            "--kokoro-voices={}",
            paths.model_dir.join("voices.bin").display()
        ),
        format!(
            "--kokoro-tokens={}",
            paths.model_dir.join("tokens.txt").display()
        ),
        format!(
            "--kokoro-data-dir={}",
            paths.model_dir.join("espeak-ng-data").display()
        ),
        "--num-threads=4".to_owned(),
        format!("--sid={}", request.speaker_id),
        format!("--output-filename={}", output.display()),
        request.text.to_owned(),
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let status = command
        .status()
        .map_err(|error| format!("could not start TTS: {error}"))?;
    if !status.success() {
        return Err(format!("TTS exited with {status}"));
    }
    let wav =
        fs::read(&output).map_err(|error| format!("could not read generated WAV: {error}"))?;
    if wav.len() < 44 || &wav[0..4] != b"RIFF" || &wav[8..12] != b"WAVE" {
        return Err("TTS output was not a valid non-empty WAV".to_owned());
    }
    Ok(wav)
}

fn require_file(path: &Path, label: &str) -> Result<(), String> {
    path.is_file()
        .then_some(())
        .ok_or_else(|| format!("{label} missing at {}", path.display()))
}

fn app_data_root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_seven_council_archetypes_have_distinct_speakers() {
        let ids = [
            Archetype::Architect,
            Archetype::Sentinel,
            Archetype::Mentor,
            Archetype::Explorer,
            Archetype::Oracle,
            Archetype::Empath,
            Archetype::Jester,
        ]
        .map(|archetype| VoiceRequest::for_archetype(archetype).speaker_id);
        let mut unique = ids.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 7);
    }

    #[test]
    fn voice_cache_is_mutable_user_data() {
        assert!(
            app_data_root().ends_with(Path::new("NeuroCognica").join("Archetypes").join("data"))
        );
    }
}
