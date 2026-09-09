# Plan: Manifestation Pedestal Visual States (Idle Symbol, Phasing Hourglass, Red X Failure, Summoned Object) — 2026-09-09 14:05

## Status
COMPLETED

## Goal
Implement accurate, unambiguous real-time feedback for the Manifestation Pedestal:
1. Initial state: Replace any pre-existing artifact with a clean floating ethereal symbol above the empty cushion.
2. Waiting state: A floating 3D hourglass that slowly rotates and phases in and out (pulsing opacity and light) with a live timer in the HUD.
3. Failed state: A floating, luminous Red 'X' above the pedestal with an explicit error description so the user instantly knows what went wrong.
4. Succeeded state: The waiting hourglass disappears right before the newly summoned 3D object appears rotating atop the pedestal cushion.
5. Fix executable/script path resolution so the installed product (`%LOCALAPPDATA%\Programs\Archetypes`) reliably executes the manifestation pipeline.
6. Support "buddha" and meditative statue iconography in `scripts/manifest_artifact.py`.
7. Verify via `cargo test --workspace` and refresh the pinned Taskbar launcher via `scripts/install_shortcut.ps1`.

## Steps

### Step 1 — Add Iconography & Robust Paths to Manifestation Pipeline
- Action:
  - In `scripts/manifest_artifact.py`, add keyword recognition for "buddha", "statue", "deity", "lotus", generating an ornate meditative figure on a lotus base with halo aureole.
  - Stage `scripts/manifest_artifact.py` in `scripts/install_shortcut.ps1` into `dist/scripts/` and `%LOCALAPPDATA%\Programs\Archetypes\scripts\`.
  - In `crates/engine/src/modes/inner_chambers/manifestation.rs`, resolve paths relative to `current_exe()`, `current_dir()`, and fallback `C:\archetypes`.
- Files touched:
  - `scripts/manifest_artifact.py`
  - `scripts/install_shortcut.ps1`
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`

### Step 2 — Implement Four Distinct 3D Pedestal States & Live HUD
- Action:
  - **Idle**: Floating hovering celestial glyph above empty cushion (no pre-loaded placeholder mesh).
  - **Waiting**: Floating 3D Hourglass (opposed cones, golden frame, glowing sands) rotating and phasing in/out with live timer.
  - **Failed**: Floating luminous Red 'X' with red warning beacon and error message.
  - **Succeeded**: Grand flash, despawning hourglass, spawning the new summoned object with turntable rotation.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`

### Step 3 — Verification & Fast Dev Taskbar Refresh
- Action: Run `cargo test --workspace` (Rule 5 Rust Gate).
- Run `pwsh -File scripts\install_shortcut.ps1` to update `dist\`, sync `%LOCALAPPDATA%\Programs\Archetypes`, and refresh the pinned Taskbar shortcut.
- Expected outcome: All tests pass and launcher updated.

### Step 4 — Ledger Update, Commit & Push
- Action: Mark plan COMPLETED, commit to `main`, and push immediately to `origin/main` (Rules 1-4).
