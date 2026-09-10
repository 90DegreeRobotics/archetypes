# Plan: Manifestation quality, orientation, and lighting — 2026-09-09 21:51

## Status
COMPLETED

## Goal
Preserve the current no-recipe manifestation speed improvement and Sentinel adult-content refusal while making generated objects materially more coherent, upright, visibly rotating, and properly exhibited under useful lighting in the buyer-launched game.

## Steps
### Step 1 — Audit the failed manifestations
- [x] Action: Inspect the newest prompt bundles, reference and prepared images, mesh receipts, timing, geometry, and the live import/exhibition path to locate each failure before changing code.
- Files touched: None.
- Expected outcome: Evidence separates reference-generation defects, reconstruction defects, orientation defects, and game presentation defects.

### Step 2 — Improve quality without recipes or the old latency
- [x] Action: Implement generic image/geometry quality improvements and rejection evidence that never substitutes a premade mesh, noun recipe, or scripted object.
- Files touched: Chronos2 reference/reconstruction pipeline and tests as evidence requires.
- Expected outcome: Cleaner single-subject inputs and better reconstructed meshes while retaining the optimized reconstruction budget.

### Step 3 — Canonicalize exhibition and make motion visible
- [x] Action: Correct generic grounding/orientation, ensure the imported artifact rotates reliably, and add balanced key/fill/underglow lighting.
- Files touched: Archetypes manifestation/import/exhibition code and tests.
- Expected outcome: The object is upright, centered, continuously rotating, and readable from all sides on the manifestation pedestal.

### Step 4 — Verify the real buyer path
- [x] Action: Run focused checks, `cargo test --workspace`, real headless Chronos generation/import probes, and `pwsh -File scripts\install_shortcut.ps1`; compare installed binary/source timestamps and hashes.
- Files touched: Generated verification artifacts only in designated locations.
- Expected outcome: The pinned launcher runs the refreshed build and evidence distinguishes proven behavior from any remaining model limitation.

### Step 5 — Document and publish the completed unit
- [x] Action: Update required architecture/runbook documentation and this plan, inspect diffs, explicitly stage only owned changes, commit on `main`, push `origin main`, and prove clean remote parity.
- Files touched: Relevant docs and this plan.
- Expected outcome: Auditable completed work on `origin/main` with no unrelated user artifact claimed.

## Verification record

- The five supplied successful bundles ran TripoSR at 256 marching-cubes resolution and 65,536-point chunks in 20–23 seconds; the speed improvement was not a low-resolution fake.
- Reference/prepared inspection proved the recurring 90-degree display failure occurred after good inputs. Chronos declared Z-up while Archetypes allowed Blender's Y-up OBJ default.
- Fresh Godzilla, F-14, and Gandalf game-GLB witnesses proved the corrected axis path; standing subjects remained upright. The new universal reference wording requests level presentation for future aircraft-like inputs.
- The old universal negative prompt included `people, person, hands`, which caused `albert einstein` to become a lightbulb. Those semantic vetoes were removed in Chronos2 commit `a6d01151` without changing Sentinel or Object-mode attempt reachability.
- DreamShaper 8 still returned a wrong-identity generic figure. Juggernaut XL produced a recognizable Einstein caricature and completed the entire governed reference → TripoSR → Blender → seal path in approximately 43 seconds. TripoSR's single-view rear/face softness remains a known limitation.
- Manifestation exhibition now uses a directed warm key spotlight, neutral fill, restrained cyan underglow, and a 0.56 rad/s turntable (about 11.2 seconds per revolution).
- Python compilation and Blender 4.5 imports/renders passed. `cargo test --workspace` passed all 113 Archetypes tests.
- `scripts\install_shortcut.ps1` initially refused the still-running old installed processes and copied nothing. After stopping only those verified installed Archetypes processes, the rerun succeeded. Source, `dist`, and installed engine/launcher/artifact hashes match; the Taskbar shortcut targets the installed launcher.
- Sentinel adult-content refusal logic was not changed.
