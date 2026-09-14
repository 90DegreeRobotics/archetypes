# Installed-game witness — altar hint and banner after review fixes

## Verdict

**Both corrected surfaces are seen in the installed game.**

- `00b_object_review.png`: during object review the corner bar shows only
  "Walking | Shift: Sprint | 2x Space: Fly | Esc: Menu". The "[E] Take it from the altar" hint no
  longer appears while pickup is blocked for review (fix `08e7db8`).
- `01_object_held_in_hand.png`: with the wolf in hand there is no top banner. The stale
  "MANIFESTED … Staged upon the sacred altar!" banner is gone (`completed_banner_visible`).
  The carry hints read "[F] Set it down [R] Duplicate [LMB] Swing".

"PRESS [E] TO MANIFEST ARTIFACT" is still visible while carrying. That is accurate: with the
altar empty, `E` opens a new creation even while an object is in hand.

## Flow observed

- Reference review at 4.2 s (cached attempt-2 wolf picture), approved with Enter.
- Object review at 285.8 s; Chronos2's seal shown as advice ("has doubts (0.38)").
- Kept with Enter; `E` pickup at 301.3 s; carried state and camera-child visual confirmed at 301.3 s.

## Identity

- Engine SHA-256: `6A4CEF808230B32E10C47B5B5926AC9A3FE3FF3F823E85E4AADB73DB00ED8F91`
- Chronos2 `chronos.exe` SHA-256: `5F2F06786E015A439A4B87436F3BB2E4A63B2F298A0298D5C55CDAE15971B3E1`
- Hunyuan emitter SHA-256: `6DE03E82AB5A6BA7C019760777F898C17EE0CF137B9359E41F358BD4757368CF`
- Artifact id: `6757eb0f-0a47-4e8a-994d-833476b1116e`
- Tests: `cargo test -p engine` 293/293 (adds
  `the_staged_banner_only_shows_while_the_object_is_on_the_altar`)
- Elapsed: `319.3 s`

## Held report

```
prompt=a wolf
carrying=true
artifact_id=6757eb0f-0a47-4e8a-994d-833476b1116e
asset=manifested/6757eb0f-0a47-4e8a-994d-833476b1116e.glb
carried_visuals=1
```

## Still open (unchanged by this fix)

- The wolf's colour lands on the wrong surfaces: the Hunyuan mesh is turned about 145 degrees
  from the picture's view (Chronos2 follow-up).
- Object size on the altar is by design (longest side 1.6 m, the cushion diameter); whether that
  is right for tall subjects is the operator's call.
