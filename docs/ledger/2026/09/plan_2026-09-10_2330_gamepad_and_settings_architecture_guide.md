# Plan: Gamepad Wiring and Settings System Architecture Guide — 2026-09-10 23:30

## Status
COMPLETED

## Goal
Design and document the full architectural specification and engineering guide for wiring USB gamepads (specifically the Xbox One S Controller) into Archetypes. The specification will provide full prose instructions for future agents and human contributors to implement:
1. Low-level gamepad driver and XInput/GilRs connection handling in Bevy 0.18.
2. Complete mapping of all current locomotion, flight, camera, and manifestation mechanics to gamepad controls.
3. An extensible, data-driven action binding architecture for remapping controls and binding future capabilities.
4. An advanced in-game Settings UI system featuring hierarchical volume mixing (Master, Music, SFX, Voice) and look sensitivity / motion curves.
5. An "Apply on Close" persistence pipeline adhering strictly to the Windows Metabolism contract (`%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json`).

## Steps
### Step 1 — Architectural Research & Engine Interface Analysis
- [x] Action: Audit Bevy 0.18 entity-based Gamepad API (`Query<&Gamepad>`, `GamepadButton`, `GamepadAxis`), camera look integration in `camera.rs`, interaction handling in `manifestation.rs`, and settings persistence under Windows Metabolism.
- Files touched: None (read-only audit).
- Expected outcome: Exact technical primitives, types, and mathematical formulas identified.

### Step 2 — Author Canonical Gamepad & Settings Guide (Full Prose)
- [x] Action: Write `docs/architecture/GAMEPAD_AND_SETTINGS_SPECIFICATION.md` and mirror/link to `GAMEPAD_AND_SETTINGS_GUIDE.md` in repository root. Cover:
  1. USB Host & Driver layer (XInput, GilRs, Bevy Gamepad entity model).
  2. Complete Xbox One S button & axis layout and default control mappings.
  3. Extensible `GameAction` and `ActionBindings` architecture for seamless future feature mapping.
  4. Advanced Settings Menu architecture with Bevy UI / Sliders.
  5. Audio volume bus hierarchy (Master vs Music vs SFX vs Voice).
  6. Look sensitivity, stick deadzones, and acceleration response curves.
  7. "Apply on Close" serialization and atomic disk persistence.
- Files touched:
  - `docs/architecture/GAMEPAD_AND_SETTINGS_SPECIFICATION.md`
  - `GAMEPAD_AND_SETTINGS_GUIDE.md`
- Expected outcome: Comprehensive, fully authored guide with complete prose and runnable Rust snippets.

### Step 3 — Verification & Push to origin/main
- [x] Action: Check file links, verify formatting, update plan to COMPLETED, commit to `main`, and push to `origin/main`.
- Files touched:
  - `docs/architecture/GAMEPAD_AND_SETTINGS_SPECIFICATION.md`
  - `GAMEPAD_AND_SETTINGS_GUIDE.md`
  - `docs/ledger/2026/09/plan_2026-09-10_2330_gamepad_and_settings_architecture_guide.md`
- Expected outcome: Clean commit on `origin/main`.
