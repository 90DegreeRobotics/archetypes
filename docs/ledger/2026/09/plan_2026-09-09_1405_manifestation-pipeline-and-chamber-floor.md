# Plan: manifestation-pipeline-and-chamber-floor — 2026-09-09 14:05

## Status
COMPLETED (floor); MANIFESTATION PIPELINE BLOCKED BY VERIFIED NO-RECIPE CONTRACT GAP

## Goal
Audit the player-facing Manifestation Pedestal path from prompt submission through Chronos2 headless generation, GLB export and validation, return into Archetypes, placement on the pedestal, and visible wait/failure states. Remove any reachable recipe or prebuilt-object substitution that violates the no-recipe law. Add a floor material with real normal/bump response only after verifying it survives the live chamber asset path.

## Steps
### Step 1 — Trace the manifest route
- [x] Action: Inspect the pedestal implementation, its child process invocation, the Chronos2 command it calls, result-file contract, and GLB import/placement logic; identify disconnected or simulated stages.
- Files touched: audit evidence and only runtime files proven defective.
- Expected outcome: A source-backed map of what is real, blocked, or theatre.

### Step 2 — Enforce no-recipe object generation
- [ ] Action: Blocked: the reachable route is itself the recipe script; Chronos2 currently lacks the validated GLB handoff and no-recipe receipt needed to replace it honestly. Findings are recorded in `docs/ledger/audits/audit_2026-09-09_manifestation-pipeline-truth.md`.
- Files touched: only the reachable manifest path and its documentation/tests.
- Expected outcome: Sentinel-authorized prompts go to Chronos2 generation or fail visibly; no prompt becomes a premade object.

### Step 3 — Author a light-reactive chamber floor
- [x] Action: Update the actual chamber runtime path with generated PBR basalt albedo and linear normal-map textures, bound to the floor material, and add a unit-level format/dimension contract.
- Files touched: the authoritative chamber authoring source and regenerated chamber asset if applicable.
- Expected outcome: The shipped chamber floor has non-emissive surface detail that responds to scene lighting.

### Step 4 — Prove, stage, and publish
- [x] Action: `cargo test --workspace` passed (96 tests); `pwsh -File scripts\install_shortcut.ps1` rebuilt and copied the release engine into `dist` and `%LOCALAPPDATA%\Programs\Archetypes`, then refreshed the Desktop, Start Menu, and pinned Taskbar shortcuts. The available native UI automation surface exposed no Windows apps, so an automated visual chamber witness could not be captured in this session; this is not treated as visual approval.
- Files touched: plan/evidence documentation and verified product files.
- Expected outcome: A real, buyer-visible pipeline or an explicit, source-backed blocker—never a simulated claim.
