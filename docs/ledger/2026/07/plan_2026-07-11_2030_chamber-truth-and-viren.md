# Plan: Chamber Truth and Viren — 2026-07-11 20:30

## Status
COMPLETED

## Goal
Assimilate the inherited Council Chamber implementation into one honest, tested unit on `origin/main`: audit the uncommitted work, repair lifecycle documentation, align Viren's theme constants with the canonical Ember Covenant manuscript, complete the visible archetype-focus choreography, update the repository truth surfaces, and pass the mandatory workspace test gate before commit and push.

## Steps
### Step 1 — Audit inherited chamber work
- [x] Action: Compare the uncommitted implementation and plans against the canonical game-direction and archetype documents.
- Files touched: None.
- Expected outcome: Exact implementation and documentation gaps are identified before code changes.

### Step 2 — Repair Viren and chamber behavior
- [x] Action: Correct Viren's theme data and implement state-driven sphere focus choreography with a deterministic focused archetype.
- Files touched: `crates/engine/src/theme/constants.rs`, `crates/engine/src/chamber/mod.rs`, `crates/engine/src/chamber/portal.rs`, `crates/engine/src/chamber/spheres.rs`.
- Expected outcome: Viren matches the canonical palette and timing data, and the chamber visibly moves a selected sphere into and out of focus.

### Step 3 — Reconcile truth documentation
- [x] Action: Correct inherited plan lifecycle state and update `README.md` and `STATUS.md` to describe the actual interactive surface and remaining stubs.
- Files touched: `docs/ledger/2026/07/plan_2026-07-11_1938_council_chamber_loop.md`, `docs/ledger/2026/07/plan_2026-07-11_1944_archetype_theme_registry.md`, `README.md`, `STATUS.md`.
- Expected outcome: Documentation no longer overstates completion and names the live controls and gaps plainly.

### Step 4 — Verify, seal, and publish
- [x] Action: Run formatting and `cargo test`, complete this plan, commit the coherent unit with inherited-work attribution, and push `main` to `origin`.
- Files touched: This plan document and any formatter-adjusted Rust files.
- Expected outcome: Tests pass, the working tree is clean, and the verified unit exists on `origin/main`.
