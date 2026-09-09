# Manifestation Pipeline Truth Audit — 2026-09-09

## Verdict

The current Manifestation Pedestal is **not** a Chronos2 generation pipeline. It is a local deterministic Blender script presented as one. Do not describe it as player prompt → Chronos2 → generated GLB → pedestal.

## Reachable Archetypes route

1. `crates/engine/src/modes/inner_chambers/manifestation.rs` accepts the player's text at the pedestal. An empty prompt is replaced with `buddha`.
2. It begins the floating-hourglass animation and launches `C:\Program Files\Blender Foundation\Blender 4.5\blender.exe -b -P scripts/manifest_artifact.py -- --prompt <text> --output <installed asset path>`.
3. `scripts/manifest_artifact.py` checks prompt word lists, then builds fixed Blender primitives for chalice, dagger, astrolabe, crown, tree, Buddha, or a generic fallback.
4. The worker only tests process exit plus existence of `manifested_artifact.glb`; it copies that file into the repo asset tree and reports `Render complete`.
5. Bevy loads `scenes/manifested_artifact.glb#Scene0` at a fixed pedestal location and adds turntable rotation.

The route does not invoke `C:\chronos2\target\release\chronos.exe`, does not submit to Chronos Director or Sentinel, carries no Chronos job identity or receipt, and does not read a Chronos bundle. The prompt word lists and Blender primitive branches are recipe behavior and violate the no-recipe law.

## Wait / failure-state truth

The floating hourglass does rotate, bob, and pulse while the local Blender child process has not returned. The visible timer is local elapsed frame time. It is not progress from Chronos2, has no timeout or cancellation, and does not know whether Comfy, TripoSR, or BlenderMCP are actually working. The red X is accurate only for the local process error; it cannot report an upstream Chronos job failure because there is no upstream job.

## Chronos2 contract observed

The current Chronos2 Create code constructs `chronos.exe first-light --prompt <text> --out-dir <unique bundle> --geometry-forge --void` for Object mode. A current successful bundle at `C:\chronos2\out\first_light\20260909_165809_operator_reference` contains `scene.blend`, provenance records, renders, and `engine_mesh\0\mesh.obj`; it does not provide the GLB handoff that Archetypes claims to consume. No local listener was live on ports 7777, 8000, 9876, or 11434 during this audit.

## Required closure before the pedestal can be real

1. Chronos2 must expose a Sentinel-authorized Object-mode handoff that produces a validated, grounded GLB and a machine-readable receipt containing the source prompt, generated artifact path, and failure state. That route must not select objects from noun lists, recipes, or fallback primitives.
2. Archetypes must launch that exact supported Chronos2 command, give each request its own output directory, read only the returned receipt, validate the returned GLB before staging it, and preserve the receipt with the in-game manifestation.
3. The wait UI must receive real stages/job state from that request; lack of updates must expire to a visible timeout. Cancellation must terminate or cancel the actual Chronos job, not just hide UI.
4. A live user-facing witness must run from the pinned Archetypes launcher with Chronos2 dependencies ready, produce a new GLB from a novel prompt, and show that exact returned mesh on the pedestal.

## Floor change in this work unit

`crates/engine/src/modes/inner_chambers/world.rs` now makes the actual runtime floor from a deterministic basalt albedo plus a linear tangent-space normal map. The `StandardMaterial` binds that map through `normal_map_texture`; it is lit PBR surface relief, not an emissive visual substitute. `cargo test --workspace` passed (96 tests) before desktop staging.
