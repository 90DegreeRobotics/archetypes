# Report: Manifested Object Quality — what is solved, and what is not

Date: 2026-09-12. Written at operator request: *"Solve what you can and create a report about
what you have yet to solve."*

Everything below is measured on this machine from artifacts kept under `artifacts/quality-lab/`
and `artifacts/visual-proof/quality-2026-09-12/`. Where a number is quoted, the file it came from
is named. Nothing here is an estimate.

---

## How quality is now measured

`scripts/manifest_quality_lab.py` generates a spread of subjects once and re-renders the *same*
meshes after each change. That separation is the whole point: Chronos2 output varies run to run,
so a change judged against a fresh generation is judged against noise.

The test subjects — astrolabe, ceramic owl, candelabra, crystal decanter, copper kettle, granite
lion — span how reconstruction *fails*, not what it is about. **They are test inputs, not
recipes**; a fix that helps only one of them is a recipe and does not ship.

The single most useful number is `preparation.subject_coverage` from each run's
`triposr_artifact.json`: what fraction of the reference image survived extraction to become the
thing TripoSR reconstructs.

---

## Solved

### 1. The live subject-extraction code was stale

`tools/triposr_mesh_emitter.py` is chronos2's source of truth and fits a background **plane** to
handle the soft vignette on a white sweep. `target/release/triposr_mesh_emitter.py` — the copy
`chronos.exe` actually loads, resolved from `current_exe().parent()` — still used a single global
border median, 34 lines shorter. The fix existed and had never run.

Its own comment describes the failure measured today: *"a pale subject swallowed wholesale (that
run's mask kept 0.18% of the frame — a sliver, not a cone)."*

All three copies (`tools/`, `target/release/`, `target/debug/`) are now identical at 485 lines.

### 2. Component selection was discarding the subject for a speck

**This was the actual cause of both catastrophic failures, and it was not where I first looked.**

`select_central_component` ranked candidates by `(centre_distance, -area)` — centre distance
first, area only as a tiebreak — and built its pool as `interior or candidates`, so a *single*
interior speck excluded every border-touching component from consideration. A subject large
enough to fill the frame touches the border.

The measurement that found it: the owl's mask retained **25.84%** of the image
(`raw_subject_coverage`), and selection then handed TripoSR **0.06%** (`subject_coverage`) — a
`retained_foreground_fraction` of 0.0025. The extraction was fine; the selection threw the body
away.

Area is now the quantity being maximised, with centrality as a penalty on it rather than the
primary key, and the interior preference is overridden when it would discard a body more than
three times larger than what it prefers.

Measured result, same prompts, fresh runs:

| subject | coverage before | coverage after | retained fraction |
|---|---|---|---|
| ceramic owl | 0.0006 | **0.2280** | 0.0025 to 0.8826 |
| cut crystal decanter | 0.0006 | **0.1932** | 0.0025 to 1.000 |

Both now reconstruct recognisably — the owl has its eyes, beak, ears and gold feet; the decanter
has its stopper, neck and faceted body where it previously came back as a flat slab. Evidence:
`component_selection_before_after.png`, `decanter_before_after.png`.

### 3. Glass and pale subjects erased by the colour test

Implemented the design already measured in `chronos2/docs/audits/glass_and_transparency.md`: run
the colour mask, measure the largest connected component's share of the retained area, and **only
when the subject has visibly been cut apart** fall back to an edge-closure silhouette (Canny at a
low threshold, dilate, close, flood the outside, keep what the flood cannot reach).

Glass transmits the backdrop, so by a colour test glass *is* background. It is not invisible to a
gradient test.

The gate is what makes this safe. The audit's rocking chair, with five genuine gaps, scores 1.000
and correctly does not trigger — edge closure would weld its holes shut.

**Honest note on what this actually contributed.** On the re-runs above the fallback did *not*
fire: once component selection stopped discarding the subject, the colour mask alone produced a
coherent body for both the owl and the decanter. So the glass fallback is shipped and armed, but
the improvement measured today is attributable to the selection fix and the stale-copy sync, not
to it. It remains the right mechanism for the audit's hourglass case; it has not yet been shown
to fire on a subject in this spread.

### 4. Objects were force-smoothed and had no material

`scripts/import_chronos_object.py` set `use_smooth = True` on every polygon, rounding off every
hard edge on top of whatever softness marching cubes had introduced. Edges above 30 degrees now
survive, baked into the geometry via EdgeSplit so they hold through glTF rather than depending on
smoothing metadata. Evidence: `shading_before_after.png`.

