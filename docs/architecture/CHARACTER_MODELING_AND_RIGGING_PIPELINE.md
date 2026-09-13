# Character Modeling & Rigging Pipeline

> **Standard operating procedure for authoring, modeling, and rigging the 8 Archetype characters in Blender 4.5 for the Archetypes game engine.**

---

## 1. Directory Architecture & Artifacts

All character authoring assets are organized under `assets/authoring/characters/`:

```
assets/authoring/characters/
├── template/
│   └── character_template.blend              # Master reusable template for any character
├── nebula_jester/
│   ├── reference_front.png                   # Raw concept reference art
│   ├── reference_front_alpha.png             # Chroma-keyed transparent alpha reference
│   └── nebula_jester_modeling.blend          # Active modeling and rigging workspace
scripts/
├── build_character_template.py               # Headless generator for character scenes
└── setup_character_scene.py                  # Image processing and chroma-key utility
```

Visual verification proofs:
- Front Orthographic View: [viewport_preview.png](file:///C:/archetypes/artifacts/visual-proof/character_setup/viewport_preview.png)
- 3/4 Perspective Proof: [perspective_preview.png](file:///C:/archetypes/artifacts/visual-proof/character_setup/perspective_preview.png)

---

## 2. Scene Organization & Collections

Opening `nebula_jester_modeling.blend` presents 4 curated collections:

| Collection | Contents | Purpose |
| :--- | :--- | :--- |
| `01_Reference` | `REF_Front_Image_Plane`, `REF_Front_Empty` | Calibrated front reference image. Grounded at $Z=0.0\text{m}$, centered at $X=0.0\text{m}$, height $1.95\text{m}$. Sits at $Y=0.04\text{m}$ just behind the mesh. |
| `02_Blockout_Mirrored` | Head, Neck, Horns, Bells, Chest, Abdomen, Pelvis, Arms, Forearms, Hands, Thighs, Shins, Feet | Quad-faced base primitives with active **Mirror Modifiers** (X-axis symmetry, clipping enabled, merge threshold $0.002\text{m}$). Object origins are at world $(0, 0, 0)$. |
| `03_Rig_Armature` | `Rig_Humanoid_Biped` (`Armature_Humanoid_Biped`) | Clean game-ready biped skeleton (`root`, `hips`, `spine`, `chest`, `neck`, `head`, `clavicle.L/R`, `upper_arm.L/R`, `forearm.L/R`, `hand.L/R`, `thigh.L/R`, `shin.L/R`, `foot.L/R`, `toe.L/R`, `horn_base.L/R`, `horn_tip.L/R`) with **X-Axis Mirror Editing** enabled. |
| `04_Guides_Studio` | `Guide_Ground_Ring`, `Camera_Front_Ortho`, Key/Fill/Rim lights | Ground floor circle at $Z=0.0\text{m}$, Front Ortho camera, and balanced studio lighting. |

---

## 3. Modeling Workflow: How to Model over the Reference

### Hotkeys & Viewport Navigation
- **Front Orthographic View**: Press `Numpad 1` (or click the $-Y$ axis bubble on the top-right navigation gizmo).
- **Toggle X-Ray Mode**: Press `Alt + Z`. In Solid mode, this makes both your 3D mesh wireframe/clay and the reference image underneath visible simultaneously.
- **Toggle Wireframe Mode**: Press `Shift + Z` or press `Z` -> `Wireframe`.
- **Edit Mode / Object Mode**: Press `Tab`.

### Step-by-Step Modeling Guide

1. **Select a Blockout Part**:
   - In Object Mode (`Tab`), click on an area you want to shape (e.g. `Blockout_Chest` or `Blockout_Head`).
   - Press `Tab` to enter **Edit Mode**. Notice you only need to edit the left side (or right side)—the `Mirror` modifier handles the opposite side automatically.
2. **Move Vertices to Match the Silhouette**:
   - Press `1` for Vertex select, `2` for Edge select, `3` for Face select.
   - Press `A` to select all, or box-select (`B`) vertices.
   - Press `G` to grab/move (press `X` or `Z` to constrain to an axis).
   - Press `S` to scale (press `Shift + Y` to scale in $X$ and $Z$ while keeping depth constant).
3. **Add Detail with Extrude and Loop Cuts**:
   - **Loop Cut (`Ctrl + R`)**: Hover over a cylinder or box and press `Ctrl + R`, then scroll the mouse wheel to add subdivisions along muscles or armor seams.
   - **Extrude (`E`)**: Select a face (e.g. shoulder epaulet, chest crest, or boot sole) and press `E` to pull out raised armor plating.
   - **Inset (`I`)**: Select a face and press `I` to create an inward border for panel lines and reactor cores.
4. **Shaping the Jester Horns**:
   - Click `Blockout_Jester_Horns` and press `Tab`.
   - Each ring of vertices can be selected with `Alt + Left Click`.
   - Press `G` to reposition rings along the horn curve and `S` to adjust thickness as it tapers toward the bell.
5. **Carving Armor Seams**:
   - Once the base volume is sculpted, use `Bevel` (`Ctrl + B`) on edges to create mechanical chamfers.
   - Extrude panel insets (`E` then `S` slightly inward) to form the segmented plates of the android body.

---

## 4. Replicating for the Other 7 Characters

To create the next character in the roster:

1. **Copy the Template**:
   ```pwsh
   Copy-Item "assets\authoring\characters\template\character_template.blend" "assets\authoring\characters\<character_name>\<character_name>_modeling.blend"
   ```
2. **Save the New Concept Reference**:
   - Place the front concept image at `assets/authoring/characters/<character_name>/reference_front.png`.
   - Run `python scripts/setup_character_scene.py` (or load directly in Blender).
3. **Swap the Reference Image in Blender**:
   - In Blender, select `REF_Front_Image_Plane` and `REF_Front_Empty`.
   - In the Material / Image tab, replace the image file with the new character's reference.
4. **Adjust Blockout Primitives and Armature**:
   - In Front Orthographic view (`Numpad 1`), select the armature and enter Edit Mode (`Tab`).
   - With `X-Axis Mirror` enabled, move the joints (shoulders, elbows, knees, ankles) to align with the new character's silhouette.
   - Move/scale the blockout meshes to fit the new character's body type.
5. **Export to Game**:
   - When modeling is complete, export as `.glb` into `assets/scenes/<character_name>.glb` with:
     - Feet grounded at $Z=0.0\text{m}$.
     - Centered horizontally at $X=0.0\text{m}, Y=0.0\text{m}$.
     - Scale calibrated for player eye level (~$2.85\text{m}$ in Bevy world space).
