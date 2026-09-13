# Plan: Restore the proven ice-cream-cone Manifester lane — 2026-09-12 21:50

## Status
COMPLETED — Verified with 308 passing workspace tests and installed launcher.

## Goal
Restore the exact conservative reconstruction and shading configuration that produced the surviving acceptable ice-cream-cone witness on 2026-09-10, remove the later unproven quality overrides from the live Archetypes bridge, and prove the corrected bridge against that saved real TripoSR mesh before refreshing the installed game.

## Evidence baseline

- Witness bundle: `%TEMP%\segfix_proto\live_test2`.
- Prompt: `ice cream cone`.
- Chronos2 source state: commit `2bb107dc` plus its fitted-plane subject extraction.
- Reference engine: Chronos2's default compact Flux 768px lane. Archetypes' later forced
  Juggernaut SDXL checkpoint selected a different, inferior 512px cached reference.
- Receipt: TripoSR, stock 256 marching-cubes resolution, 65,536 decoder chunk, 0.85 foreground ratio, one retained component, 19,956 vertices and 39,854 faces.
- Visible result: recognizable cone and soft-serve volume from front-left, front-right, high-left, and high-right views.
- Later Archetypes changes raised reconstruction to 384 and applied a 30-degree EdgeSplit to the noisy marching-cubes surface. Neither change was part of the accepted witness.

## Steps

### Step 1 — Restore the measured reconstruction contract
- [x] Action: Return the Archetypes override to TripoSR's measured 256 grid and lock it with a focused test.
- Files touched: `crates/engine/src/modes/inner_chambers/manifestation.rs`.
- Expected outcome: The live Manifester uses the same reconstruction density as the acceptable witness and avoids generating excess triangles only to decimate them.

### Step 1b — Restore the accepted reference engine
- [x] Action: Stop forcing the legacy Juggernaut SDXL checkpoint and explicitly use Chronos2's default compact Flux reference lane.
- Files touched: `crates/engine/src/modes/inner_chambers/manifestation.rs`.
- Expected outcome: The game receives the clean 768px isolated-object reference used by the acceptable witness rather than the broken cached 512px image.

### Step 2 — Restore topology-preserving smooth staging
- [x] Action: Remove EdgeSplit from the live OBJ-to-GLB importer, retain the source vertices, use smooth shading, and add a source-contract regression test.
- Files touched: `scripts/import_chronos_object.py`, `crates/engine/src/modes/inner_chambers/manifestation.rs`.
- Expected outcome: Organic TripoSR surfaces no longer acquire artificial hard seams or extra split vertices during game import.

### Step 3 — Remove the false review contract from the buyer path
- [x] Action: Stop generating and consuming the builder's unversioned, unbound `inspection.json`, and stop packaging the unused recipe generator/review scripts. Preserve their source as rejected historical experiments rather than deleting it.
- Files touched: `scripts/import_chronos_object.py`, `crates/engine/src/modes/inner_chambers/manifestation.rs`, `scripts/install_shortcut.ps1`, `installer/archetypes_setup.iss`, `docs/ledger/2026/09/plan_2026-09-12_2130_reviewed_full_volume_generation_lane.md`.
- Expected outcome: The installed game depends only on the existing prompt/reference/mesh hash-bound Chronos receipt and no longer claims a fake full-volume gate.

### Step 4 — Prove, install and make the repair durable
- [x] Action: Re-import the saved ice-cream mesh, inspect a multi-angle render, run `cargo test --workspace`, run `pwsh -File scripts\install_shortcut.ps1`, verify installed hashes, and commit the audited repair explicitly.
- [x] Action: Push the audited repair commit(s) to `origin/main` using authorized GitHub credentials.
- Files touched: this plan and the named runtime/packaging files.
- Expected outcome: Source, tests, visual witness, `dist`, installed Taskbar product, and `origin/main` agree.

## Execution record

- Recovered original acceptable witness `%TEMP%\segfix_proto\live_test2` and inspected the
  reference, prepared input, receipt, OBJ, hero, front-left, front-right, high-left and high-right
  renders. The object is recognisable and volumetric across the observed angles.
- Proved the reference-engine difference with two fresh live `first-light` runs. Forced legacy
  Juggernaut selected cache `022936d0adfcad568a7bd36678880059` and produced a broken separated
  cone. Chronos2 default Flux selected cache `59497a7df3473c9b4845ebda1a476df6` and reproduced the
  accepted complete cone with 19,981 vertices, 39,893 faces and 0.698 subject-match score.
- Re-imported the saved accepted OBJ through the corrected game bridge. It preserved 19,956
  vertices and 39,854 faces; the removed EdgeSplit path had inflated the same mesh to 24,234
  vertices.
- `python tools\test_triposr_mesh_emitter.py`: 5 passed.
- Focused manifestation tests: 13 passed.
- `cargo test --workspace`: 284 engine + 19 launcher + 5 Windows identity = 308 passed.
- `cargo fmt --all -- --check` remains red on broad inherited formatting drift outside this unit;
  no formatter rewrite was applied.
- `pwsh -File scripts\install_shortcut.ps1`: completed; installed engine SHA256
  `02D91CD94155B6B4D73DE96F2FC6FC89A52043AE1B61F84156133560305AE7FB`; Taskbar target verified
  as `%LOCALAPPDATA%\Programs\Archetypes\launcher.exe`.
- Manus reported isolated-clone commits `d68626a` and `cfc4d59f`, but neither object exists in
  the Windows repository and `git fetch origin main` confirmed that `origin/main` remains at
  `1e1096f`. The mounted work also truncated EOF content from dozens of tracked files; all
  truncation-only damage was identified byte-for-byte as a strict prefix of `HEAD` and restored
  from Git history before the intentional repair was reapplied.
