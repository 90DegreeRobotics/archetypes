# Plan: Blender Scene Authority — 2026-07-11 21:10

## Status
COMPLETED

## Goal
Replace the procedural Council Chamber imitation with the Director's `uiscene1.glb` as the authoritative visual scene, bind chamber behavior to its named nodes, retain a Bevy-owned camera because the export contains no camera or animation tracks, verify the runtime contract with tests, reconcile documentation, and publish the completed unit to `origin/main`.

## Steps
### Step 1 — Install the authoritative scene asset
- [x] Action: Prepare the Director's source `.blend` in Blender 5.0 and export a camera-free runtime GLB into the repository's central scene asset directory; retain the camera-bearing interchange GLB on Desktop.
- Files touched: `assets/scenes/uiscene1.glb`.
- Expected outcome: The authoritative Blender geometry, transforms, materials, and lights are available through Bevy's asset server without competing imported cameras.

### Step 2 — Replace procedural geometry with scene loading
- [x] Action: Load `uiscene1.glb#Scene0`, remove procedural chamber geometry, and bind merkaba and sphere behavior to the exported node names and authored transforms.
- Files touched: `crates/engine/src/chamber/mod.rs`, `crates/engine/src/chamber/merkaba.rs`, `crates/engine/src/chamber/spheres.rs`, `crates/engine/src/chamber/portal.rs`, `crates/engine/src/chamber/camera.rs`.
- Expected outcome: The live game uses the Blender scene rather than torus, cylinder, plane, and flat-orbit substitutes.

### Step 3 — Verify named-node and gameplay contracts
- [x] Action: Add tests for authoritative node mapping and run formatting plus the mandatory `cargo test` gate.
- Files touched: Chamber source files.
- Expected outcome: Exported names map deterministically to gameplay identities and all workspace tests pass.

### Step 4 — Reconcile truth surfaces and publish
- [x] Action: Update `README.md` and `STATUS.md`, complete this plan, commit, and push directly to `origin/main`.
- Files touched: `README.md`, `STATUS.md`, this plan.
- Expected outcome: Documentation matches the GLB-backed runtime, the working tree is clean, and `origin/main` contains the verified integration.
