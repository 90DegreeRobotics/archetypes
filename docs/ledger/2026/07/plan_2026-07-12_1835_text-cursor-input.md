# Plan: Input repair and locked chamber choreography — 2026-07-12 18:35

## Status
COMPLETED

## Goal
Replace append-only ritual typing with a minimal real text editor; lock the chamber's canonical visual sequence; and establish a real loading/title/main-menu boundary before Standard Mode begins.

## Steps
### Step 1 — Repair editing semantics
- [x] Action: Track a character cursor and implement Space, Left, Right, Home, End, Backspace, Delete, and character insertion at the caret.
- Files touched: `crates/engine/src/chamber/ritual.rs` and tests.
- Expected outcome: The offering/name field behaves like a keyboard-editable text field.

### Step 2 — Render and reset the caret correctly
- [x] Action: Render the caret at its cursor position and normalize it when drafts are programmatically set or cleared.
- Files touched: Ritual UI/capture logic.
- Expected outcome: Visible caret and internal editing position never diverge.

### Step 3 — Preserve the interim installation gate
- [x] Action: The prior voice-ready Desktop bundle was built and staged before this expanded startup scope; final installation is repeated in Step 7 without launching the window.
- Files touched: Status/plan plus installed outputs.
- Expected outcome: Operator can test corrected input immediately from Desktop.

### Step 4 — Lock the chamber sequence
- [x] Action: Keep the star out of the opening portal shot, reveal it only after question submission, keep archetype panels upright, return the camera to the portal for artifact results, and place the returned image over that portal.
- Files touched: Camera, panel, and artifact presentation systems plus focused tests.
- Expected outcome: The narrative camera sequence cannot drift back to an all-at-once establishing view.

### Step 5 — Give the portal table structural support
- [x] Action: Extend the Blender preparation script with a central pedestal and radial legs, re-export the existing table copy, and verify it in the live engine.
- Files touched: `scripts/prepare_table.py`, `assets/scenes/table.glb`, proof output.
- Expected outcome: The table reads as a supported physical object rather than a floating floor decal.

### Step 7 — Install and publish
- [x] Action: Run `cargo test`, rebuild release, restage the Desktop bundle, verify the shortcut and services without opening the game, update docs, commit, push, and leave the tree clean.
- Files touched: Status/plan plus installed outputs.
- Expected outcome: Operator can test corrected input and locked choreography immediately from Desktop.

### Step 6 — Build the loading veil and main menu
- [x] Action: Use `C:\Users\m\Videos\blackflame.mp4` as the authored loading motion, fade in `ARCHETYPES` and `A GAME BY MICHAEL HOLT` over black, hold the veil until required assets are ready, then fade to a settled portal-table menu with `START STANDARD MODE`.
- Files touched: Boot/menu state, generated runtime video frames, UI, installer staging, and tests.
- Expected outcome: No asset-loading lag is exposed and Standard Mode never starts automatically.

### Step 8 — Honor operator-only visual launch
- [x] Action: Verify compilation, asset completeness, shortcut target, and service readiness without opening the game; tell the operator when the installed build is ready for their visual launch.
- Files touched: Status/plan and non-visual readiness proof.
- Expected outcome: Codex never launches the game window for visual approval.

## Outcome

- Editor tests cover mid-string Space insertion and cursor-relative Backspace/Delete.
- The table now has a pedestal, four splayed legs, and feet; state gates enforce portal-only opening, clean star reveal, and portal artifact return.
- The authored MP4 became 162 JPEG frames at 8 fps and 960×540 (8.4 MB) plus a 48 kHz stereo WAV, preserving its motion and audio without a runtime codec sidecar.
- Boot waits for the film frames, seven vessels, and stargate, fades into `MainMenu`, and never auto-enters Standard Mode.
- Final installation is verified without opening the game window. The operator owns the remaining visual launch gate.
