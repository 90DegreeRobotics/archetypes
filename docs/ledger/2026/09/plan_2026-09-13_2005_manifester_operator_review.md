# Plan: Manifester operator review — 2026-09-13 20:05

## Status
IN-PROGRESS

## Goal
Put the operator in charge of whether a creation failed. The Manifester becomes two steps:
first generate and show the reference picture and ask whether to proceed; then build the object
from that exact picture and show it on the altar with Chronos2's automated check as advice, to
keep or discard. Nothing enters the permanent library until the operator keeps it.

## Evidence at start
- Operator direction (2026-09-13, after the wolf run): "maybe it should let me decide if it is a
  failure. like the first step should be create and show the reference and then prompt to
  proceed with object creation from it".
- This supersedes the same-day fail-closed `matches=false` import gate (`89d42b3`,
  `plan_2026-09-13_1256_reject_failed_manifester_artifacts.md`). Two installed runs showed the
  scorer refusing sound bodies: a correct soccer ball (0.4563, shadow counted as subject) and a
  recognisable wolf (0.3763, mesh turned 145 degrees from the picture's view).
- Integrity, digest, prompt, and Sentinel refusals stay automatic: those are facts, not taste.
- Chronos2 already builds from a supplied picture (`first-light --reference-image`); it gains
  `--reference-only` for the first step.

## Steps
### Step 1 — Chronos2 reference-only step
- [x] Action: `first-light --geometry-forge --reference-only` clears Sentinel, checks the
  Hunyuan runtime, draws the Flux reference, writes `reference_input.png`, and exits before
  Hunyuan, Blender, and sealing.
- Files touched: `C:\chronos2\crates\chronos_cli\src\main.rs`, `governed_artifact.rs`.
- Expected outcome: a picture in about five minutes with no mesh work spent.

### Step 2 — Receipt verdict becomes advice
- [x] Action: `chronos_receipt::verify` no longer refuses `matches=false` or an unchecked
  judgment; it returns the verdict (checked, matches, score, reason) on `GameArtifact`.
- Files touched: `crates/engine/src/services/chronos_receipt.rs` and its tests.
- Expected outcome: tests prove advice is carried and integrity refusals still fire.

### Step 3 — Manifester phases
- [x] Action: add reference generation, reference review (Enter build / R redraw / Esc cancel),
  object build from the approved picture, and object review (Enter keep / X discard). The worker
  stages the GLB without recording it; keep writes the library row; discard removes the staged
  file. Pickup works only after keep.
- Files touched: `crates/engine/src/modes/inner_chambers/manifestation.rs`, `objects.rs` if needed.
- Expected outcome: no creation reaches the library without an explicit keep.

### Step 4 — Capture harness and installed witness
- [x] Action: teach `manifest_capture.rs` to press through both reviews, capturing the
  reference-review and object-review frames; refresh the install; run `a wolf` end to end and
  capture the held frame.
- Files touched: `manifest_capture.rs`, evidence under `artifacts/visual-proof/`.
- Expected outcome: the operator sees the reference, the object with its advice, and the object
  in hand; the visual verdict is theirs.

## Verification record

- Chronos2 `cargo check -p chronos_cli --tests` clean; release `chronos.exe`
  `5F2F06786E015A439A4B87436F3BB2E4A63B2F298A0298D5C55CDAE15971B3E1`.
- Archetypes `cargo check -p engine --tests` clean; `cargo test -p engine` 292/292 after the seed
  assertion was updated to the reviewed flow (first draw attempt 2; redraw advances).
- Installed refresh: engine `3F807833E080848155FD096470D69874DE6BAB01D142F4DD370E2CA9097C1D59`.
- Installed witness, prompt `a wolf`, 312 s
  (`artifacts/visual-proof/a-wolf-hunyuan-held-installed-20260913-2012/`): reference review shown
  and approved; object built from that exact picture; object review showed Chronos2's seal as
  advice ("has doubts, 0.38"); keep wrote exactly one library row (20:17:23); `E` pickup confirmed
  `carrying=true` with one camera-child visual. Steps 1–4 are demonstrated in the installed UI.
- Defects found in the witness frames, open: review-time "[E] Take it from the altar" hint
  (fix in progress, `objects.rs`); stale "Staged upon the sacred altar" banner and
  "PRESS [E] TO MANIFEST ARTIFACT" while carrying; object far larger than the altar; colour
  projected onto the wrong surfaces because the mesh is turned ~145 degrees from the picture.
