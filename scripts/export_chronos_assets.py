"""
Export ChronoSophia 3D objects to game-ready GLB assets for Archetypes.
Executed via Blender 4.5 background mode.
"""

import os
import sys
import bpy
from mathutils import Vector

def process_and_export(blend_path, mesh_name, output_glb, target_max_dim=1.4):
    print(f"\n==========================================")
    print(f"Processing: {blend_path}")
    print(f"Output: {output_glb}")
    
    # Open blend file
    bpy.ops.wm.open_mainfile(filepath=blend_path)
    
    # Find target mesh object
    target_obj = None
    for obj in bpy.data.objects:
        if obj.type == 'MESH':
            if mesh_name in obj.name or obj.name == mesh_name:
                target_obj = obj
                break
    
    if not target_obj:
        # Fallback to the largest mesh object
        meshes = [o for o in bpy.data.objects if o.type == 'MESH']
        if meshes:
            target_obj = max(meshes, key=lambda m: len(m.data.vertices))
            print(f"Target '{mesh_name}' not found by name, selected largest mesh: {target_obj.name}")
        else:
            raise RuntimeError(f"No mesh objects found in {blend_path}")
            
    print(f"Found target object: {target_obj.name} with {len(target_obj.data.vertices)} verts")
    
    # Remove all non-mesh objects (lights, cameras)
    objs_to_remove = [o for o in bpy.data.objects if o != target_obj]
    for o in objs_to_remove:
        bpy.data.objects.remove(o, do_unlink=True)
        
    # Ensure active and selected
    bpy.ops.object.select_all(action='DESELECT')
    target_obj.select_set(True)
    bpy.context.view_layer.objects.active = target_obj
    
    # Apply all transforms first
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    
    # Center origin to geometry
    bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
    
    # Calculate bounding box in world space
    bbox_corners = [target_obj.matrix_world @ Vector(corner) for corner in target_obj.bound_box]
    min_x = min(c.x for c in bbox_corners)
    max_x = max(c.x for c in bbox_corners)
    min_y = min(c.y for c in bbox_corners)
    max_y = max(c.y for c in bbox_corners)
    min_z = min(c.z for c in bbox_corners)
    max_z = max(c.z for c in bbox_corners)
    
    center_x = (min_x + max_x) / 2.0
    center_y = (min_y + max_y) / 2.0
    
    # Move object so that center is at X=0, Y=0 and base is at Z=0
    target_obj.location.x -= center_x
    target_obj.location.y -= center_y
    target_obj.location.z -= min_z
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
    
    # Normalize scale to fit target_max_dim
    dim_x = max_x - min_x
    dim_y = max_y - min_y
    dim_z = max_z - min_z
    max_dim = max(dim_x, dim_y, dim_z)
    if max_dim > 0.001:
        scale_factor = target_max_dim / max_dim
        target_obj.scale = (scale_factor, scale_factor, scale_factor)
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        print(f"Scaled by {scale_factor:.4f} to fit max dimension {target_max_dim}m")
    
    # Check/Setup material with vertex color support
    mesh = target_obj.data
    color_layer = None
    if mesh.color_attributes:
        color_layer = mesh.color_attributes[0]
        print(f"Mesh has color attribute: '{color_layer.name}' (type: {color_layer.data_type})")
        
    if not target_obj.data.materials or len(target_obj.data.materials) == 0:
        mat = bpy.data.materials.new(name="Chronos_Material")
        mat.use_nodes = True
        target_obj.data.materials.append(mat)
    else:
        mat = target_obj.data.materials[0]
        if not mat.use_nodes:
            mat.use_nodes = True
            
    # If there's a color attribute, wire it to Base Color if not already
    nodes = mat.node_tree.nodes
    principled = next((n for n in nodes if n.type == 'BSDF_PRINCIPLED'), None)
    if principled and color_layer:
        # Check if vertex color or attribute node exists
        attr_node = next((n for n in nodes if n.type in ('ATTRIBUTE', 'VERTEX_COLOR')), None)
        if not attr_node:
            attr_node = nodes.new(type='ShaderNodeVertexColor')
            attr_node.layer_name = color_layer.name
            mat.node_tree.links.new(attr_node.outputs['Color'], principled.inputs['Base Color'])
            print(f"Linked VertexColor '{color_layer.name}' to Principled BSDF Base Color")
            
    # Set roughness/metallic for good display
    if principled:
        principled.inputs['Roughness'].default_value = 0.45
        principled.inputs['Metallic'].default_value = 0.05
        
    # Export to GLB
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
    print(f"Successfully exported {output_glb} ({os.path.getsize(output_glb)} bytes)")

def main():
    items = [
        {
            "blend": r"C:\chronos2\out\object_triposr_only_20260908\scene.blend",
            "mesh": "TripoSR_Object",
            "out": r"C:\archetypes\assets\scenes\chronos_teapot.glb",
            "max_dim": 1.25
        },
        {
            "blend": r"C:\chronos2\out\universal_object_witness\wedding_cake_contrast_fix_refined_20260907\scene.blend",
            "mesh": "chronos_mesh",
            "out": r"C:\archetypes\assets\scenes\chronos_cake.glb",
            "max_dim": 1.35
        },
        {
            "blend": r"C:\chronos2\out\universal_object_witness\pinecone_20260907_v2\scene.blend",
            "mesh": "chronos_mesh",
            "out": r"C:\archetypes\assets\scenes\chronos_pinecone.glb",
            "max_dim": 1.20
        },
        {
            "blend": r"C:\chronos2\out\universal_object_witness\pine_tree_refined_20260907\scene.blend",
            "mesh": "chronos_mesh",
            "out": r"C:\archetypes\assets\scenes\chronos_pinetree.glb",
            "max_dim": 1.60
        }
    ]
    
    for item in items:
        process_and_export(item["blend"], item["mesh"], item["out"], item["max_dim"])
        
    print("\nAll ChronoSophia objects successfully converted to Archetypes GLB assets!")

if __name__ == "__main__":
    main()
