# Plan: Altar banner after pickup — 2026-09-13 20:35

## Status
IN-PROGRESS

## Goal
Make the altar's status banner tell the truth after a kept object leaves the altar, and witness
in the installed game that the review-time pickup hint (commit `08e7db8`) is gone.

## Evidence at start
- Installed wolf witness `artifacts/visual-proof/a-wolf-hunyuan-held-installed-20260913-2012/`:
  - `00b_object_review.png` showed "[E] Take it from the altar" while pickup was blocked for
    review. Fixed in `objects.rs` (`08e7db8`), tests 292/292, installed-UI witness pending.
  - `01_object_held_in_hand.png` showed "MANIFESTED: "a wolf" — Staged upon the sacred altar!"
    while the wolf was in hand. `update_manifestation_hud` shows the `Completed` banner whenever
    the phase is `Completed`, whether or not anything still stands on the altar.
- Not a defect: "PRESS [E] TO MANIFEST ARTIFACT" while carrying is truthful. `take_from_altar`
  ignores `E` while carrying, and `handle_manifestation_input` opens the prompt whenever the altar
  is empty, so `E` does start a new creation. Hiding it would make the prompt lie.
- Not changed here (operator's visual call): object size on the altar. The import script
  normalises the longest axis to 1.4 m and `MANIFESTATION_OBJECT_SCALE` takes it to 1.6 m (the
  cushion diameter) with a 0.58 m lift, so a long, tall subject such as the wolf tops out near
  3.4 m, above the 3.25 m standing eye.

## Steps
### Step 1 — Banner follows the object
- [x] Action: show the `Completed` banner only while `active_artifact` is on the altar; hide it once
  the object is taken.
- Files touched: `crates/engine/src/modes/inner_chambers/manifestation.rs` (+ a focused test).
- Expected outcome: no "Staged upon the sacred altar" once the object is in hand.

### Step 2 — Installed witness
- [x] Action: `cargo test -p engine`, `scripts/install_shortcut.ps1`, then an installed capture that
  photographs object review (no pickup hint) and the held frame (no stale banner).
- Files touched: evidence under `artifacts/visual-proof/`, this plan.
- Expected outcome: both corrected surfaces seen in the installed UI; commit and push.

## Verification record

- `cargo test -p engine` 293/293, including
  `the_staged_banner_only_shows_while_the_object_is_on_the_altar`.
- Installed refresh: engine `6A4CEF808230B32E10C47B5B5926AC9A3FE3FF3F823E85E4AADB73DB00ED8F91`.
- Installed witness `artifacts/visual-proof/a-wolf-hunyuan-held-installed-20260913-2025/`: object
  review shows no "[E] Take it from the altar" hint; the held frame shows no "Staged upon the
  sacred altar" banner; pickup confirmed (`carrying=true`, one camera-child visual).
