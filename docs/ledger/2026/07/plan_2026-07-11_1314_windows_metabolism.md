# Plan: Windows Metabolism — 2026-07-11 13:14

## Status
IN-PROGRESS

## Goal
Establish a robust dual-binary workspace and define the Windows native integration layer (AppData paths, Launcher, Dependency bootstrap) to match the strict NeuroCognica SOPs from the Mirrorborn project.

## Steps
### Step 1 — Clean Ledger Creation
- [x] Action: Write this document.
- Files touched: `docs/ledger/2026/07/plan_2026-07-11_1314_windows_metabolism.md`
- Expected outcome: A local record of this architectural pivot.

### Step 2 — Workspace Restructuring
- [ ] Action: Convert the root Cargo project into a workspace with `engine` and `launcher` crates.
- Files touched: `Cargo.toml`, `crates/engine/Cargo.toml`, `crates/launcher/Cargo.toml`, `crates/*/src/main.rs`
- Expected outcome: Code is modularized; Bevy belongs strictly to the engine crate.

### Step 3 — Windows Metabolism Documentation
- [ ] Action: Define the installation and AppData paths.
- Files touched: `docs/architecture/WINDOWS_METABOLISM.md`
- Expected outcome: Clear contract for where mutable state lives.

### Step 4 — Dependency Bootstrapping
- [ ] Action: Create manifest-driven Winget scripts.
- Files touched: `scripts/dependencies.json`, `scripts/setup_windows.ps1`
- Expected outcome: Idempotent environment setup ready for Ollama injection.
