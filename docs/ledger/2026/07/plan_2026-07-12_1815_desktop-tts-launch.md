# Plan: Desktop TTS launch repair — 2026-07-12 18:15

## Status
COMPLETED

## Goal
Repair the installed desktop launch path so the launcher finds the declared offline Kokoro/sherpa-onnx voice runtime, starts the real engine from the staged Windows layout, and no longer tells the operator to run a repository-only setup command.

## Steps
### Step 1 — Trace installed-path truth
- [x] Action: Compare launcher readiness paths, dependency manifest destinations, bootstrap staging, installed files, and shortcut target/working directory.
- Files touched: This plan only during diagnosis.
- Expected outcome: Exact mismatch between the installed layout and the readiness probe.

### Step 2 — Repair the Windows metabolism
- [x] Action: Correct the owning manifest/bootstrap/launcher path logic, install or stage the declared voice artifacts idempotently, and add regression coverage.
- Files touched: Launcher, scripts, dependency manifest, and focused tests as required.
- Expected outcome: Installed voice readiness succeeds without repository-relative assumptions.

### Step 3 — Rebuild and prove desktop launch
- [x] Action: Run `cargo test`, rebuild/stage the release, refresh Desktop and Start Menu shortcuts, invoke the installed launcher with its shortcut working directory, and verify the engine process starts.
- Files touched: Generated `dist`/installed outputs and proof logs under designated locations.
- Expected outcome: Desktop entry reaches the live chamber instead of the missing-TTS stop screen.

### Step 4 — Publish
- [x] Action: Update truth docs, complete this plan, commit and push `main`, and leave the repository clean.
- Files touched: README/STATUS/plan and implementation files.
- Expected outcome: The repair is reproducible from `origin/main`.

## Outcome

- Root cause: launcher and engine only checked `%ProgramFiles%\Archetypes\speech`, while the Desktop shortcut targeted the portable `C:\archetypes\dist` installation and the complete runtime existed only under a proof directory.
- Installed runtime: sherpa-onnx v1.13.4 plus Kokoro `kokoro-en-v0_19`, including `model.onnx`, `voices.bin`, `tokens.txt`, and phoneme data, under `dist\speech`.
- Seven archetype voice mappings remain distinct speaker IDs within the installed 11-speaker Kokoro model.
- `cargo test` passed: engine 11 tests, launcher 1 test.
- `cargo build --release --workspace` passed.
- Desktop shortcut proof: launching `C:\Users\m\Desktop\Archetypes.lnk` started `C:\archetypes\dist\launcher.exe` and then the live `C:\archetypes\dist\engine.exe` chamber window.
