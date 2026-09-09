# Plan: Complete signed installer path — 2026-09-09 09:09

## Status
COMPLETED

## Goal
Complete the formal Archetypes release command already established by the current development doctrine, using the production-proven ChronoSophia Azure Artifact Signing and Inno Setup sequence while preserving the rapid `scripts/install_shortcut.ps1` gameplay iteration lane.

## Steps
### Step 1 — Map the proven donor pipeline onto Archetypes
- [x] Action: Compare the current Archetypes signing, versioning, installer, asset, and staging contracts with `C:\chronos2\installer\build.ps1` and `chronos_setup.iss`.
- Files touched: This plan only.
- Expected outcome: Exact required payloads, signing gates, and output topology are known before implementation.

### Step 2 — Implement the formal signed build
- [x] Action: Add the Archetypes Inno Setup recipe and `installer/build.ps1` with Azure payload signing, compile-time installer signing, post-sign verification, release metadata, immutable archive, and hash-verified Downloads delivery.
- Files touched: `installer/build.ps1`, `installer/archetypes_setup.iss`, supporting installer text files if required.
- Expected outcome: `pwsh installer\build.ps1` is a real executable release path rather than a documented placeholder.

### Step 3 — Correct release-state truth
- [x] Action: Ensure product policy and per-build metadata distinguish signature-required posture from an actually emitted signed package.
- Files touched: `installer/version.json`, `installer/version-history.json`, relevant Windows release docs only if command behavior requires clarification.
- Expected outcome: No metadata claims a package is signed before a signed artifact exists.

### Step 4 — Verify the complete lane
- [x] Action: Run `cargo test --workspace`, signing readiness, the formal installer build, Authenticode verification, release-manifest checks, and Downloads hash parity.
- Files touched: Generated installer outputs and release metadata under designated installer output locations.
- Expected outcome: Either a genuinely signed, hash-verified Downloads installer exists or the precise external blocker is recorded without false completion.

### Step 5 — Close and publish the unit
- [x] Action: Record actual evidence here, mark the plan COMPLETED or INTERRUPTED, review the diff, commit on `main`, and push `origin main` if all required gates pass.
- Files touched: This plan and the completed implementation.
- Expected outcome: The repository remains the complete audit surface for this unit.

## Verification evidence

- `cargo test --workspace`: PASS, 100 tests, 0 failures.
- `scripts/check_signing_ready.ps1`: PASS, all six Azure Artifact Signing gates ready.
- Formal command: `pwsh -NoProfile -ExecutionPolicy Bypass -File installer\build.ps1`.
- Clean source commit used for the recorded release: `755cff0548d2`.
- Authenticode: `Valid` for `engine.exe`, `launcher.exe`, and `Archetypes_Setup.exe`.
- Signer: `CN=Michael Holt, O=Michael Holt, L=Normal, S=il, C=US`.
- Microsoft timestamp countersignatures are present.
- Downloads artifact: `C:\Users\m\Downloads\Archetypes_Setup_1.0.0.exe`.
- Size: `160645256` bytes.
- SHA-256: `23a632e82c0d7bc0fe1bb244aef40062e6c4dd127f39a5d48cfa0acc6ee80395`.
- Compiler output, archive, repository shadow, and Downloads copy were hash-matched by the build script.
- Inno compilation completed without the elevated per-user-area warning after player-data creation was correctly left to the runtime.
