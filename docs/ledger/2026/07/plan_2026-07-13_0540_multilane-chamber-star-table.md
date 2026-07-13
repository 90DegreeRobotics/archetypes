# Plan: Multilane rebuild — chamber, star, table, text — 2026-07-13 05:40

## Status
IN-PROGRESS

## Director's note (why this exists)
The operator is done with incremental polish and low-quality output. Directive, verbatim intent:
- **New assets, designed in the reference style** (ornate, gilded, sacred-geometry) — NOT the Meshy cloud models. Author them in Blender. The chamber should be a **circular chamber with arches, possibly torch-lit**.
- **The star is awful** ("1980s wireframe"). Replace it with a **solid object, glowing slightly**. Just try a solid.
- **The text: just don't show it** for now.
- **Multilane build, directed by the operator's agents (Claude + Codex).** This plan must leave **zero ambiguity** for any lane. Each lane owns disjoint files so lanes run in parallel without collision.

The Meshy models are NOT on disk and are explicitly abandoned. Everything below is authored fresh.

## FROZEN SCENE CONTRACT (all lanes obey; do not change without updating this file)
The engine binds gameplay to exact node names. Any Blender export MUST preserve these or the build breaks:
- **Council vessels (7):** nodes named exactly `Architect`, `Sentinel`, `Jester`, `Mentor`, `Explorer`, `Oracle`, `Empath`. Bound in `crates/engine/src/chamber/spheres.rs`.
- **Panels:** each vessel has a child `<Archetype>_PanelSpinner` with children `<Archetype>_Icon_Panel` and `<Archetype>_Portrait_Panel`. Bound in `panels.rs`. (Portrait face is the visible one; do not rename.)
- **Camera:** a node named `Witness_Camera` (its transform is the establishing frame). Read in `camera.rs`.
- **Table:** the table scene root spawns under engine Name `PortalTable`; the animated disc node is named exactly `Stargate_Portal`. Bound in `portal.rs` / `camera.rs` / `ritual.rs`.
- **Star:** the authored wireframe star nodes are `Star_Tetra_A` / `Star_Tetra_B`. **Lane A removes these from the runtime look** (hidden + replaced by an engine solid). Lane B must NOT depend on them and should simply omit/leave them; Lane A hides anything named `Star_Tetra*` or `Merkaba*`.
- **Runtime GLB paths (unchanged):** chamber = `assets/scenes/uiscene1.glb`, table = `assets/scenes/table.glb`. Engine loads both in `chamber/mod.rs::load_authoritative_chamber`.

## Lane ownership (disjoint — parallel-safe)
| Lane | Owner | Files it may touch | Depends on |
|------|-------|--------------------|-----------|
| **A — Engine visual** | **Claude (me)** | `crates/engine/src/**` only | nothing (unblocked) |
| **B — Council chamber asset** | **Codex** | `scripts/author_council_chamber.py` (new), `assets/scenes/uiscene1.glb` (regenerated), `assets/textures/chamber/**` (new) | Blender 4.5 |
| **C — Council table asset** | **Codex** | `scripts/prepare_table.py` (rewrite), `assets/scenes/table.glb` (regenerated), `assets/textures/table/**` | Blender 4.5 |

No two lanes touch the same file. Engine (A) never edits a `.py`/`.glb`; asset lanes (B/C) never edit `.rs`. The node-name contract above is the only coupling, and it is frozen.

---

