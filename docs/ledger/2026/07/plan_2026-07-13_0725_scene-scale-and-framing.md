# Plan: Scene scale and framing — 2026-07-13 07:25

## Status
COMPLETED

## Goal
Assimilate Claude's completed table and chamber work, then correct the current composition by reducing the starfield density and particle scale, extending the animated portal treatment across the usable tabletop surface, raising the complete star-and-spheres council assembly as one authored group, and aiming the table camera slightly higher without disturbing the established table-first choreography.

## Steps
### Step 1 — Audit the inherited scene authority
- [x] Action: Inspect the clean committed tree, recent plans, Blender authoring scripts, runtime scene nodes, camera poses, and portal animation ownership.
- Files touched: None.
- Expected outcome: Every visual adjustment is made at its actual authority rather than guessed in Bevy.

### Step 2 — Refine authored starfield and council assembly
- [x] Action: Reduce star count and size, create or use a single parent for both tetrahedra and all seven spheres/panels, and raise that complete assembly.
- Files touched: Isolated Blender working copy, authoring/refinement scripts, chamber GLB.
- Expected outcome: Sparse restrained stars and a higher intact council constellation.

### Step 3 — Expand the tabletop portal effect and camera framing
- [x] Action: Scale the animated portal surface to fill the table's inner working surface and raise the table-view camera aim while preserving the opening/menu/table state contract.
- Files touched: Table authoring script/GLB and camera configuration.
- Expected outcome: Motion reads across the tabletop and the seated view angles upward enough to frame the raised council.

### Step 4 — Verify, install, publish
- [x] Action: Inspect exported scene structure, run `cargo test`, rebuild and stage the installed desktop runtime without launching it, update documentation, commit, push `main`, and leave the tree clean.
- Files touched: Plan/proof documentation and installed staging outputs.
- Expected outcome: A clean pushed audit surface and an explicit launch signal for the operator's visual verification.

## Verification
- Inherited tree was clean at `74ca225`; Claude's prior table, chamber, and starfield units were already on `origin/main`.
- Chamber GLB re-import: `Council_Assembly` exists at `z=2.0`; all seven vessels share it as parent; measured vessel centroid is exactly `z=2.0`; range is `-1.487..5.489`; 10 point lights; 32,262 triangles; no missing contract nodes.
- Table GLB re-import: `Stargate_Portal` is 1.44 units across within the 1.90-unit top and the complete table remains 25,138 triangles.
- Star cubemap: 180 single-texel stars per face, down from 1,300 plus clusters.
- `cargo test`: 18 passed, 0 failed across engine and launcher.
- Visual approval remains exclusively with the operator after desktop launch.
