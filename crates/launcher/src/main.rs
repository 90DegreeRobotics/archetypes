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
    fs::{self, File},
    io::{Read, Write},
    net::{Ipv4Addr, TcpListener},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ed25519_dalek::{Signer as _, SigningKey};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// A fixed loopback port used purely as a single-instance guard.
const SINGLE_INSTANCE_PORT: u16 = 47615;
const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DIRECTOR_URL: &str = "http://127.0.0.1:7777";
const COMFY_URL: &str = "http://127.0.0.1:8000";
const SENTINEL_LAUNCH_LAW: &str = "Let there be no gate before the Sentinel";
const SENTINEL_LAUNCH_ACTION: &str = "archetypes.launch";
const SENTINEL_CLIENT_ACTOR: &str = "archetypes-launcher";
const SENTINEL_SEED_LEN: usize = 32;

fn main() {
    println!("Archetypes - Council Chamber");
    println!("============================");

    // Binding the guard port fails if another instance already holds it.
    let _guard = match TcpListener::bind((Ipv4Addr::LOCALHOST, SINGLE_INSTANCE_PORT)) {
        Ok(listener) => listener,
        Err(_) => {
            fail_visible(
                "Archetypes is already running",
                "A second launch was refused because the chamber is already open.\n\
                 Close the existing Archetypes window, then try again.",
                &engine_log_path(),
            );
            return;
        }
    };

    let engine = engine_path();
    if let Err(error) = ensure_sidecars() {
        fail_visible(
            "Archetypes could not start local services",
            &format!(
                "Ollama, Chronos Director, or ComfyUI were down and could not be started.\n\n{error}\n\n\
                 If ChronoSophia is installed, leave it running, then click Archetypes again."
            ),
            &engine_log_path(),
        );
        return;
    }

    if let Err(error) = authorize_sentinel_launch(&engine) {
        let body = format!(
            "The Archetypes game engine was not started.\n\n\
             Sentinel launch authorization failed before engine execution:\n{error}\n\n\
             Chronos Director must be running on 127.0.0.1:7777 with Sentinel in enforce mode.\n\
             Open HELP from the main menu after a successful launch, or read last-failure.txt."
        );
        fail_visible(
            "Archetypes Sentinel launch refused",
            &body,
            &engine_log_path(),
        );
        return;
    }

    if !tts_installed() {
        match repair_offline_voices() {
            Ok(()) => println!("Offline voices repaired."),
            Err(error) => {
                fail_visible(
                    "Archetypes voices missing",
                    &format!(
                        "The offline council voices are not installed and repair failed:\n{error}\n\n\
                         Re-run Uninstall Archetypes / the Desktop installer, or copy the speech bundle beside launcher.exe."
                    ),
                    &engine_log_path(),
                );
                return;
            }
        }
    }

    let ready = check_readiness();
    if !ready.ollama {
        let detail = ready
            .ollama_detail
            .as_deref()
            .unwrap_or("Ollama did not answer /api/tags");
        fail_visible(
            "Archetypes cannot hear Ollama",
            &format!(
                "Ollama is not responding on 127.0.0.1:11434.\n\
                 Start Ollama, confirm model qwen2.5:7b-instruct is installed, then relaunch.\n\n\
                 Detail: {detail}"
            ),
            &engine_log_path(),
        );
        return;
    }
    if !ready.tts {
        fail_visible(
            "Archetypes voices missing",
            "The offline voices are still not installed after repair.\n\
             Re-run the Archetypes installer so the speech bundle sits beside launcher.exe.",
            &engine_log_path(),
        );
        return;
    }
    if !ready.chronos_foundry {
        let detail = ready
            .chronos_detail
            .as_deref()
            .unwrap_or("Director or Comfy did not answer");
        fail_visible(
            "Archetypes cannot reach Chronos Foundry",
            &format!(
                "Chronos Foundry is not ready for play.\n\
                 Start Chronos Foundry so Director (:7777) reports readiness and ComfyUI answers on :8000, then relaunch.\n\n\
                 Detail: {detail}"
            ),
            &engine_log_path(),
        );
        return;
    }

    if !engine.is_file() {
        fail_visible(
            "Archetypes engine missing",
            &format!(
                "engine.exe was not found next to the launcher:\n{}",
                engine.display()
            ),
            &engine_log_path(),
        );
        return;
    }

    println!("\nEntering the chamber...");
    let log_path = engine_log_path();
    match run_engine(&engine, &log_path) {
        Ok(status) if status.success() => {
            let _ = fs::remove_file(logs_dir().join("last-failure.txt"));
        }
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

    let body = attach_sentinel_launch_auth(sentinel_launch_event_body(engine))?;
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

fn attach_sentinel_launch_auth(mut body: Value) -> Result<Value, String> {
    let signer = load_or_create_sentinel_signer(&sentinel_client_keystore_path())?;
    register_sentinel_client(&signer)?;
    let authority_request = codex_append_authority_request(&body)?;
    let payload = authority_payload("codex_append", "codex_append", &authority_request)?;
    let envelope = signer.seal(payload)?;
    let object = body
        .as_object_mut()
        .ok_or_else(|| "sentinel launch event body was not a JSON object".to_owned())?;
    object.insert("auth".to_owned(), envelope);
    Ok(body)
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

fn codex_append_authority_request(body: &Value) -> Result<Value, String> {
    let requested_archetype = body
        .get("archetype")
        .and_then(Value::as_str)
        .unwrap_or("chronos_director");
    let event_type = nonempty_json_str(body, "event_type")
        .ok_or_else(|| "launch append body has no event_type".to_owned())?;
    let content = body.get("content").cloned().unwrap_or(Value::Null);
    let content_digest = blake3::hash(content.to_string().as_bytes())
        .to_hex()
        .to_string();
    Ok(json!({
        "target_archetype": requested_archetype,
        "target_event_type": event_type,
        "target_layer": body.get("layer").cloned().unwrap_or(Value::Null),
        "content_blake3": content_digest,
        "causation": body.get("causation").cloned().unwrap_or(Value::Null),
    }))
}

fn authority_payload(action: &str, resource: &str, request: &Value) -> Result<Value, String> {
    let bytes = serde_json::to_vec(request)
        .map_err(|error| format!("could not canonicalize Sentinel request: {error}"))?;
    Ok(json!({
        "action": action,
        "resource": resource,
        "request_blake3": blake3::hash(&bytes).to_hex().to_string(),
    }))
}

fn register_sentinel_client(signer: &LauncherSentinelSigner) -> Result<(), String> {
    let registration = signer.public_registration();
    let receipt = http_post_json(
        &format!("{DIRECTOR_URL}/api/v1/authority/authorize"),
        &registration,
        Duration::from_secs(8),
    )
    .map_err(|error| format!("Chronos Sentinel client-key registration failed: {error}"))?;
    if receipt.get("authorized").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "Chronos Sentinel did not authorize the launcher key: {receipt}"
        ));
    }
    let returned_key_id = nonempty_json_str(&receipt, "key_id")
        .ok_or_else(|| format!("Chronos Sentinel registration returned no key_id: {receipt}"))?;
    if returned_key_id != signer.key_id {
        return Err(format!(
            "Chronos Sentinel registered key_id {returned_key_id}, expected {}",
            signer.key_id
        ));
    }
    Ok(())
}

