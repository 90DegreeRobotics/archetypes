# Plan: Actually raise the star well above the table — 2026-07-13 18:05

## Status
IN PROGRESS

## Problem (root cause codex missed twice)
Two prior commits — `0ae8c89 "raise council"` and `dc03a58 "raise star"` — both
claimed to lift the star and neither changed its height:

- The **visible star tips are the seven glass vessels**, authored as children of the
  `Council_Assembly` node inside `uiscene1.glb`, baked at world **y = 2.0**. Their
  vertical span is y ≈ −1.49 (Jester) … +5.49 (Sentinel). The bottom tips land at
  y ≈ −1.5, essentially touching the table top (world y ≈ −1.8).
- **No runtime code ever moves `Council_Assembly`.** `COUNCIL_CENTER` (the constant
  the "raise" commits fiddled with) only positions the runtime *solid crystal core*
  (hidden until Deliberating) and the deliberation camera aim. It stayed `(0, 2, 0)`.
- Net effect: every "raise" touched things that don't change where the operator sees
  the star. The star has never left the table.

## Goal
Lift the **entire authored council constellation** (seven vessels + crystal core, and
the seven portrait `*_PanelSpinner` roots so they keep their authored offset) as one
group so the whole star floats well above the table, and make the height a single
source of truth so this class of desync cannot recur.

## Approach
1. `COUNCIL_CENTER.y`: `2.0 → 6.0`. This is now the single authoritative constellation
   height; the crystal core spawns here and the deliberation camera aims here.
   New vessel span ≈ y 2.5 … 9.5; bottom tips ~4.3 above the table top.
2. Add `AUTHORED_COUNCIL_Y = 2.0` (the baked height) so `lift = COUNCIL_CENTER.y −
   AUTHORED_COUNCIL_Y`.
3. New system `raise_council_constellation` (in `spheres.rs`, runs in Update):
   - Pin `Council_Assembly` translation.y to `COUNCIL_CENTER.y` (absolute → idempotent).
   - Add `lift` to each `*_PanelSpinner` root once (marker-gated → idempotent), so the
     portraits rise with their vessels and keep the authored vessel↔panel offset.
4. Star size math is unaffected: vessel GlobalTransforms and `COUNCIL_CENTER` shift by
   the same `lift`, so `mean_radius` (hence the crystal's size) is unchanged.

## Steps
- [x] Edit `mod.rs`: raise `COUNCIL_CENTER.y` to 6.0, add `AUTHORED_COUNCIL_Y`.
- [x] Add `raise_council_constellation` system + `Raised` marker to `spheres.rs`; register it.
- [x] `cargo build -p engine` — compiles clean (37s, 0 errors).
- [x] `cargo test --workspace` passed (18 tests green).
- [ ] HOLD for operator launch signal. Do not self-approve.

## Verification (to be filled after operator launches)
- Pixel-evidence that the star sits well clear of the table in the main menu and reads
  correctly in deliberation. Operator is the sole visual judge.

## Open questions for operator
- Height 6.0 is a first pick for "well above"; trivial to tune (one constant).
- Secondary camera poses (Witness establishing/verdict at authored y≈6.2; CouncilSpeaking
  frame at y=4.5) were composed for the old y=2 star and may need reframing next.
