# Plan: Import Archetype Council Characters (Aura, Sentinel, Oracle, Nebula Jester) — 2026-09-09 12:20

## Status
COMPLETED

## Goal
Import 4 new character models from `C:\Users\m\Downloads`:
1. `Meshy_AI_AURA_0909170750_texture.glb` -> `assets/scenes/aura.glb`
2. `Meshy_AI_Sentinel_0909170617_texture.glb` -> `assets/scenes/sentinel.glb`
3. `Meshy_AI_Oracle_0909170555_texture.glb` -> `assets/scenes/oracle.glb`
4. `Meshy_AI_Nebula_Jester_0909170446_texture.glb` -> `assets/scenes/nebula_jester.glb`

Each character will be:
- Grounded with feet flush at $Y=0$ and centered horizontally via headless Blender 4.5.
- Scaled so their eye line meets player eye height ($2.85\text{m}$) when standing on the stone floor.
- Arranged around the rotunda in an expansive radial/semicircular exhibition arc so the player can comfortably walk up to, circle around, and inspect each figure independently without crowding.
- Given tailored key and accent lighting tailored to their individual textures and silhouette.
- Given individual player obstacle collision circles so the player cannot ghost through them.
- Verified via `cargo test --workspace` and deployed to `%LOCALAPPDATA%\Programs\Archetypes` and the pinned Taskbar shortcut via `scripts/install_shortcut.ps1`.

## Steps

### Step 1 — Inspect & Process All 4 Character Assets via Headless Blender 4.5
- Action: Inspect mesh bounding box, eye height, texture nodes, center horizontally ($X=0, Z=0$), snap feet to $Y=0$, and export clean binary GLBs to `assets/scenes/`.
- Verification:
  - `aura.glb`: $13.5\,\text{MB}$, height $1.8990\,\text{m}$, eye level $1.7660\,\text{m}$, scale $1.614\times \implies 2.85\,\text{m}$.
  - `sentinel.glb`: $13.0\,\text{MB}$, height $1.8994\,\text{m}$, eye level $1.7664\,\text{m}$, scale $1.614\times \implies 2.85\,\text{m}$.
  - `oracle.glb`: $12.1\,\text{MB}$, height $1.9125\,\text{m}$, eye level $1.7787\,\text{m}$, scale $1.602\times \implies 2.85\,\text{m}$.
  - `nebula_jester.glb`: $14.6\,\text{MB}$, height $1.8976\,\text{m}$, eye level $1.7648\,\text{m}$, scale $1.615\times \implies 2.85\,\text{m}$.
- Files touched:
  - `scripts/export_characters.py`
  - `scripts/inspect_characters.py`
  - `scripts/inspect_colors.py`
  - `assets/scenes/aura.glb`
  - `assets/scenes/sentinel.glb`
  - `assets/scenes/oracle.glb`
  - `assets/scenes/nebula_jester.glb`
- Expected outcome: Grounded, centered, high-fidelity GLBs with 2K PBR materials ready for Bevy.

### Step 2 — Design Exhibition Arrangement & Lighting
- Action: Implemented grand exhibition crescent across the southern rotunda with ~5.7m separation between adjacent stations:
  1. Sentinel: $(-9.6, 0.0, 13.5)$, scale $1.614$, rot $140^\circ$, cold-white key + emerald/teal rim + visor pulse.
  2. Aura: $(-5.0, 0.0, 10.2)$, scale $1.614$, rot $157^\circ$, warm-solar key + cyan rim + golden core.
  3. Empath: $(0.0, 0.0, 7.5)$, scale $1.600$, rot $180^\circ$, crisp key + violet rim + cyan reactor glow.
  4. Oracle: $(5.0, 0.0, 10.2)$, scale $1.602$, rot $-157^\circ$, sapphire key + astral violet rim + third-eye indigo.
  5. Nebula Jester: $(9.6, 0.0, 13.5)$, scale $1.615$, rot $-140^\circ$, lavender key + neon magenta rim + cosmic spark.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: A stunning council gallery where each archetype stands proudly on the floor at player eye-level.

### Step 3 — Add Obstacle Collisions for All Figures
- Action: Added 5 obstacle boundary circles in `player_locomotion` within `crates/engine/src/modes/inner_chambers/camera.rs`, plus set player spawn to $(0.0, 2.85, 17.5)$ for an immediate panoramic view upon entering.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/camera.rs`
- Expected outcome: Player collides smoothly with each figure when walking up to them.

### Step 4 — Verification & Fast Dev Taskbar Refresh
- Action: Ran `cargo test --workspace` (Rule 5 Rust Gate) -> 100/100 tests passed.
- Ran `pwsh -File scripts\install_shortcut.ps1` -> release engine compiled, synced to `%LOCALAPPDATA%\Programs\Archetypes`, and refreshed pinned Taskbar shortcut `Archetypes.lnk` (Rule 13).
- Expected outcome: All 100 tests pass and launcher updated.

### Step 5 — Ledger Completion, Commit & Push
- Action: Update plan to COMPLETED, commit to `main`, and push immediately to `origin/main` (Rules 1-4).
- Expected outcome: Clean git audit on `origin/main`.
