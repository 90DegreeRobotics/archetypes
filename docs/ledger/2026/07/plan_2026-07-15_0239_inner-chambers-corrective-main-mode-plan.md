# Plan: Inner Chambers Corrective + Main Mode Repair Plan — 2026-07-15 02:39

## Status
COMPLETED

## Goal
Stabilize the uncommitted Inner Chambers work so it cannot break the shared menu/runtime path, then create the next concrete plan for repairing Standard Mode's visual quality. The immediate standard is safety first: no selectable half-mode, no false docs, no untested exit path, and no desktop refresh until the repo can prove the behavior.

## Steps
### Step 1 — Audit current dirty tree
- [x] Action: Inspect the uncommitted Inner Chambers changes, current docs, and mode/menu integration before editing.
- Files touched: None.
- Expected outcome: Confirmed the dirty tree came from the prior Lane B implementation: menu routing, registry availability, plugin registration, and new `modes/inner_chambers/**`. The dangerous surfaces were menu unlock, miswired exit cleanup, incorrect table reload transform, untested extraction, and docs drift.

### Step 2 — Repair or relock Inner Chambers
- [x] Action: Fix the exit lifecycle, cleanup/reload behavior, camera ownership, and extraction gate, or relock the mode if it cannot be safely completed in this pass.
- Files touched: `crates/engine/src/modes/inner_chambers/**`, `crates/engine/src/modes/game_mode.rs`, `crates/engine/src/chamber/boot.rs`, `crates/engine/src/modes/mod.rs`.
- Expected outcome: Inner Chambers remains locked in `GameMode::REGISTRY`; the parked implementation now cleans up on entering `Exiting`, restores the canonical chamber/table transform, re-enables the Witness camera, and requires node proximity for extraction.

### Step 3 — Add focused coverage
- [x] Action: Add deterministic tests for the Inner Chambers contract pieces that can be tested without launching the renderer.
- Files touched: Inner Chambers source/tests and existing mode-registry tests.
- Expected outcome: Added pure tests for truth-node count, extraction proximity, and ledger payload shape. Existing registry tests again prove Inner Chambers and Living Engine are locked.

### Step 4 — Update truth surfaces
- [x] Action: Update `README.md`, `STATUS.md`, and this plan to match the actual shipped behavior.
- Files touched: `README.md`, `STATUS.md`, this plan.
- Expected outcome: `README.md` already matched the locked state. `STATUS.md` now records the Inner Chambers safety gate, test count, and the Standard Mode visual blocker.

### Step 5 — Verify and stage desktop only if safe
- [x] Action: Run formatting, `cargo test --workspace`, and if the feature is safe/playable, refresh `dist` with `scripts/install_shortcut.ps1`.
- Files touched: build outputs only if installation is safe.
- Expected outcome: Targeted Rust formatting was applied to touched files. `cargo test --workspace` passed with 38 engine tests and 1 launcher test. `dist` was not refreshed because Inner Chambers was deliberately not shipped as a new playable desktop feature.

### Step 6 — Plan the Standard Mode visual repair
- [x] Action: Create a separate cold-start plan doc for the main/Standard Mode visual rescue: what looks bad, what files own it, what screenshots/proofs are required, and what must not break upstream/downstream.
- Files touched: new `docs/ledger/2026/07/plan_*_standard-mode-visual-repair.md`.
- Expected outcome: Created `docs/ledger/2026/07/plan_2026-07-15_0248_standard-mode-visual-repair.md`; it freezes new mode work and makes capture-led Standard Mode repair the next authority.
