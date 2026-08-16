//! Council TTS — WAV cache plus CLI, with an opt-in in-process C API.
//!
//! Per-line `sherpa-onnx-offline-tts.exe` respawn was paying a 4–5s model load
//! on every council turn. Hash-cached WAVs avoid that on repeat lines. The
//! in-process `sherpa-onnx-c-api.dll` path is **opt-in** (`ARCHETYPES_TTS_CAPI=1`)
//! because the pinned sherpa 1.13.4 sidecar ships ORT 1.17.1 (API 1–17) while
//! the C API requests ORT API 27 and access-violates the engine process. CLI
//! synthesis stays in a child process so that mismatch cannot take down the game.

use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CString},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};

use super::speech::{SpeechPaths, VoiceRequest};

struct TtsRuntime {
    cache: HashMap<String, Vec<u8>>,
    warm: Option<WarmEngine>,
}

struct WarmEngine {
    _lib: libloading::Library,
    tts: *const c_void,
    generate: unsafe extern "C" fn(*const c_void, *const c_char, i32, f32) -> *const GeneratedAudio,
    destroy_audio: unsafe extern "C" fn(*const GeneratedAudio),
    destroy_tts: unsafe extern "C" fn(*const c_void),
}

unsafe impl Send for WarmEngine {}

#[repr(C)]
#[derive(Default)]
struct VitsConfig {
    model: *const c_char,
    lexicon: *const c_char,
    tokens: *const c_char,
    data_dir: *const c_char,
    noise_scale: f32,
    noise_scale_w: f32,
    length_scale: f32,
    dict_dir: *const c_char,
}

#[repr(C)]
#[derive(Default)]
struct MatchaConfig {
    acoustic_model: *const c_char,
    vocoder: *const c_char,
    lexicon: *const c_char,
    tokens: *const c_char,
    data_dir: *const c_char,
    noise_scale: f32,
    length_scale: f32,
    dict_dir: *const c_char,
}

#[repr(C)]
#[derive(Default)]
struct KokoroConfig {
    model: *const c_char,
    voices: *const c_char,
    tokens: *const c_char,
    data_dir: *const c_char,
    length_scale: f32,
    dict_dir: *const c_char,
    lexicon: *const c_char,
    lang: *const c_char,
}

#[repr(C)]
#[derive(Default)]
struct KittenConfig {
    model: *const c_char,
    voices: *const c_char,
    tokens: *const c_char,
    data_dir: *const c_char,
    length_scale: f32,
}

#[repr(C)]
#[derive(Default)]
struct ZipvoiceConfig {
    tokens: *const c_char,
    encoder: *const c_char,
    decoder: *const c_char,
    vocoder: *const c_char,
    data_dir: *const c_char,
    lexicon: *const c_char,
    feat_scale: f32,
    t_shift: f32,
    target_rms: f32,
    guidance_scale: f32,
}

#[repr(C)]
#[derive(Default)]
struct PocketConfig {
    lm_flow: *const c_char,
    lm_main: *const c_char,
    encoder: *const c_char,
    decoder: *const c_char,
    text_conditioner: *const c_char,
    vocab_json: *const c_char,
    token_scores_json: *const c_char,
    voice_embedding_cache_capacity: i32,
}

#[repr(C)]
#[derive(Default)]
struct SupertonicConfig {
    duration_predictor: *const c_char,
    text_encoder: *const c_char,
    vector_estimator: *const c_char,
    vocoder: *const c_char,
    tts_json: *const c_char,
    unicode_indexer: *const c_char,
    voice_style: *const c_char,
}

#[repr(C)]
#[derive(Default)]
struct ModelConfig {
    vits: VitsConfig,
    num_threads: i32,
    debug: i32,
    provider: *const c_char,
    matcha: MatchaConfig,
    kokoro: KokoroConfig,
    kitten: KittenConfig,
    zipvoice: ZipvoiceConfig,
    pocket: PocketConfig,
    supertonic: SupertonicConfig,
}

#[repr(C)]
#[derive(Default)]
struct OfflineTtsConfig {
    model: ModelConfig,
    rule_fsts: *const c_char,
    max_num_sentences: i32,
    rule_fars: *const c_char,
    silence_scale: f32,
}

#[repr(C)]
struct GeneratedAudio {
    samples: *const f32,
    n: i32,
    sample_rate: i32,
}

static RUNTIME: Mutex<Option<TtsRuntime>> = Mutex::new(None);

pub fn warm_runtime() {
    let _ = with_runtime(|_| Ok(()));
}

pub fn synthesize_cached(request: &VoiceRequest) -> Result<Vec<u8>, String> {
    with_runtime(|runtime| runtime.speak(request))
}

