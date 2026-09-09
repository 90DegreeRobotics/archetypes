# Plan: Jester research verification and naming — 2026-09-09 05:47

## Status
COMPLETED

## Goal
Verify the Jester research dossier against the cited local sources and current runtimes, correct false or stale claims, and establish **Jester** as the canonical current name while preserving historical aliases only as provenance. Align the current game plan and player-facing Rust labels with that decision without changing gameplay behavior.

## Steps

### Step 1 — Verify the cited evidence
- [x] Action: Check every listed path, compare duplicated source and audio hashes, inspect the cited canon/configuration files, and distinguish source presence from current runtime availability.
- Files touched: None.
- Expected outcome: Every dossier claim is classified as verified source, historical evidence, current runtime fact, or correction required.

### Step 2 — Correct and rank the research dossier
- [x] Action: Add an evidence-status section, repair incorrect paths/runtime claims, distinguish canonical sources from historical deployments, and remove Glint from current naming.
- Files touched: `docs/architecture/JESTER_PROFILE_REPORT.md`.
- Expected outcome: The dossier becomes an auditable research report rather than an undifferentiated claim of current operational truth.

### Step 3 — Normalize current product terminology
- [x] Action: Replace Glint with Jester in the current integration plan, clarify Witness/Jester authority, and update player-facing Rust theme/chamber names from Mulligan Engine to Jester while retaining Mulligan terminology only for internal historical concepts where useful.
- Files touched: `plan_2026-09-08_archetypes_integration.md`, `crates/engine/src/theme/constants.rs`, `crates/engine/src/modes/inner_chambers/catalog.rs`.
- Expected outcome: Current design and runtime surfaces consistently call the archetype Jester.

### Step 4 — Verify the changed surfaces
- [x] Action: Review diffs and path claims, run `cargo test --workspace`, then restage the Desktop product with `pwsh -File scripts\install_shortcut.ps1` because player-facing Rust labels changed.
- Files touched: `dist/` through the canonical staging script.
- Expected outcome: Documentation is internally consistent, the workspace test gate passes, and the installed Desktop surface contains the renamed runtime labels.

### Step 5 — Record and publish the completed unit
- [x] Action: Mark this plan completed with actual verification evidence, explicitly stage only the scoped files plus the previously untracked plans they govern, commit on `main`, push `origin main`, and confirm clean-tree/remote parity.
- Files touched: This plan document and scoped worktree documents.
- Expected outcome: The verified Jester research and naming decision are preserved on `origin/main` with no work left untracked.

## Verification Record

- Jester dossier path inventory: 24 of 25 originally cited paths present; the one missing path was corrected to `assets/aura/icons/jester-icon.png`.
- Duplicate canon SHA-256: `C088F7F6AEBD82F8D3C10105583BE0D8F7577BEAA17CFECC7F4607FCB7723178`.
- Duplicate Jester profile SHA-256: `089830E92ED2816BEAF8FB076E26356FF2985C840417E1D4FFBB5D1CED67F2A7`.
- Three-copy Jester WAV SHA-256: `F8768D00B1372A678E4AD4A18096D018DFAE2C80F33F8A41437374ECF1D8A45E`.
- Live runtime audit: Ollama listening on `127.0.0.1:11434`; `mythomax` absent; port `8000` serving ComfyUI rather than Mecha FastAPI.
- `cargo test --workspace`: PASS — engine 79, launcher 16, Windows identity 5; 100 total, 0 failed.
- `pwsh -File scripts/install_shortcut.ps1`: PASS — release build, dependency readiness, distribution stage, Desktop shortcut, and Start Menu shortcut completed.
- Staged binary parity: `target/release/engine.exe` and `dist/engine.exe` both SHA-256 `898FD0A83E32E932A5803980FD09BB8D1ACFCB02DB5F948C339F8A233F15C0EC`; launcher copies both SHA-256 `D452579EE0E47AC9DC3EC56FD18CC83C93FEA6B56FC3E25FD038C05E64322A0E`.
- Shortcut targets: Desktop and Start Menu both resolve to `C:\archetypes\dist\launcher.exe` with working directory `C:\archetypes\dist`.
- Naming search: no player-facing `Glint` or `Mulligan Engine` matches remain in current Rust/product-plan surfaces; `STATUS.md` contains only the ordinary visual noun “hairline+glint.”
