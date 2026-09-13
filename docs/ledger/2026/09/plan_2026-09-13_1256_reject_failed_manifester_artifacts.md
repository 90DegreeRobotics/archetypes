# Plan: Reject failed Manifester artifacts — 2026-09-13 12:56

## Status
IN-PROGRESS

## Goal
Make the installed Archetypes manifestation path fail closed when Chronos2 records `matches=false`, so a rejected or visibly wrong reconstruction can never enter the player's persistent object library or appear on the altar.

## Steps
### Step 1 — Bind subject judgment into the receipt contract
- [x] Action: Require a readable `subject_match.json` with `checked=true` and `matches=true` before accepting a Chronos2 artifact.
- Files touched: `crates/engine/src/services/chronos_receipt.rs`
- Expected outcome: False, missing, or malformed subject judgments produce a visible subject-stage failure.

### Step 2 — Prove rejection and compatibility behavior
- [x] Action: Add focused fixtures for accepted, false, unchecked, and absent subject judgments, then run the full workspace test gate.
- Files touched: `crates/engine/src/services/chronos_receipt.rs`
- Expected outcome: Tests demonstrate that failed artifacts cannot reach import while valid current bundles still can.

### Step 3 — Refresh the operator surface
- [ ] Action: Run `scripts/install_shortcut.ps1`, verify installed hashes/timestamps, commit, and push `main`.
- Files touched: plan status plus generated distribution surface managed by the installer script.
- Expected outcome: The pinned launcher runs the fail-closed build and the repository is clean and remote-current.
