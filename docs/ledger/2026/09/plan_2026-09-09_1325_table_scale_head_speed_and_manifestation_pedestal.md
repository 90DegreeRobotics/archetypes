# Plan: Table Rescaling, Responsive Mouse Look, and Live Chronos2 Manifestation Pedestal — 2026-09-09 13:25

## Status
COMPLETED

## Goal
1. Reduce the Council Table size to 60% of its current size (scale 1.56, collision radius 1.41m, resting flush on dais top).
2. Speed up mouse look head movement (increase camera sensitivity from 0.00045 to 0.0013, expand clamping for smooth responsive FPS look).
3. Implement the Manifestation Pedestal:
   - Placed in the rotunda court on the dais at `(0.0, 0.30, 3.4)` facing the player.
   - Walking up displays an interactive `[E] Manifest Artifact` prompt.
   - Pressing 'E' opens a clean text input modal to type a creative prompt.
   - Connecting in the backend to headless Chronos2 / Blender 4.5 pipeline to generate and stage the 3D asset live into `assets/scenes/`.
   - Hovering animated wait indicator (spinning celestial gyroscopic rings + pulsing beacon) in the air above the pedestal while rendering.
   - The wait indicator disappears right before the object manifests.
   - Object appears rotating atop the pedestal via `ChronosExhibitTurntable` with dual lighting and a celebratory manifestation radiance burst.
4. Verify via `cargo test --workspace` and update the pinned Taskbar and Desktop launcher via `scripts/install_shortcut.ps1`.

## Steps

### Step 1 — Rescale Table and Tune Mouse Look
- Action:
  - In `crates/engine/src/modes/inner_chambers/world.rs`, rescaled `RotundaCouncilTable` by 60% (`Vec3::splat(1.56)`), adjusted y-offset to 1.523m, and adjusted `PortalDiscGlowLight`.
  - In `crates/engine/src/modes/inner_chambers/camera.rs`, updated table collision radius to 1.41m.
  - Increased mouse look sensitivity to 0.0013 and relaxed delta clamp to 240.0.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/modes/inner_chambers/camera.rs`
- Expected outcome: Table is balanced in size, head movement is responsive and smooth.

### Step 2 — Manifestation Pedestal & Wait Indicator System
- Action:
  - Added Manifestation Pedestal entity with ornate architectural plinth, shaft, capital, gold rim, and velvet cushion.
  - Created `ManifestationState` resource tracking `Idle`, `Prompting`, `Manifesting`, and `Completed`.
  - Created 3D wait indicator entity above the pedestal that spins along multiple axes and pulses during manifestation and despawns before appearance.
  - Implemented proximity detection and keyboard interaction (`[E]` key, typing buffer, enter/escape).
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
  - `crates/engine/src/modes/inner_chambers/mod.rs`
  - `crates/engine/src/modes/inner_chambers/camera.rs`

### Step 3 — Backend Headless Chronos2 Generation Pipeline
- Action:
  - Created background worker dispatcher in `manifestation.rs` that calls headless Blender 4.5 exporter (`scripts/manifest_artifact.py`).
  - Exports game-ready GLB to `assets/scenes/manifested_artifact.glb`.
  - Live-spawns the manifested scene entity onto the pedestal with `ChronosExhibitTurntable` and celestial entrance flash.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
  - `scripts/manifest_artifact.py`
  - `assets/scenes/manifested_artifact.glb`

### Step 4 — Verification & Fast Dev Taskbar Refresh
- Action: Ran `cargo test --workspace` (Rule 5 Rust Gate) -> 100/100 tests passed.
- Ran `pwsh -File scripts\install_shortcut.ps1` -> compiled release binaries, synced to `%LOCALAPPDATA%\Programs\Archetypes`, and refreshed pinned Taskbar shortcut `Archetypes.lnk` (Rule 13).

### Step 5 — Ledger Update, Commit & Push
- Action: Mark plan COMPLETED, commit to `main`, and push immediately to `origin/main` (Rules 1-4).
- Expected outcome: Clean audit on `origin/main`.
