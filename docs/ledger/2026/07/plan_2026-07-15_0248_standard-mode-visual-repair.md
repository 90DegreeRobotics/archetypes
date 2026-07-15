# Plan: Standard Mode Visual Repair — 2026-07-15 02:48

## Status
INTERRUPTED

## Goal
Stop adding new modes and repair the main Standard Mode experience until the first playable path looks coherent on screen. The gate is not "tests pass"; the gate is fresh screenshots from the runnable app that show the menu/table, deliberation, speaking, verdict, and artifact-return frames are visually legible and worthy of another operator playtest.

## Problem Statement
The current product blocker is visual trust. The operator's judgment is that the main mode looks bad enough to threaten the project. That overrides mechanics expansion. Inner Chambers and Living Engine stay locked while Standard Mode is repaired.

## Interruption Note
This plan was interrupted on 2026-07-15 after the operator supplied a new Standard Mode screenshot showing that the failure is structural, not just a tuning problem. The giant runtime star/crystal, detached portrait billboards, lifted dark spheres, noisy starfield, and weak chamber/table grounding are symptoms of the same issue: Standard Mode is split across authored GLBs, generated meshes, top-level panel roots, camera billboards, and runtime name-string transforms.

Do not continue this as a small visual patch plan. The active direction is the rebuild study in `docs/ledger/2026/07/plan_2026-07-15_0304_standard-mode-rebuild-study.md`: ground/sky first, then star chamber, then table, then archetype chambers, with a frozen authored hierarchy.

## Ownership Boundary
- Primary files: `crates/engine/src/chamber/camera.rs`, `crates/engine/src/chamber/star.rs`, `crates/engine/src/chamber/spheres.rs`, `crates/engine/src/chamber/portal.rs`, `crates/engine/src/chamber/ritual.rs`.
- Asset-authoring files if needed: `scripts/author_council_chamber.py`, `scripts/prepare_table.py`, `assets/scenes/uiscene1.glb`, `assets/scenes/table.glb`.
- Do not change Oracle Riddle scoring/gameplay.
- Do not unlock Inner Chambers or Living Engine.
- Do not self-approve visuals; produce frames and state what changed.

## Steps
### Step 1 — Capture current failure frames
- [x] Action: Run the app/capture path from a fresh build and save frames for `02_table`, `03_deliberating`, `05_council_speaking`, `06_witness_verdict`, and `08_artifact_result` when services allow.
- Files touched: screenshot artifacts only.
- Expected outcome: Operator supplied the current failure frame directly: a giant flat blue crystal occludes the council, portrait panels read as loose billboards, spheres are too dark, the starfield reads as white noise, and table/chamber geometry is not carrying the composition.

### Step 2 — Diagnose the visible failures
- [ ] Action: Inspect the captured frames and classify failures: composition, scale, lighting, table readability, star/council silhouette, text placement, color/materials, and artifact presentation.
- Files touched: this plan or a follow-up notes section.
- Expected outcome: A ranked punch list replaces vague "make it better" work.

### Step 3 — Repair the first viewport/menu-table shot
- [ ] Action: Fix the main menu/table composition first: camera angle, table scale/readability, portal/menu placement, and excessive darkness/clutter.
- Files touched: likely camera/portal/table/chamber files.
- Expected outcome: The first screen reads as a deliberate table interface, not a broken dark scene.

### Step 4 — Repair the council/star composition
- [ ] Action: Fix deliberation/speaking/verdict framing so the star, vessels, portrait panels, and chamber depth read clearly.
- Files touched: likely camera/star/spheres/chamber files.
- Expected outcome: The council scene has a clear subject and visual hierarchy in every Standard Mode phase.

### Step 5 — Repair returned-artifact presentation
- [ ] Action: Make the artifact return feel integrated with the table/portal instead of a pasted image or stray overlay.
- Files touched: likely `ritual.rs`, camera/portal surfaces, and artifact staging surfaces.
- Expected outcome: The reward moment is visible, framed, and emotionally legible.

### Step 6 — Verify without theater
- [ ] Action: Run `cargo test --workspace`, rebuild/stage the desktop bundle with `scripts/install_shortcut.ps1`, and capture/playtest from the actual desktop path.
- Files touched: `dist/**` through the installer script only after source is test-green.
- Expected outcome: Desktop launch shows the same repaired frames as the repo build.

## Upstream / Downstream Risk Lens
- Upstream risk: camera, scene roots, and asset transforms affect every Standard Mode state.
- Downstream risk: launcher/desktop `dist` can stay stale unless explicitly refreshed; docs can easily overclaim visual approval.
- Feature-change rule: no new mechanic is required for this rescue unless it directly fixes a visible Standard Mode problem.
- Negative-system-effect check: any change that worsens Oracle Riddle, locks out Standard Mode, breaks ledger sealing, or obscures failure states is rejected even if it makes one frame prettier.
