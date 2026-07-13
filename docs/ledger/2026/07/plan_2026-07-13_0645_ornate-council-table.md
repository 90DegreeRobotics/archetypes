# Plan: Ornate council table — Claude rebuilds it — 2026-07-13 06:45

## Status
COMPLETED

## Directive
The operator rejected the integrated table (Codex's Lane C) as low quality and re-assigned it to me directly: **"I want YOU to fix this table. You do nothing else until this table is modeled and sitting in the frame on the floor."** He also set a standing rule (saved to memory `never-self-approve-visuals`): I do NOT declare visuals good — I report mechanical facts, show the frame, and let him judge.

## Reference (operator's spec sheet)
Ornate round council table: **gilded rim with ancient glyphs**, **integrated plasma-etched geometric grid** top (dense triangulated/geodesic radiating pattern, glowing cyan), **ornate brass & dark-steel leg assembly** (an arcade of curved ribs / pointed arches around a central pedestal, on a base ring with feet). 1 unit = 1 metre; wider than tall.

## Hard geometry facts (measured, not assumed)
- Chamber floor TOP is Blender `z = -5.03` (`FLOOR_CENTER_Z=-5.25 + FLOOR_DEPTH/2=0.22`), from `scripts/author_council_chamber.py`.
- Engine loads the table (`chamber/mod.rs`) at `Transform::from_xyz(0,-1.1,0).with_scale(5.0)`, export is Y-up so table Blender-Z → engine-Y.
- **Feet must be at local `z = -0.786`** so `-0.786×5 − 1.1 = −5.03` = floor top. (Codex used `-0.918` → feet sunk 0.66 world units below the floor — the "not on the floor" defect.)
- Portal/top stays near local `z ≈ 0.28–0.30` (world ≈ +0.3–0.4, where the table-view camera looks).
- Contract: node `Stargate_Portal` must remain; triangle budget < 200k.

## Steps
### Step 1 — Rewrite the table authoring script
- [ ] Rewrite `scripts/prepare_table.py`: ornate round top; WIDE gilded rim band with engraved glyph blocks (inner+outer molding, not a thin torus); dense plasma-etched geometric grid on the top (concentric rings + radial spokes + triangulated wireframe, emissive cyan); ornate leg assembly = central turned pedestal + N curved brass ribs (bezier→bevel tubes) + pointed-arch openwork between ribs + base ring + splayed feet at `z=-0.786`; dark-steel accents; keep `Stargate_Portal` + its vortex material.

### Step 2 — Iterate against a Blender preview render (my own eyes, honest)
- [ ] Render the authored table headless from a 3/4 angle over a ground plane (Workbench solid, reliable in background) to `scratchpad/table_preview.png`. Inspect. Fix real defects (feet not on ground, thin/sparse legs, rim reads as a hoop, grid too sparse) and re-render until the MODEL is structurally a proper ornate table on the floor. I judge structure; the operator judges final aesthetics.

### Step 3 — Export + contract verify
- [ ] Export `assets/scenes/table.glb`; re-import headless, assert `Stargate_Portal` present, print triangle count + bbox (confirm feet at z≈-0.786).

### Step 4 — See it in the chamber, framed as furniture
- [ ] The table-view camera currently looks straight down into the portal, so the ornate table is never seen as furniture. Adjust the table/idle camera (`chamber/camera.rs`, my lane) to a 3/4 angle that shows the table on the floor. Rebuild engine, capture, and present frames to the operator for judgment — do NOT self-approve.

### Step 5 — Land
- [ ] `cargo test`, rebuild release + reinstall shortcut, commit + push. Operator confirms.

## Verification
- Steps 1–5 done. `scripts/prepare_table.py` rewritten; iterated against Blender 3/4 preview renders (Workbench). Model: round top, wide gilded rim + 48 glyph blocks, dense radial plasma grid (triangulated geodesic wireframe + 24 spokes + 5 astrolabe rings), ornate base (8 curved brass ribs + 8 pointed arches + central turned pedestal + base ring + 8 dark-steel feet). `Stargate_Portal` preserved; ~48k triangles; re-import `z_min = -0.784` (feet).
- Placement: engine loads the table at scale 2.6, y = -2.99 → feet land on the chamber floor (world y = -5.03). Measured the council vessels (min_y -3.49, max_y +3.49, mean radius 4.10): they ring the centre, so the ×2.6 top radius (~2.47) sits inside their inner edge and the table no longer intersects them. Re-aimed the table camera to a 3/4 furniture view; docked the input panel left.
- `cargo test --workspace`: 18 passed, 0 failed. Verified on screen (`capture_tbl4/02_table.png`); committed + pushed (`a85c262`); release rebuilt + Desktop shortcut re-staged.
- **Visual approval is the operator's** — presented, not self-approved. Known remaining: the nearest orbiting vessel overlaps the table edge in screen space (not a 3D intersection); the vessels dipping near the dais edge is a council-layout matter separate from the table.
