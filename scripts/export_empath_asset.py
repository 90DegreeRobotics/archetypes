"""
Export and prepare Meshy AI Empath model for Archetypes runtime.
Snaps feet flush to Z=0, centers horizontally at X=0, Y=0, and exports to assets/scenes/empath.glb.
"""

import os
import sys
import bpy
from mathutils import Vector

def export_empath(source_glb, output_glb):
    print(f"Loading Empath from: {source_glb}")
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=source_glb)
    
    # Locate mesh object
    mesh_objs = [o for o in bpy.data.objects if o.type == 'MESH']
    if not mesh_objs:
        raise RuntimeError(f"No mesh objects found in {source_glb}")
    
    target_obj = mesh_objs[0]
    print(f"Target object: {target_obj.name} with {len(target_obj.data.vertices)} vertices")
    
    # Remove any unwanted non-mesh objects
    for o in [o for o in bpy.data.objects if o != target_obj]:
        bpy.data.objects.remove(o, do_unlink=True)
        
    bpy.ops.object.select_all(action='DESELECT')
    target_obj.select_set(True)
    bpy.context.view_layer.objects.active = target_obj
    
    # Apply initial transforms
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    
    # Compute bounding box
    bbox = [target_obj.matrix_world @ Vector(c) for c in target_obj.bound_box]
    min_x = min(c.x for c in bbox)
    max_x = max(c.x for c in bbox)
    min_y = min(c.y for c in bbox)
    max_y = max(c.y for c in bbox)
    min_z = min(c.z for c in bbox)
    max_z = max(c.z for c in bbox)
    
    center_x = (min_x + max_x) / 2.0
    center_y = (min_y + max_y) / 2.0
    
    # Snap feet to Z=0 and center horizontally
    target_obj.location.x -= center_x
    target_obj.location.y -= center_y
    target_obj.location.z -= min_z
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
    
    # Verify post-transform bounds
    new_bbox = [target_obj.matrix_world @ Vector(c) for c in target_obj.bound_box]
    print(f"Processed bounds:")
    print(f"  X: {min(c.x for c in new_bbox):.4f} to {max(c.x for c in new_bbox):.4f} (width: {max_x - min_x:.4f}m)")
    print(f"  Y: {min(c.y for c in new_bbox):.4f} to {max(c.y for c in new_bbox):.4f} (depth: {max_y - min_y:.4f}m)")
    print(f"  Z: {min(c.z for c in new_bbox):.4f} to {max(c.z for c in new_bbox):.4f} (height: {max_z - min_z:.4f}m)")
    
    os.makedirs(os.path.dirname(output_glb), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=output_glb,
        export_format='GLB',
        use_selection=True,
        export_apply=True,
        export_yup=True,
        export_materials='EXPORT',
        export_image_format='AUTO'
    )
    print(f"Successfully exported to: {output_glb} ({os.path.getsize(output_glb)} bytes)")

if __name__ == '__main__':
    src = r"C:\Users\m\Downloads\Meshy_AI_Empath_0909165719_texture.glb"
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    dst = os.path.join(repo_root, "assets", "scenes", "empath.glb")
    export_empath(src, dst)
