//! Archetypes launcher — a small supervisor that prepares the environment and starts
//! the council-chamber engine.
//!
//! It enforces a single instance, checks that the runtime dependencies the game needs
//! are actually up (the local Ollama council model, the offline TTS voices, and the
//! Chronos Foundry that serves ComfyUI images), obtains Sentinel authorization for
//! the launch itself, then starts the engine and waits.

// Release Desktop launches must not show a command console — only the game window.
// Failures still surface via Notepad (`fail_visible`) and AppData logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs::File,
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde_json::{json, Value};

/// A fixed loopback port used purely as a single-instance guard.
const SINGLE_INSTANCE_PORT: u16 = 47615;
const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DIRECTOR_URL: &str = "http://127.0.0.1:7777";
const COMFY_URL: &str = "http://127.0.0.1:8000";
const SENTINEL_LAUNCH_LAW: &str = "Let there be no gate before the Sentinel";
const SENTINEL_LAUNCH_ACTION: &str = "archetypes.launch";

fn main() {
    println!("Archetypes - Council Chamber");
    println!("============================");

    // Binding the guard port fails if another instance already holds it.
    let _guard = match TcpListener::bind((Ipv4Addr::LOCALHOST, SINGLE_INSTANCE_PORT)) {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("Archetypes is already running.");
            pause_briefly();
            return;
        }
    };

    let legacy_allow_without_chronos =
        std::env::var_os("ARCHETYPES_ALLOW_WITHOUT_CHRONOS").is_some();
    let engine = engine_path();
    if let Err(error) = authorize_sentinel_launch(&engine) {
        let mut body = format!(
            "The Archetypes game engine was not started.\n\n\
             Sentinel launch authorization failed before engine execution:\n{error}"
        );
        if legacy_allow_without_chronos {
            body.push_str(
                "\n\nARCHETYPES_ALLOW_WITHOUT_CHRONOS was set, but it cannot bypass \
                 the Sentinel launch gate.",
            );
        }
        fail_visible(
            "Archetypes Sentinel launch refused",
            &body,
            &engine_log_path(),
        );
        return;
    }

    let ready = check_readiness();
    if !ready.ollama {
        eprintln!("\n[required] Ollama is not responding on 127.0.0.1:11434.");
        eprintln!("           The council cannot speak without it. Start Ollama, then relaunch.");
        if let Some(detail) = ready.ollama_detail.as_deref() {
            eprintln!("           Detail: {detail}");
        }
        pause_briefly();
        return;
    }
    if !ready.tts {
        eprintln!("\n[required] The offline voices are not installed.");
        eprintln!("           Re-run the Archetypes installer to repair the voice bundle.");
        pause_briefly();
        return;
    }
    if !ready.chronos_foundry {
        eprintln!("\n[required] Chronos Foundry is not ready for play.");
        eprintln!("           Start Chronos Foundry so Director (:7777) reports readiness");
        eprintln!("           and ComfyUI answers on :8000, then relaunch Archetypes.");
        if legacy_allow_without_chronos {
            eprintln!("           ARCHETYPES_ALLOW_WITHOUT_CHRONOS is ignored for launch safety.");
        }
        if let Some(detail) = ready.chronos_detail.as_deref() {
            eprintln!("           Detail: {detail}");
        }
        pause_briefly();
        return;
    }

    if !engine.is_file() {
        eprintln!(
            "\n[fatal] engine executable not found at {}",
            engine.display()
        );
        pause_briefly();
        return;
    }

    println!("\nEntering the chamber...");
    let log_path = engine_log_path();
    match run_engine(&engine, &log_path) {
        Ok(status) if status.success() => {}
        Ok(status) => {
            let tail = read_log_tail(&log_path, 4000);
            let body = format!(
                "The chamber engine exited with {status}.\n\n--- last log lines ---\n{tail}"
            );
            fail_visible("Archetypes engine failed", &body, &log_path);
        }
        Err(error) => {
            let body = format!("Could not start the engine:\n{error}");
            fail_visible("Archetypes launcher failed", &body, &log_path);
        }
    }
}

fn authorize_sentinel_launch(engine: &Path) -> Result<(), String> {
    let status = http_json(
        &format!("{DIRECTOR_URL}/api/v1/status"),
        Duration::from_secs(3),
    )?;
    require_sentinel_enforce_status(&status)?;

    let body = sentinel_launch_event_body(engine);
    let receipt = http_post_json(
        &format!("{DIRECTOR_URL}/api/v1/codex/append"),
        &body,
        Duration::from_secs(8),
    )?;
    let event_id = nonempty_json_str(&receipt, "event_id")
        .ok_or_else(|| format!("guarded launch append returned no event_id: {receipt}"))?;
    nonempty_json_str(&receipt, "integrity_hash")
        .ok_or_else(|| format!("guarded launch append returned no integrity_hash: {receipt}"))?;

    println!("Sentinel launch gate: authorized ({event_id})");
    Ok(())
}

