# Plan: sentinel-launch-gate - 2026-07-20 15:42

## Status
COMPLETED

## Goal
Make Archetypes launch itself a Sentinel-protected action. The launcher must obtain real ChronosSophia Sentinel authorization through the guarded Codex append route before starting `engine.exe`, so the game cannot begin player-facing operation ahead of Sentinel governance.

## Context
- Existing tree is dirty before this unit; this plan scopes only the Sentinel launch gate and truth docs.
- Relevant files read: `AGENTS.md`, `STATUS.md`, `crates/launcher/src/main.rs`, `crates/launcher/Cargo.toml`, `crates/engine/src/services/chronos.rs`, `crates/engine/src/services/readiness.rs`, and Chronos `crates/chronos_director/src/main.rs` status/Codex append surfaces.
- Existing launcher already checks Ollama, TTS, Chronos Director, and Comfy readiness; that is dependency readiness, not Sentinel authorization.
- The `ARCHETYPES_ALLOW_WITHOUT_CHRONOS` debug bypass conflicts with "Let there be no gate before the Sentinel" if it can launch the game without Sentinel.

## Steps

### Step 1 - Add Sentinel launch authorization
- [x] Action: Reorder launcher startup so Sentinel authorization runs before game engine start and before ordinary dependency gating.
- Files touched: `crates/launcher/src/main.rs`.
- Expected outcome: If Chronos Director is missing, degraded, not in Sentinel enforce mode, or the guarded Codex append rejects the launch event, the launcher exits before `engine.exe`.

### Step 2 - Preserve fail-visible operator truth
- [x] Action: Make the legacy debug bypass visibly ignored for Sentinel launch safety while preserving clear failure text and AppData logs.
- Files touched: `crates/launcher/src/main.rs`.
- Expected outcome: No environment variable can skip the launch Sentinel gate; failures tell the operator exactly why launch was refused.

### Step 3 - Add deterministic tests
- [x] Action: Unit-test pure parsing/body helpers for the Sentinel launch gate.
- Files touched: `crates/launcher/src/main.rs`.
- Expected outcome: Tests prove enforce-mode status is required, shadow/degraded status is rejected, and the launch event body names the carved law and game launch action.

### Step 4 - Update docs truth
- [x] Action: Update current status text to reflect the launch gate and the retired bypass behavior.
- Files touched: `STATUS.md`.
- Expected outcome: Docs no longer imply Archetypes can launch without Chronos/Sentinel in normal operation.

### Step 5 - Verify and stage
- [x] Action: Run focused launcher tests, then the mandatory `cargo test --workspace`; if that passes, run `pwsh -File scripts\install_shortcut.ps1` as required for player-facing launcher changes.
- Files touched: build/dist outputs may already be dirty from prior work; do not commit unrelated preexisting changes.
- Expected outcome: If verification passes, commit and push only this Sentinel unit plus this plan. If verification fails, mark this plan INTERRUPTED and report the exact blocker without committing.

## Verification
- `cargo test -p launcher -- --nocapture` passed: 7 launcher tests.
- `cargo test --workspace` passed: 55 engine tests + 7 launcher tests.
- `pwsh -File scripts\install_shortcut.ps1` passed; release `dist\launcher.exe` and `dist\engine.exe` are newer than `crates\launcher\src\main.rs`.