struct LauncherSentinelSigner {
    signing_key: SigningKey,
    key_id: String,
}

#[derive(Serialize)]
struct SentinelSignature {
    algorithm: &'static str,
    bytes: Vec<u8>,
}

#[derive(Serialize)]
struct SignedAuthorityEnvelope {
    actor_id: String,
    key_id: String,
    nonce: Uuid,
    payload: Value,
    signature: SentinelSignature,
}

impl LauncherSentinelSigner {
    fn from_seed(seed: [u8; SENTINEL_SEED_LEN]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        let key_id = key_id_for_verifying_key(&signing_key.verifying_key());
        Self {
            signing_key,
            key_id,
        }
    }

    fn public_registration(&self) -> Value {
        json!({
            "actor": SENTINEL_CLIENT_ACTOR,
            "key_id": self.key_id.as_str(),
            "verifying_key_hex": hex::encode(self.signing_key.verifying_key().to_bytes()),
        })
    }

    fn seal(&self, payload: Value) -> Result<Value, String> {
        let actor_id = SENTINEL_CLIENT_ACTOR.to_owned();
        let nonce = Uuid::new_v4();
        let bytes = serde_json::to_vec(&(&actor_id, &self.key_id, &nonce, &payload))
            .map_err(|error| format!("could not canonicalize Sentinel envelope: {error}"))?;
        let signature = self.signing_key.sign(&bytes);
        serde_json::to_value(SignedAuthorityEnvelope {
            actor_id,
            key_id: self.key_id.clone(),
            nonce,
            payload,
            signature: SentinelSignature {
                algorithm: "Ed25519",
                bytes: signature.to_bytes().to_vec(),
            },
        })
        .map_err(|error| format!("could not encode Sentinel envelope: {error}"))
    }
}

