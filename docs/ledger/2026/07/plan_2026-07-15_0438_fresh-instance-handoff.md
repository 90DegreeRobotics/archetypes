# Plan: Fresh Instance Handoff - 2026-07-15 04:38

## Status
COMPLETED

## Goal
Create a cold-start handoff document for a fresh Codex/agent instance so it can continue Archetypes Standard Mode work immediately: black title intro into a game-feeling main menu, Mecha-style archetype chat as source canon, old Standard Mode preserved for reference, and explicit verification/stop rules.

## Steps
### Step 1 - Gather current truth anchors
- [x] Action: Verify clean repo status, latest commits, current status, Mecha audit, Standard rebuild study, and risk-based verification rules.
- Files touched: this plan.
- Expected outcome: Handoff is grounded in checked repo state rather than chat memory.

### Step 2 - Write the full handoff
- [x] Action: Create a cold-start Markdown handoff with base commit, required reads, current design direction, implementation scope, file map, visual target, safety fences, and verification gates.
- Files touched: new handoff document and `STATUS.md`.
- Expected outcome: A fresh instance can start work without needing this conversation.

### Step 3 - Verify docs-only change
- [x] Action: Read back the handoff, run diff/status checks, and use docs-only verification under `AGENTS.md`.
- Files touched: this plan, the handoff document, and `STATUS.md`.
- Expected outcome: No cargo run is needed because no runtime code changed.

### Step 4 - Commit and push
- [x] Action: Commit and push the handoff to `origin/main`.
- Files touched: git metadata only.
- Expected outcome: The handoff exists on the shared audit surface for the fresh instance.

## Result
Created `docs/ledger/2026/07/handoff_codex_2026-07-15_standard-mode-mecha-main-menu.md` as the cold-start handoff. This is a docs-only unit; verification is readback, diff/status checks, commit, push, and clean-tree confirmation.
