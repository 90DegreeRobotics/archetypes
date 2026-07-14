# Plan: Multilane Build: Archetypes Modes of Play — 2026-07-13 20:47

## Status
PENDING

## Goal
Plan a multilane build adding several new modes of play, keeping the central thesis (the game is about legibility: surviving the crossing between two minds) and reconciling it against the actual state of `main`. Execute Lane 0 (Spine) + all three mode lanes (A/B/C) in parallel, staged in depth.

## Steps
### Step 1 — Create build contracts
- [x] Action: Write the ledger plan doc + per-lane CODEX_LANE_*.md briefs (house format).
- Files touched:
  - `docs/ledger/2026/07/plan_2026-07-13_2047_multilane-modes.md`
  - `docs/ledger/2026/07/CODEX_LANE_0_SPINE.md`
  - `docs/ledger/2026/07/CODEX_LANE_A_ORACLE.md`
  - `docs/ledger/2026/07/CODEX_LANE_B_INNER_CHAMBERS.md`
  - `docs/ledger/2026/07/CODEX_LANE_C_LIVING_ENGINE.md`
- Expected outcome: The design is formalized into frozen build contracts ready for parallel execution.

### Step 2 — Await operator confirmation
- [ ] Action: Pause build and wait for operator confirmation that other builder(s) are underway.
- Files touched: None.
- Expected outcome: Operator approval received.

### Step 3 — Land Lane 0 (Spine) and freeze contract
- [ ] Action: Implement Lane 0 spine (GameMode framework, services updates, ledger, Difficulty) and reconcile canon.
- Files touched: `crates/engine/src/modes/mod.rs`, `crates/engine/src/services/**`, `crates/engine/src/chamber/mod.rs`, `crates/engine/src/boot.rs`, `crates/engine/src/theme/constants.rs`.
- Expected outcome: Lane 0 contract is frozen and published.

### Step 4 — Remove motion-forbidding covenants and update documentation
- [ ] Action: Remove static-motion guard tests and `spheres.rs` covenant. Update STATUS.md and operator memory.
- Files touched: `crates/engine/src/chamber/spheres.rs`, `crates/engine/src/chamber/ritual.rs`, `STATUS.md`.
- Expected outcome: Spheres and stars are permitted to move for gameplay mechanics.
