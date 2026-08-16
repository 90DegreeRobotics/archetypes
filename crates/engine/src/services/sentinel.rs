//! In-engine Sentinel mediation.
//!
//! Live play asks Chronos Director to authorize a signed envelope before protected
//! work. Tests use a local deterministic guard so the workspace gate does not
//! require a live Director. Deny-all and unknown actions never authorize.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use ed25519_dalek::{Signer as _, SigningKey, VerifyingKey};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::chronos::DIRECTOR_URL;
use super::paths::app_data_root;

pub const SENTINEL_CLIENT_ACTOR: &str = "archetypes-launcher";
const SENTINEL_SEED_LEN: usize = 32;
const DIRECTOR_AUTHORIZE: &str = "/api/v1/authority/authorize";

/// Canonical Sentinel protected-action inventory (40). Keep in lockstep with
/// `docs/security/SENTINEL_PROTECTED_ACTIONS.md`.
pub const PROTECTED_ACTIONS: [&str; 40] = [
    "agent.spawn",
    "artifact.register",
    "artifact.export",
    "artifact.use",
    "browser.navigate_external",
    "capability.issue",
    "capability.consume",
    "chat.respond",
    "effect.execute",
    "external_message.send",
    "file.delete",
    "file.read_sensitive",
    "file.write",
    "game.respond",
    "game.share",
    "hardware.activate_camera",
    "hardware.activate_microphone",
    "identity.genesis",
    "identity.register",
    "identity.rebind",
    "identity.key.register",
    "identity.key.revoke",
    "identity.key.rotate",
    "installer.update",
    "memory.write",
    "memory.delete",
    "model.generate",
    "network.egress",
    "network.request",
    "payment.or_commitment",
    "plugin.install",
    "plugin.execute",
    "policy.evaluate",
    "process.spawn",
    "profile.generate",
    "robot.command",
    "shell.execute",
    "system.install",
    "tool.invoke",
    "tool.run",
];