fn load_or_create_sentinel_signer(path: &Path) -> Result<LauncherSentinelSigner, String> {
    if path.exists() {
        let bytes = fs::read(path).map_err(|error| {
            format!(
                "could not read Sentinel launcher key {}: {error}",
                path.display()
            )
        })?;
        if bytes.len() != SENTINEL_SEED_LEN {
            return Err(format!(
                "malformed Sentinel launcher key at {}: expected {SENTINEL_SEED_LEN} bytes, found {}",
                path.display(),
                bytes.len()
            ));
        }
        let mut seed = [0u8; SENTINEL_SEED_LEN];
        seed.copy_from_slice(&bytes);
        return Ok(LauncherSentinelSigner::from_seed(seed));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "could not create Sentinel launcher key directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let mut seed = [0u8; SENTINEL_SEED_LEN];
    getrandom::getrandom(&mut seed)
        .map_err(|error| format!("could not mint Sentinel launcher key: {error}"))?;
    fs::write(path, &seed).map_err(|error| {
        format!(
            "could not write Sentinel launcher key {}: {error}",
            path.display()
        )
    })?;
    Ok(LauncherSentinelSigner::from_seed(seed))
}

fn key_id_for_verifying_key(key: &ed25519_dalek::VerifyingKey) -> String {
    blake3::hash(&key.to_bytes()).to_hex().to_string()
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

fn sentinel_client_keystore_path() -> PathBuf {
    app_data_dir().join("sentinel").join("launcher_client.seed")
}

fn logs_dir() -> PathBuf {
    app_data_dir().join("logs")
}

fn app_data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("NeuroCognica").join("Archetypes")
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

fn ensure_sidecars() -> Result<(), String> {
    if !probe_ollama().0 {
        spawn_detached(Command::new("ollama").arg("serve"), "Ollama")?;
        wait_for("Ollama", Duration::from_secs(20), || probe_ollama().0)?;
    }
    if !probe_director().0 {
        start_director()?;
        wait_for("Chronos Director", Duration::from_secs(45), || probe_director().0)?;
    }
    if !probe_comfy().0 {
        start_comfy()?;
        wait_for("ComfyUI", Duration::from_secs(120), || probe_comfy().0)?;
    }
    Ok(())
}

fn start_director() -> Result<(), String> {
    if let Some(exe) = find_director_exe() {
        let root = chronos_root();
        let data = root.join("data");
        let _ = fs::create_dir_all(&data);
        let mut command = Command::new(&exe);
        command
            .current_dir(&root)
            .args([
                "--port",
                "7777",
                "--codex",
                data.join("codex.db").to_str().unwrap_or("data/codex.db"),
                "--conv-codex",
                data.join("conversation.db")
                    .to_str()
                    .unwrap_or("data/conversation.db"),
                "--renders-dir",
                root.join("renders").to_str().unwrap_or("renders"),
                "--ollama-url",
                OLLAMA_URL,
            ]);
        if let Some(cli) = find_chronos_cli() {
            command.arg("--chronos-exe").arg(cli);
        }
        return spawn_detached(&mut command, "Chronos Director");
    }
    if let Some(foundry) = find_foundry_script() {
        let pythonw = find_pythonw().ok_or_else(|| {
            format!(
                "Chronos Director is not running and pythonw.exe was not found to start {}",
                foundry.display()
            )
        })?;
        let mut command = Command::new(pythonw);
        command.arg(&foundry).current_dir(chronos_root());
        return spawn_detached(&mut command, "ChronoSophia Foundry");
    }
    Err(format!(
        "Chronos Director was not found. Looked under {} for chronos_director.exe and desktop/chronos_foundry.py.",
        chronos_root().display()
    ))
}

fn start_comfy() -> Result<(), String> {
    let comfy_dir = find_comfy_dir().ok_or_else(|| {
        "ComfyUI main.py was not found. Set ARCHETYPES_COMFYUI_DIR or install ComfyUI under Documents\\ComfyUI."
            .to_owned()
    })?;
    let python = find_comfy_python(&comfy_dir).ok_or_else(|| {
        format!(
            "No Python interpreter found to start ComfyUI at {}",
            comfy_dir.display()
        )
    })?;
    let data_dir = comfy_data_dir(&comfy_dir);
    let mut command = Command::new(python);
    command
        .current_dir(&comfy_dir)
        .args(["main.py", "--listen", "127.0.0.1", "--port", "8000"]);
    if data_dir.is_dir() {
        command.arg("--base-directory").arg(&data_dir);
    }
    spawn_detached(&mut command, "ComfyUI")
}

fn spawn_detached(command: &mut Command, label: &str) -> Result<(), String> {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;
        command.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
    }
    command
        .spawn()
        .map(|child| {
            std::mem::forget(child);
        })
        .map_err(|error| format!("could not start {label}: {error}"))
}

fn wait_for(label: &str, budget: Duration, mut ready: impl FnMut() -> bool) -> Result<(), String> {
    let started = SystemTime::now();
    loop {
        if ready() {
            return Ok(());
        }
        let elapsed = started.elapsed().unwrap_or_default();
        if elapsed >= budget {
            return Err(format!(
                "{label} did not become ready within {}s",
                budget.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn chronos_root() -> PathBuf {
    std::env::var_os("ARCHETYPES_CHRONOS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\chronos"))
}

fn director_candidates(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("target/release/chronos_director.exe"),
        root.join("target/debug/chronos_director.exe"),
        root.join("tmp/hotfix_stage/target/release/chronos_director.exe"),
        root.join("chronos_director.exe"),
    ]
}

fn find_director_exe() -> Option<PathBuf> {
    director_candidates(&chronos_root())
        .into_iter()
        .find(|path| path.is_file())
}

fn find_chronos_cli() -> Option<PathBuf> {
    let root = chronos_root();
    [
        root.join("target/release/chronos_cli.exe"),
        root.join("target/release/chronos.exe"),
        root.join("tmp/hotfix_stage/target/release/chronos_cli.exe"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

fn find_foundry_script() -> Option<PathBuf> {
    let path = chronos_root().join("desktop").join("chronos_foundry.py");
    path.is_file().then_some(path)
}

fn find_pythonw() -> Option<PathBuf> {
    which_on_path("pythonw.exe").or_else(|| {
        std::env::var_os("LOCALAPPDATA").and_then(|root| {
            let dir = PathBuf::from(root).join("Programs").join("Python");
            let mut versions = fs::read_dir(dir).ok()?.collect::<Vec<_>>();
            versions.sort_by_key(|entry| entry.as_ref().ok().map(|e| e.file_name()));
            versions.into_iter().rev().find_map(|entry| {
                let pythonw = entry.ok()?.path().join("pythonw.exe");
                pythonw.is_file().then_some(pythonw)
            })
        })
    })
}

fn find_comfy_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(explicit) = std::env::var("ARCHETYPES_COMFYUI_DIR") {
        candidates.push(PathBuf::from(explicit));
    }
    if let Ok(explicit) = std::env::var("CHRONOS_COMFYUI_DIR") {
        candidates.push(PathBuf::from(explicit));
    }
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        let docs = PathBuf::from(profile).join("Documents").join("ComfyUI");
        candidates.push(docs.clone());
        candidates.push(docs.join("ComfyUI"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local)
                .join("Programs")
                .join("ComfyUI")
                .join("resources")
                .join("ComfyUI"),
        );
    }
    candidates.into_iter().find(|path| path.join("main.py").is_file())
}

fn comfy_data_dir(comfy_dir: &Path) -> PathBuf {
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        let docs = PathBuf::from(profile).join("Documents").join("ComfyUI");
        if docs.is_dir() {
            return docs;
        }
    }
    comfy_dir.to_path_buf()
}

fn find_comfy_python(comfy_dir: &Path) -> Option<PathBuf> {
    let venv = comfy_data_dir(comfy_dir)
        .join(".venv")
        .join("Scripts")
        .join("python.exe");
    if venv.is_file() {
        return Some(venv);
    }
    let local_venv = comfy_dir.join(".venv").join("Scripts").join("python.exe");
    if local_venv.is_file() {
        return Some(local_venv);
    }
    which_on_path("python.exe")
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let candidate = dir.join(name);
        candidate.is_file().then_some(candidate)
    })
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

fn dependencies_manifest_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.join("scripts").join("dependencies.json")))
        .unwrap_or_else(|| PathBuf::from("scripts/dependencies.json"))
}

