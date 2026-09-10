# Manifestation performance and quality correction

**Date:** 2026-09-09

**Repositories:** `C:\archetypes`, `C:\chronos2`

**Status:** SOURCE, LIVE CHRONOS, INSTALL, AND PINNED LAUNCH VERIFIED; PLAYER CLICK-THROUGH RE-WITNESS REQUIRED

## What was actually broken

The 191-second TripoSR run was real, but the prior explanation overstated the
evidence. TripoSR did not spawn 2,048 Python processes. It ran one process and
evaluated the 256-cubed field through many small decoder chunks. More
importantly, the live 4K Archetypes window itself occupied most of the GPU while
Chronos was trying to infer the mesh.

The strawberry failure was upstream of GLB placement. Dreamshaper 8 was still
being driven at 1024 by the checked-in workflow despite a document claiming it
had been changed to 512. It painted a furnished room containing many
strawberries. GrabCut retained ten disconnected components. TripoSR then did
exactly what it was asked to do and reconstructed the fragments and large slab.

## Implemented correction

- Dreamshaper 8 reference generation now runs at its native 512 square, 20
  steps, with the caller-supplied negative prompt actually wired into the
  workflow.
- The prompt requests exactly one centered product subject on a featureless
  white background and generically excludes rooms, frames, furniture,
  duplicates, collages, and other scene structure.
- Segmentation measures connected components and retains the significant
  non-border component nearest the image center. This is image geometry, not a
  noun list or recipe.
- TripoSR remains at 256 marching-cubes resolution. Decoder chunk size is now
  65,536, passed explicitly by the CLI and recorded with phase timings in the
  receipt.
- Archetypes runs at a bounded 60 Hz normally and 15 Hz during manifestation,
  restoring 60 Hz after every terminal path. The wait HUD and effects continue
  updating while CUDA receives the GPU budget.
- The generic Blender importer repairs invalid decimation remnants, smooths
  shading, retains vertex color, and caps generated objects at 75,000
  triangles.

## Live strawberry evidence

Bundle:
`%LOCALAPPDATA%\Temp\NeuroCognica\Archetypes\manifestations\verification-strawberry-optimized`

- Reference: 512 by 512, one centered strawberry.
- Segmentation: one significant component; retained foreground fraction 1.0.
- TripoSR: 93,878 vertices, 187,463 faces before game import.
- TripoSR timing: 38.808 seconds total while the old 4K game process was still
  running; 8.350 seconds model initialization, 8.058 seconds inference, 7.056
  seconds extraction, 0.692 seconds mesh export, with the remaining process
  time in Python/import startup.
- Generic game import: 187,461 to 74,997 triangles; GLB 20,243,404 to 1,654,328
  bytes; one re-imported mesh; `Color` vertex attribute preserved.
- Visual inspection: one coherent strawberry. It is not studio-grade
  photogrammetry, but the prior disconnected fruit cloud and wall slab are
  absent.

## Verification truth

- Chronos Weaver: 49 passed, 1 ignored.
- Chronos CLI: 162 passed, 3 ignored.
- Chronos full workspace: the first parallel run failed at Windows link time
  with `LNK1104`; the immediate `-j 1` rerun completed with zero test failures.
- TripoSR emitter Python tests: 3 passed.
- Archetypes full workspace before the final importer contract assertion: 112
  passed, 0 failed. The gate must be rerun after the final assertion.
- A real Chronos Object-mode `strawberry` run completed through Sentinel,
  ComfyUI, TripoSR, Blender, sealing, and bundle completion.
- A second `brass astrolabe` run completed the governed bundle in 47 seconds;
  TripoSR took 33.349 seconds. Its thin geometry is recognizable but rough,
  documenting the remaining single-view quality boundary.
- `scripts\install_shortcut.ps1` completed. Target, dist, and installed engine
  hashes match at
  `9D7AA3A94ADB83513660D29277B5DC4B471ECBFA5789BDFE1C8F7D4133AF421D`.
  The pinned Taskbar shortcut resolves to the installed launcher, and launching
  that exact shortcut produced the native title
  `Archetypes 1.0.5 (build 6) — Council Chamber`.
- Trusted Windows Computer Use was unavailable (`Trusted RPC service is not
  configured: sky`), so a single continuous post-change player click-through
  could not be automated. The earlier operator screenshots prove the same game
  dispatch path; the corrected downstream pipeline and new installed buyer
  launch were proven separately and still require one physical combined
  re-witness.

## Engineering failure report

The earlier handoff failed the operator in three specific ways: it documented a
512 workflow that still contained 1024, prescribed an unmeasured 128-resolution
change as though it were the answer, and claimed the pinned product was updated
without proving the running executable had changed. Those are verification
failures, not communication problems. The corrective standard is simple:
inspect checked-in values, benchmark the same input, run the buyer entry point,
compare installed hashes and timestamps, and leave any unproven gate explicitly
pending.
