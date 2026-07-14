# Plan: Oracle Riddle Guessable Scoring — 2026-07-14 16:57

## Status
COMPLETED

## Goal
Make Oracle Riddle playable as an actually guessable image-prompt game: replace obscure abstract prompts with visual noun/adjective/action triples, make scoring understandable, add player-facing reward feedback, and audit upstream/downstream impact before shipping.

## Steps
### Step 1 — Audit current Oracle Riddle truth
- [x] Action: Inspect prompt selection, generation, scoring, result UI, ledger payload, tests, and mode integration.
- Files touched: None.
- Expected outcome: Found an existing but weak score path: ordered embedding scores, no robust fallback, and no meaningful player reward beyond per-word numbers. Oracle Riddle is isolated under `modes::oracle_riddle`; Standard Mode and shared services do not need to change.

### Step 2 — Design safer prompt and score contract
- [x] Action: Define guessable prompt triples and player-facing tiers that do not require hidden semantics or impossible metaphors.
- Files touched: None.
- Expected outcome: Replaced abstract prompt targets with concrete visual triples; guesses can be entered in any order; scoring gives exact/alias credit first and only uses embeddings as a soft bonus/fallback.

### Step 3 — Implement gameplay correction
- [x] Action: Update Oracle Riddle prompts/scoring/result copy/tests so rounds are guessable and reward feedback is meaningful.
- Files touched: Oracle Riddle source and docs only.
- Expected outcome: Result screen now shows matched guesses, total score, rank, and Insight reward. Ledger payload seals matched guesses, total score, reward points, and reward label.

### Step 4 — Verify impact
- [x] Action: Run focused tests plus `cargo test --workspace`; check dirty scope and installed refresh needs.
- Files touched: Generated install/dist files only if a refresh is needed.
- Expected outcome: Focused Oracle tests and full workspace tests passed. First install refresh was blocked because the old desktop game process was running; stopped only the `C:\archetypes\dist` launcher/engine PIDs and reran successfully.

### Step 5 — Land work
- [x] Action: Update this ledger with verification, commit, push, and report exact effects.
- Files touched: Git metadata.
- Expected outcome: Verified change is on `origin/main` with a clean working tree.

## Impact Audit
- Score system before: yes, but only ordered embedding similarity with fragile player feedback.
- Score system after: order-insensitive exact/alias scoring, optional embedding soft match, lexical fallback if embeddings are down.
- Player reward before: no real reward loop; only per-word scores.
- Player reward after: immediate Insight reward tier (`Perfect Read`, `Clear Read`, `Partial Read`, `Faint Echo`) plus points sealed into the Oracle ledger.
- Upstream impact: prompt generation still calls the same Chronos artifact request path; shared Chronos and LLM service APIs are unchanged.
- Downstream impact: Standard Mode, council ritual, chamber assets, mode registry semantics, and launcher contract are unchanged. Oracle ledger payload gains extra fields; existing consumers reading old fields still have `target`, `guess`, `scores`, and `error`.
- Negative-system check: no new dependency, no new asset path, no change to desktop shortcut contract. The only runtime interruption was closing the stale running desktop build so Windows could overwrite `dist\engine.exe`.

## Verification
- `cargo test -p engine oracle_riddle`: passed, 9 tests.
- `cargo test --workspace`: passed, 35 engine tests and 1 launcher test.
- `scripts\install_shortcut.ps1`: passed after closing stale `dist` launcher/engine processes; `dist\engine.exe` refreshed at `2026-07-14 17:03`.
