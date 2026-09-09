# Plan: Inner Chambers Animated Table Rotunda — 2026-09-09 07:07

## Status
COMPLETED

## Goal
Transform the Inner Chambers mode from an empty open void into a massive enclosed hall (castle great room with floor, perimeter stone walls, cross-beam ceiling, atmospheric illumination) featuring the animated Council Table (`scenes/table.glb` with its rotating, pulsing `Stargate_Portal` vortex) at its center, navigable via 360-degree free-flight camera so the player can view the animated table from any angle.

## Steps

### Step 1 — Author Enclosed Great Hall & Animated Table World Setup
- [x] Action: Refactor `crates/engine/src/modes/inner_chambers/world.rs` to spawn:
  1. A solid, dark stone floor slab ($76\text{m} \times 76\text{m}$) and decorative two-tiered circular stone dais ($R=6.0\text{m}$ step, $R=4.8\text{m}$ platform at $y = 0.30\text{m}$).
  2. Four enclosing perimeter walls ($76\text{m}$ span, $22\text{m}$ high) with buttress columns.
  3. An enclosed vaulted ceiling ($y = 22.0\text{m}$) with dark structural cross beams so the space is completely enclosed.
  4. Atmospheric castle lighting: 8 perimeter torch/brazier point lights and dual overhead illumination pools (central table light at $y = 8.5\text{m}$, vault fill light at $y = 18.0\text{m}$, and directional sun shaft).
  5. The animated `scenes/table.glb#Scene0` placed at the center ($y = 2.34\text{m}$, scale $2.6$), tagged with `InnerWorldElement` and `Name::new("PortalTable")` so `PortalPlugin` binds and spins the `Stargate_Portal` emissive vortex.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: The world loads a majestic enclosed hall with the animated portal table at its center.

### Step 2 — Position Camera for 360-Degree Free-Flight Orbit & Viewing
- [x] Action: Adjust `setup_camera` in `crates/engine/src/modes/inner_chambers/camera.rs` to spawn the player spectator camera at an elevated offset ($y = 4.2\text{m}, z = 9.5\text{m}$) looking directly down toward the table center, with tuned flight speed ($10.0$) and multi-key descend controls (`ShiftLeft`, `ShiftRight`, `KeyC`) for smooth inspection from any angle and altitude.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/camera.rs`
  - `crates/engine/src/modes/inner_chambers/extraction.rs`
- Expected outcome: Player immediately sees the animated table upon entering the mode and can fly around it effortlessly.

### Step 3 — Verification Gate & Desktop Staging
- [x] Action: Run `cargo test --workspace` to verify that all unit and integration tests pass without regression.
  - Result: 100 tests passed (79 engine, 16 launcher, 5 windows_identity), 0 failed.
- [x] Action: Run `pwsh -File scripts\install_shortcut.ps1` to stage the desktop launcher and verify `dist\`.
  - Result: `dist\engine.exe` built and staged cleanly with current timestamp.
- [x] Action: Commit and push cleanly to `origin/main` per AGENTS.md rules.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/modes/inner_chambers/camera.rs`
  - `crates/engine/src/modes/inner_chambers/extraction.rs`
  - `docs/ledger/2026/09/plan_2026-09-09_0707_inner-chambers-animated-table-rotunda.md`
- Expected outcome: The working tree is clean, tested, staged, and pushed.

