use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

use super::paths::app_data_root;

#[derive(Serialize, Deserialize, Clone)]
pub struct LedgerEntry {
    pub timestamp: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub previous_hash: String,
    pub hash: String,
}

pub fn ledger_path() -> PathBuf {
    app_data_root().join("ledger.jsonl")
}

pub fn append_to_ledger(event_type: &str, payload: serde_json::Value) -> Result<(), String> {
    let path = ledger_path();
    let parent = path.parent().ok_or("No parent dir")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let previous_hash = if path.exists() {
        // Simple read of last line to get previous hash (in a real system this would be more robust)
        let content = fs::read_to_string(&path).unwrap_or_default();
        if let Some(last_line) = content.lines().last() {
            if let Ok(entry) = serde_json::from_str::<LedgerEntry>(last_line) {
                entry.hash
            } else {
                "0".to_owned()
            }
        } else {
            "0".to_owned()
        }
    } else {
        "0".to_owned()
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let pre_hash = format!("{}:{}:{}:{}", timestamp, event_type, payload.to_string(), previous_hash);
    let mut hasher = Sha256::new();
    hasher.update(pre_hash.as_bytes());
    let hash = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();

    let entry = LedgerEntry {
        timestamp,
        event_type: event_type.to_owned(),
        payload,
        previous_hash,
        hash,
    };

    let line = serde_json::to_string(&entry).map_err(|e| e.to_string())? + "\n";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    
    file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_hash_chaining() {
        let _ = fs::remove_file(ledger_path()); // Ensure clean state for test
        append_to_ledger("test_start", serde_json::json!({"test": 1})).unwrap();
        append_to_ledger("test_continue", serde_json::json!({"test": 2})).unwrap();
        
        let content = fs::read_to_string(ledger_path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        
        let e1: LedgerEntry = serde_json::from_str(lines[0]).unwrap();
        let e2: LedgerEntry = serde_json::from_str(lines[1]).unwrap();
        
        assert_eq!(e1.previous_hash, "0");
        assert_eq!(e2.previous_hash, e1.hash);
        let _ = fs::remove_file(ledger_path());
    }
}
