# Plan: Star Tetrahedron Correction — 2026-07-12 04:35

## Status
IN-PROGRESS

## Origin
Direct Director correction. The central object is a **star tetrahedron** — one fixed,
solid form of two interlocked tetrahedra — **not a merkaba**. A merkaba implies an
active counter-rotating light-vehicle; that framing seeded a counter-spin
(`Merkaba_Diamond +0.16` / `Merkaba_Emerald -0.11`) that was never called for and that
actively destroys the one profound view (the fixed star resolving into the 2D hexagram
along a single axis). The corrected model:

- The star tetrahedron is **fixed and static**. It does not rotate. No counter-rotation.
- The seven glass spheres are **locked to the star's tips**; the Witness sphere is the
  fixed center. Nothing on the star moves.
- The **camera is the only moving element.** It flies to the alignment where the
  selected archetype's sphere faces the Witness and the star resolves to the hexagram.
- Lock every coordinate once, fly the camera — "more with less," no per-object math.
- Remove **all** "merkaba" verbiage and labeling from the project surfaces.

OUT OF SCOPE here (separate design discussion, not built yet): the temple enclosure
(choosing a north, hanging the star in the sky, closing the open-edge void into an
ancient-temple interior with a circular table whose center is the stargate/prompt
wavefield, camera looking up) and the real camera-alignment flight itself.

## Steps
### Step 1 — Static star, locked spheres (behavior)
- [ ] Delete the rotation system (`chamber/merkaba.rs`) and its plugin registration.
  The star renders statically from the authored GLB.
- [ ] `chamber/spheres.rs`: keep identity binding, remove all sphere movement
  (including the just-added move-to-center). Spheres stay locked at their authored tips.
- Expected: star and spheres are completely fixed; only camera/theme systems animate.

### Step 2 — Purge "merkaba" from code, script, and living docs
- [ ] Code/plugins (`mod.rs`), Blender script comment, `README.md`, `STATUS.md`,
  `docs/architecture/COUNCIL_WORLD_ENGINE.md`, `docs/architecture/COUNCIL_CHAMBER_DIRECTION.md`,
  and this session's plan docs → "star tetrahedron". Correct the motion-model
  statements (continuous motion / sphere-glides-to-center) to the fixed-star,
  camera-moves model.
- Expected: no living surface calls it a merkaba or prescribes its motion.

### Step 3 — Verify and publish
- [ ] `cargo test --workspace` + build green; commit; push `main`.

## Boundary items to raise with the Director (not silently changed)
- The **Blender `.blend`/`.glb` node names** `Merkaba_Diamond` / `Merkaba_Emerald` are
  authored in the Director's scene; the export script must match them, so they cannot be
  renamed from code without breaking the pipeline. Rename them in Blender, then the
  script's `STATIC_CHAMBER_NODES` list is updated to match.
- **Historical ledger/handoff docs** and the **`Archetypes of AURA final.txt`** manuscript
  also contain the word; those are frozen records / source material — left as-is pending
  the Director's call.