/// Actions this product actually performs and therefore mediates at runtime.
const MEDIATED_RUNTIME: &[&str] = &[
    "artifact.register",
    "chat.respond",
    "file.write",
    "game.respond",
    "memory.write",
    "model.generate",
    "profile.generate",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardPolicy {
    DenyAll,
    MediatedRuntime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardDecision {
    Allow,
    Deny,
}

impl GuardDecision {
    pub fn authorizes_effect(self) -> bool {
        matches!(self, Self::Allow)
    }
}

pub fn authorize_local(policy: GuardPolicy, action: &str) -> GuardDecision {
    if !PROTECTED_ACTIONS.contains(&action) {
        return GuardDecision::Deny;
    }
    match policy {
        GuardPolicy::DenyAll => GuardDecision::Deny,
        GuardPolicy::MediatedRuntime => {
            if MEDIATED_RUNTIME.contains(&action) {
                GuardDecision::Allow
            } else {
                GuardDecision::Deny
            }
        }
    }
}

/// Fail-closed mediation. Tests use the local mediated-runtime policy so they do
/// not require Chronos. The live engine always asks Director.
pub fn mediate(action: &str, resource: &str, request: &Value) -> Result<(), String> {
    if cfg!(test) {
        return match authorize_local(GuardPolicy::MediatedRuntime, action) {
            GuardDecision::Allow => Ok(()),
            GuardDecision::Deny => Err(format!(
                "Sentinel denied {action} on {resource} (local fail-closed policy)"
            )),
        };
    }
    chronos_authorize(action, resource, request)
}

fn chronos_authorize(action: &str, resource: &str, request: &Value) -> Result<(), String> {
    if !PROTECTED_ACTIONS.contains(&action) {
        return Err(format!("unknown Sentinel action {action} is denied"));
    }
    if authorize_local(GuardPolicy::MediatedRuntime, action) != GuardDecision::Allow {
        return Err(format!(
            "Sentinel denied {action}: this product does not perform that work"
        ));
    }
    let signer = load_or_create_signer(&client_keystore_path())?;
    register_client(&signer)?;
    let mut body = json!({
        "archetype": "archetypes",
        "event_type": "archetypes_protected_action",
        "content": {
            "source": "archetypes_engine",
            "protected_action": action,
            "resource": resource,
            "request": request,
        }
    });
    let authority_request = json!({
        "target_archetype": "archetypes",
        "target_event_type": "archetypes_protected_action",
        "content_blake3": blake3::hash(body["content"].to_string().as_bytes()).to_hex().to_string(),
    });
    let payload = authority_payload("codex_append", action, &authority_request)?;
    let envelope = signer.seal(payload)?;
    body.as_object_mut()
        .ok_or_else(|| "Sentinel event body was not an object".to_owned())?
        .insert("auth".to_owned(), envelope);
    let receipt = http_post_json(
        &format!("{DIRECTOR_URL}/api/v1/codex/append"),
        &body,
        Duration::from_secs(8),
    )
    .map_err(|error| format!("Sentinel authorize failed for {action}: {error}"))?;
    if receipt.get("event_id").and_then(Value::as_str).is_none() {
        return Err(format!("Sentinel denied {action}: {receipt}"));
    }
    Ok(())
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

struct EngineSentinelSigner {
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

impl EngineSentinelSigner {
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

fn register_client(signer: &EngineSentinelSigner) -> Result<(), String> {
    let receipt = http_post_json(
        &format!("{DIRECTOR_URL}{DIRECTOR_AUTHORIZE}"),
        &signer.public_registration(),
        Duration::from_secs(8),
    )
    .map_err(|error| format!("Sentinel client-key registration failed: {error}"))?;
    if receipt.get("authorized").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "Chronos Sentinel did not authorize the engine key: {receipt}"
        ));
    }
    Ok(())
}

fn load_or_create_signer(path: &Path) -> Result<EngineSentinelSigner, String> {
    if path.exists() {
        let bytes = fs::read(path).map_err(|error| {
            format!(
                "could not read Sentinel engine key {}: {error}",
                path.display()
            )
        })?;
        if bytes.len() != SENTINEL_SEED_LEN {
            return Err(format!(
                "malformed Sentinel key at {}: expected {SENTINEL_SEED_LEN} bytes",
                path.display()
            ));
        }
        let mut seed = [0u8; SENTINEL_SEED_LEN];
        seed.copy_from_slice(&bytes);
        return Ok(EngineSentinelSigner::from_seed(seed));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut seed = [0u8; SENTINEL_SEED_LEN];
    getrandom::getrandom(&mut seed)
        .map_err(|error| format!("could not mint Sentinel engine key: {error}"))?;
    fs::write(path, seed).map_err(|error| error.to_string())?;
    Ok(EngineSentinelSigner::from_seed(seed))
}

fn key_id_for_verifying_key(key: &VerifyingKey) -> String {
    blake3::hash(&key.to_bytes()).to_hex().to_string()
}

fn client_keystore_path() -> PathBuf {
    app_data_root()
        .parent()
        .unwrap_or(app_data_root().as_path())
        .join("sentinel")
        .join("launcher_client.seed")
}

fn http_post_json(url: &str, body: &Value, timeout: Duration) -> Result<Value, String> {
    ureq::post(url)
        .timeout(timeout)
        .send_json(body.clone())
        .map_err(|error| error.to_string())?
        .into_json()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_all_paralyzes_every_protected_action() {
        for action in PROTECTED_ACTIONS {
            let decision = authorize_local(GuardPolicy::DenyAll, action);
            assert!(
                !decision.authorizes_effect(),
                "{action} must not run under deny-all"
            );
        }
    }

    #[test]
    fn unknown_action_is_denied_even_under_mediated_runtime_policy() {
        let decision = authorize_local(GuardPolicy::MediatedRuntime, "unknown.execute");
        assert!(!decision.authorizes_effect());
    }

    #[test]
    fn mediated_runtime_allows_only_wired_game_actions() {
        assert!(authorize_local(GuardPolicy::MediatedRuntime, "chat.respond").authorizes_effect());
        assert!(authorize_local(GuardPolicy::MediatedRuntime, "memory.write").authorizes_effect());
        assert!(!authorize_local(GuardPolicy::MediatedRuntime, "shell.execute").authorizes_effect());
        assert!(!authorize_local(GuardPolicy::MediatedRuntime, "system.install").authorizes_effect());
    }

    #[test]
    fn inventory_covers_forty_canonical_actions() {
        assert_eq!(PROTECTED_ACTIONS.len(), 40);
        let mut unique = PROTECTED_ACTIONS.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 40);
    }
}
