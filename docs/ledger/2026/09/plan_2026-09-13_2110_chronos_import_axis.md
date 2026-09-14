# Plan: Chronos import axis — 2026-09-13 21:10

## Status
IN-PROGRESS

## Goal
Import Chronos2 Object-mode meshes the way they are authored (Z-up, front facing -Y), so the side
the reference picture shows is the side the altar presents first and the side a held object turns
toward the player.

## Evidence at start
- `scripts/import_chronos_object.py` imports with `forward_axis="NEGATIVE_Y"`, `up_axis="Z"`.
- Measured in Blender 4.5 on the aligned Hunyuan wolf: that declaration turns the mesh 180 degrees
  about Z (file vertex (0.319306, 0.158549, 0.934946) imported as (-0.319306, -0.158549, 0.934946),
  rotation 180.0). A camera at -Y then sees the back, which carries only nearest-visible fill
  colour; the camera at +Y sees the painted picture view.
- Chronos2 plan `plan_2026-09-13_2045_hunyuan_view_alignment.md` records the same defect in its
  showcase import and the decision to fix the importers rather than turn meshes in the emitter.
- Existing library objects are converted GLBs and are never re-imported, so they do not change.

## Steps
### Step 1 — Correct the declaration
- [ ] Action: import with `forward_axis="Y"`, `up_axis="Z"` after Blender confirms that is the
  identity for this convention; add a source test that pins the declaration.
- Files touched: `scripts/import_chronos_object.py`, a focused test in the engine crate.
- Expected outcome: imported vertices equal file vertices.

### Step 2 — Installed witness
- [ ] Action: `cargo test -p engine`, reinstall, installed capture of `a wolf` through both reviews
  and pickup, with the aligned emitter staged beside Chronos2.
- Files touched: evidence under `artifacts/visual-proof/`, this plan.
- Expected outcome: the review and held frames show the painted side; the operator judges the look.
