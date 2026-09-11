"""Normalize Meshy character GLBs for the Inner Castle using Blender background mode.

The source characters retain their authored materials and hierarchy.  This script only
grounds and centers their complete imported scene, then exports an application-ready
Y-up GLB.  It deliberately does not manufacture a pose or merge meshes.
"""

import os
import bpy
from mathutils import Vector

ROOT = r"C:\archetypes"
DOWNLOADS = os.path.expanduser(r"~\Downloads")

ASSETS = (
    ("Meshy_AI_Architect_0911001938_texture.glb", "architect.glb"),
    ("Meshy_AI_Explorer_0911001908_texture.glb", "explorer.glb"),
    ("Meshy_AI_Mentor_0909214927_texture.glb", "mentor.glb"),
)


def bounds(meshes):
    corners = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    return (
        min(c.x for c in corners), max(c.x for c in corners),
        min(c.y for c in corners), max(c.y for c in corners),
        min(c.z for c in corners), max(c.z for c in corners),
    )


def normalize(source_name, output_name):
    source = os.path.join(DOWNLOADS, source_name)
    destination = os.path.join(ROOT, "assets", "scenes", output_name)
    if not os.path.isfile(source):
        raise RuntimeError(f"Missing source GLB: {source}")

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=source)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError(f"No mesh objects imported from {source}")

    min_x, max_x, min_y, max_y, min_z, max_z = bounds(meshes)
    # glTF imports Y-up into Blender's Z-up world.  Center horizontally, ground on Z.
    offset = Vector((-(min_x + max_x) / 2.0, -(min_y + max_y) / 2.0, -min_z))
    roots = [obj for obj in bpy.context.scene.objects if obj.parent is None]
    for root in roots:
        root.location += offset

    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.export_scene.gltf(
        filepath=destination,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_materials="EXPORT",
        export_attributes=True,
        export_yup=True,
    )
    after = bounds(meshes)
    print(f"{output_name}: meshes={len(meshes)} bounds={after} bytes={os.path.getsize(destination)}")


for asset in ASSETS:
    normalize(*asset)