## LANE A — Engine visual (Claude)
### A1 — Replace the wireframe star with a solid glowing merkaba
- [x] New `crates/engine/src/chamber/star.rs` (`StarPlugin`):
  - Every frame, force-hide any scene node whose `Name` contains `Star_Tetra` or `Merkaba` (kills the orange wireframe permanently).
  - Once the 7 vessels are bound, spawn ONE solid stellated-octahedron (star-tetrahedron) mesh, built in code, centred on the vessels' centroid and sized so its points reach toward the vessel shell. Emissive glowing crystal material (`cull_mode=None`, translucent-solid, cyan/white glow). Static (no rotation — camera-only motion law).
  - Gate the solid's visibility to the star-reveal states only: `Deliberating`, `CouncilSpeaking`, `WitnessVerdict`, `ArtifactPending`. Hidden at `Booting/MainMenu/Onboarding/IdleAtTable/ArtifactResult`.
  - Remove the now-dead `Merkaba_` handling from `camera.rs::gate_star_visibility` so the two systems don't fight.
- Register `StarPlugin` in `chamber/mod.rs`.
- Verify on a rebuilt binary + capture that the star is a solid glowing object, not wireframe.

### A2 — Hide the text ("just don't show it")
- [x] In `ritual.rs`: the transcript drawer now shows ONLY in `Onboarding`/`IdleAtTable` (where typing is required); it is hidden in every watching state. The floating `SpeakerBubble` and the small status/hint line are suppressed entirely. Verified on screen — the deliberation/verdict scene is clean of text overlays.

### A3 — Land it
- [x] `cargo test --workspace`: 18 passed, 0 failed. `install_shortcut.ps1` release rebuild + shortcut re-stage in progress; commit + push follows.

---

## LANE B — Council chamber (Codex). SELF-CONTAINED BRIEF
**Goal:** Replace the boxy temple with a **circular sacred chamber**: a ring of arches on columns, torch-lit, gilded, in the reference style (deep navy + brass/gold + warm torch glow). It must still enclose the existing council (7 vessels + star region + central dais) and preserve the FROZEN SCENE CONTRACT above.

**File to create:** `scripts/author_council_chamber.py` (a headless Blender authoring script, same pattern as the existing `scripts/author_temple_overhaul.py`).

**Exact requirements:**
1. Start from the current authoritative working blend that already contains the 7 named vessels, their `_PanelSpinner` panels, `Witness_Camera`, and `Star_Tetra_A/B`. DO NOT rename or move those nodes. (If working from scratch, import the vessels/panels/camera from the existing `assets/scenes/uiscene1.glb` and keep names byte-identical.)
2. DELETE the old boxy temple shell nodes (`Temple_*`, `Council_Dais*` may be kept or replaced) and author a NEW shell:
   - A **circular floor** (radius ~14 units) of dark polished stone with a subtle gilded inlay ring.
   - **8–12 arches** evenly spaced around the ring, each an extruded arch (two columns + a semicircular top), brass/gold trim over dark stone. Columns ~6 units tall.
   - A **low torch** at each column: a small emissive cone/flame mesh (warm orange, emissive strength high) plus a real Blender POINT light (energy ~150, warm color 1.0/0.55/0.2) so the chamber reads as torch-lit. These lights ARE exported (`export_lights=True`).
   - Keep the ceiling OPEN (no vault) so the starfield skybox shows above — do not add a solid dome.
   - A central **dais** (low cylinder, ~4 unit radius) under the star region, dark stone with a warm gold rim.
3. Materials: PBR, metallic gold (`metallic 0.9, roughness 0.25`) for trim, dark stone (`base 0.02–0.05, roughness 0.7`) for structure, strong emissive for torch flames.
4. Export to BOTH the working blend (save) and `assets/scenes/uiscene1.glb` via `bpy.ops.export_scene.gltf(..., export_format="GLB", export_lights=True, export_cameras=True, export_apply=True, export_yup=True)`.
5. **Do not exceed ~150k triangles total** (arches are cheap — use low-segment cylinders/curves). This shares the GPU with the engine.

**Verification (Codex must do before declaring done):**
- Re-import the exported GLB headless and assert all 7 vessel names, all `_PanelSpinner`/`_Portrait_Panel` names, `Witness_Camera`, and at least 8 POINT lights are present. Print the counts.
- The engine is NOT yours to run; Lane A owns the on-screen check. Hand off the GLB + the import-assertion output.

