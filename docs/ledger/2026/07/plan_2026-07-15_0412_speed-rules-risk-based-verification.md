# Plan: Speed Rules and Risk-Based Verification - 2026-07-15 04:12

## Status
COMPLETED

## Goal
Update the repository operating rules so development moves faster without weakening truth. The operator wants more collaboration and explanation, not quieter execution. The target is risk-based verification: full cargo gates stay mandatory for Rust/user-facing runtime behavior, but docs-only and asset-only units should not automatically pay the full Rust test tax unless they make runtime claims or need a commit gate.

## Steps
### Step 1 - Rewrite verification rules
- [x] Action: Replace the blanket `cargo test` before every commit language with tiered verification gates by change type.
- Files touched: `AGENTS.md`.
- Expected outcome: Agents can use docs/asset-specific verification for docs/assets and reserve full workspace cargo tests for Rust/runtime/user-facing behavior.

### Step 2 - Rewrite planning rule
- [x] Action: Change "plan before every action" into one plan per meaningful work unit before edits/builds/deploys, so agents stop creating or implying micro-plans for every tiny step.
- Files touched: `AGENTS.md`.
- Expected outcome: Planning remains mandatory, but it stops becoming ceremony that slows execution.

### Step 3 - Preserve collaboration reporting
- [x] Action: Add explicit guidance that the operator wants more explanation, not silence, and that speed changes must not reduce status narration.
- Files touched: `AGENTS.md`.
- Expected outcome: Faster verification does not become quiet, unexplained tool use.

### Step 4 - Update current status
- [x] Action: Record the rule change and verification performed.
- Files touched: `STATUS.md`.
- Expected outcome: The repo truth surface explains why future docs/assets work may not always run a full cargo test.

## Result
Root `AGENTS.md` now uses risk-based verification gates and one plan per meaningful work unit. The collaboration/reporting rule explicitly says speed must not come from silence. `STATUS.md` records the rule change and the narrowed verification basis for this docs/rules-only unit.
