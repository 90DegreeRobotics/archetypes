# Plan: Manifested Object Quality — 2026-09-12 19:30

## Status
IN PROGRESS — three changes shipped and measured. One root-cause fix is written up below and
**needs an operator decision**, because it edits a sibling product.

## Operator directive

> I want the quality increased. Objects must look real. Keep trying random objects and fine
> tuning

---

## The loop

`scripts/manifest_quality_lab.py`, in two deliberately separate phases:

```
python scripts/manifest_quality_lab.py --generate 6   # slow; keeps the bundles
python scripts/manifest_quality_lab.py --render       # fast; re-runnable after every change
```

Generating once and re-rendering the *same* meshes is what makes a tuning result mean anything:
Chronos2 output varies run to run, so a change judged against a fresh generation is judged
against noise. `scripts/render_object_turntable.py` renders each mesh from three angles on a
neutral studio rig — mid-grey ground, not white, because a white floor flatters everything
equally.

The test subjects span how reconstruction *fails*, not what it is about: hard-edged, organic,
thin-limbed, transparent, reflective, rough stone. **They are test inputs, not recipes** — see
`memory: no-recipes-is-the-north-star`. A fix that only helps one of them is a recipe and does
not ship.

---

## What the measurements say

| subject | subject_coverage | outcome |
|---|---|---|
| brass astrolabe | 0.4456 | recognisable |
| polished copper kettle | 0.1298 | recognisable |
| wrought iron candelabra | 0.0947 | recognisable, genuinely good |
| carved granite lion | 0.6024 | background included; lion embedded in a slab |
| **ceramic owl** | **0.0006** | noise |
| **cut crystal decanter** | **0.0006** | noise |
| relic (live run) | 0.1403 | partial; glass sphere cut away before reconstruction |

**The dominant variable is subject extraction, not reconstruction.** Where the image handed to
TripoSR is clean, TripoSR does a decent job. Where extraction collapses, nothing downstream can
recover — it is reconstructing a blank frame.

The two catastrophic subjects have one thing in common and it is not their subject matter: both
are close in colour to their own backdrop. A white ceramic figure on white, and clear glass on
grey.

## Ruled out, with evidence

- **Colour space.** GLB `COLOR_0` equals `srgb_to_linear(OBJ)` to four decimal places on every
  vertex sampled. The round trip is exactly right; there is no gamma fault. The pale gold in the
  first render was my own turntable key light.
- **`subject_match.json` as a quality gate.** It scored **0.260** for the destroyed owl and
  **0.259** for the good astrolabe. It does not discriminate here, and gating on it would reject
  good work while passing noise.
- **Vertex colours being dropped.** They survive import and export intact.

---

## Shipped

1. **Angle-based shading, and a real material.** `import_chronos_object.py` forced
   `use_smooth = True` on every polygon, rounding off every hard edge — a plinth, a camera body,
   a machined rim all came out melted on top of whatever softness marching cubes had introduced.
   Now edges above 30 degrees stay sharp, via an EdgeSplit that bakes the discontinuity into the
   geometry so it survives glTF rather than depending on smoothing metadata. The export also
   previously carried `COLOR_0` but **no material at all** (`materials: []`), leaving every
   consumer to invent one; it now writes an explicit material reading the vertex colour.
   Evidence: `artifacts/visual-proof/quality-2026-09-12/shading_before_after.png`.

2. **Marching-cubes resolution 256 to 384.** Chronos2 exposes `CHRONOS_TRIPOSR_MC_RESOLUTION`
   and defaults to 256; the engine now sets 384. Measured on the same cached reference so
   nothing else varied: 141,269 faces to 320,670 for five seconds more, and after both were
   decimated to the same 75k game budget the finer source kept the knurled case rim and dial
   face that 256 rounded away. Decimating from a finer surface picks better collapses than
   reconstructing coarsely to begin with. Evidence:
   `artifacts/visual-proof/quality-2026-09-12/mc_resolution_256_vs_384.png`.

3. **A coverage floor.** Below 2% coverage the manifestation now **fails with the measurement**
   instead of staging noise on the altar and calling it the player's work. The floor separates
   the two groups cleanly — nothing good measured below 0.09, nothing broken above 0.0006. The
   bundle is kept so the failure is inspectable.

---

## The root cause — needs an operator decision

It is in `C:\chronos2\tools\triposr_mesh_emitter.py::subject_mask`, and I could not apply it:
writing outside this repository was refused, correctly, because it changes a sibling product.

The function floods the border-connected backdrop, then refines with GrabCut seeded from pixels
"confidently" far from the measured background colour:

```python
confident = distance >= max(0.10, tolerance * 2.5)
...
return np.isin(grab_mask, (cv2.GC_FGD, cv2.GC_PR_FGD))
```

That seeding is **purely a colour distance**, so a subject close in colour to its backdrop
contributes almost no seed pixels. GrabCut then builds its foreground model from a handful of
stray specks — an orange beak, a glass highlight — and returns a mask containing almost nothing.
**That result is accepted unconditionally**, even though the flood mask it was refining was fine.

The proposed change is a guard, not a heuristic about subject matter:

```python
refined = np.isin(grab_mask, (cv2.GC_FGD, cv2.GC_PR_FGD))
flood_area, refined_area = int(initial_subject.sum()), int(refined.sum())
if flood_area > 0 and refined_area < flood_area * COLLAPSE_FRACTION:   # 0.35
    return initial_subject
return refined
```

It compares the two masks against each other and keeps the flood when the refinement has
collapsed. It carries no noun, material or expected shape, so it does not violate the no-recipe
law. Expected effect: the owl and the decanter stop being erased, which is the difference
between two of seven subjects being noise and none of them being noise.

Note also that the three copies of that emitter have **drifted** — `tools/`, `target/release/`
and `target/debug/` have different hashes, and the live one is `target/release/`, resolved from
`current_exe().parent()`. Any fix has to go to both the source of truth and the live copy.

## Still open

- The granite lion's opposite failure: coverage 0.6024 because the extraction *included*
  background, giving a lion relief embedded in a slab. The same guard does not address this;
  it needs an upper bound or a better border model.
- Single-view reconstruction cannot see an object's back. A flat-based subject photographed
  three-quarter on will always have an invented rear. Multi-view would be the real answer and is
  a much larger piece of work.
