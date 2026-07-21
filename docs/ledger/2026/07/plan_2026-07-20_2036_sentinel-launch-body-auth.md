# Plan: Sentinel Launch Body Auth - 2026-07-20 20:36

## Status
COMPLETED

## Goal
Harden the Archetypes launcher so its Chronos Sentinel launch append is client-signed and bound to the exact launch event body. The launcher must not rely on an implicit Chronos-side trust path when Chronos requires client signing for protected actions.

## Steps
### Step 1 - Inspect launcher and Sentinel contract
- [x] Action: Read the launcher source, launcher tests, security docs, and repo workflow law.
- Files touched: None.
- Expected outcome: Confirm the current gap and required verification gates.

### Step 2 - Bind launch append to Archetypes client authority
- [x] Action: Add a launcher-owned Sentinel keystore, Chronos client-key registration, body-bound `codex_append` authority request, and signed `auth` envelope insertion before posting the guarded launch event.
- Files touched: `crates/launcher/Cargo.toml`, `crates/launcher/src/main.rs`.
- Expected outcome: Launches fail closed unless Chronos Sentinel accepts the exact signed Archetypes launch body.

### Step 3 - Update security and status docs
- [x] Action: Update the public docs to remove stale bypass language and record the signed-body launch gate.
- Files touched: `README.md`, `STATUS.md`, `docs/security/SENTINEL_ADOPTION_STATUS.md`, `docs/security/SENTINEL_PROTECTED_ACTIONS.md`.
- Expected outcome: Docs match the runtime path and remaining blockers.

### Step 4 - Verify installed desktop surface
- [x] Action: Run launcher tests, full workspace tests, Sentinel certification, and `scripts\install_shortcut.ps1`.
- Files touched: Generated build/dist artifacts as required by repo workflow.
- Expected outcome: Runtime proof exists for source and Desktop `dist` launcher parity; any certification blockers are explicit.

### Step 5 - Commit and push
- [x] Action: Commit the verified unit on `main`, push to `origin/main`, and confirm clean parity.
- Files touched: Git metadata only.
- Expected outcome: `origin/main` is the handoff surface and working tree is clean.
