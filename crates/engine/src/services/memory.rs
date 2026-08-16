//! Persistent world memory — accepted artifacts become lineage in the chamber.
//!
//! Each completed Chronos return is sealed here with provenance and ancestry.
//! The chamber recalls the lineage on the next session and manifests tokens
//! around the portal table so the world is visibly changed.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use super::paths::app_data_root;
use super::sentinel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryShard {
    pub id: String,
    pub parent_id: Option<String>,
    pub offering: String,
    pub verdict: String,
    pub artifact_id: Option<String>,
    pub png_path: Option<String>,
    pub proof_receipt_id: Option<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct WorldMemory {
    pub shards: Vec<MemoryShard>,
}

pub fn memory_path() -> PathBuf {
    app_data_root().join("world_memory.json")
}

pub fn load_world_memory() -> WorldMemory {
    let path = memory_path();
    let Ok(body) = fs::read_to_string(&path) else {
        return WorldMemory::default();
    };
    serde_json::from_str(&body).unwrap_or_default()
}

pub fn remember_artifact(
    offering: &str,
    verdict: &str,
    artifact_id: Option<String>,
    png_path: Option<String>,
    proof_receipt_id: Option<String>,
) -> Result<MemoryShard, String> {
    let request = serde_json::json!({
        "kind": "world_memory_append",
        "artifact_id": artifact_id,
    });
    sentinel::mediate(
        "memory.write",
        "archetypes://world_memory",
        &request,
    )?;

    let mut memory = load_world_memory();
    let parent_id = memory.shards.last().map(|shard| shard.id.clone());
    let id = artifact_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("mem-{}", memory.shards.len() + 1));
    let hash = hash_shard(
        &id,
        parent_id.as_deref(),
        offering,
        verdict,
        png_path.as_deref(),
    );
    let shard = MemoryShard {
        id,
        parent_id,
        offering: offering.to_owned(),
        verdict: verdict.to_owned(),
        artifact_id,
        png_path,
        proof_receipt_id,
        hash,
    };
    memory.shards.push(shard.clone());
    persist_world_memory(&memory)?;
    Ok(shard)
}

fn persist_world_memory(memory: &WorldMemory) -> Result<(), String> {
    let path = memory_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let body = serde_json::to_string_pretty(memory).map_err(|error| error.to_string())?;
    fs::write(path, format!("{body}\n")).map_err(|error| error.to_string())
}

fn hash_shard(
    id: &str,
    parent_id: Option<&str>,
    offering: &str,
    verdict: &str,
    png_path: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(parent_id.unwrap_or("0").as_bytes());
    hasher.update(offering.as_bytes());
    hasher.update(verdict.as_bytes());
    hasher.update(png_path.unwrap_or("").as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lineage_chains_parent_to_child() {
        let first = hash_shard("a", None, "offer", "verdict", None);
        let second = hash_shard("b", Some("a"), "offer-2", "verdict-2", Some("x.png"));
        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
        assert_eq!(second.len(), 64);
    }

    #[test]
    fn empty_memory_has_no_shards() {
        let memory = WorldMemory::default();
        assert!(memory.shards.is_empty());
    }
}
