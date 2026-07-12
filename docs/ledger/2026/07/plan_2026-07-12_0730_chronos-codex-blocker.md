# Plan: Chronos canvas seam and council text — 2026-07-12 07:30

## Status
COMPLETED

## Goal
Reuse Chronos's already-live direct image-on-canvas generation seam for Archetypes, then replace the chamber's monolithic text wall with a closable full-conversation drawer and a concise themed speech bubble anchored to the speaking sphere. Preserve fail-closed artifact truth and prepare deterministic visual proof for operator review rather than self-approving the game's appearance.

## Steps
### Step 1 — Establish cross-repository truth
- [x] Action: Read Chronos governance, inspect both repositories and live processes, reproduce the prior request, and identify the direct canvas-image route already owned by Chronos.
- Files touched: This plan only.
- Expected outcome: Exact root cause with evidence, not a restart superstition.

### Step 2 — Wire the existing canvas seam
- [x] Action: Replace Archetypes' hybrid scene-render request with Chronos's direct Comfy canvas-image endpoint, map its synchronous response truthfully, and retain verified-PNG staging into the existing in-game image panel.
- Files touched: `crates/engine/src/chamber/ritual.rs` and focused tests/docs.
- Expected outcome: Archetypes asks Chronos for the same class of standalone image used by the museum-wall canvas flow, without invoking Blender scene rendering.

### Step 3 — Build the council text presentation
- [x] Action: Preserve the full conversation in a closable side drawer, add short archetype-themed chat bubbles projected beside the active sphere, separate prompts and voice status, and animate new speech with restrained fly/fade motion.
- Files touched: Chamber UI/state modules and focused tests.
- Expected outcome: Dialogue remains readable and reviewable without obscuring the temple or disconnecting each line from its speaker.

### Step 4 — Verify and publish the audit surface
- [x] Action: Run focused tests and `cargo test`, prepare deterministic screenshots for operator visual approval, reconcile documentation, complete this plan, commit and push `main`, and leave the tree clean.
- Files touched: Proof artifacts, README/STATUS, and this plan.
- Expected outcome: Functional proof is on `origin/main`; visual acceptance remains explicitly assigned to the operator.

## Outcome

- `cargo test`: 12 passed, 0 failed.
- Live app proof: concept `2b2ec8d1-eb6e-4c1a-b958-6e86a1ad8171`, completion event `e3e99f9a-503c-4367-9691-e0f32c73b4de`, staged PNG 2,188,686 bytes.
- Review candidates: `artifacts/visual-proof/council-text-ui-final/05_council_speaking.png` and `08_artifact_result.png`. These prove rendering and wiring; only the operator can approve the visual direction.
