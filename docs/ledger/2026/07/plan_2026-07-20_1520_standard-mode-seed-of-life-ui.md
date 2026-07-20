# Plan: standard-mode-seed-of-life-ui — 2026-07-20 15:20

## Status
COMPLETED

## Goal
Make Standard Mode match the operator Seed-of-Life canon: pure black, spaced gold ARCHETYPES title with glinted hairline, seven thin gold rings (Sentinel center). Same black+gold chrome in chat/switching. Stop using Mecha uxbacklayer.

## Canon reference
- `assets/standard_mecha/canon/seed-of-life-selector.png`
- `docs/ledger/2026/07/canon_2026-07-20_standard-mode-seed-of-life.png`
- Font: `assets/fonts/Cinzel-Regular.ttf` (SIL OFL; see `Cinzel-OFL.txt`)

## Steps
### Step 1 — Bundle serif font + rewrite selector/chat chrome
- [x] Done in `crates/engine/src/modes/standard_mecha/mod.rs`

### Step 2 — Tests + install_shortcut.ps1 + STATUS
- [x] `cargo test --workspace` 55+3; Desktop restaged
