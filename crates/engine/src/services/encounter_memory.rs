//! Mode-neutral consentful memory for archetype encounters.
//!
//! A spoken or typed turn is not automatically remembered. Every completed turn is logged
//! as `Transient` for audit/provenance, but only becomes part of the player's recallable
//! history after an explicit `Remember`. `Forget` appends an auditable withdrawal event
//! that removes a record from recall forever — it does not rewrite or delete history, so
//! the withdrawal itself stays provable, but no reader of `recallable_records` (RAG, a
//! future prompt, a "view your history" screen) can see forgotten content again.
//!
//! The journal is an append-only JSONL event log, deliberately mirroring the tamper-evident
//! shape of `services::ledger`: state is never mutated in place, only superseded by a later
//! event. "Restart-safe" therefore falls out for free — recall is recomputed by folding the
//! file from scratch every time, so there is no in-memory cache to go stale or lie.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::ledger::append_to_ledger;
use super::paths::app_data_root;
use crate::modes::game_mode::GameMode;

pub const ENCOUNTER_RECORD_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionState {
    Transient,
    Remembered,
    Forgotten,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterRecord {
    pub id: String,
    pub version: u32,
    pub archetype: String,
    pub capability: String,
    pub created_at_utc_ms: u64,
    pub player_input: String,
    pub response: String,
    pub source_refs: Vec<String>,
    pub content_hash: String,
}

impl EncounterRecord {
    pub fn new(archetype: &str, capability: &str, player_input: &str, response: &str) -> Self {
        let created_at_utc_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let id = uuid::Uuid::new_v4().to_string();
        let content_hash =
            content_hash(&id, archetype, capability, created_at_utc_ms, player_input, response);
        Self {
            id,
            version: ENCOUNTER_RECORD_VERSION,
            archetype: archetype.to_owned(),
            capability: capability.to_owned(),
            created_at_utc_ms,
            player_input: player_input.to_owned(),
            response: response.to_owned(),
            source_refs: Vec::new(),
            content_hash,
        }
    }
}

fn content_hash(
    id: &str,
    archetype: &str,
    capability: &str,
    created_at_utc_ms: u64,
    player_input: &str,
    response: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(archetype.as_bytes());
    hasher.update(capability.as_bytes());
    hasher.update(created_at_utc_ms.to_string().as_bytes());
    hasher.update(player_input.as_bytes());
    hasher.update(response.as_bytes());
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum EncounterEvent {
    Created { record: EncounterRecord },
    Remembered { id: String, at_utc_ms: u64 },
    Forgotten { id: String, at_utc_ms: u64, reason: Option<String> },
}

pub fn encounters_dir() -> PathBuf {
    app_data_root().join("encounters")
}

pub fn journal_path() -> PathBuf {
    encounters_dir().join("journal.jsonl")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Appends one line to the journal file only. Ledger sealing is a separate step (see
/// callers below) so tests can exercise the journal/retention-state logic against an
/// isolated scratch file without racing every other test in the binary over the one
/// real, process-wide, hash-chained `ledger.jsonl`.
fn append_event(path: &Path, event: &EncounterEvent) -> Result<(), String> {
    let parent = path.parent().ok_or("journal path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let line = serde_json::to_string(event).map_err(|error| error.to_string())? + "\n";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes()).map_err(|error| error.to_string())
}

fn seal_in_ledger(kind: &str, event: &EncounterEvent) -> Result<(), String> {
    let payload = match event {
        EncounterEvent::Created { record } => serde_json::json!({
            "id": record.id, "archetype": record.archetype, "capability": record.capability,
            "content_hash": record.content_hash,
        }),
        EncounterEvent::Remembered { id, .. } => serde_json::json!({ "id": id }),
        EncounterEvent::Forgotten { id, reason, .. } => serde_json::json!({ "id": id, "reason": reason }),
    };
    append_to_ledger(GameMode::InnerChambers, kind, payload)
}

/// Logs a completed turn as `Transient`. Transient turns are provable and inspectable
/// (they are on disk, hash-receipted, and ledger-sealed) but are excluded from
/// `recallable_records` until the player explicitly remembers them.
pub fn record_turn(archetype: &str, capability: &str, player_input: &str, response: &str) -> Result<EncounterRecord, String> {
    let record = EncounterRecord::new(archetype, capability, player_input, response);
    let event = EncounterEvent::Created { record: record.clone() };
    append_event(&journal_path(), &event)?;
    seal_in_ledger("inner_castle_encounter_created", &event)?;
    Ok(record)
}

#[cfg(test)]
fn record_turn_at(path: &Path, archetype: &str, capability: &str, player_input: &str, response: &str) -> Result<EncounterRecord, String> {
    let record = EncounterRecord::new(archetype, capability, player_input, response);
    append_event(path, &EncounterEvent::Created { record: record.clone() })?;
    Ok(record)
}

/// Explicit player consent to keep a turn as part of their recallable history.
pub fn remember(id: &str) -> Result<(), String> {
    let event = EncounterEvent::Remembered { id: id.to_owned(), at_utc_ms: now_ms() };
    append_event(&journal_path(), &event)?;
    seal_in_ledger("inner_castle_encounter_remembered", &event)
}

#[cfg(test)]
fn remember_at(path: &Path, id: &str) -> Result<(), String> {
    append_event(path, &EncounterEvent::Remembered { id: id.to_owned(), at_utc_ms: now_ms() })
}

/// Auditable withdrawal. A forgotten record is never removed from the append-only journal
/// (the withdrawal itself must stay provable), but it is permanently excluded from
/// `recallable_records`, so nothing downstream can feed it back into a prompt.
pub fn forget(id: &str, reason: Option<&str>) -> Result<(), String> {
    let event = EncounterEvent::Forgotten { id: id.to_owned(), at_utc_ms: now_ms(), reason: reason.map(str::to_owned) };
    append_event(&journal_path(), &event)?;
    seal_in_ledger("inner_castle_encounter_forgotten", &event)
}

#[cfg(test)]
fn forget_at(path: &Path, id: &str, reason: Option<&str>) -> Result<(), String> {
    append_event(path, &EncounterEvent::Forgotten { id: id.to_owned(), at_utc_ms: now_ms(), reason: reason.map(str::to_owned) })
}

fn load_events(path: &Path) -> Result<Vec<EncounterEvent>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let event: EncounterEvent = serde_json::from_str(&line)
            .map_err(|error| format!("encounter journal line {} is invalid JSON: {error}", index + 1))?;
        events.push(event);
    }
    Ok(events)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordStatus {
    pub record: EncounterRecord,
    pub retention_state: RetentionState,
}

/// Folds the full journal into the current status of every record that was ever created,
/// most-recent transition wins per id (a later `Forgotten` always beats an earlier
/// `Remembered`). This is the single source of truth `view_record` reads from.
fn fold_status(events: &[EncounterEvent]) -> HashMap<String, RecordStatus> {
    let mut statuses: HashMap<String, RecordStatus> = HashMap::new();
    for event in events {
        match event {
            EncounterEvent::Created { record } => {
                statuses.insert(record.id.clone(), RecordStatus { record: record.clone(), retention_state: RetentionState::Transient });
            }
            EncounterEvent::Remembered { id, .. } => {
                if let Some(status) = statuses.get_mut(id) {
                    status.retention_state = RetentionState::Remembered;
                }
            }
            EncounterEvent::Forgotten { id, .. } => {
                if let Some(status) = statuses.get_mut(id) {
                    status.retention_state = RetentionState::Forgotten;
                }
            }
        }
    }
    statuses
}

/// The player-visible, recall-eligible history: only records explicitly remembered and not
/// since forgotten. A declined (never-remembered) or a forgotten turn never appears here,
/// regardless of how many times the journal is reloaded — this is what a future prompt
/// assembler or a "your history" screen must read from, never the raw journal.
pub fn recallable_records(archetype: Option<&str>) -> Result<Vec<EncounterRecord>, String> {
    recallable_records_at(&journal_path(), archetype)
}

fn recallable_records_at(path: &Path, archetype: Option<&str>) -> Result<Vec<EncounterRecord>, String> {
    let events = load_events(path)?;
    let statuses = fold_status(&events);
    let mut records: Vec<EncounterRecord> = statuses
        .into_values()
        .filter(|status| status.retention_state == RetentionState::Remembered)
        .filter(|status| archetype.map(|a| a == status.record.archetype).unwrap_or(true))
        .map(|status| status.record)
        .collect();
    records.sort_by_key(|record| record.created_at_utc_ms);
    Ok(records)
}

/// Full transparency for the `[View record]` control: the record's content and its current
/// retention state, regardless of what that state is.
pub fn view_record(id: &str) -> Result<Option<RecordStatus>, String> {
    view_record_at(&journal_path(), id)
}

fn view_record_at(path: &Path, id: &str) -> Result<Option<RecordStatus>, String> {
    let events = load_events(path)?;
    Ok(fold_status(&events).remove(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_path(stem: &str) -> PathBuf {
        std::env::temp_dir().join(format!("archetypes-encounters-{stem}-{}.jsonl", std::process::id()))
    }

    #[test]
    fn a_declined_turn_never_recalls() {
        let path = scratch_path("declined");
        let _ = fs::remove_file(&path);
        record_turn_at(&path, "Oracle", "inner_castle_encounter", "hello", "welcome").unwrap();
        let recallable = recallable_records_at(&path, None).unwrap();
        assert!(recallable.is_empty(), "a transient turn must not be recallable before Remember");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_remembered_turn_recalls_and_survives_a_reload() {
        let path = scratch_path("remembered");
        let _ = fs::remove_file(&path);
        let record = record_turn_at(&path, "Mentor", "inner_castle_encounter", "hi", "greetings").unwrap();
        remember_at(&path, &record.id).unwrap();

        // "Restart" is simulated honestly here: recall is recomputed fresh from disk, not
        // read back from any in-process cache, so re-invoking this is the real test.
        let recallable = recallable_records_at(&path, None).unwrap();
        assert_eq!(recallable.len(), 1);
        assert_eq!(recallable[0].id, record.id);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn forgetting_removes_a_previously_remembered_record_from_recall() {
        let path = scratch_path("forgotten");
        let _ = fs::remove_file(&path);
        let record = record_turn_at(&path, "Architect", "inner_castle_encounter", "plan", "sure").unwrap();
        remember_at(&path, &record.id).unwrap();
        forget_at(&path, &record.id, Some("changed my mind")).unwrap();

        let recallable = recallable_records_at(&path, None).unwrap();
        assert!(recallable.is_empty(), "a forgotten record must never be recallable again");

        // But the withdrawal itself remains auditable in the journal.
        let status = view_record_at(&path, &record.id).unwrap().expect("record must still exist for audit");
        assert_eq!(status.retention_state, RetentionState::Forgotten);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn recallable_records_filter_by_archetype() {
        let path = scratch_path("filter");
        let _ = fs::remove_file(&path);
        let oracle = record_turn_at(&path, "Oracle", "inner_castle_encounter", "q", "a").unwrap();
        let mentor = record_turn_at(&path, "Mentor", "inner_castle_encounter", "q2", "a2").unwrap();
        remember_at(&path, &oracle.id).unwrap();
        remember_at(&path, &mentor.id).unwrap();

        let oracle_only = recallable_records_at(&path, Some("Oracle")).unwrap();
        assert_eq!(oracle_only.len(), 1);
        assert_eq!(oracle_only[0].archetype, "Oracle");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn tampering_with_a_persisted_record_changes_its_content_hash() {
        let record = EncounterRecord::new("Sentinel", "inner_castle_encounter", "hi", "guarded");
        let original_hash = record.content_hash.clone();
        let tampered_hash = content_hash(&record.id, &record.archetype, &record.capability, record.created_at_utc_ms, &record.player_input, "a different reply entirely");
        assert_ne!(original_hash, tampered_hash);
    }

    #[test]
    fn view_record_reports_transient_before_any_decision() {
        let path = scratch_path("view");
        let _ = fs::remove_file(&path);
        let record = record_turn_at(&path, "Explorer", "inner_castle_encounter", "hi", "onward").unwrap();
        let status = view_record_at(&path, &record.id).unwrap().unwrap();
        assert_eq!(status.retention_state, RetentionState::Transient);
        let _ = fs::remove_file(&path);
    }
}