fn require_sentinel_enforce_status(status: &Value) -> Result<(), String> {
    if status.get("readiness").and_then(Value::as_str) != Some("ready") {
        return Err(format!(
            "Chronos Director readiness is {:?}; Sentinel launch requires ready.",
            status.get("readiness")
        ));
    }
    let authority = status
        .get("authority")
        .ok_or_else(|| "Chronos Director status has no authority block.".to_owned())?;
    let mode = authority
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let enforced = authority.get("enforced").and_then(Value::as_bool);
    if mode != "enforce" || enforced != Some(true) {
        return Err(format!(
            "Chronos Sentinel authority is not enforcing (mode={mode}, enforced={enforced:?})."
        ));
    }
    nonempty_json_str(authority, "actor")
        .ok_or_else(|| "Chronos Sentinel authority did not report an actor.".to_owned())?;
    nonempty_json_str(authority, "key_id")
        .ok_or_else(|| "Chronos Sentinel authority did not report a key_id.".to_owned())?;
    Ok(())
}

fn sentinel_launch_event_body(engine: &Path) -> Value {
    json!({
        "archetype": "archetypes",
        "event_type": "archetypes_launch_requested",
        "content": {
            "source": "archetypes_launcher",
            "protected_action": SENTINEL_LAUNCH_ACTION,
            "law": SENTINEL_LAUNCH_LAW,
            "engine_path": engine.display().to_string(),
            "requested_unix_ms": unix_now_millis(),
        }
    })
}

fn nonempty_json_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn unix_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn run_engine(engine: &Path, log_path: &Path) -> std::io::Result<std::process::ExitStatus> {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let log = File::create(log_path)?;
    let stdout = log.try_clone()?;
    let mut command = Command::new(engine);
    if let Some(dir) = engine.parent() {
        command.current_dir(dir);
    }
    command.stdout(Stdio::from(stdout)).stderr(Stdio::from(log));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Belt-and-suspenders: never flash a console for the game child.
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.status()
}

fn engine_log_path() -> PathBuf {
    logs_dir().join("last-engine.log")
}

fn logs_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("NeuroCognica").join("Archetypes").join("logs")
}

fn read_log_tail(path: &Path, max_bytes: usize) -> String {
    let Ok(mut file) = File::open(path) else {
        return "(no engine log was written)".to_owned();
    };
    let mut bytes = Vec::new();
    if file.read_to_end(&mut bytes).is_err() {
        return "(engine log could not be read)".to_owned();
    }
    let start = bytes.len().saturating_sub(max_bytes);
    String::from_utf8_lossy(&bytes[start..])
        .replace('\0', "")
        .trim()
        .to_owned()
}

/// Desktop shortcuts often have no console. Persist the failure and open it in
/// Notepad so the operator always sees why the chamber died.
fn fail_visible(title: &str, body: &str, log_path: &Path) {
    let summary_path = logs_dir().join("last-failure.txt");
    let summary = format!(
        "{title}\n\n{body}\n\nFull engine log:\n{}\n",
        log_path.display()
    );
    let _ = std::fs::write(&summary_path, &summary);
    eprintln!("\n{title}");
    eprintln!("{body}");
    eprintln!("\nFailure summary: {}", summary_path.display());
    eprintln!("Engine log:      {}", log_path.display());
    let _ = Command::new("notepad.exe").arg(&summary_path).spawn();
    pause_briefly();
}

struct Readiness {
    ollama: bool,
    tts: bool,
    chronos_foundry: bool,
    ollama_detail: Option<String>,
    chronos_detail: Option<String>,
}

fn check_readiness() -> Readiness {
    let (ollama, ollama_detail) = probe_ollama();
    let (director, director_detail) = probe_director();
    let (comfy, comfy_detail) = probe_comfy();
    let tts = tts_installed();
    let chronos_foundry = director && comfy;
    let chronos_detail = match (director, comfy) {
        (true, true) => None,
        (false, _) => director_detail,
        (true, false) => comfy_detail,
    };

    println!("Ollama (11434):     {}", status_word(ollama));
    println!("TTS voices:         {}", status_word(tts));
    println!("Chronos Director:   {}", status_word(director));
    println!("ComfyUI (8000):     {}", status_word(comfy));

    Readiness {
        ollama,
        tts,
        chronos_foundry,
        ollama_detail,
        chronos_detail,
    }
}

fn probe_ollama() -> (bool, Option<String>) {
    match http_json(&format!("{OLLAMA_URL}/api/tags"), Duration::from_secs(2)) {
        Ok(_) => (true, None),
        Err(error) => (false, Some(error)),
    }
}

fn probe_director() -> (bool, Option<String>) {
    match http_json(
        &format!("{DIRECTOR_URL}/api/v1/status"),
        Duration::from_secs(3),
    ) {
        Ok(body) => {
            if body.get("readiness").and_then(Value::as_str) == Some("ready") {
                (true, None)
            } else {
                (
                    false,
                    Some(format!(
                        "Director answered but readiness is {:?}",
                        body.get("readiness")
                    )),
                )
            }
        }
        Err(error) => (false, Some(error)),
    }
}

