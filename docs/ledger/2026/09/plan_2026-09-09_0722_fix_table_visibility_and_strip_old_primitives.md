# Plan: Fix Table Visibility & Strip Old Primitives from Rotunda — 2026-09-09 07:22

## Status
COMPLETED

## Goal
Root-cause and eliminate the bug where entering Inner Chambers displayed the old placeholder shapes (cubes, spheres, wall shards) with no visible table:
1. Root cause 1: `crates/engine/src/chamber/camera.rs` had a global visibility gate `gate_boot_and_table_visibility` that matched `Name == "PortalTable"` and forced `Visibility::Hidden` whenever the mode was not the ritual chamber.
2. Root cause 2: `crates/engine/src/modes/inner_chambers/world.rs` was still calling `spawn_chamber()` which generated the 7 archetype primitive placeholder geometries (floating cubes, spheres, line meshes) into the room.

We will rename the rotunda table entity to `RotundaCouncilTable`, safeguard `camera.rs` so it never hides non-ritual tables, and remove `spawn_chamber()` so the giant castle room is completely clean, showing only the floor, dais, 4 walls, ceiling, cross-beams, torches, lighting, and the animated table.

## Steps

### Step 1 — Unblock Table Visibility in Engine & Rename Rotunda Table
- [x] Action:
  1. In `crates/engine/src/modes/inner_chambers/world.rs`, change table entity name from `PortalTable` to `RotundaCouncilTable` and add `PortalDiscGlowLight`.
  2. In `crates/engine/src/chamber/camera.rs`, update `gate_boot_and_table_visibility` so it only gates visibility when not in exclusive world modes (`if !exclusive`).
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/chamber/camera.rs`
- Expected outcome: The table is 100% visible and un-gated in the rotunda.

### Step 2 — Strip All Legacy Abstract Shapes from the Castle Room
- [x] Action:
  1. In `crates/engine/src/modes/inner_chambers/world.rs`, remove `spawn_chamber()` and the legacy geometry generators (`spawn_law_geometry`, `InnerTruthNode`, etc.).
  2. Clean up unused imports and functions.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: The room is a giant, clean castle great hall containing only floor, dais, walls, ceiling, lighting, and the animated council table.

### Step 3 — Verification Gate, Desktop Recompile & Push
- [x] Action:
  1. Run `cargo test --workspace` to ensure all tests pass (100 passed, 0 failed).
  2. Run `pwsh -File scripts\install_shortcut.ps1` to rebuild release binaries and restage to `dist\`.
  3. Verify `dist\engine.exe` timestamp (`9/9/2026 7:21:55 AM`).
  4. Commit and push cleanly to `origin/main`.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/chamber/camera.rs`
  - `docs/ledger/2026/09/plan_2026-09-09_0722_fix_table_visibility_and_strip_old_primitives.md`
- Expected outcome: The Desktop icon immediately loads the clean castle hall with the animated table.

