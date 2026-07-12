# Plan: First World-Engine Slice — 2026-07-11 21:30

## Status
IN-PROGRESS

## Goal
Build the smallest honest end-to-end council-world vertical slice through the live Bevy application: establish a persistent Witness profile, accept a portal offering, perform the chamber reveal and Architect focus, enter an Architect interior, produce a Witness-authorized verdict, and invoke a verified local Chronos artifact route when available while failing closed and visibly when it is not.

## Steps
### Step 1 — Audit the live Chronos artifact seam
- [ ] Action: Read Chronos truth surfaces and inspect callable local artifact routes, request schemas, outputs, and readiness probes.
- Files touched: None in `C:\chronos`.
- Expected outcome: The integration targets a verified route or command rather than a fabricated generator.

### Step 2 — Implement persistent Witness onboarding and portal input
- [ ] Action: Add AppData-backed profile storage and a Bevy UI flow for profile sealing and portal text submission.
- Files touched: Engine profile, persistence, portal, and chamber modules.
- Expected outcome: A first-run player creates a durable Witness identity and can submit a real text offering from the live game.

### Step 3 — Complete one deliberation and Architect interior
- [ ] Action: Drive the submitted offering through upward reveal, Architect focus, an Architect-themed interior state, and a visible Witness authorization step.
- Files touched: Chamber state, camera, spheres, theme, and UI modules.
- Expected outcome: The player traverses one complete ritual sequence without terminal-only controls.

### Step 4 — Return one truthful artifact result
- [ ] Action: Call the verified Chronos seam, present its returned artifact and provenance in-game, or present a precise fail-closed readiness error when external prerequisites are unavailable.
- Files touched: Chronos bridge, artifact state, UI, and persistence modules.
- Expected outcome: No fake output is shown; success has an actual artifact path and failure has an actionable checked reason.

### Step 5 — Verify, document, and publish
- [ ] Action: Add focused tests, run `cargo test`, build and visually exercise the live path, update truth docs, complete this plan, commit, and push `main`.
- Files touched: Tests, `README.md`, `STATUS.md`, and this plan.
- Expected outcome: The vertical slice is proven in proportion to available external readiness and exists on `origin/main` with a clean tree.
