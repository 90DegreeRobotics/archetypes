# Plan: Blank Slate Shell — 2026-07-15 05:25

## Status
COMPLETED

## Goal
Reset the desktop-facing Archetypes launch experience so the installed launcher shows only the black intro sequence and a new blank-slate main menu. The old chamber/table/menu/Mecha visual surfaces remain preserved in source as standby/reference code, but they must not appear from the normal desktop launch path before the new GLB scene is supplied.

## Steps
### Step 1 — Rewire startup visibility
- [x] Action: Stop the authoritative chamber/table GLBs from spawning during the default launcher boot path without deleting their code or assets.
- Files touched: `crates/engine/src/chamber/mod.rs`.
- Expected outcome: Normal launch no longer presents the old table, council chamber, portal, or Mecha visual surface.
- Result: Default launch does not add the legacy chamber/table/portal/sky/star visual stack. The legacy scene remains preserved behind the explicit `ARCHETYPES_LEGACY_CHAMBER` opt-in.

### Step 2 — Replace the main menu shell
- [x] Action: Replace the portal/table menu with a clean black-slate Bevy UI main menu that keeps modes visibly in standby/locked states.
- Files touched: `crates/engine/src/chamber/boot.rs`.
- Expected outcome: Boot fades from title/subtitle into a real main menu that does not route into old Standard Mode or Oracle visuals while the new GLB is pending.
- Result: The menu is a standalone black-shell UI. Standard Mode and Oracle Riddle show `STANDBY`; Inner Chambers and Living Engine show `LOCKED`; no menu action dispatches into the old Standard/Oracle/Inner/Living visuals.

### Step 3 — Add witness capture for the new launch path
- [x] Action: Add or adapt a deterministic runtime capture path that screenshots intro and main menu from the actual engine executable.
- Files touched: `crates/engine/src/chamber/boot.rs`, docs/status if needed.
- Expected outcome: Visual proof can be generated from the desktop-facing runtime without relying on old Mecha/chamber capture paths.
- Result: `ARCHETYPES_BLANK_CAPTURE=1` captures title, creator credit, and the blank main menu using Bevy's screenshot pipeline.

### Step 4 — Verify, package, and document
- [x] Action: Run formatting, Rust verification, asset/path checks needed for the changed surface, refresh desktop distribution, witness the installed launcher behavior, then update status and this plan.
- Files touched: `STATUS.md`, this plan, generated proof artifacts if required.
- Expected outcome: The tree records the reset truthfully, the launcher shortcut points at the refreshed build, and screenshots prove no old table/chamber/menu appears.
- Result: `cargo test --workspace` passed, `cargo build -p engine` passed, `scripts\install_shortcut.ps1` refreshed `dist` and the Desktop/Start Menu shortcuts, and `dist\launcher.exe` captured the final blank-shell frames in `artifacts/visual-proof/blank-slate-shell-2026-07-15_0525/`.
