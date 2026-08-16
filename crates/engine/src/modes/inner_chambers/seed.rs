use serde::{Deserialize, Serialize};
use std::fs;

use crate::services::paths::app_data_root;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeededTruth {
    pub words: [String; 3],
    pub node_index: usize,
    #[serde(default)]
    pub archetype: String,
}

fn seed_path() -> std::path::PathBuf {
    app_data_root().join("inner_chambers").join("last_truth.json")
}

pub fn persist_extracted_truth(
    words: [&str; 3],
    node_index: usize,
    archetype: &str,
) -> Result<(), String> {
    let path = seed_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let seeded = SeededTruth {
        words: [
            words[0].to_owned(),
            words[1].to_owned(),
            words[2].to_owned(),
        ],
        node_index,
        archetype: archetype.to_owned(),
    };
    fs::write(
        path,
        serde_json::to_string_pretty(&seeded).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

/// Consume the last Architect extraction once so it can seed a single Oracle round.
pub fn take_seeded_truth() -> Option<[String; 3]> {
    let path = seed_path();
    let body = fs::read_to_string(&path).ok()?;
    let seeded: SeededTruth = serde_json::from_str(&body).ok()?;
    let _ = fs::remove_file(&path);
    if seeded.words.iter().all(|word| !word.trim().is_empty()) {
        Some(seeded.words)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_truth_round_trips_three_concrete_words() {
        let seeded = SeededTruth {
            words: ["Order".into(), "Structure".into(), "Grid".into()],
            node_index: 2,
            archetype: "Luminous Blueprint".into(),
        };
        let encoded = serde_json::to_string(&seeded).unwrap();
        let decoded: SeededTruth = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, seeded);
        assert_eq!(decoded.words.len(), 3);
    }
}
