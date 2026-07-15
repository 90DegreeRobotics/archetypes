# Plan: Lore Chamber Integration — 2026-07-15 05:39

## Status
COMPLETED

## Goal
Turn the supplied Blender chamber generator into a repo-owned, lore-compliant chamber asset and integrate it into the current blank-slate Archetypes launch path. The script must use the repository's canonical archetype names/colors instead of guessed lore, export a verified GLB, and the desktop-facing app must visibly load the new chamber without reviving the rejected legacy table/chamber/Mecha surfaces.

## Steps
### Step 1 — Ground the lore contract
- [x] Action: Read the canonical archetype registry/theme constants and map the supplied script's guessed names/colors to the real seven council archetypes.
- Files touched: plan only during this step.
- Expected outcome: The authored Blender source uses repo canon for archetype ids, display names, colors, seating, and node names.

### Step 2 — Create repo-owned Blender generator
- [x] Action: Add a hardened Blender Python script based on the supplied code, with fixed export paths, lore-correct archetypes, stable node names, validation output, and no one-off console assumptions.
- Files touched: `scripts/author_lore_chamber.py`.
- Expected outcome: The script can run headless or in Blender's Scripting tab, rebuilds idempotently, and exports a GLB under `assets/scenes/`.

### Step 3 — Generate and verify the GLB
- [x] Action: Run Blender if available, export the chamber, re-import/inspect the GLB, and capture a rendered proof frame. If Blender is unavailable, stop and report the blocker without pretending the asset exists.
- Files touched: `assets/scenes/lore_chamber.glb`, `artifacts/visual-proof/...`.
- Expected outcome: A real GLB exists with lights/cameras/geometry, correct archetype nodes, bounded scale, and visual proof.

### Step 4 — Wire the chamber into the launch path
- [x] Action: Load the new lore chamber in the default app path after the black intro while preserving old chamber/ritual/Mecha code in standby.
- Files touched: `crates/engine/src/chamber/mod.rs`, `crates/engine/src/chamber/boot.rs`, related docs/status as needed.
- Expected outcome: Desktop launch fades into the new chamber-backed main menu, not the old rejected scene, with modes still safely held in standby unless explicitly re-enabled later.

### Step 5 — Verify, package, witness, and publish
- [x] Action: Run the Rust gate, engine build, asset checks, `scripts\install_shortcut.ps1`, packaged launcher capture, screenshot inspection, docs update, commit, and push.
- Files touched: `STATUS.md`, this plan, generated proof artifacts.
- Expected outcome: `origin/main` contains the integrated lore chamber with clean status and actual runnable-app screenshots.

## Completion Evidence
- Blender 4.5 headless export/re-import succeeded for `assets/scenes/lore_chamber.glb`: `objects=121`, `meshes=104`, `lights=17`, `cameras=0`, `point_lights=16`, `triangles=35074`, `missing=[]`.
- Blender proof render: `artifacts/visual-proof/lore-chamber-2026-07-15_0539/blender_lore_chamber.png`.
- `cargo test --workspace` passed with 45 engine tests and 1 launcher test.
- `cargo build -p engine` passed.
- `scripts\install_shortcut.ps1` rebuilt release binaries, staged `dist`, verified Ollama/TTS dependencies, and refreshed Desktop / Start Menu shortcuts.
- Packaged launcher witness: `dist\launcher.exe` ran with `ARCHETYPES_LORE_CAPTURE=1`, all readiness probes reported ready, and `artifacts/visual-proof/lore-chamber-runtime-2026-07-15_0539/` contains black title, black creator-credit, and chamber-backed main-menu screenshots.
