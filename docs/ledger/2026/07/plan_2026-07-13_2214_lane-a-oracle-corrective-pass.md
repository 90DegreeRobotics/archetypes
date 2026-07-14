# Plan: Lane A Oracle corrective pass — 2026-07-13 22:14

## Status
COMPLETED

## Goal
Finish Lane A against its actual contract rather than the overclaimed first pass. This pass will add real difficulty-tier prompt selection, visible invalid-input feedback, a next-round loop, deterministic tests for exact/partial scoring and ledger payload shape, and truthful docs/plan status. It must preserve Standard Mode, keep Inner Chambers and Living Engine locked, run `cargo test --workspace`, commit, push `origin/main`, and leave the tree clean.

## Steps
### Step 1 — Repair truth surfaces
- [x] Action: Correct the prior Lane A plan checklist/status claims and update `STATUS.md` / `README.md` to match the real corrective scope.
- Files touched: `docs/ledger/2026/07/plan_2026-07-13_2205_lane-a-oracle-riddle.md`, `STATUS.md`, `README.md`.
- Expected outcome: Docs no longer claim missing tests/features are done before they exist.

### Step 2 — Implement difficulty-tier prompt selection
- [x] Action: Replace the single hardcoded prompt with explicit semantic-distance tiers and deterministic selection helpers.
- Files touched: `crates/engine/src/modes/oracle_riddle/scoring.rs`.
- Expected outcome: The mode can choose valid 3-word prompts from Beginner through Impossible tiers.

### Step 3 — Improve playable loop feedback
- [x] Action: Surface invalid 3-word input feedback, add a next-round path, and keep return-to-menu behavior.
- Files touched: `crates/engine/src/modes/oracle_riddle/mod.rs`, `crates/engine/src/modes/oracle_riddle/ui.rs`.
- Expected outcome: The player is not silently ignored on bad input and can play another Oracle round without relaunching the mode.

### Step 4 — Add missing deterministic tests
- [x] Action: Add pure tests for exact/partial scoring from injected vectors, ledger payload shape, prompt tiers, and mode lock state.
- Files touched: `crates/engine/src/modes/oracle_riddle/scoring.rs`, `crates/engine/src/modes/oracle_riddle/ui.rs`; existing `game_mode.rs` tests continue to verify the mode lock state.
- Expected outcome: The tests prove the claims made in status docs without requiring live Ollama or Chronos.

### Step 5 — Verify and publish
- [x] Action: Run `cargo test --workspace`, mark this plan completed, commit, push `origin/main`, and confirm clean status.
- Files touched: this plan.
- Expected outcome: Lane A corrective pass is on `origin/main` and the repo is clean.

## Verification
- `cargo test --workspace`: 30 engine tests + 1 launcher test passed, 0 failed.
