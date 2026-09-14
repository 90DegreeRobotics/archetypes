# Installed-game witness — `a wolf`, Hunyuan3D-2 lane (refused)

## Verdict

**Pipeline: completed. Body: a recognisable wolf. Colour: wrong. Import: refused by the scorer.**

The installed engine ran the real Manifester flow for `a wolf`. Flux drew a standing wolf in a
front three-quarter view on the mid-grey backdrop. Hunyuan3D-2 built a closed body with four
legs, a tail, and ears (`hero.png` in the bundle). Chronos2 sealed `matches=false` and the game
refused the import, so nothing was held.

## Identity

- Engine SHA-256: `D0DF1B61746E5AB976FE9A48E059BC16BF5AC56D0E3C9F7A23EE962A9E4B4F88`
- Chronos2 `chronos.exe` SHA-256: `6326ECD9EED0A10B78FF45D247F2B207FEAAD009F050D9501188BA24E1B31D2C`
- Hunyuan emitter SHA-256: `6DE03E82AB5A6BA7C019760777F898C17EE0CF137B9359E41F358BD4757368CF`
- Bundle: `%TEMP%\NeuroCognica\Archetypes\manifestations\5eee8aff-111c-45db-841c-c849df3e3ec1`
- Elapsed: `592.4 s`

## Measurements

- Mesh: 279,693 vertices, 559,382 faces, watertight.
- Subject match (front, against the consumed silhouette): score `0.3763`, covered `0.4653`,
  invented `0.3370`.

## Why it failed

Hunyuan3D-2 turns the subject to its own canonical pose instead of keeping the picture's view.
An orthographic silhouette search over the mesh (same 64-grid IoU as the seal) scored `0.3981`
at the recorded front and `0.7316` at yaw 145 / pitch 10. So the seal compared the wrong angle,
and the front/back colour projection painted the wolf's face onto its hip and tail.

## Operator direction after this run

The operator decides whether an object failed. The Manifester becomes two steps: show the
reference and ask to proceed, then build from that exact picture and show the result with the
automated check as advice to keep or discard. Plans:
`docs/ledger/2026/09/plan_2026-09-13_1905_hunyuan_football_held_witness.md` and
`C:\chronos2\docs\plans\plan_2026-09-13_1905_hunyuan3d_object_lane.md`.
