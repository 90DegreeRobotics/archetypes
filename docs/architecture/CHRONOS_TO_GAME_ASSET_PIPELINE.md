# ChronoSophia to In-Game Asset Pipeline & Exhibition Doctrine

**Canonical Reference for the Archetypes Repository**  
*The "Gold Standard" for converting Chronos creations into stunning in-game assets*

---

## 1. Core Philosophy & Design Intent

ChronoSophia is the generative forge where 3D geometry, TripoSR meshes, and AI-carved forms are born. Archetypes is the living, interactive world where players encounter, examine, and interact with these creations.

To make an imported object feel truly premium, state-of-the-art, and awe-inspiring, it is never enough to simply dump a raw mesh into a scene. Every Chronos object imported into Archetypes must obey **The Five Laws of Artifact Exhibition**:

1. **Grounded & Centered Geometry:** Objects must have their local origin centered horizontally ($X=0, Z=0$) and their base resting flush at $Y=0$ so they never float haphazardly or clip through surfaces.
2. **Material & Vertex Color Preservation:** Chronos meshes generate rich vertex colors (e.g. `Color` or `carve` layers). These must be properly bound to PBR shader nodes (`ShaderNodeVertexColor`) during export so they render with full vibrancy in Bevy.
3. **Architectural Pedestals:** Exhibition objects sit atop bespoke, multi-tiered architectural plinths (dark stone, polished obsidian, burnished gold trim, and velvet display cushions).
4. **Dual-Lighting Doctrine:** Every exhibition object requires two complementary light sources:
   - A crisp overhead spotlight illuminating the top contours, and
   - A soft, thematic underglow point light nestled at the base highlighting silhouettes and casting ambient color.
5. **Living Turntable Motion:** Every exhibited artifact rotates smoothly around its Y-axis at a gentle pace ($\sim 15^\circ–20^\circ/\text{s}$), allowing 360-degree inspection from any flight angle.

---

## 2. ChronoSophia Asset Discovery

ChronoSophia stores generated 3D creations and witness outputs in `C:\chronos2\out\`:

```
C:\chronos2\out\
  ├── object_triposr_only_<timestamp>\   # TripoSR image-to-3D outputs
  │     ├── scene.blend                   # Full Blender scene
  │     ├── engine_mesh\0\mesh.obj        # Raw extracted OBJ mesh
  │     ├── reference_input.png           # Source image prompt
  │     └── human_prompt.txt              # User text prompt (e.g. "ceramic teapot")
  └── universal_object_witness\<name>\    # Universal object witness runs
        ├── scene.blend                   # Refined mesh with vertex colors
        ├── renders\hero.png              # Reference turntable render
        └── human_prompt.txt              # Subject description
```

### Discovery Rule:
Inspect `C:\chronos2\out\` sorted by `LastWriteTime` to identify the most recent creations. Check `human_prompt.txt` to understand the artifact's lore, name, and aesthetic theme.

---

## 3. Headless Blender Conversion Pipeline

All conversions run automatically via Blender 4.5 in headless background mode:
```powershell
& "C:\Program Files\Blender Foundation\Blender 4.5\blender.exe" --background --python scripts\export_chronos_assets.py
```

### 3.1 Script Implementation (`scripts/export_chronos_assets.py`)
Below is the canonical Blender Python pipeline that handles mesh cleanup, centering, grounding, scaling, vertex color wiring, and GLTF/GLB export:

```python
import os
import bpy
from mathutils import Vector

