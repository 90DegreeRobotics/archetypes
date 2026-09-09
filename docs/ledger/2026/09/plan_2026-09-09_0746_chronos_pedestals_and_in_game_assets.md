# Plan: ChronoSophia Objects on Lit Pedestals — 2026-09-09 07:46

## Status
COMPLETED

## Goal
Export recently created ChronoSophia 3D objects into optimized game assets (`.glb`), construct architectural lit display pedestals around the Council Chamber perimeter, and display the Chronos objects atop each pedestal with focused spotlights and subtle rotating showcases so the operator can inspect how Chronos creations translate into live in-game assets.

## Steps

### Step 1 — Export ChronoSophia Objects to `.glb` Assets
- Action: Using Blender in background mode (`C:\Program Files\Blender Foundation\Blender 4.5\blender.exe`), export the recent ChronoSophia models from `C:\chronos2\out\` into `assets/scenes/`:
  1. Ceramic Teapot (`chronos_teapot.glb`) from `out/object_triposr_only_20260908/scene.blend`
  2. Multi-tiered Altar Cake (`chronos_cake.glb`) from `out/universal_object_witness/wedding_cake_contrast_fix_refined_20260907/scene.blend`
  3. Golden Pinecone (`chronos_pinecone.glb`) from `out/universal_object_witness/pinecone_20260907_v2/scene.blend`
  4. Evergreen Pine (`chronos_pinetree.glb`) from `out/universal_object_witness/pine_tree_refined_20260907/scene.blend`
- Files touched:
  - `assets/scenes/chronos_teapot.glb`
  - `assets/scenes/chronos_cake.glb`
  - `assets/scenes/chronos_pinecone.glb`
  - `assets/scenes/chronos_pinetree.glb`
- Expected outcome: Four self-contained, high-fidelity `.glb` meshes with vertex colors/materials ready for Bevy PBR rendering.

### Step 2 — Construct Lit Exhibition Pedestals in Castle Rotunda
- Action: In `crates/engine/src/modes/inner_chambers/world.rs`, spawn 4 exhibition pedestals arranged symmetrically around the outer edge of the Council Dais ($R \approx 9.5\text{m}$ at 45°, 135°, 225°, 315°):
  1. Multi-tiered architectural pedestal: Dark obsidian octagonal column with stepped base and gold-trimmed display plinth.
  2. Dedicated lighting per pedestal: Focused downward spotlight and soft warm upward accent light highlighting the artifact.
  3. Load and position each ChronoSophia `.glb` asset atop its respective pedestal with proper bounding scale and orientation.
  4. Add an animated gentle turntable rotation component so the objects slowly revolve for 360° inspection.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: The castle chamber features four illuminated artifact showcases displaying Chronos objects around the animated Council Table.

### Step 3 — Verify & Update Desktop / Taskbar Launcher
- Action: Run `cargo test --workspace` to ensure all tests pass cleanly. Run `pwsh -File scripts\install_shortcut.ps1` to compile the new `dist\engine.exe`, sync to `%LOCALAPPDATA%\Programs\Archetypes`, and update the operator's pinned Taskbar icon.
- Files touched: `dist/*`, `%LOCALAPPDATA%\Programs\Archetypes/*`
- Expected outcome: The operator launches immediately from their pinned taskbar shortcut to view the Chronos objects on lit pedestals.

### Step 4 — Commit and Push to Main
- Action: Record completion in ledger, verify git status is clean, and push to `origin/main`.
- Files touched: `docs/ledger/2026/09/plan_2026-09-09_0746_chronos_pedestals_and_in_game_assets.md`
- Expected outcome: Complete, pushed commit on `main`.
