# Plan: Starfield sky + reflections (kill the black void) — 2026-07-13 05:20

## Status
IN-PROGRESS

## Goal
The operator confirmed the previous session's ritual/render fixes are live on the desktop launcher, then named the next look-breaker: the enforced **absolute-black void** behind the chamber. "I said black sky and it broke the whole look. We are gonna need stars to give a reflection or something. Black void won't work." Getting the look right is the explicit gate before moving on to new game mechanics. This plan replaces the black void with a procedural starfield that is both visible behind the open temple and reflective on the gilded/glass surfaces.

## Verification of the prior build (operator's first ask)
- `dist/engine.exe` = `2026-07-12 22:45`, identical inode to `target/release/engine.exe`, built from the clean working tree at commit `2219388` (the ritual/render fixes). The Desktop shortcut (`22:45`) → `launcher.exe` → that engine. So the panel/HUD/camera/art-style fixes ARE what the operator is running.
- `dist/launcher.exe` is older (`19:38`) but unchanged this cycle — only engine code changed, and the launcher just supervises + starts the engine. A fresh release rebuild lands at the end of this plan regardless, so the operator is provably on the newest build.

## Root cause of the black void
`crates/engine/src/chamber/interior.rs` hard-codes `CEREMONIAL_VOID = Color::BLACK`, sets it as the `ClearColor`, re-asserts `clear.0 = CEREMONIAL_VOID` every frame in `drive_interior_environment`, and even has a test (`the_render_void_is_absolute_black`) *enforcing* pure black. The earlier `plan_2026-07-12_1945_voice-table-black-void` deliberately removed the temple ceiling/vault from the export, so the chamber is now open to that black background above and behind — which is exactly why it reads as a dead void. There is no skybox or environment map anywhere in the engine (`assets/` has no star/sky/cubemap asset either).

## Approach
Add a self-contained starfield with no external assets (Bevy 0.18 API already confirmed against the crate source):
- Procedurally generate a **star cubemap `Image`** in code (deep-navy base + scattered white/warm/cool star pixels, 6 faces, `TextureViewDimension::Cube`).
- Attach a **`Skybox`** (`bevy::core_pipeline::Skybox`) to the Witness camera → stars visible behind the open temple.
- Attach an **`EnvironmentMapLight`** (`bevy::light::EnvironmentMapLight`) built from the same cubemap → the gold table rim and glass vessels reflect starlight (the "reflection" the operator asked for).
- Lift `CEREMONIAL_VOID` off pure black to a matching deep-space navy so any frame the skybox doesn't cover is seamless, and update the now-obsolete "absolute black" test to the new deep-space intent.

The title/boot screen keeps its black overlay (it covers the 3D view until reveal), so the starfield only appears once the chamber is shown.

## Steps

### Step 1 — New `chamber/sky.rs`: procedural star cubemap + skybox + env map
- [ ] Action: Add `SkyPlugin`. A Startup system builds the star cubemap `Image` and stores its handle in a resource; an `Added<WitnessCamera>` system attaches `Skybox` + `EnvironmentMapLight` to the camera the frame it appears. Register the plugin in `chamber/mod.rs`.
- Files touched: `crates/engine/src/chamber/sky.rs` (new), `crates/engine/src/chamber/mod.rs`.
- Expected outcome: stars render behind the chamber; gold/glass catch starlight.

### Step 2 — Lift the clear color off pure black
- [ ] Action: Change `CEREMONIAL_VOID` in `interior.rs` from `Color::BLACK` to a deep-space navy matching the skybox base, and update the `the_render_void_is_absolute_black` test to the new intent (dark, not pure black).
- Files touched: `crates/engine/src/chamber/interior.rs`.
- Expected outcome: no pure-black backdrop anywhere; test reflects reality.

### Step 3 — Tune and verify on screen
- [ ] Action: `cargo build -p engine`, run `ARCHETYPES_CAPTURE=1`, inspect the table / deliberation / council-speaking frames. Tune `SKYBOX_BRIGHTNESS` and `ENV_INTENSITY` until the stars read clearly and the gold/glass visibly catch light without washing out. Confirm no skybox/env-map validation crash in the log (fall back to skybox-only if the non-mipped specular map is rejected).
- Files touched: `crates/engine/src/chamber/sky.rs` (constants).
- Expected outcome: a real captured frame shows a starfield backdrop with reflective surfaces, no black void.

### Step 4 — Rebuild release, reinstall shortcut, docs, commit/push
- [ ] Action: `cargo test`, `scripts/install_shortcut.ps1` to rebuild release + re-stage dist + shortcut, update STATUS.md, commit and push. Confirm the operator is on the new version.
- Files touched: `STATUS.md`, dist/ + shortcut (generated).
- Expected outcome: the desktop launch shows the starfield; tree clean on origin/main.

## Out of scope (flagged for the operator)
The ornate gilded council-chamber and the Flower-of-Life table from the operator's reference images (the Meshy "Atrium of the Seven Portals" / "Astral Convergence Council Table" models) are a separate, larger asset-pipeline task — Meshy export (~5.8M faces) → decimation → Blender prep → `.glb`, following the existing `scripts/prepare_table.py` pattern. That is the natural next chunk once the starfield look is approved; this plan deliberately does the lighting/void fix first because it's the specific thing the operator said "broke the whole look."

## Verification
(to be filled in as steps complete)