**Must NOT:** touch any `.rs` file; rename any contract node; add a solid ceiling; blow the triangle budget.

---

## LANE C — Council table (Codex). SELF-CONTAINED BRIEF
**Goal:** Replace the thin, barely-visible procedural table with an ornate **Flower-of-Life astrolabe table** in the reference style: a circular top with a **gilded rim carrying engraved glyphs**, a **plasma-etched Flower-of-Life geometric grid** inlaid on a dark surface, an **ornate brass + dark-steel splayed leg assembly**, and the existing living **`Stargate_Portal`** disc at the centre (keep it — the operator already approved the portal).

**File to rewrite:** `scripts/prepare_table.py` (it already builds `assets/scenes/table.glb` and the `Stargate_Portal`; keep the portal generation and its texture, upgrade everything else).

**Exact requirements:**
1. Keep the node named exactly `Stargate_Portal` and its emissive vortex material/texture unchanged (operator-approved). Keep the export root behaviour so the engine still spawns it under `PortalTable`.
2. Circular top (~1.0 unit radius in table-local space; engine scales it ×5): dark polished surface. Inlay a **Flower-of-Life pattern** as thin emissive cyan geometry (overlapping circles), contained within the rim — brighter and denser than the current sparse rings so it reads clearly in the dark chamber.
3. **Gilded rim** (torus, `metallic 0.92 roughness 0.2`) noticeably thicker than the current thin rim, with a ring of small engraved-glyph geometry (simple extruded notches are fine) so the rim reads ornate, not a plain hoop.
4. **Leg assembly:** an ornate central pedestal + 4 splayed brass legs with dark-steel feet, visibly substantial (the current legs are too thin/dim). Add a subtle emissive under-glow ring so the table doesn't vanish into the dark.
5. Export `assets/scenes/table.glb` (same export call already in the script).

**Verification (Codex):** re-import headless; assert `Stargate_Portal` still present; assert triangle count < ~200k; print bounding box so Lane A can confirm scale. Hand off the GLB + assertion output.

**Must NOT:** touch any `.rs`; rename `Stargate_Portal`; remove the portal; touch `uiscene1.glb`.

---

## Integration
Lane A codes against the frozen contract, so when Codex drops the new `uiscene1.glb` / `table.glb` into `assets/scenes/`, the engine picks them up with no code change. Lane A's final release rebuild bundles whatever assets are present. If B/C land after A, a follow-up `install_shortcut.ps1` re-stages them — no code merge needed.

## Verification
- **Lane A (Claude) — DONE, verified on a rebuilt binary (`ARCHETYPES_CAPTURE=1`):**
  - The orange `Star_Tetra` wireframe is force-hidden in all states; a single engine-authored **solid stellated-octahedron (star tetrahedron)** now sits at the council centre. First pass read as a flat pale cutout (viewed head-on down a symmetry axis + flooded by uniform ambient); fixed by (a) tilting it to a 3/4 orientation so facets foreshorten, and (b) deepening it to a low-roughness metallic sapphire with only a faint inner glow so facets shade by directional light + starfield reflection. The `03_deliberating` capture shows a clean 3D glowing crystal with distinct per-facet shading — no wireframe, no flat cutout.
  - All ritual HUD text is suppressed in the watching states (transcript drawer, floating bubble, status line); the drawer remains only where the Witness types.
  - `cargo test --workspace`: 18 passed, 0 failed. Release rebuilt and the Desktop/Start-Menu shortcut re-staged so the operator's launcher runs this build.
  - Lane A did NOT add bloom; the star glows via emissive + reflection only. Bloom is an available follow-up if the operator wants a stronger glow.
- **Lane B (chamber) / Lane C (table):** briefs issued (`CODEX_LANE_B_*`, `CODEX_LANE_C_*`), pushed to `origin/main`; awaiting Codex.
