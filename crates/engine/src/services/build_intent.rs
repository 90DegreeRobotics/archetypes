//! Player-owned planning artifacts for the Architect workshop.
//!
//! A `BuildIntent` is the player's plan, not the model's. It is created, edited, sequenced and
//! closed only by an explicit confirmed player action; nothing here is ever written as a side
//! effect of an archetype reply. The local model may later *propose* a line, but proposal and
//! persistence are separate steps by construction: this module exposes no entry point that a
//! model response can reach without passing through a player confirmation in the workshop UI.
//!
//! Storage mirrors `encounter_memory`: an append-only JSONL event log folded from scratch on
//! every read, so there is no in-memory cache that can drift from disk and a restart simply
//! re-derives the same state. Editing a plan never rewrites history — a later event supersedes
//! an earlier one, and the whole sequence stays auditable.
//!
//! Sentinel note: writes are sealed through `services::ledger`, which mediates the existing
//! `memory.write` protected action. A plan journal is a memory write; `PROTECTED_ACTIONS` is a
//! fixed 40-entry inventory locked by test and mirrored in `docs/security/SENTINEL_PROTECTED_ACTIONS.md`,
//! and expanding that security inventory for this feature would not be justified.

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

pub const BUILD_INTENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatus {
    Open,
    Stalled,
    Closed,
}

