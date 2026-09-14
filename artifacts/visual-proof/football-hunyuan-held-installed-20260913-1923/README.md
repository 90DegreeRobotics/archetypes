# Installed-game witness — `football`, Hunyuan3D-2 lane (refused)

## Verdict

**Pipeline: completed. Import: correctly refused. Held object: none.**

The installed engine ran the real Manifester flow for the exact prompt `football`. Chronos2's
Flux reference (new flat mid-grey backdrop) drew a soccer ball. Hunyuan3D-2 built a watertight
body from it. Chronos2's subject-match seal then recorded `matches=false`, and Archetypes refused
to import the artifact, so nothing reached the altar, the library, or the player's hand. The
frame in this folder is the refusal shown in game.

## Identity

- Engine SHA-256: `D0DF1B61746E5AB976FE9A48E059BC16BF5AC56D0E3C9F7A23EE962A9E4B4F88`
- Chronos2 `chronos.exe` SHA-256: `92367F7CB012ADDBD03AAEA4BDF6B83D66225862FAA51C23CFD166E92ACB24EC`
- Hunyuan emitter SHA-256: `ECA5873F1D8A32555F312722EA25E9394C9285A1D3A200B184A11609B66C7F89`
- Bundle: `%TEMP%\NeuroCognica\Archetypes\manifestations\0d31e22d-c1d0-4c7b-90b3-405455db5a59`
- Elapsed: `585.7 s`

## Measurements

- Mesh: 668,779 vertices, 1,337,554 faces, watertight.
- Subject match (front only): score `0.4563`, covered `0.9274`, invented `0.5268`.
- Refusal text: "Chronos2 rejected its own reconstruction: checked=true, matches=false,
  score=0.4563 … Nothing was imported or saved to your object library."

## Why the seal failed

A Python replica of `chronos_vision`'s mask and comparison reproduced `0.4563` exactly. Its
corner-luminance mask counted the Flux picture's floor shadow as subject, widening the reference
bounds to 490x361 against the consumed cutout's 346x342. Scored against the silhouette of the
cutout Hunyuan actually consumed, the same render gives `0.9228` (invented `0.0155`). The body
was right; the reference outline was wrong. Chronos2 now seals single-image bodies against that
consumed silhouette (see `C:\chronos2\docs\plans\plan_2026-09-13_1905_hunyuan3d_object_lane.md`).

The prompt `football` itself is ambiguous: Flux drew association football. The follow-up run
uses `American football`.
