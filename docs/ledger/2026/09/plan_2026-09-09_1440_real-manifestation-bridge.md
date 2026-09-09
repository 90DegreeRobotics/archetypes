# Plan: real-manifestation-bridge — 2026-09-09 14:40

## Status
IN-PROGRESS

## Goal
Replace the exposed recipe-based manifestation facade with a prompt-bound, Sentinel-governed Chronos2 Object-mode request and receipt-driven GLB handoff. Make the chamber floor visibly legible as physical stone under the existing lighting rather than relying on subtle image-space normal detail.

## Steps
### Step 1 — Freeze the no-recipe boundary
- [ ] Action: Make the pedestal incapable of invoking `manifest_artifact.py` or accepting its outputs. Require a Chronos2 receipt declaring a TripoSR-generated source mesh and a prompt digest.
- Files touched: manifestation bridge, tests, truth docs.
- Expected outcome: No prompt can resolve to a named primitive, fallback object, or bundled recipe.

### Step 2 — Build the real Chronos2 handoff
- [ ] Action: Invoke Chronos2 Object mode with a unique bundle directory, consume its generated mesh/receipt, run generic headless OBJ-to-GLB conversion, validate the GLB, and stage only the validated output.
- Files touched: bridge runtime and generic conversion helper.
- Expected outcome: Player prompt reaches Sentinel and TripoSR; the returned mesh is the one mounted on the pedestal.

### Step 3 — Make progress and failure truthful
- [ ] Action: Stream/parse Chronos stage output, bind the floating indicator and HUD to real stages, expose Sentinel refusal distinctly, and add cancellation/timeout truth.
- Files touched: manifestation runtime and tests.
- Expected outcome: Waiting UI reports actual request state rather than a decorative timer.

### Step 4 — Make floor relief visible
- [ ] Action: Replace the broad flat floor visual with visibly raised/recessed stone courses while retaining PBR normal response.
- Files touched: inner chamber world/runtime tests.
- Expected outcome: The floor reads as stonework from the player camera in the staged game.

### Step 5 — Verify and publish
- [ ] Action: Run Chronos2 focused gates, Archetypes workspace tests, live object attempt when services are available, refresh the desktop launcher, inspect a visible witness, commit, and push both main branches with explicit paths.
- Files touched: plans, audit records, tests, product code.
- Expected outcome: Auditable cross-repo state with no disguised recipe fallback.

### Step 6 — Harden and re-run the pinned-launcher golden path
- [x] Action: Made the deployment script fail before copying over a live installed process, verified byte-identical engine and launcher hashes across build, dist, and installed locations, reread the Taskbar shortcut target, then rebuilt and reinstalled through the documented path.
- Files touched: Windows deployment script, this plan, deployment audit record.
- Expected outcome: A deployment cannot be reported successful merely because a copy command ran; the installed Taskbar target is proved against the release artifacts.
- Result: `cargo test --workspace` passed (80 engine, 16 launcher, 5 identity tests). `scripts/install_shortcut.ps1` rebuilt, restaged, installed, hash-verified both executables, and reread the persisted pinned shortcut. A new foreground Taskbar launch is still required for a buyer-runtime witness.