impl PlanStatus {
    pub fn label(&self) -> &'static str {
        match self {
            PlanStatus::Open => "OPEN",
            PlanStatus::Stalled => "STALLED",
            PlanStatus::Closed => "CLOSED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanAction {
    pub id: String,
    pub description: String,
    pub done: bool,
    pub created_at_utc_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildIntent {
    pub id: String,
    pub version: u32,
    pub title: String,
    pub intent_statement: String,
    pub constraints: Vec<String>,
    pub risks: Vec<String>,
    pub actions: Vec<PlanAction>,
    pub supporting_ids: Vec<String>,
    pub status: PlanStatus,
    pub created_at_utc_ms: u64,
    pub updated_at_utc_ms: u64,
    pub completion_evidence: Option<String>,
    pub content_hash: String,
}

impl BuildIntent {
    pub fn open_actions(&self) -> usize {
        self.actions.iter().filter(|action| !action.done).count()
    }

    pub fn is_closed(&self) -> bool {
        self.status == PlanStatus::Closed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum PlanEvent {
    Created {
        id: String,
        title: String,
        intent_statement: String,
        supporting_ids: Vec<String>,
        at_utc_ms: u64,
    },
    ConstraintAdded { plan_id: String, text: String, at_utc_ms: u64 },
    RiskAdded { plan_id: String, text: String, at_utc_ms: u64 },
    ActionAdded { plan_id: String, action_id: String, description: String, at_utc_ms: u64 },
    ActionCompleted { plan_id: String, action_id: String, at_utc_ms: u64 },
    ActionMoved { plan_id: String, action_id: String, new_index: usize, at_utc_ms: u64 },
    StatusChanged { plan_id: String, status: PlanStatus, at_utc_ms: u64 },
    Closed { plan_id: String, completion_evidence: Option<String>, at_utc_ms: u64 },
}

pub fn plans_dir() -> PathBuf {
    app_data_root().join("plans")
}

pub fn journal_path() -> PathBuf {
    plans_dir().join("journal.jsonl")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn content_hash(plan: &BuildIntent) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plan.id.as_bytes());
    hasher.update(plan.title.as_bytes());
    hasher.update(plan.intent_statement.as_bytes());
    for constraint in &plan.constraints {
        hasher.update(constraint.as_bytes());
    }
    for risk in &plan.risks {
        hasher.update(risk.as_bytes());
    }
    for action in &plan.actions {
        hasher.update(action.id.as_bytes());
        hasher.update(action.description.as_bytes());
        hasher.update(if action.done { b"1" } else { b"0" });
    }
    hasher.update(plan.status.label().as_bytes());
    hasher.update(plan.completion_evidence.clone().unwrap_or_default().as_bytes());
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Appends one event to the journal file only. Ledger sealing is a separate call so tests can
/// drive the fold logic against a scratch file without racing the one process-wide hash chain.
fn append_event(path: &Path, event: &PlanEvent) -> Result<(), String> {
    let parent = path.parent().ok_or("plan journal path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let line = serde_json::to_string(event).map_err(|error| error.to_string())? + "\n";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes()).map_err(|error| error.to_string())
}

/// What actually happened to a write.
///
/// The journal and the audit ledger are two different files and can fail independently. A
/// journal failure is a real failure and nothing is kept. A seal failure is *not* allowed to
/// masquerade as one: the player's plan is on disk, so the honest report is "written, but
/// unsealed", carrying the reason. Claiming "not written" when the row exists would be a lie
/// the next load contradicts.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteReceipt {
    pub sealed: bool,
    pub seal_error: Option<String>,
}

impl WriteReceipt {
    /// A short player-facing suffix, empty when the write was fully sealed.
    pub fn caveat(&self) -> String {
        match &self.seal_error {
            Some(error) => format!(" (kept, but the audit ledger did not accept it: {error})"),
            None => String::new(),
        }
    }
}

fn write(kind: &str, payload: serde_json::Value, event: &PlanEvent) -> Result<WriteReceipt, String> {
    append_event(&journal_path(), event)?;
    match append_to_ledger(GameMode::InnerChambers, kind, payload) {
        Ok(()) => Ok(WriteReceipt { sealed: true, seal_error: None }),
        Err(error) => Ok(WriteReceipt { sealed: false, seal_error: Some(error) }),
    }
}

pub fn create_plan(
    title: &str,
    intent_statement: &str,
    supporting_ids: Vec<String>,
) -> Result<WriteReceipt, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let event = PlanEvent::Created {
        id: id.clone(),
        title: title.to_owned(),
        intent_statement: intent_statement.to_owned(),
        supporting_ids: supporting_ids.clone(),
        at_utc_ms: now_ms(),
    };
    write(
        "architect_plan_created",
        serde_json::json!({ "id": id, "title": title, "supporting_ids": supporting_ids }),
        &event,
    )
}

pub fn add_action(plan_id: &str, description: &str) -> Result<WriteReceipt, String> {
    let action_id = uuid::Uuid::new_v4().to_string();
    let event = PlanEvent::ActionAdded {
        plan_id: plan_id.to_owned(),
        action_id: action_id.clone(),
        description: description.to_owned(),
        at_utc_ms: now_ms(),
    };
    write(
        "architect_plan_action_added",
        serde_json::json!({ "plan_id": plan_id, "action_id": action_id }),
        &event,
    )
}

pub fn complete_action(plan_id: &str, action_id: &str) -> Result<WriteReceipt, String> {
    let event = PlanEvent::ActionCompleted {
        plan_id: plan_id.to_owned(),
        action_id: action_id.to_owned(),
        at_utc_ms: now_ms(),
    };
    write(
        "architect_plan_action_completed",
        serde_json::json!({ "plan_id": plan_id, "action_id": action_id }),
        &event,
    )
}

pub fn add_constraint(plan_id: &str, text: &str) -> Result<WriteReceipt, String> {
    let event = PlanEvent::ConstraintAdded {
        plan_id: plan_id.to_owned(),
        text: text.to_owned(),
        at_utc_ms: now_ms(),
    };
    write(
        "architect_plan_constraint_added",
        serde_json::json!({ "plan_id": plan_id }),
        &event,
    )
}

pub fn add_risk(plan_id: &str, text: &str) -> Result<WriteReceipt, String> {
    let event = PlanEvent::RiskAdded {
        plan_id: plan_id.to_owned(),
        text: text.to_owned(),
        at_utc_ms: now_ms(),
    };
    write("architect_plan_risk_added", serde_json::json!({ "plan_id": plan_id }), &event)
}

pub fn set_status(plan_id: &str, status: PlanStatus) -> Result<WriteReceipt, String> {
    let event = PlanEvent::StatusChanged {
        plan_id: plan_id.to_owned(),
        status,
        at_utc_ms: now_ms(),
    };
    write(
        "architect_plan_status_changed",
        serde_json::json!({ "plan_id": plan_id, "status": status.label() }),
        &event,
    )
}

pub fn close_plan(plan_id: &str, completion_evidence: Option<&str>) -> Result<WriteReceipt, String> {
    let event = PlanEvent::Closed {
        plan_id: plan_id.to_owned(),
        completion_evidence: completion_evidence.map(str::to_owned),
        at_utc_ms: now_ms(),
    };
    write(
        "architect_plan_closed",
        serde_json::json!({ "plan_id": plan_id, "has_evidence": completion_evidence.is_some() }),
        &event,
    )
}

fn load_events(path: &Path) -> Result<Vec<PlanEvent>, String> {
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
        let event: PlanEvent = serde_json::from_str(&line)
            .map_err(|error| format!("plan journal line {} is invalid JSON: {error}", index + 1))?;
        events.push(event);
    }
    Ok(events)
}

/// Folds the event log into current plan state. An event naming a plan that was never created
/// is ignored rather than fabricating one, so a partially copied journal degrades honestly.
fn fold(events: Vec<PlanEvent>) -> HashMap<String, BuildIntent> {
    let mut plans: HashMap<String, BuildIntent> = HashMap::new();
    for event in events {
        match event {
            PlanEvent::Created { id, title, intent_statement, supporting_ids, at_utc_ms } => {
                plans.insert(
                    id.clone(),
                    BuildIntent {
                        id,
                        version: BUILD_INTENT_VERSION,
                        title,
                        intent_statement,
                        constraints: Vec::new(),
                        risks: Vec::new(),
                        actions: Vec::new(),
                        supporting_ids,
                        status: PlanStatus::Open,
                        created_at_utc_ms: at_utc_ms,
                        updated_at_utc_ms: at_utc_ms,
                        completion_evidence: None,
                        content_hash: String::new(),
                    },
                );
            }
            PlanEvent::ConstraintAdded { plan_id, text, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    plan.constraints.push(text);
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
            PlanEvent::RiskAdded { plan_id, text, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    plan.risks.push(text);
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
            PlanEvent::ActionAdded { plan_id, action_id, description, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    plan.actions.push(PlanAction {
                        id: action_id,
                        description,
                        done: false,
                        created_at_utc_ms: at_utc_ms,
                    });
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
            PlanEvent::ActionCompleted { plan_id, action_id, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    if let Some(action) = plan.actions.iter_mut().find(|action| action.id == action_id) {
                        action.done = true;
                    }
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
            PlanEvent::ActionMoved { plan_id, action_id, new_index, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    if let Some(from) = plan.actions.iter().position(|action| action.id == action_id) {
                        let action = plan.actions.remove(from);
                        let to = new_index.min(plan.actions.len());
                        plan.actions.insert(to, action);
                    }
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
            PlanEvent::StatusChanged { plan_id, status, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    plan.status = status;
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
            PlanEvent::Closed { plan_id, completion_evidence, at_utc_ms } => {
                if let Some(plan) = plans.get_mut(&plan_id) {
                    plan.status = PlanStatus::Closed;
                    plan.completion_evidence = completion_evidence;
                    plan.updated_at_utc_ms = at_utc_ms;
                }
            }
        }
    }
    for plan in plans.values_mut() {
        plan.content_hash = content_hash(plan);
    }
    plans
}

/// Every plan the player owns, oldest first.
pub fn load_plans() -> Result<Vec<BuildIntent>, String> {
    load_plans_at(&journal_path())
}

fn load_plans_at(path: &Path) -> Result<Vec<BuildIntent>, String> {
    let mut plans: Vec<BuildIntent> = fold(load_events(path)?).into_values().collect();
    plans.sort_by_key(|plan| plan.created_at_utc_ms);
    Ok(plans)
}

/// Keeps only the supporting encounter ids that are still recallable.
///
/// A forgotten encounter must not resurface as plan provenance, so the caller passes the
/// currently recallable ids (from `encounter_memory::recallable_records`) and this filters
/// against them. Pure by design: the consent decision lives in `encounter_memory`, and this
/// module never reaches around it to read the raw journal.
pub fn filter_supporting_ids(requested: &[String], recallable: &[String]) -> Vec<String> {
    requested
        .iter()
        .filter(|id| recallable.iter().any(|allowed| allowed == *id))
        .cloned()
        .collect()
}

/// Measurable, on-disk signals only — open/stalled counts and the age of the oldest untouched
/// plan. Deliberately not a psychological read of the player.
pub fn plan_signals(plans: &[BuildIntent], now_utc_ms: u64) -> PlanSignals {
    let open = plans.iter().filter(|plan| plan.status == PlanStatus::Open).count();
    let stalled = plans.iter().filter(|plan| plan.status == PlanStatus::Stalled).count();
    let open_actions = plans
        .iter()
        .filter(|plan| !plan.is_closed())
        .map(BuildIntent::open_actions)
        .sum();
    let oldest_untouched_days = plans
        .iter()
        .filter(|plan| !plan.is_closed())
        .map(|plan| now_utc_ms.saturating_sub(plan.updated_at_utc_ms) / 86_400_000)
        .max()
        .unwrap_or(0);
    PlanSignals { open, stalled, open_actions, oldest_untouched_days }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanSignals {
    pub open: usize,
    pub stalled: usize,
    pub open_actions: usize,
    pub oldest_untouched_days: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_path(stem: &str) -> PathBuf {
        std::env::temp_dir().join(format!("archetypes-plans-{stem}-{}.jsonl", std::process::id()))
    }

    fn created(path: &Path, title: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        append_event(
            path,
            &PlanEvent::Created {
                id: id.clone(),
                title: title.to_owned(),
                intent_statement: "because it matters".to_owned(),
                supporting_ids: Vec::new(),
                at_utc_ms: 1_000,
            },
        )
        .unwrap();
        id
    }

    #[test]
    fn a_created_plan_starts_open_with_no_actions() {
        let path = scratch_path("created");
        let _ = fs::remove_file(&path);
        created(&path, "Ship the workshop");
        let plans = load_plans_at(&path).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].title, "Ship the workshop");
        assert_eq!(plans[0].status, PlanStatus::Open);
        assert!(plans[0].actions.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn actions_complete_and_survive_a_reload_from_disk() {
        let path = scratch_path("actions");
        let _ = fs::remove_file(&path);
        let plan_id = created(&path, "Plan");
        let action_id = uuid::Uuid::new_v4().to_string();
        append_event(&path, &PlanEvent::ActionAdded { plan_id: plan_id.clone(), action_id: action_id.clone(), description: "first step".into(), at_utc_ms: 2_000 }).unwrap();
        append_event(&path, &PlanEvent::ActionCompleted { plan_id: plan_id.clone(), action_id: action_id.clone(), at_utc_ms: 3_000 }).unwrap();

        // Recall is recomputed from the file every call, so this is a genuine restart test.
        let plans = load_plans_at(&path).unwrap();
        assert_eq!(plans[0].actions.len(), 1);
        assert!(plans[0].actions[0].done);
        assert_eq!(plans[0].open_actions(), 0);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn closing_records_evidence_and_marks_the_plan_closed() {
        let path = scratch_path("closed");
        let _ = fs::remove_file(&path);
        let plan_id = created(&path, "Plan");
        append_event(&path, &PlanEvent::Closed { plan_id, completion_evidence: Some("shipped".into()), at_utc_ms: 9_000 }).unwrap();
        let plans = load_plans_at(&path).unwrap();
        assert!(plans[0].is_closed());
        assert_eq!(plans[0].completion_evidence.as_deref(), Some("shipped"));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn action_order_is_player_controlled_and_survives_a_move() {
        let path = scratch_path("order");
        let _ = fs::remove_file(&path);
        let plan_id = created(&path, "Plan");
        let mut ids = Vec::new();
        for (index, text) in ["one", "two", "three"].iter().enumerate() {
            let action_id = uuid::Uuid::new_v4().to_string();
            ids.push(action_id.clone());
            append_event(&path, &PlanEvent::ActionAdded { plan_id: plan_id.clone(), action_id, description: (*text).into(), at_utc_ms: 2_000 + index as u64 }).unwrap();
        }
        append_event(&path, &PlanEvent::ActionMoved { plan_id, action_id: ids[2].clone(), new_index: 0, at_utc_ms: 4_000 }).unwrap();
        let plans = load_plans_at(&path).unwrap();
        let order: Vec<&str> = plans[0].actions.iter().map(|action| action.description.as_str()).collect();
        assert_eq!(order, vec!["three", "one", "two"]);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn an_event_for_an_unknown_plan_is_ignored_rather_than_fabricating_one() {
        let path = scratch_path("orphan");
        let _ = fs::remove_file(&path);
        append_event(&path, &PlanEvent::ActionAdded { plan_id: "does-not-exist".into(), action_id: "a".into(), description: "ghost".into(), at_utc_ms: 1 }).unwrap();
        assert!(load_plans_at(&path).unwrap().is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_sealed_write_carries_no_caveat_and_an_unsealed_one_says_why() {
        let sealed = WriteReceipt { sealed: true, seal_error: None };
        assert_eq!(sealed.caveat(), "");
        let unsealed = WriteReceipt {
            sealed: false,
            seal_error: Some("ledger line 34 hash mismatch".to_owned()),
        };
        // The player must not be told "not written" when the row is on disk; they are told it
        // is kept and why it is unsealed.
        assert!(unsealed.caveat().contains("kept"));
        assert!(unsealed.caveat().contains("ledger line 34 hash mismatch"));
    }

    #[test]
    fn a_forgotten_encounter_cannot_be_used_as_plan_provenance() {
        let requested = vec!["kept".to_owned(), "forgotten".to_owned()];
        let recallable = vec!["kept".to_owned()];
        assert_eq!(filter_supporting_ids(&requested, &recallable), vec!["kept".to_owned()]);
    }

    #[test]
    fn content_hash_changes_when_the_plan_content_changes() {
        let path = scratch_path("hash");
        let _ = fs::remove_file(&path);
        let plan_id = created(&path, "Plan");
        let first = load_plans_at(&path).unwrap()[0].content_hash.clone();
        append_event(&path, &PlanEvent::RiskAdded { plan_id, text: "scope creep".into(), at_utc_ms: 5_000 }).unwrap();
        let second = load_plans_at(&path).unwrap()[0].content_hash.clone();
        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn signals_count_only_measurable_state() {
        let path = scratch_path("signals");
        let _ = fs::remove_file(&path);
        let plan_id = created(&path, "Plan");
        append_event(&path, &PlanEvent::ActionAdded { plan_id: plan_id.clone(), action_id: "a".into(), description: "step".into(), at_utc_ms: 2_000 }).unwrap();
        let plans = load_plans_at(&path).unwrap();
        let signals = plan_signals(&plans, 2_000 + 3 * 86_400_000);
        assert_eq!(signals.open, 1);
        assert_eq!(signals.stalled, 0);
        assert_eq!(signals.open_actions, 1);
        assert_eq!(signals.oldest_untouched_days, 3);
        let _ = fs::remove_file(&path);
    }
}
