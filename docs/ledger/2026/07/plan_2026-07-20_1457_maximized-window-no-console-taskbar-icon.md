# Plan: maximized-window-no-console-taskbar-icon — 2026-07-20 14:57

## Status
COMPLETED

## Goal
Desktop launch opens the game window maximized, hides the launcher/engine console, and shows a proper taskbar/shortcut icon for Archetypes.

## Steps
### Step 1 — Inspect launcher/engine window + Windows subsystem
- [x] Action: Confirmed Bevy 0.18 has no `WindowMode::Maximized`; use `Window::set_maximized(true)`. Release builds need `windows_subsystem = "windows"`.
- Files touched: (read-only)
- Expected outcome: Clear approach for maximize + hide console + icon embed.

### Step 2 — Maximize primary window; windows_subsystem; set window icon
- [x] Action: Engine Startup maximizes primary window; both crates use `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`; winres embeds `assets/icons/archetypes.ico`; launcher spawns engine with `CREATE_NO_WINDOW`; install script refreshes Desktop `.ico`.
- Files touched: `crates/engine/src/main.rs`, `crates/launcher/src/main.rs`, `crates/engine/build.rs`, `crates/launcher/build.rs`, `crates/*/Cargo.toml`, `scripts/install_shortcut.ps1`, `assets/icons/archetypes.ico`
- Expected outcome: Release Desktop launch shows only maximized game window + taskbar/shortcut icon; no console flash.

### Step 3 — cargo test + install_shortcut.ps1
- [x] Action: `cargo test --workspace` (54 engine + 3 launcher). `scripts\install_shortcut.ps1` release-staged dist + Desktop/Start Menu shortcuts. PE check: both exes are `WINDOWS_GUI`.
- Files touched: `STATUS.md`, this plan
- Expected outcome: Desktop surface matches source; operator can confirm maximize / no console / icon on next launch.
