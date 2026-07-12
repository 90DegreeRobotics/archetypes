# Plan: Portal Table — Materials, Stargate Effect, Chamber Integration — 2026-07-12 16:52

## Status
COMPLETED

## Origin
Director dropped a new asset `C:\Users\m\Desktop\table.glb` — the golden clock-astrolabe
table from the reference video. It is a raw AI export: one fused mesh, **1.42M triangles /
55 MB**, a single baked-albedo material (gold + engraved rings + blue portal all painted in),
no metallic, no emission, no animation. Director wants: materials + a stargate portal effect,
**and** integration into the council chamber (table under the star, prompt input on the portal,
camera sweep table→star).

## Steps
### 1 — Blender asset prep (`scripts/prepare_table.py`) → `assets/scenes/table.glb`
- [x] Completed.
- Import; **decimate** to ~120k tris (ratio ~0.08) — mandatory for real-time on the 3060.
- Material: keep the baked albedo; set metallic ~0.5 / roughness ~0.4 so the gold reads as
  struck metal; add an **emissive map** (`table_emissive.png`, the blue/cyan regions of the
  albedo) so the engraved geometry and portal glow.
- Add a **`Stargate_Portal`** disc at the table's centre-top with an emissive vortex texture
  (`portal_vortex.png`) — the surface the engine will animate.
- Export a lean GLB (`export_apply=True`, `export_yup=True`).

### 2 — Engine integration
- [x] Completed.
- Load `table.glb` as a second scene root, placed at the chamber floor centre under the star,
  scaled to the room (table is ~1.9 u; scale ~4-5x).
- **Stargate animation:** rotate the `Stargate_Portal` disc (two-layer swirl / spin) + pulse
  emission — a small `portal.rs` system (the currently-empty PortalPlugin gets a real body).
- **Prompt on the portal:** the Onboarding/IdleAtTable prompt text projects at the portal's
  screen position (the intent surface), not a screen-corner box.
- **Camera sweep:** a "table pose" (camera low, looking down at the portal) for
  Booting/Onboarding/IdleAtTable; on submit it sweeps up to the star/establishing pose for
  Deliberating/CouncilSpeaking. Reuse `camera.rs` (add the table pose + per-state target).

### 3 — Verify + publish
- [x] `cargo test` passed (11 tests), `cargo build -p engine` passed, and deterministic capture produced the table and post-submit council frames.
- `cargo build -p engine`; capture run showing the materialed table with the glowing animated
  portal at the table view, the sweep to the star on submit, and the council. Commit + push.

## Facts / reuse
- Table bbox (glTF Y-up): X/Z ±0.95, Y [-0.286, 0.282]; diameter ~1.9, top at Y≈0.28; portal
  at centre (0, ~0.29, 0).
- `scripts/make_avatars.py` pattern for texture generation; `camera.rs` per-state targets;
  `chamber/portal.rs` (empty `PortalPlugin`) is the home for the stargate animation.
- Generated emissive/vortex inputs are preserved in `assets/textures/table/`; the source export keeps its albedo embedded.

## Outcome

The live engine loads the 10.6 MB prepared table, decodes its embedded JPEG albedo, animates and pulses the authored stargate disc, positions intent text from the portal projection, and changes from the table pose to the council pose after submission. Texture-generation inputs needed by the preparation script are preserved under `assets/textures/table/`. Visual composition remains subject to operator approval.
