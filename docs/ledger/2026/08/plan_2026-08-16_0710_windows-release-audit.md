# Plan: Windows release lifecycle audit — 2026-08-16 07:10

## Status
COMPLETED

## Goal
Operator-initiated full-repo audit of Windows product lifecycle (install, uninstall, desktop icon, help, sidecar downloads), mode completeness, and whether the shipped game still showcases the AURA conversational council. Produce a ledger audit plus a navigable canvas. No product-code changes in this unit.

## Steps
### Step 1 — Read canon and status
- [x] Action: Read README, STATUS, PROTOCOL, architecture docs, Sentinel docs, Windows metabolism, mode contracts, launcher/engine source, install scripts.
- Files touched: none (reads only).
- Expected outcome: Ground truth of what is documented vs what Desktop actually launches.

### Step 2 — Parallel mode and lifecycle exploration
- [x] Action: Exhaustive inventory of installer/uninstall/help/sidecar surfaces and of all four GameMode lanes including parked ritual code.
- Files touched: none.
- Expected outcome: Confirmed gaps: no MSI/uninstaller/help; Standard is Mecha 1:1 chat; council ritual is orphaned; Inner Chambers locked prototype; Living Engine has no implementation.

### Step 3 — Write audit artifacts
- [x] Action: Write ledger plan, `docs/ledger/audits/` report, update `CURRENT_HANDOFF.md`, and a Cursor canvas.
- Files touched: this plan, audit markdown, `docs/ledger/CURRENT_HANDOFF.md`, canvas outside the repo.
- Expected outcome: Operator can navigate findings without re-reading the tree.

### Step 4 — Close without product edits
- [x] Action: Mark plan COMPLETED. Do not commit unless the operator asks.
- Files touched: this plan.
- Expected outcome: Audit unit complete; implementation remains a follow-up unit.
