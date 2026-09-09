# Plan: Complete signed installer path — 2026-09-09 09:09

## Status
IN-PROGRESS

## Goal
Complete the formal Archetypes release command already established by the current development doctrine, using the production-proven ChronoSophia Azure Artifact Signing and Inno Setup sequence while preserving the rapid `scripts/install_shortcut.ps1` gameplay iteration lane.

## Steps
### Step 1 — Map the proven donor pipeline onto Archetypes
- [x] Action: Compare the current Archetypes signing, versioning, installer, asset, and staging contracts with `C:\chronos2\installer\build.ps1` and `chronos_setup.iss`.
- Files touched: This plan only.
- Expected outcome: Exact required payloads, signing gates, and output topology are known before implementation.

### Step 2 — Implement the formal signed build
- [ ] Action: Add the Archetypes Inno Setup recipe and `installer/build.ps1` with Azure payload signing, compile-time installer signing, post-sign verification, release metadata, immutable archive, and hash-verified Downloads delivery.
- Files touched: `installer/build.ps1`, `installer/archetypes_setup.iss`, supporting installer text files if required.
- Expected outcome: `pwsh installer\build.ps1` is a real executable release path rather than a documented placeholder.

### Step 3 — Correct release-state truth
- [ ] Action: Ensure product policy and per-build metadata distinguish signature-required posture from an actually emitted signed package.
- Files touched: `installer/version.json`, `installer/version-history.json`, relevant Windows release docs only if command behavior requires clarification.
- Expected outcome: No metadata claims a package is signed before a signed artifact exists.

### Step 4 — Verify the complete lane
- [ ] Action: Run `cargo test --workspace`, signing readiness, the formal installer build, Authenticode verification, release-manifest checks, and Downloads hash parity.
- Files touched: Generated installer outputs and release metadata under designated installer output locations.
- Expected outcome: Either a genuinely signed, hash-verified Downloads installer exists or the precise external blocker is recorded without false completion.

### Step 5 — Close and publish the unit
- [ ] Action: Record actual evidence here, mark the plan COMPLETED or INTERRUPTED, review the diff, commit on `main`, and push `origin main` if all required gates pass.
- Files touched: This plan and the completed implementation.
- Expected outcome: The repository remains the complete audit surface for this unit.