def process_and_export(blend_path, mesh_name, output_glb, target_max_dim=1.4):
    # 1. Open the source Chronos .blend file
    bpy.ops.wm.open_mainfile(filepath=blend_path)
    
    # 2. Locate target mesh object
    target_obj = None
    for obj in bpy.data.objects:
        if obj.type == 'MESH' and (mesh_name in obj.name or obj.name == mesh_name):
            target_obj = obj
            break
            
    if not target_obj:
        meshes = [o for o in bpy.data.objects if o.type == 'MESH']
        if meshes:
            target_obj = max(meshes, key=lambda m: len(m.data.vertices))
        else:
            raise RuntimeError(f"No mesh found in {blend_path}")

    # 3. Strip away unwanted lights and cameras from Chronos scene
    for o in [o for o in bpy.data.objects if o != target_obj]:
        bpy.data.objects.remove(o, do_unlink=True)
        
    bpy.ops.object.select_all(action='DESELECT')
    target_obj.select_set(True)
    bpy.context.view_layer.objects.active = target_obj
    
    # 4. Apply all existing transforms
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    
    # 5. Center horizontally and ground vertically at Z=0
    bbox = [target_obj.matrix_world @ Vector(c) for c in target_obj.bound_box]
    min_x, max_x = min(c.x for c in bbox), max(c.x for c in bbox)
    min_y, max_y = min(c.y for c in bbox), max(c.y for c in bbox)
    min_z, max_z = min(c.z for c in bbox), max(c.z for c in bbox)
    
    target_obj.location.x -= (min_x + max_x) / 2.0
    target_obj.location.y -= (min_y + max_y) / 2.0
    target_obj.location.z -= min_z
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
    
    # 6. Normalize bounding scale to target dimension (e.g. 1.2m - 1.6m)
    max_dim = max(max_x - min_x, max_y - min_y, max_z - min_z)
    if max_dim > 0.001:
        scale_factor = target_max_dim / max_dim
        target_obj.scale = (scale_factor, scale_factor, scale_factor)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    
    # 7. Preserve and wire Vertex Color Attributes to Principled BSDF
    mesh = target_obj.data
    color_layer = mesh.color_attributes[0] if mesh.color_attributes else None
    
    if not target_obj.data.materials:
        mat = bpy.data.materials.new(name="Chronos_Material")
        mat.use_nodes = True
        target_obj.data.materials.append(mat)
    else:
        mat = target_obj.data.materials[0]
        mat.use_nodes = True
        
    nodes = mat.node_tree.nodes
    principled = next((n for n in nodes if n.type == 'BSDF_PRINCIPLED'), None)
    if principled and color_layer:
        attr_node = next((n for n in nodes if n.type in ('ATTRIBUTE', 'VERTEX_COLOR')), None)
        if not attr_node:
            attr_node = nodes.new(type='ShaderNodeVertexColor')
            attr_node.layer_name = color_layer.name
            mat.node_tree.links.new(attr_node.outputs['Color'], principled.inputs['Base Color'])
            
    if principled:
        principled.inputs['Roughness'].default_value = 0.45
        principled.inputs['Metallic'].default_value = 0.05
        
    # 8. Export to GLB with Y-up and attributes enabled
    os.makedirs(os.path.dirname(output_glb), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=output_glb,
        export_format='GLB',
        use_selection=True,
        export_apply=True,
        export_materials='EXPORT',
        export_attributes=True,
        export_yup=True
    )
