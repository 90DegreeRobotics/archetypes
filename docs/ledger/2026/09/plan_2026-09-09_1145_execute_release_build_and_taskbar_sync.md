# Plan: Execute Signed Release Build and Pinned Taskbar Sync — 2026-09-09 11:45

## Status
COMPLETED

## Goal
Execute the standard delivery protocol to produce both:
1. Fast Inner Loop sync (`scripts\install_shortcut.ps1`): compiles release binaries and synchronizes them directly to `%LOCALAPPDATA%\Programs\Archetypes` and the operator's pinned Windows Taskbar shortcut (`Archetypes.lnk`).
2. Formal Signed Release (`installer\build.ps1`): increments `build_serial` to 2 in `installer/version.json`, records `build_serial: 2` in `installer/version-history.json`, signs payloads with Azure Artifact Signing (`CN=Michael Holt`), compiles the Inno Setup installer with compile-time authenticode signing, and delivers `Archetypes_Setup_1.0.1.exe` directly to `%USERPROFILE%\Downloads`.

## Steps

### Step 1 — Update Version Documents for Formal Release Serial 2
- [x] Action:
  - In `installer/version.json`, set `product_version` to `"1.0.1"` and `build_serial` to `2`.
  - In `installer/version-history.json`, append a pending record for `build_serial: 2` (version `"1.0.1"`).
- Files touched:
  - `installer/version.json`
  - `installer/version-history.json`
- Expected outcome: Version manifest ready for `installer/build.ps1`.

### Step 2 — Fast Inner Loop & Taskbar Sync
- [x] Action: Run `pwsh -File scripts\install_shortcut.ps1` to compile release binaries and synchronize them to `%LOCALAPPDATA%\Programs\Archetypes`, updating the pinned Windows Taskbar shortcut.
- Expected outcome: The operator's pinned Taskbar icon immediately runs the updated build.

### Step 3 — Formal Azure-Signed Release Build
- [x] Action: Run `pwsh installer\build.ps1` to sign binaries with Azure Artifact Signing (`CN=Michael Holt`), compile Inno Setup installer, verify Authenticode signature, and deliver `Archetypes_Setup_1.0.1.exe` to `%USERPROFILE%\Downloads`.
- Expected outcome: A verified, signed setup executable in `%USERPROFILE%\Downloads`.

### Step 4 — Verification, Ledger Update, Commit & Push
- [x] Action: Verify signatures and files, mark plan COMPLETED, commit changes, and push to `origin/main` per Rule 4.
- Expected outcome: Clean repository state on `main` pushed to `origin`.

