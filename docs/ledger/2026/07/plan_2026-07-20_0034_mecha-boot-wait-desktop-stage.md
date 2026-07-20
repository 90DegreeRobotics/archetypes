# Plan: mecha-boot-wait-desktop-stage — 2026-07-20 00:34

## Status
COMPLETED

## Goal
Fix the boot flash of the main menu before the black title veil, make chat waiting honest and visible, keep in-chat Auto-fit artifacts/portraits correct, and permanently stage desktop `dist` + shortcut after product changes so launches match source.

## Root cause (desktop miss)
Desktop `Archetypes.lnk` → `C:\archetypes\dist\launcher.exe`. `dist\engine.exe` was last written 2026-07-15; source edits from 2026-07-19 never ran `install_shortcut.ps1`, so the operator saw the old UI.

## Steps
### Step 1 — Plan + diagnose
- [x] Action: Confirm dist age vs source; write this plan

### Step 2 — Boot flash
- [x] Action: Keep chamber/main-menu hidden until Booting veil completes; black first frame
- Files: `crates/engine/src/chamber/boot.rs`, `interior.rs`, `mod.rs`, `camera.rs`

### Step 3 — Chat wait + images
- [x] Action: Stronger in-channel waiting indicator; verify artifact embed + portrait aspect
- Files: `crates/engine/src/modes/standard_mecha/mod.rs`

### Step 4 — Permanent launcher rule
- [x] Action: `.cursor/rules` + AGENTS.md: product changes must refresh `dist` via `install_shortcut.ps1`

### Step 5 — Stage desktop + verify
- [x] Action: `cargo test --workspace`; `scripts\install_shortcut.ps1`; STATUS update

## Test gate
```
cargo test --workspace
scripts\install_shortcut.ps1
```
