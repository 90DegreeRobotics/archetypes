# Plan: Football held-object witness — 2026-09-13 11:46

## Status
COMPLETED

## Goal
Generate a new football through the installed Archetypes Manifester, pick it up through the same interaction path used by a player, and produce a buyer-visible screenshot showing the football held in first-person. Extend the existing capture harness only as needed to make that end-to-end behavior repeatable and auditable.

## Steps
### Step 1 — Trace the live pickup and capture paths
- [x] Action: Read the manifestation capture state machine, altar pickup system, held-object presentation, and system ordering closely enough to identify the smallest truthful harness change.
- Files touched: None.
- Expected outcome: A concrete implementation that drives the real interact input and observes real held state.

### Step 2 — Add a held-object capture phase
- [x] Action: Add an opt-in capture mode that waits for successful manifestation, presses the normal pickup control, confirms the artifact is held, allows the held presentation to settle, and writes a screenshot and report evidence.
- Files touched: `crates/engine/src/modes/inner_chambers/manifest_capture.rs` and focused tests in the same runtime area as needed.
- Expected outcome: The harness cannot report a held witness unless the game actually transitions through its pickup path.

### Step 3 — Verify and refresh the installed product
- [x] Action: Run focused checks, `cargo test --workspace`, then `pwsh -File scripts\install_shortcut.ps1`; confirm installed engine freshness and launcher routing.
- Files touched: Build/install outputs only.
- Expected outcome: The Taskbar/Desktop product contains the verified change.

### Step 4 — Generate and inspect the football witness
- [x] Action: Launch the installed engine with prompt `football`, wait for Chronos reconstruction, inspect multi-angle reconstruction evidence, and capture the live held-object view.
- Files touched: Designated visual-proof artifacts plus the normal user artifact library.
- Expected outcome: A screenshot that visibly proves the newly generated football is held in game, with quality limitations reported honestly.

The exact prompt produced a hybrid soccer/American-football reference and a washed-out spherical reconstruction. A second installed-game run using `American football` produced the requested semantic reference, but TripoSR reconstructed an open, over-thick shell. Both objects were successfully taken through the real `E` path; the second run recorded `carrying=true`, artifact id `838b4a2b-3acc-4c89-8fde-d23586280881`, and one `CarriedVisual`. Chronos itself marked that second reconstruction `matches=false` with `0.4692` invented outline. Gameplay holding is proven; object quality is explicitly failed.

### Step 5 — Record, commit, and publish the completed unit
- [x] Action: Update this plan with actual evidence, restore any test-mutated tracked scene asset, commit only this unit, push `main`, and confirm clean local/remote parity.
- Files touched: This plan and the focused runtime/proof files from the steps above.
- Expected outcome: The implementation and witness are preserved on `origin/main` with no unrelated work included.

## Verification record

- Focused compile gate: `cargo test -p engine manifest_capture --no-fail-fast` compiled the engine successfully; no test name matched, so 288 tests were filtered rather than misreported as executed.
- Required runtime gate: `cargo test --workspace` passed `312/312` tests (`288` engine, `19` launcher, `5` Windows identity).
- Installed refresh: `pwsh -File scripts\install_shortcut.ps1` completed successfully.
- Installed engine SHA-256: `54DD72D49DB4A5EA6E443B9EF1A59F0D202B68D3611BF41CC144A78FF40A7C6B`.
- Installed launcher SHA-256: `D45E3CF72C96144DB47E5E66E86147EA5B9C1A6A38B496CDAA3EC0F651A49A65`.
- Pinned Taskbar shortcut target: `C:\Users\m\AppData\Local\Programs\Archetypes\launcher.exe`.
- Exact `football` live run: manifestation succeeded; pickup and one carried visual observed at `397.5s`; quality failed because the reference mixed soccer geometry with American-football laces and the reconstruction lost most material contrast.
- Disambiguated `American football` live run: manifestation succeeded; pickup and one carried visual observed at `372.1s`; quality failed because the mesh was an open, thick shell. Chronos recorded `matches=false`, score `0.5024`, and invented-outline fraction `0.4692`, but Archetypes still accepted it.
- Persistent installed artifacts: `bb63bb94-190c-4b95-ac16-3f5633018b3c.glb` and `838b4a2b-3acc-4c89-8fde-d23586280881.glb` remain individually addressable under the installed `assets/manifested` library.
- Shared `assets/scenes/manifested_artifact.glb` was restored from `HEAD` after the live runs; the per-artifact outputs and proof records were preserved.
- `git diff --check` passed. The only message was Git's informational LF-to-CRLF working-copy warning.