```

---

## 4. Architectural Pedestals in Bevy (`world.rs`)

Exhibition pedestals are spawned in `crates/engine/src/modes/inner_chambers/world.rs`. Each pedestal is a 5-tier architectural structure:

| Tier | Geometry | Dimensions | Material |
|---|---|---|---|
| **Base Plinth** | `Cylinder` | Radius: $1.35\text{m}$, Height: $0.25\text{m}$ ($y \in [0.0, 0.25]$) | Dark Granite (`Color::srgb(0.09, 0.095, 0.11)`, roughness 0.8) |
| **Main Shaft** | `Cylinder` | Radius: $0.95\text{m}$, Height: $1.00\text{m}$ ($y \in [0.25, 1.25]$) | Polished Obsidian (`Color::srgb(0.05, 0.055, 0.07)`, roughness 0.35) |
| **Upper Capital** | `Cylinder` | Radius: $1.20\text{m}$, Height: $0.20\text{m}$ ($y \in [1.25, 1.45]$) | Beveled Stone (`Color::srgb(0.12, 0.125, 0.15)`, roughness 0.5) |
| **Gold Trim Rim** | `Cylinder` | Radius: $1.22\text{m}$, Height: $0.04\text{m}$ ($y \in [1.45, 1.49]$) | Burnished Gold (`Color::srgb(0.85, 0.68, 0.28)`, metallic 0.85) |
| **Velvet Cushion** | `Cylinder` | Radius: $0.85\text{m}$, Height: $0.05\text{m}$ ($y \in [1.49, 1.54]$) | Deep Indigo Velvet (`Color::srgb(0.08, 0.07, 0.13)`, roughness 0.9) |

The artifact model is spawned directly on the cushion surface at **$y = 1.54\text{m}$**.

---

## 5. The Dual-Lighting Doctrine

A single omnidirectional light renders 3D models flat. Every exhibition pedestal MUST have dual illumination:

```
                  [ Overhead Spotlight ] (y = 5.2m, Crisp Ivory White, Range: 12.0m)
                            │
                            ▼
                    ┌───────────────┐
                    │ Chronos Model │
                    └───────┬───────┘
                            ▲
                            │
                  [ Showcase Underglow ] (y = 1.75m, Thematic Amber/Gold/Rose/Emerald, Range: 4.5m)
                    ┌───────────────┐
                    │ Velvet Cushion│
                    └───────────────┘
```

1. **Overhead Spotlight:**
   - Position: $(x, 5.2, z)$ directly above the model.
   - Color: Crisp clean ivory (`Color::srgb(1.0, 0.98, 0.94)`).
   - Intensity: `65_000.0`, Range: `12.0m`.
   - Role: Defines edges, top specular highlights, and structural form.

2. **Showcase Underglow Light:**
   - Position: $(x, 1.75, z)$ right at the base of the model.
   - Color: Tailored to the artifact's narrative essence:
     - Porcelain / Ceramic: Warm Amber (`Color::srgb(1.0, 0.88, 0.65)`)
     - Ceremonial / Altar: Soft Rose Ivory (`Color::srgb(1.0, 0.75, 0.85)`)
     - Mystic / Relic: Deep Burnished Gold (`Color::srgb(1.0, 0.82, 0.35)`)
     - Nature / Sylvan: Emerald Verdant Glow (`Color::srgb(0.55, 0.95, 0.65)`)
   - Intensity: `22_000.0`, Range: `4.5m`.
   - Role: Uplights under-surfaces, fills crevices, and bathes the cushion in ambient glow.

---

## 6. Turntable Showcase Component (`ChronosExhibitTurntable`)

To allow effortless 360-degree viewing as the player flies around the chamber, each model entity is assigned the `ChronosExhibitTurntable` component:

```rust
#[derive(Component)]
pub struct ChronosExhibitTurntable {
    pub speed: f32,
}

fn rotate_chronos_exhibits(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ChronosExhibitTurntable)>,
) {
    let delta = time.delta_secs();
    for (mut transform, turntable) in &mut query {
        transform.rotate_y(turntable.speed * delta);
    }
}
```
Standard recommended turntable speed is **$0.25$ to $0.35\text{ rad/s}$** ($\approx 14^\circ–20^\circ/\text{second}$).

---

## 7. Mandatory Verification & Delivery Law

Whenever a 3D asset, pedestal, or world element is added or updated:

1. **Verify Rust Gate:**
   ```powershell
   cargo test --workspace
   ```
   All tests must pass.
2. **Refresh Desktop and Taskbar Surface:**
   ```powershell
   pwsh -File scripts\install_shortcut.ps1
   ```
   This compiles `dist\engine.exe`, copies new `.glb` assets to `dist\assets\scenes\` and `%LOCALAPPDATA%\Programs\Archetypes\assets\scenes\`, and refreshes the **pinned Windows Taskbar icon**.
3. **Commit & Push:**
   Commit to `main` and push to `origin` immediately.
4. **Handoff:**
   Inform the operator that the changes are live and ready to be tested directly from their pinned Taskbar icon.