fn repair_offline_voices() -> Result<(), String> {
    let install_root = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .ok_or_else(|| "could not locate launcher directory".to_owned())?;
    let speech_root = install_root.join("speech");
    fs::create_dir_all(&speech_root).map_err(|error| error.to_string())?;

    let manifest_path = dependencies_manifest_path();
    let body = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "dependency manifest missing at {}: {error}",
            manifest_path.display()
        )
    })?;
    let manifest: Value =
        serde_json::from_str(&body).map_err(|error| format!("invalid dependencies.json: {error}"))?;
    let artifacts = manifest
        .get("download_artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| "dependencies.json has no download_artifacts".to_owned())?;

    for artifact in artifacts {
        let name = artifact
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("artifact");
        let url = artifact
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{name} has no url"))?;
        let expected = artifact
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{name} has no sha256"))?;
        let expected_rel = if name.contains("sherpa-onnx") {
            "sherpa-onnx-v1.13.4-win-x64-shared-MD-Release/bin/sherpa-onnx-offline-tts.exe"
        } else {
            "kokoro-en-v0_19/model.onnx"
        };
        if speech_root.join(expected_rel).is_file() {
            continue;
        }
        let archive = std::env::temp_dir().join(format!("archetypes-{expected}.tar.bz2"));
        download_url(url, &archive)?;
        let actual = sha256_file(&archive)?;
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!(
                "hash mismatch for {name}: expected {expected}, got {actual}"
            ));
        }
        let status = Command::new("tar.exe")
            .args(["-xf", &archive.display().to_string(), "-C"])
            .arg(&speech_root)
            .status()
            .map_err(|error| format!("could not run tar.exe: {error}"))?;
        if !status.success() {
            return Err(format!("tar.exe failed extracting {name}"));
        }
    }

    if speech_root_ready(&speech_root) {
        Ok(())
    } else {
        Err("voices extracted but readiness paths are still missing".to_owned())
    }
}

