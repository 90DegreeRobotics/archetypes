# Plan: Taskbar Pinned Shortcut Pipeline Integration — 2026-09-09 07:30

## Status
COMPLETED

## Goal
Fix the deployment pipeline gap where player-facing builds only staged to `C:\archetypes\dist\` and refreshed Desktop shortcuts, leaving the operator's pinned Taskbar shortcut (`%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk`) executing a stale binary from `%LOCALAPPDATA%\Programs\Archetypes\` (dated August 25, 2026).

We update the pipeline scripts so that every deployment step automatically synchronizes the new build to `%LOCALAPPDATA%\Programs\Archetypes\`, updates the pinned Taskbar shortcut target, and refreshes the Windows icon cache.

## Steps

### Step 1 — Audit & Root Cause Pinned Shortcut Target
- [x] Action: Inspected the taskbar shortcut `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Archetypes.lnk`.
  - Discovered it points to `%LOCALAPPDATA%\Programs\Archetypes\launcher.exe`, which contained an un-updated August 25 build.
- Files touched:
  - `scripts/inspect_shortcut.ps1`
- Expected outcome: Identified why clicking the pinned Taskbar icon launched the old version.

### Step 2 — Update Staging Pipeline to Sync to Programs and Taskbar
- [x] Action:
  1. Updated `scripts/install_shortcut.ps1` to automatically invoke `scripts/install_product.ps1` whenever `%LOCALAPPDATA%\Programs\Archetypes` exists.
  2. Added Taskbar pinned shortcut detection and target refresh in `scripts/install_shortcut.ps1`.
  3. Executed `scripts/install_product.ps1` and confirmed `C:\Users\m\AppData\Local\Programs\Archetypes\engine.exe` is updated to the latest build (`9/9/2026 7:21:55 AM`).
- Files touched:
  - `scripts/install_shortcut.ps1`
  - `scripts/inspect_shortcut.ps1`
- Expected outcome: The pinned Taskbar shortcut launches the exact current engine build.

### Step 3 — Verification Gate, Documentation & Push
- [x] Action:
  1. Ran `cargo test --workspace` to ensure all tests pass.
  2. Verified timestamp of `C:\Users\m\AppData\Local\Programs\Archetypes\engine.exe`.
  3. Committed and pushed to `origin/main`.
- Files touched:
  - `scripts/install_shortcut.ps1`
  - `scripts/inspect_shortcut.ps1`
  - `docs/ledger/2026/09/plan_2026-09-09_0730_taskbar_pinned_pipeline_integration.md`
- Expected outcome: The pipeline guarantees any future build reaches the Taskbar icon.
