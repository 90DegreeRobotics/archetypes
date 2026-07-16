use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DIRECTOR_URL: &str = "http://127.0.0.1:7777";

pub fn comfyui_url() -> String {
    std::env::var("ARCHETYPES_COMFYUI_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_owned())
}

pub fn comfyui_output_dir() -> String {
    std::env::var("ARCHETYPES_COMFYUI_OUTPUT_DIR")
        .unwrap_or_else(|_| r"C:\Users\m\Documents\ComfyUI\output".to_owned())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactOutcome {
    pub status: String,
    pub job_id: Option<String>,
    pub artifact_id: Option<String>,
    pub png_path: Option<String>,
    pub proof_receipt_id: Option<String>,
    pub detail: String,
}

pub fn request_chronos_artifact(prompt: &str) -> ArtifactOutcome {
    request_chronos_artifact_with_style(
        prompt,
        "modern realistic digital painting, natural lighting and color, crisp detail, readable silhouette, no text",
        "archetypes-council",
    )
}

pub fn request_chronos_artifact_with_style(
    prompt: &str,
    style: &str,
    session_id: &str,
) -> ArtifactOutcome {
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

    let _ = ureq::post("http://127.0.0.1:11434/api/generate")
        .timeout(Duration::from_secs(20))
        .send_json(json!({
            "model": std::env::var("ARCHETYPES_OLLAMA_MODEL")
                .unwrap_or_else(|_| "qwen2.5:7b-instruct".to_owned()),
            "prompt": "",
            "keep_alive": 0,
        }));

    let submit: Value = match response_json(
        ureq::post(&format!("{DIRECTOR_URL}/api/v1/pipeline/concept-thumbnail"))
            .timeout(Duration::from_secs(480))
            .send_json(json!({
                "prompt": prompt,
                "style": style,
                "fidelity": "refined",
                "comfyui_url": comfyui_url(),
                "comfyui_output_dir": comfyui_output_dir(),
                "session_id": session_id,
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

pub fn resolve_chronos_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        Path::new("C:\\chronos").join(path)
    }
}
