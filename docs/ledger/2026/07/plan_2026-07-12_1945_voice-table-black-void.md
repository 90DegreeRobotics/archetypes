# Plan: Exact voices, council table, black void — 2026-07-12 19:45

## Status
COMPLETED

## Goal
Make spoken output exactly match every generated council bubble, replace the noisy AI-derived table/floor treatment with the clean supported council-table language in the operator references, remove the oversized yellow ring, and enforce a black void behind all illuminated scene assets.

## Steps
### Step 1 — Speak the actual council text
- [x] Action: Replace signature-line playback with per-turn Kokoro synthesis of the full generated `CouncilLine`, and prevent speaker advancement until synthesis/playback completes or fails visibly.
- Files touched: Council/speech systems and tests.
- Expected outcome: Bubble text and heard speech are identical without truncation or repetition.

### Step 2 — Re-author the table surface and portal
- [x] Action: Preserve the raw imported source but exclude it from export; build a clean dark tabletop, gilded rim, contained cyan geometric inlay, supported leg assembly, and layered non-pinwheel stargate using a purpose-built portal texture.
- Files touched: Blender preparation script, generated texture, `assets/scenes/table.glb`.
- Expected outcome: No geodesic underfloor; the geometry reads as one physical council table matching the references.

### Step 3 — Remove scene clutter and sky
- [x] Action: Identify and remove/hide the oversized yellow ring from the authored chamber export and enforce black clear/background color while retaining local lighting on assets.
- Files touched: Blender chamber copy/export and environment code.
- Expected outcome: Black above/behind every lit composition with no giant yellow loop.

### Step 4 — Verify and install without launching
- [x] Action: Run tests and Blender structural inspection, rebuild/stage the release and shortcuts, verify services/assets/process state without opening the game, update truth docs, commit, push, and leave the tree clean.
- Files touched: Plans/status/proof metadata and installed outputs.
- Expected outcome: Operator receives the explicit launch signal and remains the only visual verifier.

## Verification
- `cargo test`: 14 passed, 0 failed.
- Blender 4.5 re-exported the isolated `uiscene1.codex-temple.blend`; the operator's original was not opened or modified.
- The runtime chamber export omits `Temple_Star_Halo`, `Temple_Ceiling_Ring`, and `Temple_Vault` while retaining the authored walls and lit assets.
- The table export contains only the clean authored table, inlay, support, and `Stargate_Portal`; the raw AI mesh remains preserved outside the runtime export.
