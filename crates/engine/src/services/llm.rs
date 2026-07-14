use serde_json::{json, Value};
use std::time::Duration;

const OLLAMA_CHAT_URL: &str = "http://127.0.0.1:11434/api/chat";
const OLLAMA_EMBED_URL: &str = "http://127.0.0.1:11434/api/embeddings";

pub fn ollama_model() -> String {
    std::env::var("ARCHETYPES_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:7b-instruct".to_owned())
}

pub fn ollama_chat(model: &str, system: &str, user: &str) -> Result<String, String> {
    let response = ureq::post(OLLAMA_CHAT_URL)
        .timeout(Duration::from_secs(90))
        .send_json(json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "stream": false,
            "options": {"temperature": 0.85, "num_predict": 200},
        }))
        .map_err(|error| format!("Ollama is unreachable ({error}). Is `ollama serve` running?"))?;
    let body: Value = response
        .into_json()
        .map_err(|error| format!("Ollama returned invalid JSON: {error}"))?;
    let text = body
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .ok_or("Ollama returned no message content")?
        .trim()
        .to_owned();
    if text.is_empty() {
        return Err("Ollama returned an empty council voice".to_owned());
    }
    Ok(text)
}

pub fn embed(text: &str) -> Result<Vec<f32>, String> {
    let model = ollama_model();
    let response = ureq::post(OLLAMA_EMBED_URL)
        .timeout(Duration::from_secs(30))
        .send_json(json!({
            "model": model,
            "prompt": text,
        }))
        .map_err(|error| format!("Ollama embeddings unreachable: {error}"))?;
    let body: Value = response
        .into_json()
        .map_err(|error| format!("Ollama returned invalid JSON: {error}"))?;
    let embedding = body
        .get("embedding")
        .and_then(|v| v.as_array())
        .ok_or("Ollama returned no embedding vector")?;
    
    let mut vec = Vec::with_capacity(embedding.len());
    for val in embedding {
        let f = val.as_f64().ok_or("Invalid float in embedding")? as f32;
        vec.push(f);
    }
    Ok(vec)
}