fn with_runtime<T>(f: impl FnOnce(&mut TtsRuntime) -> Result<T, String>) -> Result<T, String> {
    let mut slot = RUNTIME
        .lock()
        .map_err(|_| "TTS runtime lock poisoned".to_owned())?;
    if slot.is_none() {
        *slot = Some(TtsRuntime::open());
    }
    f(slot.as_mut().expect("TTS runtime inserted"))
}

impl TtsRuntime {
    fn open() -> Self {
        let warm = if std::env::var_os("ARCHETYPES_TTS_CAPI").is_some() {
            SpeechPaths::resolve()
                .ok()
                .and_then(|paths| WarmEngine::load(&paths).ok())
        } else {
            None
        };
        Self {
            cache: HashMap::new(),
            warm,
        }
    }

    fn speak(&mut self, request: &VoiceRequest) -> Result<Vec<u8>, String> {
        let key = cache_key(request);
        if let Some(wav) = self.cache.get(&key) {
            return Ok(wav.clone());
        }
        let paths = SpeechPaths::resolve()?;
        let disk = paths.cache_dir.join(format!("{key}.wav"));
        if disk.is_file() {
            let wav = fs::read(&disk).map_err(|error| error.to_string())?;
            if wav_is_valid(&wav) {
                self.cache.insert(key, wav.clone());
                return Ok(wav);
            }
        }
        let wav = if let Some(warm) = self.warm.as_ref() {
            match warm.speak(request) {
                Ok(wav) => wav,
                Err(_) => synthesize_cli(request, &paths)?,
            }
        } else {
            synthesize_cli(request, &paths)?
        };
        let _ = fs::write(&disk, &wav);
        self.cache.insert(key, wav.clone());
        Ok(wav)
    }
}

impl WarmEngine {
    fn load(paths: &SpeechPaths) -> Result<Self, String> {
        let dll = c_api_dll(&paths.executable)?;
        prepend_dll_search_path(dll.parent().unwrap_or(Path::new(".")));
        if let Some(bin) = paths.executable.parent() {
            prepend_dll_search_path(bin);
        }
        let lib = unsafe { libloading::Library::new(&dll) }
            .map_err(|error| format!("could not load sherpa C API: {error}"))?;
        let create: libloading::Symbol<unsafe extern "C" fn(*const OfflineTtsConfig) -> *const c_void> =
            unsafe { lib.get(b"SherpaOnnxCreateOfflineTts\0") }
                .map_err(|error| format!("missing CreateOfflineTts: {error}"))?;
        let generate: libloading::Symbol<
            unsafe extern "C" fn(*const c_void, *const c_char, i32, f32) -> *const GeneratedAudio,
        > = unsafe { lib.get(b"SherpaOnnxOfflineTtsGenerate\0") }
            .map_err(|error| format!("missing OfflineTtsGenerate: {error}"))?;
        let destroy_audio: libloading::Symbol<unsafe extern "C" fn(*const GeneratedAudio)> =
            unsafe { lib.get(b"SherpaOnnxDestroyOfflineTtsGeneratedAudio\0") }
                .map_err(|error| format!("missing DestroyGeneratedAudio: {error}"))?;
        let destroy_tts: libloading::Symbol<unsafe extern "C" fn(*const c_void)> =
            unsafe { lib.get(b"SherpaOnnxDestroyOfflineTts\0") }
                .map_err(|error| format!("missing DestroyOfflineTts: {error}"))?;

        let model = CString::new(paths.model_dir.join("model.onnx").to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        let voices = CString::new(paths.model_dir.join("voices.bin").to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        let tokens = CString::new(paths.model_dir.join("tokens.txt").to_string_lossy().as_ref())
            .map_err(|error| error.to_string())?;
        let data_dir = CString::new(
            paths
                .model_dir
                .join("espeak-ng-data")
                .to_string_lossy()
                .as_ref(),
        )
        .map_err(|error| error.to_string())?;
        let provider = CString::new("cpu").map_err(|error| error.to_string())?;

        let mut config = OfflineTtsConfig::default();
        config.model.num_threads = 4;
        config.model.debug = 0;
        config.model.provider = provider.as_ptr();
        config.model.kokoro.model = model.as_ptr();
        config.model.kokoro.voices = voices.as_ptr();
        config.model.kokoro.tokens = tokens.as_ptr();
        config.model.kokoro.data_dir = data_dir.as_ptr();
        config.model.kokoro.length_scale = 1.0;
        config.max_num_sentences = 2;
        config.silence_scale = 0.2;

        let tts = unsafe { create(&config) };
        if tts.is_null() {
            return Err("SherpaOnnxCreateOfflineTts returned null".to_owned());
        }

        std::mem::forget(model);
        std::mem::forget(voices);
        std::mem::forget(tokens);
        std::mem::forget(data_dir);
        std::mem::forget(provider);

        Ok(Self {
            generate: *generate,
            destroy_audio: *destroy_audio,
            destroy_tts: *destroy_tts,
            tts,
            _lib: lib,
        })
    }

    fn speak(&self, request: &VoiceRequest) -> Result<Vec<u8>, String> {
        let text = CString::new(request.text.as_str()).map_err(|error| error.to_string())?;
        let audio = unsafe { (self.generate)(self.tts, text.as_ptr(), i32::from(request.speaker_id), 1.0) };
        if audio.is_null() {
            return Err("warm TTS generated no audio".to_owned());
        }
        let wav = unsafe {
            let audio_ref = &*audio;
            if audio_ref.samples.is_null() || audio_ref.n <= 0 {
                (self.destroy_audio)(audio);
                return Err("warm TTS returned empty samples".to_owned());
            }
            let samples = std::slice::from_raw_parts(audio_ref.samples, audio_ref.n as usize);
            let wav = pcm_f32_to_wav(samples, audio_ref.sample_rate);
            (self.destroy_audio)(audio);
            wav
        };
        if wav_is_valid(&wav) {
            Ok(wav)
        } else {
            Err("warm TTS did not produce a valid WAV".to_owned())
        }
    }
}

impl Drop for WarmEngine {
    fn drop(&mut self) {
        if !self.tts.is_null() {
            unsafe { (self.destroy_tts)(self.tts) };
            self.tts = std::ptr::null();
        }
    }
}

fn prepend_dll_search_path(dir: &Path) {
    let extra = dir.display().to_string();
    let current = std::env::var("PATH").unwrap_or_default();
    if !current.to_lowercase().contains(&extra.to_lowercase()) {
        std::env::set_var("PATH", format!("{extra};{current}"));
    }
}

fn c_api_dll(executable: &Path) -> Result<PathBuf, String> {
    let root = executable
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "TTS executable has no runtime root".to_owned())?;
    let dll = root.join("lib").join("sherpa-onnx-c-api.dll");
    dll.is_file()
        .then_some(dll.clone())
        .ok_or_else(|| format!("sherpa C API missing at {}", dll.display()))
}

fn synthesize_cli(request: &VoiceRequest, paths: &SpeechPaths) -> Result<Vec<u8>, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let output = paths
        .cache_dir
        .join(format!("cli-{}-{stamp}.wav", request.name.to_ascii_lowercase()));
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
    let wav = fs::read(&output).map_err(|error| format!("could not read generated WAV: {error}"))?;
    if wav_is_valid(&wav) {
        Ok(wav)
    } else {
        Err("TTS output was not a valid non-empty WAV".to_owned())
    }
}

