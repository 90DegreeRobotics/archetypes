//! Archetypes launcher — a small supervisor that prepares the environment and starts
//! the council-chamber engine.
//!
//! It enforces a single instance, checks that the runtime dependencies the game needs
//! are actually up (the local Ollama council model, the offline TTS voices, and the
//! Chronos Foundry that serves ComfyUI images), then launches the engine and waits.

use std::{
    io::Write,
    net::{Ipv4Addr, TcpListener},
    path::PathBuf,
    process::Command,
    time::Duration,
};

use serde_json::Value;

/// A fixed loopback port used purely as a single-instance guard.
const SINGLE_INSTANCE_PORT: u16 = 47615;
const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DIRECTOR_URL: &str = "http://127.0.0.1:7777";
const COMFY_URL: &str = "http://127.0.0.1:8000";

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

    let allow_without_chronos = std::env::var_os("ARCHETYPES_ALLOW_WITHOUT_CHRONOS").is_some();
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
        if allow_without_chronos {
            eprintln!("\n[debug] ARCHETYPES_ALLOW_WITHOUT_CHRONOS is set — launching without Chronos.");
            eprintln!("        Standard/Oracle image paths will fail closed until Foundry is up.");
        } else {
            eprintln!("\n[required] Chronos Foundry is not ready for play.");
            eprintln!("           Start Chronos Foundry so Director (:7777) reports readiness");
            eprintln!("           and ComfyUI answers on :8000, then relaunch Archetypes.");
            if let Some(detail) = ready.chronos_detail.as_deref() {
                eprintln!("           Detail: {detail}");
            }
            pause_briefly();
            return;
        }
    }

    let engine = engine_path();
    if !engine.is_file() {
        eprintln!(
            "\n[fatal] engine executable not found at {}",
            engine.display()
        );
        pause_briefly();
        return;
    }

    println!("\nEntering the chamber...");
    let mut command = Command::new(&engine);
    if let Some(dir) = engine.parent() {
        command.current_dir(dir);
    }
    match command.status() {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("The engine exited with {status}."),
        Err(error) => eprintln!("Could not start the engine: {error}"),
    }
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
}
