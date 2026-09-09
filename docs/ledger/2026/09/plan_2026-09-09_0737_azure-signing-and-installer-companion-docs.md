# Plan: Azure Signing Pipeline & Installer Wizard Companion Docs — 2026-09-09 07:37

## Status
COMPLETED

## Goal
Document the complete 6-step Azure Trusted Signing pipeline (leveraged from `C:\chronos2`), the versioned executable workflow, and the customized Inno Setup installer/uninstaller wizard architecture for the Archetypes game. Establish clear guidelines in `AGENTS.md` so every future session agent understands that the operator is a verified Microsoft author, respects the two distinct development lanes (fast dev staging to pinned taskbar vs formal signed versioned installer release), and never leaves work unfinished or expects the operator to use the CLI.

## Steps
### Step 1 — Create Azure Trusted Signing Pipeline Documentation
- Action: Author `docs/windows/AZURE_TRUSTED_SIGNING.md` detailing the 6-step Azure Trusted Signing pipeline, authentication (`az login` / SP), `signtool.exe` resolution, `Azure.CodeSigning.Dlib.dll` integration, timestamp authority (`http://timestamp.acs.microsoft.com/`), and the compile-time ISCC signing flag (`-sarchetypes="<cmd>"`) required to prevent Smart App Control Error 4551.
- Files touched: `docs/windows/AZURE_TRUSTED_SIGNING.md`
- Expected outcome: A comprehensive, production-grade guide matching the active signing infrastructure in `C:\chronos2`.

### Step 2 — Create Versioning and Release Specification Documentation
- Action: Author `docs/windows/VERSIONING_AND_RELEASES.md` describing how versioned executables are managed (`version.json`, `version-history.json`, build serials, release metadata `release.json`, and automatic delivery to the operator's `Downloads` folder).
- Files touched: `docs/windows/VERSIONING_AND_RELEASES.md`
- Expected outcome: Clear rules on semantic versioning, build serial monotonic progression, payload integrity hashing, and delivery contracts.

### Step 3 — Create Installer / Uninstaller Wizard Specification Documentation
- Action: Author `docs/windows/INSTALLER_WIZARD_SPEC.md` specifying how the Inno Setup wizard from `chronos2` is borrowed and dressed specifically for the Archetypes game (ceremonial castle rotunda branding, dark fantasy color palette, components, paths, and graceful uninstall contract).
- Files touched: `docs/windows/INSTALLER_WIZARD_SPEC.md`
- Expected outcome: Complete design and architectural specification for the game's installer and uninstaller.

### Step 4 — Update Agent Operating Procedures in `AGENTS.md`
- Action: Update `AGENTS.md` to formally document the verified author standing, the 6-step Azure signing pipeline, the reconciliation between the Fast Inner Loop (`scripts/install_shortcut.ps1` -> Pinned Taskbar) and the Formal Signed Release Lane (`installer/build.ps1` -> Signed Installer in `Downloads`), and agent obligations.
- Files touched: `AGENTS.md`
- Expected outcome: Canonical rules for future session agents.

### Step 5 — Verify, Commit, and Push
- Action: Run verification checks (docs diff, links, markdown rendering, git status) and push cleanly to `origin/main`.
- Files touched: `docs/ledger/2026/09/plan_2026-09-09_0737_azure-signing-and-installer-companion-docs.md`
- Expected outcome: Clean repository state on `main` pushed to `origin`.
