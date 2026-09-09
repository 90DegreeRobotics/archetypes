# Plan: Player Ground Locomotion, Gravity & Flight Mode — 2026-09-09 08:00

## Status
COMPLETED

## Goal
Implement embodied player physics and locomotion in Inner Chambers mode: grounded walking with realistic eye height, gravity, dais step heights, smooth non-touchy mouse look, standard FPS WASD strafing/forward-back movement, and an intuitive flight mode transition (double-jump + hold Space for 2.0s to fly; triple-tap Space to exit flight and resume gravity).

## Steps

### Step 1 — Author Locomotion State & Player Physics Component
- [x] Action: Refactor `crates/engine/src/modes/inner_chambers/camera.rs` to replace raw spectator flight with:
  1. `LocomotionMode`: `Walking` (default) vs `Flying`.
  2. Ground height resolution: base floor at $y=0$, dais outer step at $y=0.15\text{m}$, upper dais platform at $y=0.30\text{m}$, clamped within room boundaries.
  3. Physical parameters: eye height ($1.75\text{m}$), gravity ($-16.0\text{m/s}^2$), jump velocity ($+5.2\text{m/s}$), walk speed ($6.5\text{m/s}$), flight speed ($11.0\text{m/s}$).
  4. Initial spawn orientation: eye height $1.75\text{m}$ looking slightly downward ($\approx -8.5^\circ$) toward the table and artifact pedestals.
  5. Calibrated mouse sensitivity ($0.0009$) with smooth interpolation to eliminate the "touchy" feeling and provide intuitive FPS mouse look.
  6. Standard FPS horizontal strafing and movement with A/D and W/S aligned to camera yaw.
- Files touched: `crates/engine/src/modes/inner_chambers/camera.rs`
- Expected outcome: Player walks naturally on the castle floor with gravity and standard FPS controls.

### Step 2 — Implement Double-Jump Hold & Triple-Tap Flight Mode Transitions
- [x] Action: In `crates/engine/src/modes/inner_chambers/camera.rs`:
  1. In `Walking` mode: Track space press cadence. If a second Space press occurs within $0.4\text{s}$ and is held for $2.0\text{s}$ (`flight_engage_timer >= 2.0`), switch to `Flying` mode.
  2. In `Flying` mode: Enable 6DOF vertical flight (Space up, Shift/C down). If 3 Space presses occur within $0.75\text{s}$, immediately cancel flight and restore `Walking` mode with gravity.
  3. Update HUD overlay in `crates/engine/src/modes/inner_chambers/world.rs` to clearly display current locomotion status (`WALKING` vs `FLYING`) and control prompts.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/camera.rs`
  - `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: Seamless, intuitive transition between grounded walking physics and free flight.

### Step 3 — Workspace Verification & Desktop/Taskbar Launcher Sync
- [x] Action: Run `cargo test --workspace` to ensure all tests pass. Run `pwsh -File scripts\install_shortcut.ps1` to compile `dist\engine.exe`, sync to `%LOCALAPPDATA%\Programs\Archetypes`, and refresh the pinned Taskbar icon.
- Files touched: `dist/*`, `%LOCALAPPDATA%\Programs\Archetypes/*`
- Expected outcome: The operator tests the new locomotion physics directly from their pinned Taskbar icon.

### Step 4 — Ledger Update, Commit & Push
- [x] Action: Mark plan COMPLETED, verify clean git status, and push to `origin/main`.
- Files touched: `docs/ledger/2026/09/plan_2026-09-09_0800_player_gravity_locomotion_and_flight.md`
- Expected outcome: Clean repository state on `main` pushed to `origin`.
