# Plan: Intro Runtime Truth — 2026-07-14 16:43

## Status
COMPLETED

## Goal
Audit why recent work is not visible from the desktop launch path, remove the candle video intro, slow the black/title/subtext/main-menu transition, verify with tests, and refresh the installed launch surface if the repository scripts support it.

## Steps
### Step 1 — Runtime status report
- [x] Action: Inspect git state, recent commits, release/install scripts, desktop shortcut targets, and installed output timestamps.
- Files touched: None.
- Expected outcome: Desktop shortcut was confirmed to launch `C:\archetypes\dist\launcher.exe` with working directory `C:\archetypes\dist`; `dist` can be stale unless `scripts\install_shortcut.ps1` is rerun after source changes.

### Step 2 — Locate intro implementation
- [x] Action: Find the candle video and intro/menu code paths without editing unrelated lanes.
- Files touched: None.
- Expected outcome: `crates/engine/src/chamber/boot.rs` was the runtime intro path; it loaded `loading/blackflame/frame_*.jpg` and `blackflame.wav`.

### Step 3 — Implement intro change
- [x] Action: Replace candle-video intro behavior with black fade, title fade, subtext fade, and slow transition to main menu.
- Files touched: Source files and docs directly required by the intro behavior.
- Expected outcome: Runtime intro no longer loads or plays `blackflame`; title/subtitle enter sequentially over black and the reveal fade is longer.

### Step 4 — Verify and refresh launch surface
- [x] Action: Run focused checks plus `cargo test`, rebuild or restage via the repository script if appropriate, then report exact status.
- Files touched: Generated install/dist files only if the repo script updates them.
- Expected outcome: `cargo test --workspace` passed; `scripts\install_shortcut.ps1` rebuilt release binaries, restaged `dist`, and recreated Desktop/Start Menu shortcuts.

### Step 5 — Land completed work
- [x] Action: Commit and push only this scoped unit once tests pass.
- Files touched: Git metadata.
- Expected outcome: Verified change is on `origin/main`; unrelated dirty work stays untouched.

## Verification
- Runtime search: no `blackflame`, `BootFrame`, or loading-frame constants remain in `crates/engine/src/chamber/boot.rs`.
- `cargo test --workspace`: passed, 32 engine tests and 1 launcher test.
- `scripts\install_shortcut.ps1`: passed; Desktop shortcut target is `C:\archetypes\dist\launcher.exe`, working directory `C:\archetypes\dist`.
- Refreshed installed files: `dist\engine.exe` timestamp `2026-07-14 16:49`, `dist\launcher.exe` timestamp `2026-07-14 16:49`.
