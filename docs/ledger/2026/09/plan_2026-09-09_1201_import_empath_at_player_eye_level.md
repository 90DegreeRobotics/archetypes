# Plan: Import Empath Archetype at Player Eye Level on Floor — 2026-09-09 12:01

## Status
COMPLETED

## Goal
Import the Meshy AI Empath 3D model (`C:\Users\m\Downloads\Meshy_AI_Empath_0909165719_texture.glb`) into the Archetypes world.
In accordance with operator instructions:
1. Ground the mesh geometry with feet flush at $Y=0$ and centered at $X=0, Z=0$, and export to `assets/scenes/empath.glb`.
2. Place the Empath directly on the floor (no pedestal).
3. Scale the model so it stands eye-level with the player ($2.85\text{m}$ eye level).
4. Light the Empath from an angle that naturally complements the lighting, iridescence, and specular sheen baked into its material (soft frontal key with cyan/magenta rim and emissive chest/visor core illumination).
5. Add collision so the player cannot walk directly through the Empath figure.
6. Verify via `cargo test --workspace` and update the pinned Taskbar and Desktop launcher via `scripts/install_shortcut.ps1`.

## Steps

### Step 1 — Inspect & Process Empath Model via Headless Blender 4.5
- Action: Write and execute a headless Blender script to:
  - Ground feet at $Y=0$.
  - Center horizontally at $X=0, Z=0$.
  - Verify and preserve all 2K PBR textures (Base Color, Metallic/Roughness, Normal Map).
  - Export optimized binary glTF to `assets/scenes/empath.glb`.
- Files touched:
  - `scripts/export_empath_asset.py`
  - `assets/scenes/empath.glb`
- Expected outcome: Clean, centered, grounded GLB asset ready for Bevy.
- Verification: Export completed successfully (5,727,516 bytes).

### Step 2 — Position and Illuminate Empath in the Rotunda
- Action: In `crates/engine/src/modes/inner_chambers/world.rs`:
  - Spawn `SceneRoot(asset_server.load("scenes/empath.glb#Scene0"))`.
  - Position on the stone floor at $y=0$ in front of the dais (`Vec3::new(0.0, 0.0, 7.2)`), facing the player entrance.
  - Scale by $1.60\times$ so the glowing eye visor aligns with player eye height ($2.85\text{m}$).
  - Configure three-point tailored lighting:
    - Key Light (`Transform::from_xyz(1.8, 4.2, 9.8)`, intensity 95k, warm white).
    - Rim/Kicker Light (`Transform::from_xyz(-2.2, 4.0, 5.0)`, intensity 60k, purple/violet).
    - Core Resonance Glow (`Transform::from_xyz(0.0, 2.2, 7.5)`, intensity 18k, cyan).
  - Add physical player obstacle collision at `Vec2::new(0.0, 7.2)` with radius $0.85\text{m}$ in `crates/engine/src/modes/inner_chambers/camera.rs`.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/modes/inner_chambers/camera.rs`
- Expected outcome: The player stands face-to-face, eye-to-eye with the embodied Empath on the castle floor.

### Step 3 — Verification & Fast Dev Taskbar Sync
- Action: Run `cargo test --workspace` to ensure all tests pass. Run `pwsh -File scripts\install_shortcut.ps1` to sync `%LOCALAPPDATA%\Programs\Archetypes` and refresh the pinned Taskbar shortcut.
- Verification:
  - `cargo test --workspace` passed 100/100 tests cleanly (79 engine, 16 launcher, 5 windows_identity).
  - `scripts\install_shortcut.ps1` built release binaries in 1m 26s, staged to `dist\`, synced to `%LOCALAPPDATA%\Programs\Archetypes`, and refreshed pinned Taskbar shortcut `Archetypes.lnk`.

### Step 4 — Ledger Update, Commit & Push
- Action: Mark plan COMPLETED, commit with descriptive message, and push to `origin/main`.
- Expected outcome: Clean git audit on `origin/main`.
