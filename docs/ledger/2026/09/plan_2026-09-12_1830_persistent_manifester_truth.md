# Plan: Persistent Manifester truth and player library — 2026-09-12 18:30

## Status

COMPLETE - all four steps shipped, gated and pushed to `origin/main` on 2026-09-12.
Plan authored by Codex; executed and amended by Claude, with the amendments recorded below.

## Goal

Turn the Manifester's currently partial persistence path into an honest player-owned object system while preserving the boundary that generated geometry quality belongs to Chronos2. Every successfully manifested artifact must retain a durable identity and provenance, placements must survive restart, and the player must be able to browse and summon a prior creation. The bridge must consume a verified Chronos2 game-artifact contract rather than infer success from a private bundle layout.

## Evidence at start

- `cargo test --workspace` passed on 2026-09-12: 248 engine, 19 launcher, and 5 Windows-identity tests.
- `objects.rs` implements take, carry, place, `R` duplicate, and an append-only placement ledger. It does not expose a library/summon interaction.
- `manifestation.rs` invokes Chronos2 and requires an OBJ plus the presence of `triposr_artifact.json`, but does not validate schema, hashes, source-prompt binding, Sentinel disposition, or a game-ready output digest.
- The quality-lab's multi-angle renders show that current single-view TripoSR output does not meet an all-around object-quality bar. This plan does not represent a presentation fix as a geometry fix.

## Steps

### Step 1 — Specify and validate the game artifact receipt

- [x] Action: Consume only a versioned Chronos2 game-artifact receipt that binds the prompt digest, Sentinel outcome, source mesh digest, game GLB digest, and no-recipe declaration; fail visibly on every mismatch.
- Files touched: manifestation bridge, focused tests, truth documentation.
- Expected outcome: Archetypes never stages a mesh merely because a process exited successfully or an internal path happens to exist.

### Step 2 — Make the player library reachable

- [x] Action: Add an in-game catalog interaction that reads the durable artifact library, presents player-created objects with their original prompt/provenance, and summons a chosen artifact into the player's hand without duplicating or overwriting the artifact file.
- Files touched: Inner Chambers interaction/UI/object systems, artifact service tests, player documentation.
- Expected outcome: Any retained player creation can be called back at any time; carrying, placement, and duplication remain one coherent lifecycle.

### Step 3 — Prove persistence and lifecycle end to end

- [x] Action: Exercise manifest-to-library, summon, place, duplicate, withdraw, reload, and re-summon against a real artifact fixture; refresh the desktop launcher and capture a visible buyer-path witness after runtime changes.
- Files touched: tests and designated visual-proof artifacts only.
- Expected outcome: The system is proven as a player-facing loop, not a collection of independent unit tests.

### Step 4 — Publish only proven state

- [x] Action: Update status/truth documents, run the full Rust gate, run `pwsh -File scripts\install_shortcut.ps1` for player-facing changes, inspect the installed launcher witness, stage explicit paths, commit, push `main`, and verify clean remote parity.
- Files touched: plan and required truth surfaces.
- Expected outcome: The main branch contains a truthful, auditable handoff.

## Dependency boundary

Chronos2 owns the high-quality full-volume generation engine and its game-artifact contract. This plan cannot make TripoSR's single-view inferred rear surface accurate; it refuses to hide that limitation.

---

## Execution record

### Amendments to the plan as written

**Step 1 asked for two fields that do not exist, and both were dropped deliberately rather than
faked.**

- *A game GLB digest.* Chronos2 cannot supply one: the GLB is produced afterwards by this
  repository's own Blender import, so no hash of it can exist in a bundle sealed before that ran.
  Archetypes measures it itself and records it on the library row. A check that can only ever
  fail is a check nobody keeps.
- *A no-recipe declaration.* No such field exists, and a bundle attesting to its own innocence
  would prove nothing. No-recipe stays enforced on the sending side, by
  `test_no_recipe_or_keyword_in_manifestation_dispatch`, which reads this repository's dispatch
  source and fails if it branches on prompt content.

Everything else Step 1 asked for turned out to be real and is now enforced: the versioned receipt
schema, Chronos2's own integrity verdict and Codex chain, the Sentinel's disposition, the prompt
binding, and the source-mesh and reference-image digests.

### What the work found that the plan did not anticipate

1. **A bundle carries two digest algorithms.** `manifest.json`'s `bundle_files` map is blake3
   (`chronos_cli`'s `hex_bytes`); `triposr_artifact.json` is sha256 (the Python emitter). Both
   render as 64 lowercase hex characters, so comparing the wrong one looks like a corrupt bundle.

2. **Hand-built fixtures cannot find a wrong contract.** Every fixture agreed with itself and
   passed while the contract was wrong. The test that reads bundles Chronos2 really wrote is what
   caught it, and it now runs against all 10 local bundles.

3. **The asset root was decided in two places and they disagreed.** Bevy resolves a relative
   `AssetPlugin.file_path` against the *executable's* directory, not the working directory.
   `paths::asset_root` is now the single source of truth, returns an absolute path, and is what
   both `main.rs` and the ledger's resolver read.

4. **A test that could never fail.** `resetting_restores_every_default` overwrote its fixture with
   the default and asserted the default equalled the default. The reset was inline in a Bevy
   system and unreachable from a test; it is now a function the test drives.

### Witness

`artifacts/visual-proof/library-2026-09-12/`, captured by walking with real keys through the
**installed** build. Frame 14 shows the library open over the Council floor with real rows; frame
15 shows the summoned artifact in hand with the carry hint, `carrying=yes` in the report and the
pickup cue counted. Zero `Path not found` lines in the engine log for the run.

**A harness trap worth not repeating.** The first capture ran `target/release/engine.exe`, which
has no `assets/` beside it, so every texture and every glTF silently failed to load. Objects the
report counted as standing appeared nowhere in frame, and it read exactly like a rendering bug in
the code just written. It was not. `walk_capture.rs` now documents the requirement, and the check
is the engine's own stderr: a correct run logs zero `Path not found` lines.

### Gate

`cargo test --workspace` - 282 engine, 19 launcher, 5 Windows identity, all passing, no warnings.
`scripts\install_shortcut.ps1` run; installed `engine.exe` verified at
`AC76484585D2F0CB956721FC96E724CED05808C2A81ED13FB3CD57BB587E8048`.

### Still open, and deliberately not claimed

The object-quality limits in
`docs/ledger/2026/09/report_2026-09-12_object_quality_unsolved.md` are untouched by this work.
Single-view reconstruction still invents the back of every object, and this plan's own dependency
boundary says so. Nothing here improves geometry; it makes what Chronos2 returns honest,
attributable and durable.