The export also carried `COLOR_0` but **no material at all** (`materials: []`), leaving every
consumer to invent one. It now writes an explicit material reading the vertex colour.

### 5. Geometry resolution

`CHRONOS_TRIPOSR_MC_RESOLUTION` raised from Chronos2's default 256 to 384. Measured on the same
cached reference so nothing else varied: 141,269 faces to 320,670 for five seconds more, and
after both were decimated to the same 75k budget the finer source kept the knurled case rim and
dial face that 256 rounded away. Evidence: `mc_resolution_256_vs_384.png`.

### 6. Noise is no longer presented as the player's work

A subject-coverage floor of 0.02 fails the manifestation with the measurement rather than staging
a blob on the altar.

### Ruled out with evidence, not assumption

- **Colour space is correct.** GLB `COLOR_0` equals `srgb_to_linear(OBJ)` to four decimal places
  on every vertex sampled. There is no gamma fault. An earlier suspicion of one was my own
  turntable key light.
- **`subject_match.json` is not a usable gate.** It scored 0.260 for the destroyed owl and 0.259
  for the good astrolabe.

---

## Not solved

### A. The owl still loses part of its body

Coverage went from 0.0006 to 0.2280 and the result is recognisable, but the image handed to
TripoSR still has a section bitten out of the owl's white flank where it met the white backdrop,
and the mesh is scooped there. The fragmentation gate does not fire because what survives is one
coherent body — it is missing a piece, not cut into lumps. Detecting "coherent but incomplete"
is a different measurement and has not been attempted.

### B. The opposite failure: extraction that includes too much

The granite lion measured **0.6024** coverage — not too little, too much. The mask kept a
rectangular slab of backdrop along with the subject, and the reconstruction is a lion relief
embedded in a block.

The fragmentation gate does not help: that mask is one coherent body, so its largest-component
share is high and the gate correctly declines to fire. What is needed is an upper bound, or a
border model that does not accept a straight-edged region as subject. **No measurement has been
done on this yet** — one sample is not a basis for a threshold.

### C. Single-view reconstruction invents the back of every object

TripoSR sees one image. A subject photographed three-quarter on has a rear the model has never
seen, so it is confabulated — this is why the relic's plinth came back as an open bent frame and
why the astrolabe's reverse is a smooth lump.

No amount of extraction or resolution tuning addresses this. The real answer is multi-view
reconstruction, which is a substantially larger piece of work and an engine change, not a
parameter.

### D. Edge closure returns a solid, not glass

For a transparent subject the closure gives the correct *shape* — a decanter-shaped body beats
three floating lumps — but it is opaque. TripoSR bakes vertex colour, and the colour behind clear
glass is the backdrop, so the body comes out pale and flat where the glass was.

Making it read as glass means assigning a transmissive material at staging time. Nothing
currently does that, and doing it from the prompt text would be a recipe. A measured route might
be to detect low colour variance within the closure region against high gradient energy at its
boundary — untested, and I am not proposing it as done.

### E. Thresholds are under-calibrated

- `FRAGMENTATION_TRIGGER = 0.70` is read off **five** references in the chronos2 audit, which
  states plainly that it must be run across all 40 before shipping, checking specifically that no
  row with real holes starts firing. It has now shipped at operator instruction; that limit is
  unchanged and is recorded here rather than quietly dropped.
- `MIN_SUBJECT_COVERAGE = 0.02` separates seven measured samples cleanly — nothing good below
  0.09, nothing broken above 0.0006 — but seven is seven.

Neither has been run against the full corpus.

### F. The three emitter copies can drift again

They were at three different hashes, and the live one was the stale one. Nothing copies
`tools/` into `target/` — no `build.rs` references it — so they are kept in step by hand. They
are in step now. There is no mechanism stopping them diverging again, and the failure mode is
silent: the product runs the old code and every symptom points somewhere else.

### G. Canny thresholds are tuned to a clean studio backdrop

12/40 comes from the audit and is tuned to the white sweep the reference generator produces. A
busy or low-contrast reference is untested.

---

## What I would do next, in order

1. Run the fragmentation gate across the full 40-reference corpus and calibrate 0.70 properly,
   watching for rows with genuine holes that begin to fire.
2. Measure the over-inclusion failure (A) on more than one sample before proposing a bound.
3. Make `tools/` the only copy, or have something copy it at build time, so (E) cannot recur.
4. Treat multi-view (B) as its own piece of work; it is the ceiling on everything else.