fn download_url(url: &str, dest: &Path) -> Result<(), String> {
    let mut reader = ureq::get(url)
        .timeout(Duration::from_secs(180))
        .call()
        .map_err(|error| format!("download failed: {error}"))?
        .into_reader();
    let mut file = File::create(dest).map_err(|error| error.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|error| error.to_string())?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode_upper(hasher.finalize()))
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

    #[test]
    fn codex_append_authority_request_binds_launch_content() {
        let first = json!({
            "archetype": "archetypes",
            "event_type": "archetypes_launch_requested",
            "content": {
                "source": "archetypes_launcher",
                "protected_action": SENTINEL_LAUNCH_ACTION,
                "law": SENTINEL_LAUNCH_LAW,
                "engine_path": r"C:\archetypes\dist\engine.exe",
                "requested_unix_ms": 1000_u64,
            }
        });
        let second = json!({
            "archetype": "archetypes",
            "event_type": "archetypes_launch_requested",
            "content": {
                "source": "archetypes_launcher",
                "protected_action": SENTINEL_LAUNCH_ACTION,
                "law": SENTINEL_LAUNCH_LAW,
                "engine_path": r"C:\archetypes\dist\engine.exe",
                "requested_unix_ms": 2000_u64,
            }
        });

        let first_request = codex_append_authority_request(&first).unwrap();
        let second_request = codex_append_authority_request(&second).unwrap();

        assert_eq!(first_request["target_archetype"], "archetypes");
        assert_eq!(
            first_request["target_event_type"],
            "archetypes_launch_requested"
        );
        assert_eq!(first_request["target_layer"], Value::Null);
        assert_eq!(first_request["causation"], Value::Null);
        assert!(first_request["content_blake3"]
            .as_str()
            .is_some_and(|digest| digest.len() == 64));
        assert_ne!(
            first_request["content_blake3"], second_request["content_blake3"],
            "launch content digest must be body-sensitive"
        );
    }

    #[test]
    fn sentinel_payload_digest_is_request_sensitive() {
        let request_a = json!({"content_blake3": "a"});
        let request_b = json!({"content_blake3": "b"});
        let payload_a = authority_payload("codex_append", "codex_append", &request_a).unwrap();
        let payload_b = authority_payload("codex_append", "codex_append", &request_b).unwrap();

        assert_eq!(payload_a["action"], "codex_append");
        assert_eq!(payload_a["resource"], "codex_append");
        assert!(payload_a["request_blake3"]
            .as_str()
            .is_some_and(|digest| digest.len() == 64));
        assert_ne!(payload_a["request_blake3"], payload_b["request_blake3"]);
    }

    #[test]
    fn sentinel_envelope_and_registration_match_chronos_contract() {
        let signer = LauncherSentinelSigner::from_seed([7u8; SENTINEL_SEED_LEN]);
        let registration = signer.public_registration();
        assert_eq!(registration["actor"], SENTINEL_CLIENT_ACTOR);
        assert_eq!(registration["key_id"], signer.key_id.as_str());
        assert!(registration["verifying_key_hex"]
            .as_str()
            .is_some_and(|value| value.len() == 64));

        let payload = json!({
            "action": "codex_append",
            "resource": "codex_append",
            "request_blake3": "0".repeat(64),
        });
        let envelope = signer.seal(payload.clone()).unwrap();

        assert_eq!(envelope["actor_id"], SENTINEL_CLIENT_ACTOR);
        assert_eq!(envelope["key_id"], signer.key_id.as_str());
        assert_eq!(envelope["payload"], payload);
        assert_eq!(envelope["signature"]["algorithm"], "Ed25519");
        assert!(envelope["nonce"].as_str().is_some());
        assert_eq!(
            envelope["signature"]["bytes"].as_array().unwrap().len(),
            64,
            "Ed25519 signature is 64 bytes"
        );
    }

    #[test]
    fn dependencies_manifest_is_expected_beside_the_launcher() {
        let path = dependencies_manifest_path();
        let text = path.to_string_lossy().replace('\\', "/").to_lowercase();
        assert!(
            text.ends_with("scripts/dependencies.json"),
            "{text}"
        );
    }

    #[test]
    fn sha256_file_is_stable_for_known_bytes() {
        let path = std::env::temp_dir().join(format!("archetypes-sha-{}", Uuid::new_v4()));
        std::fs::write(&path, b"archetypes").unwrap();
        let digest = sha256_file(&path).unwrap();
        assert_eq!(digest.len(), 64);
        assert_eq!(digest, sha256_file(&path).unwrap());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sentinel_signer_keystore_is_durable_and_strict() {
        let root =
            std::env::temp_dir().join(format!("archetypes-launcher-sentinel-{}", Uuid::new_v4()));
        let path = root.join("sentinel").join("launcher_client.seed");

        let first = load_or_create_sentinel_signer(&path).unwrap();
        let second = load_or_create_sentinel_signer(&path).unwrap();
        assert_eq!(first.key_id, second.key_id);
        assert_eq!(fs::read(&path).unwrap().len(), SENTINEL_SEED_LEN);

        fs::write(&path, b"bad").unwrap();
        let error = match load_or_create_sentinel_signer(&path) {
            Ok(_) => panic!("malformed Sentinel launcher key was accepted"),
            Err(error) => error,
        };
        assert!(error.contains("malformed Sentinel launcher key"), "{error}");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn director_candidates_stay_under_chronos_root() {
        let root = PathBuf::from(r"C:\chronos");
        let candidates = director_candidates(&root);
        assert!(candidates.iter().all(|path| path.starts_with(&root)));
        assert!(candidates
            .iter()
            .any(|path| path.ends_with("chronos_director.exe")));
    }

    #[test]
    fn comfy_discovery_uses_userprofile_not_a_baked_account() {
        let source = include_str!("main.rs");
        assert!(source.contains("USERPROFILE"));
        assert!(source.contains("ARCHETYPES_COMFYUI_DIR"));
        let baked = ["Users", "m", "Documents", "ComfyUI"].join("\\");
        assert!(
            !source.contains(&format!("C:\\{baked}")),
            "Comfy discovery must not bake a single operator account"
        );
    }
}
