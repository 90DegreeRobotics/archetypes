# Plan: Nebula Jester posable avatar — 2026-09-13 07:00

## Status
IN-PROGRESS

## Goal
Prove the full chain from the existing static `assets/scenes/nebula_jester.glb` to a skinned,
animated Jester playing an idle in the installed game. One character, automated steps first,
so the operator can judge auto-rig quality before the other seven are committed to.

## Evidence
- Every in-game character GLB has 0 skins, 0 joints, 0 animations.
- `nebula_jester.glb`: 1 mesh, 209k verts / 357k tris, 67k non-manifold edges, 1 UV set,
  3 x 2048 textures, 1.90m tall, feet at Z=0, facing -Y. Too dense and too broken for
  Blender bone-heat weights directly.
- Antigravity's `nebula_jester_modeling.blend` has a 26-bone armature bound to nothing and
  box/tube blockouts; no weights, no actions, no export.

## Decision
Weights are solved on a voxel-remeshed watertight proxy (bone heat works on it), then
transferred by nearest-face data transfer onto a decimated (~40k tri) copy of the original
that keeps its UVs and textures. Joints are placed from measured mesh cross-sections, not
eyeballed. Only generic humanoid logic; nothing keyed to this character's name.

## Steps

### Step 1 — Rig script
- [x] `scripts/rig_character_avatar.py`: import GLB, decimate, build armature from slices,
  proxy weights, transfer, author `Idle` and `Wave` actions, export skinned GLB to
  `assets/scenes/nebula_jester_rigged.glb`. Render posed proof frames.

### Step 2 — Engine playback
- [x] Load the rigged GLB with its animation clips via `AnimationGraph`, play `Idle` looped on
  the Council host Jester. Unit test the clip/graph wiring.

### Step 3 — Verify in the installed app
- [ ] `cargo test --workspace`, install, walk capture near the Jester, pixel-diff frames to
  prove motion, send frames to the operator. Operator is the visual judge.

### Step 4 — Land
- [x] Commit on `main`, push. Plan stays IN-PROGRESS until the operator judges the frames.

## Execution record
- Rig script: 357,494 -> 44,998 tris; 22 bones (21 deform); proxy 24,624 verts, 0 unweighted;
  game mesh 22,428 verts, 0 unweighted. GLB 5.6 MB, 1 skin / 21 joints, clips `Idle`, `Wave`.
- `cargo test --workspace`: 287 engine + 19 launcher + 5 = 311 passed.
- Installed engine SHA256 `C7D56295F7216696B88871556452A1EDAFF33FDE5FE387D4717C947CFC99936B`;
  Taskbar target verified.
- Installed walk capture: frame 02 shows the Jester arms-down (mesh rest pose is T-pose, so the
  clip is being applied). Frames 02 and 02b, 0.7s apart with no input: 1.32% of pixels changed,
  all inside the Jester's bounding box (x 2086..2494, y 565..2018); rest of frame identical.
  No `avatar:` warnings, no panics in engine stderr.
- Visual quality verdict: pending operator.
- Step 3 capture beat 03 no longer frames the Jester in the rectangular hall (it shoots a wall);
  route beats still reference the retired rotunda/ascent layout.
