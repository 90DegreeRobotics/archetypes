# Plan: Lane 0 spine audit fix — 2026-07-13 21:47

## Status
COMPLETED

## Goal
Close the concrete gaps found after commit `edbd91a`: make the Lane 0 spine match its contract instead of merely compiling. The corrective pass will make the mode registry real, consolidate app-data paths through `services::paths`, seal live events to the ledger, fix the completed plan checklist, update truth docs, run the workspace tests, and publish the clean result to `origin/main`.

## Steps
### Step 1 — Audit the pushed Lane 0 surface
- [x] Action: Inspect HEAD, status, touched files, and the new mode/services code for contract mismatches.
- Files touched: None.
- Expected outcome: Defects are identified before any edit.

### Step 2 — Repair mode registry and menu truth
- [x] Action: Replace placeholder A/B/C enum labels with named mode variants, expose registry metadata, render all planned modes in the menu, and prevent incomplete modes from launching.
- Files touched: `crates/engine/src/modes/game_mode.rs`, `crates/engine/src/chamber/boot.rs`.
- Expected outcome: The selector is real, while incomplete lanes are visibly unavailable and not playable stubs.

### Step 3 — Consolidate paths and seal ledger events
- [x] Action: Remove duplicated app-data helpers in chamber modules, use `services::paths`, and seal key live Standard Mode events to `services::ledger`.
- Files touched: `crates/engine/src/chamber/ritual.rs`, `crates/engine/src/chamber/speech.rs`, `crates/engine/src/services/ledger.rs`.
- Expected outcome: Shared paths are actually shared, and the Standard Mode loop writes append-only ledger entries.

### Step 4 — Fix documentation truth
- [x] Action: Update the prior Lane 0 plan checklist/status and the status docs so they describe the actual completed contract.
- Files touched: `docs/ledger/2026/07/plan_2026-07-13_2132_lane-0-spine.md`, `STATUS.md`, this plan.
- Expected outcome: The ledger no longer says completed work is unchecked, and public status no longer overstates the first Lane 0 commit.

### Step 5 — Verify, commit, and push
- [x] Action: Run `cargo test --workspace`, commit, push `origin main`, and confirm `git status` is clean.
- Files touched: None.
- Expected outcome: Corrected Lane 0 is on `origin/main` with a clean tree.

## Verification
- `cargo test --workspace`: 24 engine tests + 1 launcher test passed, 0 failed.
