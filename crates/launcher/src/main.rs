//! Archetypes launcher — a small supervisor that prepares the environment and starts
//! the council-chamber engine.
//!
//! It enforces a single instance, checks that the runtime dependencies the game needs
//! are actually up (the local Ollama council model, the offline TTS voices, and the
//! Chronos Foundry that serves ComfyUI images), then launches the engine and waits.

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    time::Duration,
};

/// A fixed loopback port used purely as a single-instance guard.
const SINGLE_INSTANCE_PORT: u16 = 47615;

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

    let ready = check_readiness();
    if !ready.ollama {
        eprintln!("\n[required] Ollama is not responding on 127.0.0.1:11434.");
        eprintln!("           The council cannot speak without it. Start Ollama, then relaunch.");
        pause_briefly();
        return;
    }
    if !ready.tts {
        eprintln!("\n[required] The offline voices are not installed.");
        eprintln!("           Run scripts\\setup_windows.ps1 to install them, then relaunch.");
        pause_briefly();
        return;
    }
    if !ready.comfy {
        eprintln!("\n[optional] ComfyUI + Chronos Director are not both up — council images");
        eprintln!("           will fail closed until you launch the Chronos Foundry. The");
        eprintln!("           chamber itself still runs.");
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
    comfy: bool,
}

fn check_readiness() -> Readiness {
    let ollama = port_open(11434);
    let director = port_open(7777);
    let comfy = port_open(8000);
    let tts = tts_installed();
    println!("Ollama (11434):     {}", status_word(ollama));
    println!("TTS voices:         {}", status_word(tts));
    println!("Chronos Director:   {}", status_word(director));
    println!("ComfyUI (8000):     {}", status_word(comfy));
    Readiness {
        ollama,
        tts,
        comfy: comfy && director,
    }
}

fn port_open(port: u16) -> bool {
    TcpStream::connect_timeout(
        &SocketAddr::from((Ipv4Addr::LOCALHOST, port)),
        Duration::from_millis(600),
    )
    .is_ok()
}

fn tts_installed() -> bool {
    let Some(root) = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .map(|program_files| program_files.join("Archetypes").join("speech"))
    else {
        return false;
    };
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
