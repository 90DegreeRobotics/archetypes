# Plan: Council Chamber Asset — 2026-07-13 06:05

## Status
COMPLETED

## Goal
Regenerate `assets/scenes/uiscene1.glb` as the Lane B circular arched torch-lit council chamber while preserving the frozen node contract and avoiding all Rust files, table files, and Lane A work.

## Steps
### Step 1 — Preserve contract
- [x] Action: Import the current chamber GLB and keep all required vessels, panel nodes, `Witness`, and `Witness_Camera`.
- Files touched: None.
- Expected outcome: The authored chamber starts from the existing runtime contract.

### Step 2 — Replace shell
- [x] Action: Remove the old temple/star shell objects and add the circular floor, 10 arch bays, central dais, brass trim, and torch point lights.
- Files touched: `scripts/author_council_chamber.py`, `assets/scenes/uiscene1.glb`.
- Expected outcome: The chamber reads as a circular arched torch-lit space with open ceiling.

### Step 3 — Verify export
- [x] Action: Re-import `uiscene1.glb` headless and print contract, light, and triangle counts.
- Files touched: None.
- Expected outcome: All required nodes are present, at least 10 point lights export, and triangles remain below 150k.

## Verification
- Blender 4.5 re-import of `assets/scenes/uiscene1.glb`: `vessels=7`, `panel_nodes=21`, `point_lights=10`, `triangles=32262`, `missing=[]`.