fn probe_comfy() -> (bool, Option<String>) {
    match http_json(&format!("{COMFY_URL}/system_stats"), Duration::from_secs(3)) {
        Ok(_) => (true, None),
        Err(error) => (false, Some(error)),
    }
}

fn http_json(url: &str, timeout: Duration) -> Result<Value, String> {
    ureq::get(url)
        .timeout(timeout)
        .call()
        .map_err(|error| error.to_string())?
        .into_json()
        .map_err(|error| error.to_string())
}

fn http_post_json(url: &str, body: &Value, timeout: Duration) -> Result<Value, String> {
    ureq::post(url)
        .timeout(timeout)
        .send_json(body.clone())
        .map_err(|error| error.to_string())?
        .into_json()
        .map_err(|error| error.to_string())
}

fn tts_installed() -> bool {
    speech_roots()
        .into_iter()
        .any(|root| speech_root_ready(&root))
}

fn speech_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(explicit) = std::env::var_os("ARCHETYPES_SPEECH_ROOT") {
        roots.push(PathBuf::from(explicit));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("speech"));
        }
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        roots.push(
            PathBuf::from(program_files)
                .join("Archetypes")
                .join("speech"),
        );
    }
    roots
}

fn speech_root_ready(root: &std::path::Path) -> bool {
    root.join("sherpa-onnx-v1.13.4-win-x64-shared-MD-Release")
        .join("bin")
        .join("sherpa-onnx-offline-tts.exe")
        .is_file()
        && root.join("kokoro-en-v0_19").join("model.onnx").is_file()
}

fn engine_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.join("engine.exe")))
        .unwrap_or_else(|| PathBuf::from("engine.exe"))
}

fn status_word(ok: bool) -> &'static str {
    if ok {
        "ready"
    } else {
        "MISSING"
    }
}

fn pause_briefly() {
    print!("\nPress Enter to close...");
    let _ = std::io::stdout().flush();
    let mut buffer = String::new();
    let _ = std::io::stdin().read_line(&mut buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_speech_root_contract_checks_runtime_and_model() {
        let root = std::env::temp_dir().join("archetypes-launcher-speech-contract");
        let exe = root
            .join("sherpa-onnx-v1.13.4-win-x64-shared-MD-Release/bin/sherpa-onnx-offline-tts.exe");
        let model = root.join("kokoro-en-v0_19/model.onnx");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        std::fs::create_dir_all(model.parent().unwrap()).unwrap();
        std::fs::write(&exe, b"runtime").unwrap();
        std::fs::write(&model, b"model").unwrap();
        assert!(speech_root_ready(&root));
    }

    #[test]
    fn status_word_is_honest() {
        assert_eq!(status_word(true), "ready");
        assert_eq!(status_word(false), "MISSING");
    }

    #[test]
    fn logs_dir_is_under_neurocognica_archetypes() {
        let dir = logs_dir();
        let text = dir.to_string_lossy().replace('\\', "/").to_lowercase();
        assert!(text.contains("neurocognica/archetypes/logs"), "{text}");
    }

    #[test]
    fn sentinel_launch_status_requires_enforce_mode() {
        let status = json!({
            "readiness": "ready",
            "authority": {
                "mode": "enforce",
                "enforced": true,
                "actor": "local_operator",
                "key_id": "abc123"
            }
        });
        require_sentinel_enforce_status(&status).unwrap();
    }

    #[test]
    fn sentinel_launch_status_rejects_shadow_mode() {
        let status = json!({
            "readiness": "ready",
            "authority": {
                "mode": "shadow",
                "enforced": false,
                "actor": "local_operator",
                "key_id": "abc123"
            }
        });
        let error = require_sentinel_enforce_status(&status).unwrap_err();
        assert!(error.contains("not enforcing"), "{error}");
    }

    #[test]
    fn sentinel_launch_status_rejects_degraded_director() {
        let status = json!({
            "readiness": "degraded",
            "authority": {
                "mode": "enforce",
                "enforced": true,
                "actor": "local_operator",
                "key_id": "abc123"
            }
        });
        let error = require_sentinel_enforce_status(&status).unwrap_err();
        assert!(error.contains("requires ready"), "{error}");
    }

    #[test]
    fn sentinel_launch_event_body_carries_carved_law_and_action() {
        let body = sentinel_launch_event_body(Path::new(r"C:\archetypes\dist\engine.exe"));
        assert_eq!(body["archetype"], "archetypes");
        assert_eq!(body["event_type"], "archetypes_launch_requested");
        assert_eq!(body["content"]["protected_action"], SENTINEL_LAUNCH_ACTION);
        assert_eq!(body["content"]["law"], SENTINEL_LAUNCH_LAW);
        assert_eq!(
            body["content"]["engine_path"],
            r"C:\archetypes\dist\engine.exe"
        );
        assert!(body["content"]["requested_unix_ms"]
            .as_u64()
            .is_some_and(|stamp| stamp > 0));
    }
}
