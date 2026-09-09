"""
Export and prepare Meshy AI Archetype characters for Archetypes runtime:
- AURA
- Sentinel
- Oracle
- Nebula Jester

Snaps feet flush to Z=0, centers horizontally at X=0, Y=0, and exports clean GLBs to assets/scenes/.
Reports unscaled bounding box heights so exact Bevy scaling can be computed for player eye level (2.85m).
"""

import os
import sys
import bpy
from mathutils import Vector

CHARACTERS = [
    {
        "name": "aura",
        "src": r"C:\Users\m\Downloads\Meshy_AI_AURA_0909170750_texture.glb",
        "dst": "aura.glb",
    },
    {
        "name": "sentinel",
        "src": r"C:\Users\m\Downloads\Meshy_AI_Sentinel_0909170617_texture.glb",
        "dst": "sentinel.glb",
    },
    {
        "name": "oracle",
        "src": r"C:\Users\m\Downloads\Meshy_AI_Oracle_0909170555_texture.glb",
        "dst": "oracle.glb",
    },
    {
        "name": "nebula_jester",
        "src": r"C:\Users\m\Downloads\Meshy_AI_Nebula_Jester_0909170446_texture.glb",
        "dst": "nebula_jester.glb",
    },
]

def process_character(char_info, assets_dir):
    src = char_info["src"]
    dst = os.path.join(assets_dir, char_info["dst"])
    name = char_info["name"]

    print(f"\n========================================================")
    print(f"Processing character: {name.upper()}")
    print(f"Source: {src}")
    print(f"Target: {dst}")

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=src)

    mesh_objs = [o for o in bpy.data.objects if o.type == 'MESH']
    if not mesh_objs:
        raise RuntimeError(f"No mesh objects found in {src}")

    # If there are multiple meshes, join them into one, or choose the primary mesh
    if len(mesh_objs) > 1:
        print(f"Found {len(mesh_objs)} mesh objects, joining into unified mesh...")
        bpy.ops.object.select_all(action='DESELECT')
        for o in mesh_objs:
            o.select_set(True)
        bpy.context.view_layer.objects.active = mesh_objs[0]
        bpy.ops.object.join()
        target_obj = bpy.context.view_layer.objects.active
    else:
        target_obj = mesh_objs[0]

    print(f"Mesh object: {target_obj.name} with {len(target_obj.data.vertices)} vertices")

    # Remove any non-mesh objects (empties, cameras, lights)
    for o in [o for o in bpy.data.objects if o != target_obj]:
        bpy.data.objects.remove(o, do_unlink=True)

    bpy.ops.object.select_all(action='DESELECT')
    target_obj.select_set(True)
    bpy.context.view_layer.objects.active = target_obj

    # Apply all transforms first
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    # Compute bounding box
    bbox = [target_obj.matrix_world @ Vector(c) for c in target_obj.bound_box]
    min_x = min(c.x for c in bbox)
    max_x = max(c.x for c in bbox)
    min_y = min(c.y for c in bbox)
    max_y = max(c.y for c in bbox)
    min_z = min(c.z for c in bbox)
    max_z = max(c.z for c in bbox)

    width = max_x - min_x
    depth = max_y - min_y
    raw_height = max_z - min_z

    center_x = (min_x + max_x) / 2.0
    center_y = (min_y + max_y) / 2.0

    # Ground feet flush to Z=0 and center horizontally at (0, 0)
    target_obj.location.x -= center_x
    target_obj.location.y -= center_y
    target_obj.location.z -= min_z
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)

    # Compute eye line: approximate eye height is ~93% of overall figure height (from soles to eyes)
    # Let's also check top 5% vertex z
    verts_z = sorted([v.co.z for v in target_obj.data.vertices])
    # Sole is now at 0.0
    top_z = verts_z[-1]
    # Eye line is typically between 91% and 94% of top height
    eye_z_raw = top_z * 0.93
    target_eye_height = 2.85  # Player standing eye level in meters
    recommended_scale = target_eye_height / eye_z_raw

    print(f"Grounded & Centered:")
    print(f"  Width (X): {width:.4f}m")
    print(f"  Depth (Y): {depth:.4f}m")
    print(f"  Raw Height (Z): {raw_height:.4f}m (sole=0.0m, top={top_z:.4f}m)")
    print(f"  Estimated raw eye Z: {eye_z_raw:.4f}m")
    print(f"  Recommended Bevy scale for 2.85m eye height: {recommended_scale:.4f}x")

    # Materials report
    print(f"  Materials ({len(target_obj.data.materials)}):")
    for mat in target_obj.data.materials:
        if mat:
            print(f"    - {mat.name}")

    os.makedirs(os.path.dirname(dst), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=dst,
        export_format='GLB',
        use_selection=True,
        export_apply=True,
        export_yup=True,
        export_materials='EXPORT',
        export_image_format='AUTO'
    )
    print(f"Successfully exported {name} to: {dst} ({os.path.getsize(dst)} bytes)")

def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(repo_root, "assets", "scenes")
    for char_info in CHARACTERS:
        process_character(char_info, assets_dir)

if __name__ == '__main__':
    main()
