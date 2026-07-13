# Plan: Flower Table Asset — 2026-07-13 06:05

## Status
COMPLETED

## Goal
Regenerate `assets/scenes/table.glb` as the Lane C ornate Flower-of-Life astrolabe table while preserving the existing `Stargate_Portal` node and its approved vortex material/texture.

## Steps
### Step 1 — Preserve portal
- [x] Action: Review current `scripts/prepare_table.py` portal generation and keep the `Stargate_Portal` disc/material behavior intact.
- Files touched: None.
- Expected outcome: The portal contract survives the rebuild.

### Step 2 — Upgrade table shell
- [x] Action: Rework the table top, rim glyphs, Flower-of-Life inlay, pedestal, legs, feet, and under-glow.
- Files touched: `scripts/prepare_table.py`, `assets/scenes/table.glb`.
- Expected outcome: The physical table reads clearly and ornamentally at the existing local scale.

### Step 3 — Verify export
- [x] Action: Re-import `table.glb` headless and print `Stargate_Portal`, triangle count, and bounding box.
- Files touched: None.
- Expected outcome: Portal node is present, triangles remain below 200k, and scale is reported for Lane A.

## Verification
- Blender 4.5 re-import of `assets/scenes/table.glb`: `Stargate_Portal=present`, `triangles=38970`, `bbox_min=(-0.985,-0.985,-0.918)`, `bbox_max=(0.985,0.985,0.377)`.
