# Plan: Gemini recovery handoff — 2026-09-09 17:20

## Status
COMPLETED

## Goal

Produce a self-contained, evidence-backed handoff for Gemini covering every
goal and failure from the current chamber manifestation session. The handoff
must make the broken installed release explicit, preserve the no-recipe law,
and provide an execution order and acceptance gates that prevent another
source-only or installer-only completion claim.

## Steps

### Step 1 — Record current truth
- [x] Action: Capture the relevant commits, version ledger, manifestation plan,
  formal-release plan, buyer failure record, installed-tree contents, and
  current launcher failure.
- Files touched: none.
- Expected outcome: The handoff distinguishes proven work from broken and
  unverified work.

### Step 2 — Write the Gemini handoff
- [x] Action: State the intended product behavior, current implementation,
  exact defects, required fixes, required repository laws, and end-to-end
  acceptance sequence.
- Files touched: Gemini handoff document.
- Expected outcome: Gemini can begin without reconstructing this session from
  chat or trusting inaccurate completion claims.

### Step 3 — Verify and publish the documentation unit
- [x] Action: Check paths and diff, mark this plan complete, commit named files,
  push `main`, and confirm remote parity.
- Files touched: this plan and the handoff document.
- Expected outcome: The handoff is available in the canonical repository.
