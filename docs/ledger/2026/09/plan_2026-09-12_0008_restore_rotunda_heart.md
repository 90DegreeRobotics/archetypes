# Plan: Restore the Rotunda Heart — 2026-09-12 00:08

## Status
COMPLETED

## Goal

Restore the live Council centrepiece that was disconnected during the Seed-of-Life castle rewrite: the authored Council table with its real animated Stargate portal. Place the manifestation altar directly above that portal, and keep collision and interaction tied to the same shared altar position so the presentation and playable contracts agree.

## Steps

### Step 1 — Measure the authored centrepiece and map its live dependencies

- [x] Action: Read the rotunda handoff, current world spawn, portal binding, manifestation position, interaction distance, and player obstacle logic; inspect the table asset bounds before choosing placement heights.
- Files touched: none.
- Expected outcome: The world-space table and altar positions are derived from the authored model and current floor rather than legacy coordinates.

### Step 2 — Reconnect the real portal and centre the altar

- [x] Action: Replaced the procedural floor-inlay stand-in with `table.glb` in the live Inner Chambers world, positioned it from its measured authored foot and disc coordinates, moved the manifestation pedestal to the centre above the visible portal, and derived its obstacle from the shared constant.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`, `crates/engine/src/modes/inner_chambers/manifestation.rs`, `crates/engine/src/modes/inner_chambers/camera.rs`.
- Expected outcome: `PortalPlugin` binds the table's `Stargate_Portal`; the real vortex spins under the usable altar; no obsolete visual or invisible collision remains.

### Step 3 — Prove source and installed behaviour

- [x] Action: Added position and table-footprint collision regressions, ran `cargo test --workspace`, restaged with `pwsh -File scripts\install_shortcut.ps1`, then ran both capture harnesses from `dist` and inspected the resulting frames and walk report.
- Files touched: tests as required; `artifacts/visual-proof/rotunda-heart-2026-09-12/`.
- Expected outcome: Achieved. The full Rust gate passed: 201 tests (177 engine, 19 launcher, 5 Windows identity). `artifacts/visual-proof/rotunda-heart-2026-09-12/01_council_floor_inlay.png` visibly shows the real animated portal ring under the altar. `artifacts/visual-proof/rotunda-heart-walk-2026-09-12-final/walk_report.txt` records a real walking approach stopped at the 2.65m physical table boundary while resolving `ManifestationAltar`.

### Step 4 — Record and publish the verified unit

- [x] Action: Marked this plan completed, updated `STATUS.md` with bounded claims and evidence paths, then commit and push this unit.
- Files touched: this plan, `STATUS.md`, verified source and evidence files.
- Expected outcome: The repository and remote `main` are the complete audit surface, with no uncommitted work.
