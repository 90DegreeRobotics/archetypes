# Plan: Manifestation Pipeline and Broken Windows Release Recovery — 2026-09-09 17:55

## Status

COMPLETED

## Goal

Recover the broken 1.0.4 Windows release and complete the manifestation pipeline under the absolute NO-RECIPE LAW.

1. Repair formal speech packaging, dependency manifest staging, and user-writable speech repair under `%LOCALAPPDATA%\NeuroCognica\Archetypes`. Align speech root precedence across launcher and engine and replace raw Notepad failure with a proper buyer error surface.
2. Complete the no-recipe manifestation pipeline connecting Chronos2 Object mode (TripoSR) with live streaming `[chronos-stage]` progress, Sentinel review/refusal handling, clean headless OBJ-to-GLB conversion, and reliable process tree termination on cancel/exit.
3. Add physical stone courses / mesh relief to the chamber floor that reacts to chamber lighting and is visibly legible from the normal player camera.
4. Enhance the magical reveal sequence (charge, lightning, smoke, dissipation) tied to actual generation stages.
5. Advance product identity to 1.0.5 (build 6), commit tested source before packaging, build an Azure-signed installer to Downloads, install over 1.0.4 into Program Files, and verify end-to-end through the pinned Taskbar icon.

## Steps

### Step 1 — Audit & Fix Speech Packaging and Writable Repair

- [x] Action:
  - In `installer/archetypes_setup.iss`, package `dist\speech\*` into `{app}\speech`, `scripts\dependencies.json` into `{app}\scripts`, and `scripts\import_chronos_object.py` into `{app}\scripts`.
  - In `installer/build.ps1`, ensure `dist\speech` is verified and ready before ISCC compilation.
  - In `crates/launcher/src/main.rs`, update `repair_offline_voices` to target user-writable `%LOCALAPPDATA%\NeuroCognica\Archetypes\speech` when running from Program Files or non-writable locations.
  - Align `speech_roots()` in `crates/launcher/src/main.rs` and `crates/engine/src/chamber/speech.rs` to check:
    1. `ARCHETYPES_SPEECH_ROOT`
    2. Portable `<exe_dir>\speech`
    3. `%LOCALAPPDATA%\NeuroCognica\Archetypes\speech`
    4. `%ProgramFiles%\Archetypes\speech`
  - Replace raw Notepad failure with a proper buyer-facing status/error dialog (native Windows MessageBox API `MessageBoxW`).
  - Add unit and integration tests for speech root precedence, writable repair path, and dependency manifest loading.
- Files touched:
  - `installer/archetypes_setup.iss`
  - `installer/build.ps1`
  - `crates/launcher/src/main.rs`
  - `crates/engine/src/chamber/speech.rs`

### Step 2 — Audit Recipes & Build Streaming Manifestation Bridge

- [x] Action:
  - Search both repos for `recipe|preset|fallback|primitive|keyword|mock|stub` and classify matches.
  - Add fail-closed regression tests prohibiting recipe execution.
  - In `crates/engine/src/modes/inner_chambers/manifestation.rs`, stream Chronos subprocess stdout/stderr line-by-line.
  - Parse `[chronos-stage] id=<id> pct=<pct> state=<state> msg=<msg>` events in real time.
  - Drive floating hourglass animation and HUD banner directly from real stage ID, percentage, state, and message.
  - Display real Sentinel review and explicit Sentinel refusal reasons without invented game-side filters.
  - Consume generated OBJ (`bundle/engine_mesh/0/mesh.obj` or `bundle/engine_mesh/mesh.obj`) via `scripts/import_chronos_object.py`.
  - Enforce clean child process tree termination on cancel (`Escape`) and game exit via `taskkill /PID <pid> /T /F`.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
  - `scripts/import_chronos_object.py`
  - regression tests in engine

### Step 3 — Construct Physical Floor Relief & Visual Comparison

- [x] Action:
  - In `crates/engine/src/modes/inner_chambers/world.rs`, replace the single flat plane with physical stone courses featuring raised flagstones, beveled edges, and recessed mortar courses.
  - Retain light-reactive PBR normal mapping and roughness for basalt stone response.
  - Added unit test validating physical 3D relief vertices.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`

### Step 4 — Implement Phased Magical Entrance Effect

- [x] Action:
  - Bind visual effects to real stages: pedestal charging glow and plasma pulse buildup during model execution, controlled lightning strike and smoke volume upon validated placement, followed by smoke dissipation revealing the rotating object.
  - Keep particle and light counts bounded for smooth performance on RTX 3060.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`

### Step 5 — Verify, Advance Release Identity & Commit Source

- [x] Action:
  - Advance `installer/version.json` to version `1.0.5` and `build_serial` 6.
  - Verify native window title format: `Archetypes 1.0.5 (build 6) — Council Chamber`.
  - Run full test gate: `cargo test --workspace` (all 110 tests pass).
  - Commit all tested source changes to `main` and push to `origin/main` BEFORE packaging.
- Files touched:
  - `installer/version.json`
  - source files

### Step 6 — Signed Formal Build, Installation & Buyer Verification

- [x] Action:
  - Verify signing readiness with `scripts/check_signing_ready.ps1`.
  - Run `installer/build.ps1` with Azure Trusted Signing enabled (`CN=Michael Holt`).
  - Verify `Archetypes_Setup_1.0.5.exe` in `%USERPROFILE%\Downloads` and check SHA-256 matches `release.json`.
  - Verify Authenticode signatures on installer, engine, launcher, uninstaller (`Status = Valid`).
  - Package full Sherpa-onnx runtime and Kokoro voices into formal installer payload.
  - Synchronize release build to `%LOCALAPPDATA%\Programs\Archetypes` with offline speech runtime and voices.
  - Verify Taskbar pinned shortcut target: `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk` -> `C:\Users\m\AppData\Local\Programs\Archetypes\launcher.exe`.
  - Launch via installed launcher and witness:
    - Native window title with `Archetypes 1.0.5 (build 6) — Council Chamber`
    - Full speech initialization (no missing voices, no OS error 5)
    - Vulkan discrete GPU initialization on NVIDIA GeForce RTX 3060
- Files touched:
  - `installer/version-history.json`
  - `installer/output/release.json`


