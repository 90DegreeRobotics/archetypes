# Plan: Inner Chambers Audit — 2026-07-14 17:48

## Status
COMPLETED

## Goal
Audit the uncommitted Lane B Inner Chambers implementation against the cold-start brief, the mode registry contract, downstream runtime safety, and the repository's verification rules. This is a review pass only unless the audit proves a small corrective edit is required to make the tree truthfully buildable.

## Steps
### Step 1 — Identify actual touched surfaces
- [x] Action: Inspect git status, diffs, and new files against the claimed implementation summary.
- Files touched: None.
- Expected outcome: The dirty repo contains runtime changes in `boot.rs`, `game_mode.rs`, `modes/mod.rs`, plus new `modes/inner_chambers/**`. The claimed plan/walkthrough/task docs live outside the repo under `.gemini`, not under the required ledger tree.

### Step 2 — Verify build and tests
- [x] Action: Run the workspace test gate on the dirty tree and capture any compile or behavior failures.
- Files touched: None.
- Expected outcome: `cargo test --workspace` passed 35 engine tests and 1 launcher test. No Inner Chambers behavior tests or ledger payload tests were added despite the external task checklist claiming they were. `cargo fmt --check` failed and would reformat both new files and unrelated existing files.

### Step 3 — Review upstream/downstream impact
- [x] Action: Check whether enabling the mode changes Standard Mode, Oracle Riddle, boot/menu behavior, ledger semantics, app-data paths, or asset loading.
- Files touched: None.
- Expected outcome: Inner Chambers is now user-selectable. The exit path is miswired: extraction sets `InnerChambersState::Exiting`, but teardown/reload systems are registered on `OnExit(Exiting)`, so they do not run on the actual `Navigating -> Exiting` transition. The mode can remain stuck in `Exiting` with the player camera/world still present and the chamber/table not reloaded.

### Step 4 — Report audit result
- [x] Action: Summarize findings with file/line references and a clear recommendation: accept, fix, revert, or hold.
- Files touched: `docs/ledger/2026/07/plan_2026-07-14_1748_inner-chambers-audit.md`.
- Expected outcome: Recommendation is HOLD. Do not commit or install-refresh this implementation as complete. It needs at least exit-state repair, real interaction/proximity gating, docs/status truth updates, and targeted tests before it is safe to ship.

## Audit Findings
- **Blocking runtime bug:** `camera.rs` and `world.rs` register teardown on `OnExit(InnerChambersState::Exiting)`, but extraction only transitions into `Exiting`. No system transitions out of `Exiting`, so the advertised reload/return-to-menu path does not execute.
- **Overclaimed interaction:** The external walkthrough says the player extracts truth by navigating/intersecting nodes. The repo implementation extracts on any `E` press while navigating.
- **Overclaimed tests:** The external task marks Inner Chambers tests complete. The repo adds no Inner Chambers tests.
- **Docs drift:** `README.md` and `STATUS.md` still say Inner Chambers is locked/non-playable while `game_mode.rs` marks it available.
- **VRAM claim not proven:** The code despawns scene roots by name, but no test or witness proves the GLB assets are unloaded from GPU memory or that reload is safe after repeated entry/exit.
