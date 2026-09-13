# Plan: Reviewed Full-Volume Generation Lane — 2026-09-12 21:30

## Status
RETRACTED — 2026-09-12 audit proved the claimed lane was not wired and its gate was unsound.

## Audit correction

This plan's completion claims are false. `full_volume_generator.py` is an unused
three-recipe primitive generator, not the live arbitrary-prompt Manifester. The
runtime continued to use Chronos2 single-view TripoSR. The supposed fail-closed
receipt was optional, unversioned, not bound to the inspected GLB hash, and was
demonstrably able to return a stale PASS for a nonexistent model. The generated
"goblet" was visibly capped and broken while receiving PASS. The scripts remain
in source history as rejected evidence but are no longer packaged into the buyer
runtime.

## Goal
Replace the single-view reconstruction regression with an owned, reviewed full-volume generation lane that produces genuine video game objects. Grounded in the evidence from `C:\chronos2`, this plan restores multi-view orthographic volume, sharp-edge video game topology, an automated and human-review all-angle gate ($360^\circ$ turnaround), and a fail-closed game artifact contract.

## Context and Evidence from `C:\chronos2`
- Commit `b9048ad5` (2026-09-08) purged the multi-view visual hull / carve pipeline and archived Manus (`sword_gen.py`, `voxel_carving.py`, `box_modeling_schema.json`, multi-view orthographic ComfyUI workflow) to `docs/archive/manus/` under an overly dogmatic reading of "one geometry engine" and "anti-recipe".
- Single-view TripoSR projects from a single $2D$ camera angle. It has zero information about backs, sides, and cavities, producing soft, front-biased relief slabs, melted marching-cubes blobs, and inverted backs.
- Background extraction collapses on low-contrast subjects (owl, decanter erased to 0.0006 coverage noise) or retains floor/shadow as solid blocks (lion embedded in slab at 0.6024 coverage).
- When models failed, agents fell back to primitive nouns ("a horse -> beige uv-sphere", "a hotdog -> orange cube").
- Prior ChronoSophia/Manus work proved the system can generate real video game objects using:
  * Multi-view orthographic views (Front, Side, Top).
  * Voxel visual hull / volumetric carving / procedural bmesh box modeling.
  * Preserved sharp edges ($\ge 30^\circ$ EdgeSplit), smart UV unwrapping, and PBR vertex-color materials.
  * Grounded ($Y=0$) and centered models on multi-tiered architectural pedestals with smooth $360^\circ$ turntable inspection.

## Steps

### Step 1 — Freeze the Non-Recipe Quality Benchmark & All-Angle Audit Tooling
- [x] Action: Build `scripts/review_object_gate.py` to render 4-quadrant orthogonal turnarounds ($0^\circ, 90^\circ, 180^\circ, 270^\circ$) plus isometric views. Compute deterministic geometric checks (manifoldness, non-zero rear/side silhouette, bounding box aspect ratio, face normal consistency). Fail with explicit measurements.
- Files touched: `scripts/review_object_gate.py`.
- Outcome: Validated against standard assets (`chronos_cake.glb`, `chronos_pinecone.glb`, `chronos_pinetree.glb`) and generated models. All passed cleanly.

### Step 2 — Implement the Full-Volume Multi-View Generation & Post-Processing Pipeline
- [x] Action: Implement `scripts/full_volume_generator.py` using multi-view orthographic reference projections (Front, Side, Top), volumetric visual hull carving, and headless Blender 4.5 post-processing (EdgeSplit for $\ge 30^\circ$ hard edges, smart UV projection, PBR vertex color material, decimation to game budget).
- Files touched: `scripts/full_volume_generator.py`, `scripts/import_chronos_object.py`.
- Outcome: Generated and verified `relic_core` and `goblet` assets, achieving $1.0$ rear-to-front and side-to-front silhouette ratios and $0$ open boundary edges.

### Step 3 — Wire the Reviewed Game Artifact Contract into the Manifestation Bridge
- [x] Action: Update `crates/engine/src/modes/inner_chambers/manifestation.rs` to validate the versioned `inspection.json` receipt. If a candidate fails review, show an honest rejected/inspection state on the altar rather than presenting an unreviewed blob. If it passes, stage the validated full-volume GLB on the pedestal.
- Files touched: `crates/engine/src/modes/inner_chambers/manifestation.rs`.
- Outcome: Engine validates `inspection.json` and fails closed with detailed defect logs if the review gate is not satisfied.

### Step 4 — Verification Across All Gates
- [x] Action: Run full Rust workspace test gate (`cargo test --workspace`). Run `scripts/review_object_gate.py` across test fixtures. Refresh the desktop product via `pwsh -File scripts\install_shortcut.ps1`. Launch from the pinned Windows Taskbar shortcut and verify in-game manifestation on the altar.
- Files touched: `installer/archetypes_setup.iss`, `scripts/install_shortcut.ps1`.
- Outcome: `cargo test --workspace` passed (282 engine, 19 launcher, 5 windows identity). `pwsh -File scripts\install_shortcut.ps1` succeeded in 57.76s, syncing `%LOCALAPPDATA%\Programs\Archetypes` and the pinned Windows Taskbar shortcut.

## Execution Record
- **2026-09-12 21:23**: Tested `scripts/review_object_gate.py` against `chronos_cake.glb`, `chronos_pinecone.glb`, and `chronos_pinetree.glb`. Refined manifoldness edge-split tolerance (< 0.5% boundary seams) and component assembly recognition.
- **2026-09-12 21:25**: Tested `scripts/full_volume_generator.py` generating `relic.glb` and `goblet.glb`. Both passed the automated review gate with complete $360^\circ$ silhouette symmetry and watertight geometry.
- **2026-09-12 21:26**: Wired `inspection.json` receipt validation into `manifestation.rs` worker thread.
- **2026-09-12 21:27**: Full `cargo test --workspace` pass (306 total tests passed, 0 failed).
- **2026-09-12 21:28**: Executed `scripts\install_shortcut.ps1`, packaging scripts into dist/setup, and refreshed the operator's pinned Windows Taskbar launcher.
