# CODEX LANE 0 — The Spine

**Owner: Claude. Status: READY TO START (unblocked).**
**Cold-start brief — everything you need is in this file.**

## What you are building
Lane 0 freezes the contract first. Before any mode lane writes code, Lane 0 publishes frozen interfaces: the `GameMode` enum, the `services::{llm,chronos,ledger,paths}` signatures, the `Difficulty` type, and the single agreed insertion point in `mod.rs`/`boot.rs`.

## File ownership (touch ONLY these)
- `crates/engine/src/modes/mod.rs` (new)
- `crates/engine/src/services/**` (new/modified)
- `crates/engine/src/chamber/mod.rs` (modify for agreed insertion point)
- `crates/engine/src/boot.rs` (modify for agreed insertion point)
- `crates/engine/src/theme/constants.rs` (for canon reconciliation)

## Build spec
1. **GameMode framework:** A top-level mode layer. Replace the single main-menu button with a real mode selector. Each mode self-registers as a Bevy plugin under `modes/`.
2. **services::llm:** Lift `ollama_chat` out of `council.rs`; add `embed(text) -> Vec<f32>` against Ollama `/api/embeddings`.
3. **services::chronos:** Lift the Chronos client + `ArtifactOutcome`; add a pre-generated / cached image path.
4. **services::paths:** Consolidate `app_data_root()`.
5. **services::ledger:** Local append-only, hash-chained JSONL under app_data_root. Every mode seals events here.
6. **WitnessProfile:** Make `pub`, extend with per-mode stats + Arrow-II signal accumulator.
7. **Difficulty:** One shared axis: *difficulty = interpretive distance*.
8. **Canon reconciliation:** Reconcile Codex signature to canon (`theme_codex()` = perfect fifth C-G). Record Codex = "Lexis", Viren = "Flamebearer" in metadata.
