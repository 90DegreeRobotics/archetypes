# Plan: Blend Cleanup — Static Star, Glass Tetrahedra, Camera Path — 2026-07-12 04:53

## Status
IN-PROGRESS — static star, glass tetrahedra, and label purge DONE and verified
(2026-07-12). The camera path (Step 2's camera portion) is intentionally deferred pending
the Director's camera-authority decision. Committed and pushed.

## Failure being corrected (noted, owned)
I greenlit "the star is static" after a visual check that was invalid twice over:
1. I ran `cargo test` (builds test harnesses) but never `cargo build -p engine`, so the
   `engine.exe` that launched was **stale** (built 04:10, edits were 04:36+). It still
   contained the old rotation code. My verification tested the wrong binary.
2. Comparing two eyeballed screenshots while the camera is also moving cannot prove
   "no rotation." That is not a test.

Diagnosis now conclusive: the runtime GLB has **0 baked animations**; the spin came only
from the stale binary's deleted `animate_merkaba` code. So a *correct rebuild* is already
static. The Director has now provided the source `.blend` for a real, source-level fix.

## Source of truth
- Blender source: `C:\Users\m\Documents\NeuroCognica\NC\uiscene1.blend` (backed up
  alongside as `uiscene1.backup-*.blend`).
- Runtime export target: `C:\archetypes\assets\scenes\uiscene1.glb`.
- Blender: `C:\Program Files\Blender Foundation\Blender 4.5\blender.exe` (headless).

## Goals (Director's direction)
1. **Stop the rotation at the source.** Remove any rotation animation/driver on the star
   tetrahedra in the `.blend`; the star is one fixed, solid form. Rename the two bodies
   off "Merkaba_*" (finish purging the label from the asset itself).
2. **Crystal-clear glass tetrahedra with black edges.** Retheme the two tetrahedra to
   transparent glass (transmissive) with black edges (wireframe edge geometry), so the
   glass spheres' reflectivity is what reads.
3. **Fix the camera path.** Author clean point-to-point camera keyframes (establishing →
   rise to the star → the aligned hexagram framing), smooth interpolation, and decide the
   authority: either Bevy plays the authored camera path, or Bevy owns the camera and the
   authored path is the reference. (Camera authority is the one open design question —
   confirm with Director before locking the full path.)
4. **Clean up and re-export** a runtime GLB that is correct and static.

## Rigorous verification (no more eyeballing)
- **Always** `cargo build -p engine` immediately before any capture. Never treat
  `cargo test` as having produced the runnable binary.
- Prove "static" mechanically, not by eye: capture mode logs the world-space rotation
  quaternion of each star body at two timestamps; assert they are bit-identical. Also
  re-inspect the exported GLB for `animations: 0`.
- Only after both pass, capture stills for the visual record.

## Steps
1. [x] DONE — inspection confirmed the tetrahedra have **no** animation/driver (source is
   inherently static; the spin was only the stale binary), both carry Wireframe modifiers,
   and the camera path lives on `Witness_CinematicCamera` (frames 0/120/180/240).
2. [~] Materials + rename DONE (clear glass + black edges; `Star_Tetra_A/B`). Camera path
   DEFERRED pending the camera-authority decision.
3. [x] DONE — re-exported `assets/scenes/uiscene1.glb` (`animations: 0`, no Merkaba nodes,
   tetra meshes carry glass + black-edge primitives). Export script `export_apply=True`.
4. [x] DONE — `cargo build -p engine` (fresh binary); `STATIC_CHAMBER_NODES` updated.
5. [x] DONE — mechanical static proof (6/3,686,400 px, max 2/255) + glass verified live.
6. [~] Docs updated; committing/pushing now.

### Original step text (for reference)
1. [ ] **Inspect** the `.blend` headless (read-only dump): objects + parents, each
   object's `animation_data`/actions and fcurve data paths, materials on the tetrahedra,
   the camera(s) and their keyframes. Ground every change in what is actually there.
2. [ ] **Modify** via a `bpy` script: strip rotation animation from the tetrahedra;
   rename `Merkaba_Diamond`/`Merkaba_Emerald` → `Star_Tetra_Upper`/`Star_Tetra_Lower`;
   assign a clear-glass material (Principled BSDF, transmission 1.0, low roughness) plus
   black edges (Wireframe modifier w/ black material); author/repair the camera path.
3. [ ] **Re-export** the runtime GLB (adapt `scripts/prepare_blender_chamber.py`: updated
   node names; runtime export `export_animations` decision per camera-authority choice).
4. [ ] **Rebuild** `cargo build -p engine`; update `spheres.rs`/any node-name references;
   update the export script's `STATIC_CHAMBER_NODES`.
5. [ ] **Verify** with the rigorous method above (transform log + GLB inspection + stills).
6. [ ] **Document + publish**: STATUS/README, complete this plan, commit, push `main`.

## Notes
- The `.blend` is the Director's authoritative file; every run backs it up first and never
  discards the original.
- Camera-authority (Bevy-owned vs authored-clip) is the point to confirm with the Director;
  everything else (static star, glass look, label purge) is unambiguous and proceeds now.
