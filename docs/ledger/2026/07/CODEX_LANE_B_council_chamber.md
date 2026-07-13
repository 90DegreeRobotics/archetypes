# CODEX LANE B — Council Chamber (circular, arched, torch-lit)

**Owner: Codex.  Status: READY TO START (unblocked).  Director: Claude (Lane A).**
**Cold-start brief — everything you need is in this file. Do not wait on anyone.**

---

## What you are building
Replace the boxy stone temple with a **circular sacred chamber**: a ring of **arches on columns**, **torch-lit**, gilded brass over dark stone, in a deep-navy sacred-geometry style. It must still enclose the existing council (7 vessels + a central dais) and keep the ceiling OPEN so the starfield shows above.

You are authoring a Blender scene and exporting it to **`assets/scenes/uiscene1.glb`** (the runtime chamber GLB the engine already loads).

## Your file ownership (touch ONLY these)
- CREATE: `scripts/author_council_chamber.py` (headless Blender authoring script; model it on the existing `scripts/author_temple_overhaul.py` and `scripts/prepare_blender_chamber.py`).
- REGENERATE: `assets/scenes/uiscene1.glb`.
- CREATE (if needed): `assets/textures/chamber/**`.
- **NEVER touch any `.rs` file, `scripts/prepare_table.py`, or `assets/scenes/table.glb`.** Those belong to other lanes.

## FROZEN SCENE CONTRACT — the export MUST preserve these node names exactly
The engine binds gameplay by exact node name. If you rename/drop any of these, the build breaks:
- **7 vessels:** `Architect`, `Sentinel`, `Jester`, `Mentor`, `Explorer`, `Oracle`, `Empath`.
- **Panels (children of each vessel):** `<Archetype>_PanelSpinner` with children `<Archetype>_Icon_Panel` and `<Archetype>_Portrait_Panel`.
- **Camera:** `Witness_Camera` (a real camera node; its transform is the establishing frame).
- Keep whatever `Witness` node exists.
- **The star:** the old `Star_Tetra_A` / `Star_Tetra_B` nodes are being replaced by an engine-side solid (Lane A). **Delete them from the chamber, or leave them — Lane A force-hides anything named `Star_Tetra*`/`Merkaba*`.** Do not build the chamber to depend on them.

The simplest safe path: **start from the current working blend / import the existing `assets/scenes/uiscene1.glb`, keep the vessels + panels + `Witness_Camera` byte-identical, delete only the temple shell (`Temple_*`) and the star, and author the new circular chamber around them.**

## Build spec (unambiguous)
1. **Circular floor:** radius ~14 units, dark polished stone (`base 0.02–0.05, roughness 0.7, metallic 0`), with one subtle gilded inlay ring (thin gold torus) near the rim.
2. **Arch ring:** **10 arches** evenly spaced on a circle of radius ~12. Each arch = 2 columns (~6 units tall, ~0.5 radius, low-segment cylinders) + a semicircular top (a half-torus or an arc-swept profile). Brass/gold trim (`metallic 0.9, roughness 0.25`) over dark stone.
3. **Torches (this is what makes it torch-lit):** at each column top, place:
   - a small emissive flame mesh (a low cone, warm orange, high emissive strength), AND
   - a real Blender **POINT light**, energy ~150, warm color RGB ≈ (1.0, 0.55, 0.2), positioned just above the flame.
   These lights ARE exported (`export_lights=True`). 10 columns → at least 10 point lights.
4. **Open ceiling:** NO vault, NO dome. Leave the top open so the engine's starfield skybox shows above the arches.
5. **Central dais:** a low cylinder (~4 unit radius, ~0.6 tall) under the star region, dark stone with a warm gold rim torus. Keep it centred at the world origin region where the vessels ring sits.
6. **Triangle budget:** keep the whole chamber **under ~150k triangles** (use low-segment cylinders/tori; arches are cheap). It shares the GPU with the live engine.
7. **Materials:** PBR only. Gold/brass trim `metallic 0.9 roughness 0.25`; dark stone `roughness ~0.7`; torch flame strong emissive warm orange.

## Export (exact call)
```python
bpy.ops.export_scene.gltf(
    filepath=str(export_path),            # C:/archetypes/assets/scenes/uiscene1.glb
    export_format="GLB",
    use_selection=False,
    export_cameras=True,
    export_lights=True,
    export_apply=True,
    export_yup=True,
)
```
Run headless: `blender --background --python scripts/author_council_chamber.py -- <args>`.
Blender is at `C:\Program Files\Blender Foundation\Blender 4.5\blender.exe`.

## Verification you MUST do before declaring done
Re-import the exported `uiscene1.glb` headless and assert/print:
- all 7 vessel names present,
- every `<Archetype>_PanelSpinner`, `_Icon_Panel`, `_Portrait_Panel` present,
- `Witness_Camera` present and is a CAMERA,
- **at least 10 POINT lights** present,
- total triangle count (must be < ~150k).
Print the counts. **You do NOT run the game** — that is Lane A's on-screen check. Hand off the GLB + your import-assertion output + the new script.

## Hard rules (repo doctrine)
- Plan doc first: create `docs/ledger/2026/07/plan_<date>_council-chamber-asset.md` before you touch anything.
- Work only on `main`. Never delete files; `git mv` if renaming.
- Do not present a stub as real. If you can't preserve the contract, STOP and report — do not ship a broken chamber.
- Commit + push to `origin/main` when the GLB re-imports clean with the assertions passing.
