# Installed-game witness — `a wolf`, operator-review flow (kept and held)

## Verdict

**Two-step review flow: works end to end in the installed game. Held object: yes.**
Visual quality is the operator's call; mechanical facts and known defects are listed below.

## Flow observed

1. Prompt `a wolf` submitted through the real altar prompt at 4.1 s.
2. Chronos2 `first-light --reference-only` drew the reference and stopped. The picture was laid on
   the altar and the game entered reference review (`00a_reference_review.png`): banner
   "[Enter] build the object from it · [R] draw a different picture · [Esc] cancel".
3. The harness pressed Enter. Chronos2 rebuilt from that exact picture (`--reference-image`):
   Hunyuan3D-2 produced a watertight 279,693-vertex / 559,382-face body.
4. Object review at 278.0 s (`00b_object_review.png`): the wolf on the altar, Chronos2's seal
   shown as advice — "Automated check has doubts (0.38): At least one reconstruction-aligned view
   failed; weakest score 0.376. · [Enter] keep it · [X] discard it".
5. The harness pressed Enter (keep). Phase `Completed`; the library row was written then.
6. `E` pressed at 293.6 s; carried state and camera-child visual confirmed at 293.6 s
   (`01_object_held_in_hand.png`).

## Identity

- Engine SHA-256: `3F807833E080848155FD096470D69874DE6BAB01D142F4DD370E2CA9097C1D59`
- Chronos2 `chronos.exe` SHA-256: `5F2F06786E015A439A4B87436F3BB2E4A63B2F298A0298D5C55CDAE15971B3E1`
- Hunyuan emitter SHA-256: `6DE03E82AB5A6BA7C019760777F898C17EE0CF137B9359E41F358BD4757368CF`
- Artifact id: `5b8b4bd7-03a3-4435-9d42-c16cc1309423`
- Installed asset: `manifested/5b8b4bd7-03a3-4435-9d42-c16cc1309423.glb` (1,801,260 bytes)
- Library: exactly one row for this id in
  `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\artifacts\library.jsonl`, written 20:17:23 (after keep)
- Elapsed: `312 s`

## Held report

```
prompt=a wolf
carrying=true
artifact_id=5b8b4bd7-03a3-4435-9d42-c16cc1309423
asset=manifested/5b8b4bd7-03a3-4435-9d42-c16cc1309423.glb
carried_visuals=1
```

## Known defects seen in these frames

- During object review the corner interaction hint reads "[E] Take it from the altar", although
  pickup is correctly blocked until keep.
- After pickup the top banner still reads "MANIFESTED … Staged upon the sacred altar!" and the
  "PRESS [E] TO MANIFEST ARTIFACT" prompt is visible while carrying.
- The wolf stands far larger than the altar: only its legs are in the review frame.
- The body colour is mostly tan/grey with streaks: the Hunyuan mesh is turned about 145 degrees
  from the picture's view, so front/back colour projection lands on the wrong surfaces.
- Banner icon glyphs render as empty boxes (font has no emoji glyphs; pre-existing).
- The reference step returned in seconds because Chronos2's reference cache held the attempt-2
  wolf picture from the 19:43 run.
