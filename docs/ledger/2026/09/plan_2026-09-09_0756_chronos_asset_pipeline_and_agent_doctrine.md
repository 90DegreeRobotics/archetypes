# Plan: Document Chronos-to-Game Asset Pipeline & Agent Doctrine — 2026-09-09 07:56

## Status
COMPLETED

## Goal
Fully document the end-to-end Chronos-to-Archetypes 3D asset conversion, architectural pedestal staging, dual-lighting doctrine, turntable showcase component, and mandatory pinned taskbar sync workflow. Codify this exact process into `docs/architecture/CHRONOS_TO_GAME_ASSET_PIPELINE.md` and enshrine it into `AGENTS.md` as canonical repository doctrine so every future session agent replicates this gold-standard workflow.

## Steps

### Step 1 — Author the Canonical Pipeline Guide
- Action: Create `docs/architecture/CHRONOS_TO_GAME_ASSET_PIPELINE.md` covering:
  1. Discovery of Chronos creation artifacts in `C:\chronos2\out\`.
  2. Headless Blender 4.5 execution, mesh extraction, grounding, bounding box normalization, and vertex color attribute preservation (`ShaderNodeVertexColor`).
  3. Standardized export parameters (`bpy.ops.export_scene.gltf`).
  4. Exhibition pedestal architecture, dimensions, materials, and coordinates.
  5. Dual-lighting doctrine (overhead crisp spotlight + thematic underglow point light).
  6. Smooth 360-degree turntable animation via `ChronosExhibitTurntable`.
  7. Verification, staging to `%LOCALAPPDATA%\Programs\Archetypes`, and refreshing the pinned Windows Taskbar shortcut.
- Files touched: `docs/architecture/CHRONOS_TO_GAME_ASSET_PIPELINE.md`
- Expected outcome: A comprehensive, copy-pasteable, battle-tested standard.

### Step 2 — Update AGENTS.md with Rule 16
- Action: Update `AGENTS.md` to enshrine Rule 16 ("Chronos-to-Game 3D Asset Import and Staging Doctrine — The Gold Standard"), mandating that whenever Chronos assets or 3D models are introduced into the game, agents follow this exact pipeline and verify via the pinned Taskbar launcher.
- Files touched: `AGENTS.md`
- Expected outcome: Enforced repository law for all future session agents.

### Step 3 — Ledger Update, Verification, Commit & Push
- Action: Mark plan COMPLETED, verify docs diffs and git status, commit, and push cleanly to `origin/main`.
- Files touched: `docs/ledger/2026/09/plan_2026-09-09_0756_chronos_asset_pipeline_and_agent_doctrine.md`
- Expected outcome: Clean repository state on `main` pushed to `origin`.