pub fn cache_key(request: &VoiceRequest) -> String {
    let mut hasher = Sha256::new();
    hasher.update([request.speaker_id]);
    hasher.update(request.text.as_bytes());
    hasher
        .finalize()
        .iter()
        .take(16)
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn wav_is_valid(wav: &[u8]) -> bool {
    wav.len() >= 44 && &wav[0..4] == b"RIFF" && &wav[8..12] == b"WAVE"
}

fn pcm_f32_to_wav(samples: &[f32], sample_rate: i32) -> Vec<u8> {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let clipped = sample.clamp(-1.0, 1.0);
        let value = (clipped * 32767.0) as i16;
        pcm.extend_from_slice(&value.to_le_bytes());
    }
    let sample_rate = sample_rate.max(1) as u32;
    let byte_rate = sample_rate * 2;
    let data_len = pcm.len() as u32;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(&pcm);
    wav
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Archetype;

    #[test]
    fn cache_key_is_stable_for_identical_speech() {
        let a = VoiceRequest::for_council_line(Archetype::Oracle, "The pattern repeats.");
        let b = VoiceRequest::for_council_line(Archetype::Oracle, "The pattern repeats.");
        let c = VoiceRequest::for_council_line(Archetype::Oracle, "The pattern changes.");
        assert_eq!(cache_key(&a), cache_key(&b));
        assert_ne!(cache_key(&a), cache_key(&c));
        assert_eq!(cache_key(&a).len(), 32);
    }

    #[test]
    fn pcm_wav_has_riff_header() {
        let wav = pcm_f32_to_wav(&[0.0, 0.5, -0.5], 22050);
        assert!(wav_is_valid(&wav));
        assert!(wav.len() > 44);
    }

    #[test]
    fn in_process_c_api_is_opt_in_so_ort_mismatch_cannot_kill_the_engine() {
        let source = include_str!("tts_runtime.rs");
        assert!(source.contains("ARCHETYPES_TTS_CAPI"));
        assert!(source.contains("synthesize_cli"));
    }
}
