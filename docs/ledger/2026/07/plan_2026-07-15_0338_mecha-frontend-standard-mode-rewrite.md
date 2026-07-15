# Plan: Mecha Frontend Standard Mode Rewrite — 2026-07-15 03:38

## Status
COMPLETED

## Goal
Fully examine `C:\mecha\aura-mechanician\frontend\src` as preserved source canon for the Archetypes Standard Mode rewrite. The target functionality is not optional inspiration: Archetypes should let the player chat with archetypes, and when the player chats with an archetype the behavior and visual language should carry forward from the Mecha frontend. Preserve the old Standard Mode for reference while planning a replacement.

## Steps
### Step 1 — Inventory the full Mecha frontend source
- [x] Action: List every file under `C:\mecha\aura-mechanician\frontend\src`, including assets, themes, utilities, scripts, and entrypoints.
- Files touched: this plan and `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`.
- Expected outcome: No part of the source folder is ignored or summarized from memory.

### Step 2 — Read and classify functionality
- [x] Action: Read the source files and classify all behaviors: chat, council mode, archetype switching, theme loading, asset loading, splash/startup, state monitors, portrait presentation, Electron shell, and any supporting scripts/styles.
- Files touched: this plan and `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`.
- Expected outcome: A file-by-file map of what exists and what it does.

### Step 3 — Map Mecha features into Archetypes
- [x] Action: Define how the Mecha frontend behavior should become hard-coded game functionality in Bevy/Rust without losing the old design.
- Files touched: this plan and `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`.
- Expected outcome: A concrete Standard Mode rewrite contract, not a vague recommendation.

### Step 4 — Protect reference and name risks
- [x] Action: Specify how to preserve the old Archetypes Standard Mode for reference, identify upstream/downstream risks, and define verification gates.
- Files touched: this plan, `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`, and `STATUS.md`.
- Expected outcome: The rewrite can proceed without accidentally deleting reference behavior, breaking Oracle Riddle, or pretending partial UI parity is complete.

## Result
The full Mecha frontend audit is captured in `docs/ledger/2026/07/mecha_frontend_full_audit_2026-07-15.md`. This is an audit/contract unit only. No game code was changed yet.
