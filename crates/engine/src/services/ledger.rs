use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::paths::app_data_root;
use crate::modes::game_mode::GameMode;

#[derive(Serialize, Deserialize, Clone)]
pub struct LedgerEntry {
    pub timestamp: u64,
    pub mode: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub previous_hash: String,
    pub hash: String,
}

pub fn ledger_path() -> PathBuf {
    app_data_root().join("ledger.jsonl")
}

pub fn append_to_ledger(
    mode: GameMode,
    kind: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    append_to_path(ledger_path(), mode, kind, payload)
}

fn append_to_path(
    path: PathBuf,
    mode: GameMode,
    kind: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let parent = path.parent().ok_or("No parent dir")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let previous_hash = last_hash(&path)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let mode = mode.id().to_owned();
    let kind = kind.to_owned();
    let hash = hash_entry(timestamp, &mode, &kind, &payload, &previous_hash);

    let entry = LedgerEntry {
        timestamp,
        mode,
        kind,
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

pub fn verify_ledger() -> Result<(), String> {
    verify_ledger_path(ledger_path())
}

fn verify_ledger_path(path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let file = fs::File::open(&path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut expected_previous = "0".to_owned();
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: LedgerEntry = serde_json::from_str(&line)
            .map_err(|error| format!("ledger line {} is invalid JSON: {error}", index + 1))?;
        if entry.previous_hash != expected_previous {
            return Err(format!(
                "ledger line {} expected previous hash {}, got {}",
                index + 1,
                expected_previous,
                entry.previous_hash
            ));
        }
        let expected_hash = hash_entry(
            entry.timestamp,
            &entry.mode,
            &entry.kind,
            &entry.payload,
            &entry.previous_hash,
        );
        if entry.hash != expected_hash {
            return Err(format!("ledger line {} hash mismatch", index + 1));
        }
        expected_previous = entry.hash;
    }
    Ok(())
}

fn last_hash(path: &PathBuf) -> Result<String, String> {
    if !path.exists() {
        return Ok("0".to_owned());
    }
    verify_ledger_path(path.clone())?;
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let Some(last_line) = content.lines().rev().find(|line| !line.trim().is_empty()) else {
        return Ok("0".to_owned());
    };
    let entry: LedgerEntry = serde_json::from_str(last_line)
        .map_err(|error| format!("ledger tail is invalid JSON: {error}"))?;
    Ok(entry.hash)
}

fn hash_entry(
    timestamp: u64,
    mode: &str,
    kind: &str,
    payload: &serde_json::Value,
    previous_hash: &str,
) -> String {
    let pre_hash = format!(
        "{}:{}:{}:{}:{}",
        timestamp, mode, kind, payload, previous_hash
    );
    let mut hasher = Sha256::new();
    hasher.update(pre_hash.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_hash_chaining_and_verification() {
        let path = test_ledger_path("chain");
        let _ = fs::remove_file(&path);
        append_to_path(
            path.clone(),
            GameMode::Standard,
            "test_start",
            serde_json::json!({"test": 1}),
        )
        .unwrap();
        append_to_path(
            path.clone(),
            GameMode::Standard,
            "test_continue",
            serde_json::json!({"test": 2}),
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let e1: LedgerEntry = serde_json::from_str(lines[0]).unwrap();
        let e2: LedgerEntry = serde_json::from_str(lines[1]).unwrap();

        assert_eq!(e1.previous_hash, "0");
        assert_eq!(e2.previous_hash, e1.hash);
        assert_eq!(e1.mode, "standard");
        assert_eq!(e2.kind, "test_continue");
        verify_ledger_path(path.clone()).unwrap();
        let _ = fs::remove_file(path);
    }

    #[test]
    fn ledger_verification_rejects_tampering() {
        let path = test_ledger_path("tamper");
        let _ = fs::remove_file(&path);
        append_to_path(
            path.clone(),
            GameMode::Standard,
            "test_start",
            serde_json::json!({"test": 1}),
        )
        .unwrap();

        let tampered = fs::read_to_string(&path)
            .unwrap()
            .replace("\"test\":1", "\"test\":9");
        fs::write(&path, tampered).unwrap();

        assert!(verify_ledger_path(path.clone()).is_err());
        let _ = fs::remove_file(path);
    }

    fn test_ledger_path(stem: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "archetypes-{stem}-{}-ledger.jsonl",
            std::process::id()
        ))
    }
}
