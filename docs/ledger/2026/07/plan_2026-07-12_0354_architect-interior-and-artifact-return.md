# Plan: Architect Interior & Truthful Artifact Return — 2026-07-12 03:54

## Status
COMPLETED — 2026-07-12. All five steps done and proven end-to-end in the live GPU
window against a running Chronos Director (readiness: ready). See the outcome notes
under each step and the verification block in `STATUS.md`.

## Origin
Follows a hard-nosed reconciliation of the "New Direction" report against the live
code (see the report critique in session). The report is strategically sound but
conflates *Chronos capabilities* with *game features*. Two gaps between its promise
and the running app are the highest-value, self-contained increments — and both are
fully demonstrable in the live Bevy window without any new external dependency:

1. **`ArchitectInterior` is a caption, not a world.** The entire `ArchetypeTheme`
   system — including a fully specified `theme_architect()` "Luminous Blueprint"
   palette — is **dead code**. It is never applied to the rendered scene. Entering
   the Architect changes only a string in one bottom overlay `Text` node.
   `spheres.rs` even lets the focused sphere drift back out of center during
   `ArchitectInterior` because it only holds center on `FocusArchetype`.
2. **The returned artifact is never shown.** `ArtifactResult` prints the PNG
   *filesystem path* as text. By this project's own rule (THE UI IS THE ONLY GREEN
   CHECK), the artifact return is not done until the image is displayed in-game.

Explicitly OUT OF SCOPE (report over-reach we are NOT building now): seven archetype
worlds, Chronos lineage/mutation/family lanes, `WorldSeedArtifact` descent,
genealogical world memory, dual-sided sphere faces, audio. These stay off the
roadmap until the single Architect loop is green in the UI.

## Goal
Make the dead `ArchetypeTheme` system *live* by translating the focused archetype's
tokens into an actual rendered environment inversion, scoped to the one archetype the
slice reaches (Architect), via a general mechanism (driven by `CurrentFocus`, no
hardcoding). Hold the focused sphere at center through the interior. Replace the
printed PNG path with the actual returned image displayed in the chamber. Prove all
of it in the live window with screenshots.

## Steps

### Step 1 — Activate the archetype environment (world encapsulation)
- [x] DONE. `crates/engine/src/chamber/interior.rs` lerps `ClearColor` and
  `GlobalAmbientLight` toward `focus.0.theme()` inside the interior states. Proven in
  frame `05_architect_interior`: the void inverts from ceremonial near-black to the
  luminous silver-white Blueprint world.
- [ ] Action: Add an interior-environment system that lerps the world `ClearColor`
  and an accent light toward `focus.0.theme()` tokens during
  `FocusArchetype` / `ArchitectInterior` / `WitnessVerdict`, and back to the
  ceremonial dark void otherwise. This finally consumes `ArchetypeTheme` (currently
  dead) and inverts the Architect's void from near-black to luminous blueprint white.
- Files touched: `chamber/theme`-application (new system in engine), `theme/mod.rs`
  (helper to expose the void/accent as usable render values if needed).
- Expected outcome: Crossing into the Architect visibly changes the *law of the
  world*, not a caption — the environment inverts, driven generically by focus.

### Step 2 — Hold the focused sphere at center through the interior
- [x] DONE. `spheres.rs` now holds center across the whole interior tail, not just
  `FocusArchetype`. (Known refinement: dead-center overlaps the merkaba core; a
  forward focus corridor is future work, tracked in `STATUS.md`.)
- [ ] Action: Fix `spheres.rs` so the focused sphere stays centered and scaled
  across `FocusArchetype` AND `ArchitectInterior` (and the verdict/artifact tail),
  instead of drifting back out the instant the interior begins.
- Files touched: `chamber/spheres.rs`.
- Expected outcome: The Architect remains the sovereign center for the whole
  interior sequence; no visual regression on entry.

### Step 3 — Display the returned artifact image in-game
- [x] DONE. `ritual.rs` stages the verified PNG into `assets/artifacts/` and shows it
  as an `ImageNode`, only when Chronos returned a verified on-disk render. Proven in
  frame `08_artifact_result`: a real ~120 s Blender render displayed in-chamber with
  `Status: complete` and a real `C:\chronos\renders\...png` path.
- [ ] Action: In `ArtifactResult`, load the verified PNG from disk as a Bevy UI
  `ImageNode` and display it in the chamber, keeping the provenance line
  (status / artifact_id / proof id) as caption. Fail visibly if the image cannot be
  loaded (never claim a render that is not shown).
- Files touched: `chamber/ritual.rs`.
- Expected outcome: A real Chronos render returns *as an image the player sees*, not
  a filepath string. Display mechanism proven with a real on-disk PNG; full Chronos
  round-trip remains gated on Director + Ollama + codex readiness (fail-closed path
  already handled).

### Step 4 — Verify in the live window
- [x] DONE. `cargo test --workspace` 9/0; deterministic capture mode
  (`ARCHETYPES_CAPTURE`) drove the full ritual through the live GPU window and saved
  eight stage frames via Bevy's screenshot pipeline, against a ready Chronos Director.
  (Computer-use pixel-driving was unavailable: its resolver does not recognize a raw
  cargo dev `.exe`, so the self-capture mode is the reproducible substitute.)
- [ ] Action: `cargo test` and `cargo build -p engine`; run the engine headed; walk
  the ritual Onboarding → table → deliberation → Architect focus → Architect interior
  → verdict → artifact, capturing screenshots of the interior inversion and the
  artifact display (using a real local PNG for the display proof if the live Chronos
  Director is not ready on this machine).
- Files touched: tests as needed.
- Expected outcome: The two gaps are demonstrably closed on screen, not just in code.

### Step 5 — Document and publish
- [x] DONE. `README.md` and `STATUS.md` rewritten to the true ritual loop, interior,
  and artifact return (and the stale `Space`-loop text removed); this plan completed;
  committed and pushed to `origin/main`.
- [ ] Action: Update `README.md` and `STATUS.md` to reflect the now-live Architect
  interior and in-game artifact display (and honestly state what remains: other
  worlds, lineage, Chronos readiness dependency). Complete this plan, commit, push
  `main`.
- Files touched: `README.md`, `STATUS.md`, this plan.
- Expected outcome: Work exists on `origin/main` with a clean tree and truthful docs.
