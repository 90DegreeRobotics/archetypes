# Plan: Portal menu and star position — 2026-07-13 12:45

## Status
COMPLETED

## Goal
Correct the visually failed star lift, preserve the successful full-table portal effect while slowing its motion, tighten and steepen the table-first camera composition, and move Standard Mode menu choices into the portal area so the main menu is primarily the living table rather than a detached overlay.

## Steps
### Step 1 — Prove and correct star placement
- [x] Action: Inspect the runtime star spawn and authored council transforms, then bind the solid star to an explicit shared assembly position rather than relying on an indirectly observed centroid.
- Files touched: Star/chamber runtime and tests as required.
- Expected outcome: The star is visibly raised with the council constellation.

### Step 2 — Reframe and slow the table portal
- [x] Action: Move the table camera closer with a steeper angle and reduce portal rotation speed without changing the approved portal footprint.
- Files touched: Camera and portal systems.
- Expected outcome: The table dominates the opening frame and the plasma movement feels deliberate.

### Step 3 — Integrate the main menu into the portal
- [x] Action: Reposition and restyle game-mode controls over the portal motion while retaining keyboard and mouse activation.
- Files touched: Boot/main-menu UI.
- Expected outcome: The main-menu shot reads as a table interface with options emerging from its center.

### Step 4 — Verify, install, publish
- [x] Action: Run tests, rebuild and stage the desktop runtime without launching, update verification, commit, push, and leave the tree clean.
- Files touched: Plan/proof documentation and installed staging outputs.
- Expected outcome: Operator receives a clean launch signal for visual verification.

## Verification
- The star no longer derives its placement from the asynchronous GLTF transform observed on its spawn frame; `COUNCIL_CENTER = (0, 2, 0)` is shared by star and deliberation camera.
- Table camera moved from `(0, -0.8, 8.0)` to `(0, 0.4, 6.4)` and retains a table-centered target at `(0, -2.5, 0)`.
- Portal rotation reduced from `0.5` to `0.16` radians per second; the approved full-table portal mesh is unchanged.
- Main-menu root is transparent and centered over the portal, with Standard Mode retaining mouse and Enter activation.
- `cargo test`: 18 passed, 0 failed across engine and launcher.
- Visual approval remains with the operator after desktop launch.
