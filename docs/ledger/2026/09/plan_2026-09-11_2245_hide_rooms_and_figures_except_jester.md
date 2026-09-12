# Plan: Hide Satellite Rooms and Figures Except Jester — 2026-09-11 22:45

## Status
COMPLETED — 2026-09-11 22:50

## Goal
The operator explicitly requested: "get rid of the archetypes except for Jester. that was clear. just hide them for now. I want the rooms and furniture and all off that hidden for now. i may make the inner chambers through archways. can you launch the game and walkthrough inner chamber mode and take screens so you can see how it looks?"

This plan hides the 6 satellite archetype rooms (platforms, cobblestone walls, doorways, bridges over the abyss, room furniture, drafting bench, and lights), hides AURA at the central circle, and hides the 6 room embodiment figures, while preserving the Jester as host in the Council circle and leaving all code completely intact for future re-enabling or archway relocation (Rule 1: Never delete). It also adjusts character obstacle collision so the player doesn't bump into invisible hidden entities, updates unit tests, runs the full test suite and shortcut installer, launches the game to walk through inner chamber mode, and collects visual screenshot proof.

## Steps
### Step 1 — Hide satellite rooms and character meshes except Jester in world.rs
- [x] Action:
  - Add clean gating in `world.rs` (`const SPAWN_OUTER_ROOMS: bool = false;` and `const SPAWN_AURA: bool = false;`).
  - Keep `Jester_CouncilHost` fully spawned and active at `(10.2, 0.42, 6.2)`.
  - Gate `spawn_seed_room` loop and `AURA_CentralEmbodiment` spawn, preserving all functions intact.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: Only the Council circle, Jester, manifestation pedestal/inlay, and the perimeter arcade/stairs/vault fresco are spawned; satellite rooms, furniture, and non-Jester characters are hidden.

### Step 2 — Adjust character obstacle colliders and tests in camera.rs
- [x] Action:
  - Update `character_obstacles()` in `camera.rs` to reflect the active entities (Jester and Manifestation Pedestal, removing Aura and the 6 hidden room figures so the player encounters no invisible barriers in open space).
  - Provide `canonical_room_figure_obstacles()` helper in `camera.rs` to pin the room formulas in unit tests.
- Files touched: `crates/engine/src/modes/inner_chambers/camera.rs`
- Expected outcome: No invisible phantom colliders on the Council floor or in space; unit tests pass.

### Step 3 — Update capture harnesses for rotunda walking and vantage inspection
- [x] Action:
  - Update `walk_capture.rs` and `capture.rs` to walk and photograph the Council circle, Jester host, portal area, and monumental ascent/vault from real walking and vantage cameras.
- Files touched: `crates/engine/src/modes/inner_chambers/walk_capture.rs`, `crates/engine/src/modes/inner_chambers/capture.rs`
- Expected outcome: Captures showcase the clean hall, Jester, center area, and perimeter architecture without aiming into empty room voids.

### Step 4 — Verify with full test suite and install shortcut
- [x] Action:
  - Run `cargo test --workspace` to ensure all 199+ tests pass (199 passed).
  - Run `pwsh -File scripts\install_shortcut.ps1` to rebuild release and sync to `%LOCALAPPDATA%\Programs\Archetypes`.
- Files touched: `dist/*`, `%LOCALAPPDATA%\Programs\Archetypes`
- Expected outcome: 100% passing tests, verified binaries staged to Desktop/Taskbar launcher surface.

### Step 5 — Launch walkthrough capture and inspect visual proof
- [x] Action:
  - Run `dist\engine.exe` with `ARCHETYPES_INNER_CAPTURE=1` and `ARCHETYPES_WALK_CAPTURE=1` to take high-resolution rendered frames of the inner chamber mode.
  - Review and inspect screenshots under `artifacts/visual-proof/` to evaluate the visual presentation and confirm everything requested is hidden and Jester is present.
- Files touched: `artifacts/visual-proof/rotunda-clean-2026-09-11/`, `artifacts/visual-proof/rotunda-walk-2026-09-11/`
- Expected outcome: High-resolution screenshots captured and inspected showing the spacious rotunda with only the Jester and center active. All 17 frames inspected and verified.
