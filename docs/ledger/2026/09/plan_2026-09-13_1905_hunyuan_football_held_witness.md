# Plan: Hunyuan3D football held in hand — 2026-09-13 19:05

## Status
IN-PROGRESS

## Goal
Accept Chronos2's new Hunyuan3D-2 Object-mode receipt and prove, in the installed game, that a
newly generated football is created through the Manifester, picked up with `E`, and carried.

## Evidence at start
- `9c6432f` held witness: carrying works, but the TripoSR football was an open malformed shell.
- `89d42b3`: `matches=false`, unchecked, or missing subject judgment now refuses import.
- Chronos2 plan `plan_2026-09-13_1905_hunyuan3d_object_lane.md` replaces the geometry engine.
- The game has no visible player hand/arms; the held object floats in first-person hold position.

## Steps
### Step 1 — Receipt contract
- [x] Action: accept `chronosophia.hunyuan3d-mesh.v1` (single source image digest, mesh digest,
  coverage) alongside the existing schemas; keep every refusal path.
- Files touched: `crates/engine/src/services/chronos_receipt.rs`, bridge env in `manifestation.rs`.
- Expected outcome: fixtures for accept, digest mismatch, and `matches=false` all behave.

### Step 2 — Installed-game witness
- [ ] Action: refresh the installed build, create "football" through the Manifester, pick it up,
  capture the held frame with the walking capture harness, and record hashes.
- Files touched: evidence under `artifacts/visual-proof/`, this plan.
- Expected outcome: a held-football frame shown to the operator; verdict is theirs.

## Run log

- 2026-09-13 19:23, prompt `football`, installed engine `D0DF1B61…`: pipeline completed in
  585.7 s, Hunyuan3D-2 built a watertight soccer ball, Chronos2 sealed `matches=false` (0.4563),
  and the game refused the import with the full reason on screen. Gate behaved as designed;
  nothing held. Cause was the seal's reference outline (floor shadow counted as subject), not the
  body; replica score against the consumed silhouette is 0.9228. Evidence:
  `artifacts/visual-proof/football-hunyuan-held-installed-20260913-1923/`.
- Next: prompt `American football` after Chronos2 seals single-image bodies against the consumed
  silhouette.
