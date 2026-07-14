# Plan: Lane A Oracle Riddle — 2026-07-13 22:05

## Status
COMPLETED

## Goal
Build the first real non-standard playable mode: a reverse-prompt puzzle. A hidden 3-word prompt generates or resolves to an image. The player reconstructs the prompt. Scoring is embedding-tolerant per word. Each completed round seals to the Lane 0 ledger.

## Steps
### Step 1 — Register Mode
- [x] Action: Make `Oracle Riddle` playable in `ModeRegistration` and route it from `boot.rs`.
- Files touched: `crates/engine/src/modes/game_mode.rs`, `crates/engine/src/chamber/boot.rs`.
- Expected outcome: Oracle Riddle becomes selectable; selecting it routes to an Oracle-specific state or mode setup.

### Step 2 — Base Mode Plugin
- [x] Action: Create `oracle_riddle` module and plugin.
- Files touched: `crates/engine/src/modes/mod.rs`, `crates/engine/src/modes/oracle_riddle/mod.rs`.
- Expected outcome: Bevy app registers and runs `OracleRiddlePlugin`.

### Step 3 — The Game Loop State
- [x] Action: Add Oracle mode specific States and Systems.
- Files touched: `crates/engine/src/modes/oracle_riddle/mod.rs` and submodules.
- Expected outcome: Loop covers: choose hidden 3-word prompt -> cached/generated image -> accept player 3-word guess -> embedding-tolerant per-word scoring -> seal round to ledger -> allow next round or return to menu.

### Step 4 — Embedding Similarity & Pure Tests
- [x] Action: Compute cosine similarity between guess words and target words using `services::llm::embed`. Add pure deterministic tests for prompt selection shape, scoring partial/exact matches, cosine helper, mode registry, and ledger payload shape.
- Files touched: `crates/engine/src/modes/oracle_riddle/scoring.rs` and tests.
- Expected outcome: Honest scoring with visible failure if embeddings/Chronos are unavailable (no stubs).

### Step 5 — Verify & Seal
- [x] Action: Run tests, update STATUS.md and README.md, commit, and push.
- Files touched: `STATUS.md`, `README.md`.
- Expected outcome: Done definitions met, workspace passes tests.

## Verification
- `cargo test --workspace` passed (27 engine tests, 1 launcher test).
- Tests added for Oracle Riddle's exact/partial vector scoring, cosine math, registry availability, prompt tiers, and ledger payload shape after the corrective pass in `plan_2026-07-13_2214_lane-a-oracle-corrective-pass.md`.
- `OracleRiddle` correctly overrides `ChamberState::MainMenu` UI and enters its independent Bevy State Loop, protecting `Standard` ritual.
- A local hash-chained ledger event is appended on round completion or failure.
