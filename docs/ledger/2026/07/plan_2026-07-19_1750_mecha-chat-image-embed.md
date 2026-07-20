# Plan: mecha-chat-image-embed — 2026-07-19 17:50

## Status
COMPLETED

## Goal
Fix stretched Mecha chat portraits and Comfy artifacts in Standard Mode, and embed each completed artifact inline in the left channel for every archetype. Profile portraits stay in the right panel at correct aspect; artifacts leave the sidebar strip.

## Steps
### Step 1 — Ledger plan
- [x] Action: Write this plan
- Files touched: `docs/ledger/2026/07/plan_2026-07-19_1750_mecha-chat-image-embed.md`
- Expected outcome: Work unit has a dated plan before code edits

### Step 2 — Portrait native aspect
- [x] Action: Add per-archetype `portrait_aspect`; size portrait with `NodeImageMode::Auto`, width-only constraint, `aspect_ratio`
- Files touched: `crates/engine/src/modes/standard_mecha/mod.rs`
- Expected outcome: Sidebar portraits match 2:3 or 1:1 source assets

### Step 3 — Chat-embedded artifacts
- [x] Action: Replace transcript text blob with scrollable message rows; inline Auto-fit artifact images; remove sidebar Comfy image
- Files touched: `crates/engine/src/modes/standard_mecha/mod.rs`
- Expected outcome: Artifacts appear under reply text in the left channel

### Step 4 — Reveal + status retarget
- [x] Action: Point soft reveal at newest in-chat artifact; keep short status under portrait
- Files touched: `crates/engine/src/modes/standard_mecha/mod.rs`
- Expected outcome: Fade-in still works; status remains player-facing

### Step 5 — Docs + verification
- [x] Action: Update STATUS.md; run `cargo test --workspace`
- Files touched: `STATUS.md`, this plan
- Expected outcome: Gate green; status reflects new chat image presentation

## Test gate
```
cargo test --workspace
```

## Rollback
Revert the unit commit. Never force-push.
