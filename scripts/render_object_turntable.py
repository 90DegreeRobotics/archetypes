"""Render one manifested object from three angles, on a neutral studio backdrop.

Used by `scripts/manifest_quality_lab.py` to judge reconstruction quality. Deliberately neutral:
a soft three-point rig and a mid-grey ground, so what is being judged is the mesh and its
vertex colour, not a flattering light.

Run headless:

  blender -b --factory-startup -P scripts/render_object_turntable.py -- \
      --glb artifacts/quality-lab/glb/hard_edge.glb --out-dir artifacts/quality-lab/renders/hard_edge
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

ANGLES = (35.0, 140.0, 250.0)
RESOLUTION = 640
SAMPLES = 48


def log(*parts: object) -> None:
    print("[turntable]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--glb", required=True)
    parser.add_argument("--out-dir", required=True)
    return parser.parse_args(argv)


def bounds(objects):
    points = [obj.matrix_world @ Vector(c) for obj in objects for c in obj.bound_box]
    lo = Vector((min(p.x for p in points), min(p.y for p in points), min(p.z for p in points)))
    hi = Vector((max(p.x for p in points), max(p.y for p in points), max(p.z for p in points)))
    return lo, hi


def main() -> None:
    args = parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(Path(args.glb).resolve()))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise SystemExit("no mesh in the GLB")

    lo, hi = bounds(meshes)
    centre = (lo + hi) * 0.5
    extent = max((hi - lo).x, (hi - lo).y, (hi - lo).z)
    if extent <= 0:
        raise SystemExit("mesh has empty bounds")

    # A mid-grey ground under the object. Not white: a white floor blows out the exposure and
    # flatters everything equally, which is the opposite of what a quality check wants.
    ground = bpy.data.materials.new("lab_ground")
    ground.use_nodes = True
    ground.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (
        0.22, 0.22, 0.23, 1.0
    )
    ground.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.85
    bpy.ops.mesh.primitive_plane_add(size=extent * 14.0, location=(centre.x, centre.y, lo.z))
    bpy.context.active_object.data.materials.append(ground)

    # Anything the importer left without a material gets one that shows its vertex colour, so
    # the render judges what the mesh actually carries rather than a default grey.
    for obj in meshes:
        if obj.data.materials:
            continue
        material = bpy.data.materials.new(f"lab_vcol_{obj.name}")
        material.use_nodes = True
        tree = material.node_tree
        principled = tree.nodes["Principled BSDF"]
        colours = obj.data.color_attributes
        if colours:
            attribute = tree.nodes.new("ShaderNodeVertexColor")
            attribute.layer_name = colours[0].name
            tree.links.new(attribute.outputs["Color"], principled.inputs["Base Color"])
        principled.inputs["Roughness"].default_value = 0.55
        obj.data.materials.append(material)

    key = bpy.data.lights.new("key", type="AREA")
    key.energy = 900.0 * extent * extent
    key.size = extent * 2.2
    key_object = bpy.data.objects.new("key", key)
    bpy.context.collection.objects.link(key_object)
    key_object.location = centre + Vector((extent * 1.6, -extent * 2.0, extent * 2.2))
    key_object.rotation_euler = (
        (centre - key_object.location).to_track_quat("-Z", "Y").to_euler()
    )

    fill = bpy.data.lights.new("fill", type="AREA")
    fill.energy = 260.0 * extent * extent
    fill.size = extent * 3.0
    fill_object = bpy.data.objects.new("fill", fill)
    bpy.context.collection.objects.link(fill_object)
    fill_object.location = centre + Vector((-extent * 2.4, -extent * 1.2, extent * 1.0))
    fill_object.rotation_euler = (
        (centre - fill_object.location).to_track_quat("-Z", "Y").to_euler()
    )

    world = bpy.data.worlds.new("lab")
    bpy.context.scene.world = world
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.05, 0.06, 1.0)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6

    camera_data = bpy.data.cameras.new("cam")
    camera_data.lens = 52.0
    camera = bpy.data.objects.new("cam", camera_data)
    bpy.context.collection.objects.link(camera)
    bpy.context.scene.camera = camera

    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE_NEXT"
    scene.render.resolution_x = RESOLUTION
    scene.render.resolution_y = RESOLUTION
    scene.render.film_transparent = False
    if hasattr(scene, "eevee") and hasattr(scene.eevee, "taa_render_samples"):
        scene.eevee.taa_render_samples = SAMPLES

    distance = extent * 2.4
    for index, angle in enumerate(ANGLES):
        radians = math.radians(angle)
        camera.location = centre + Vector(
            (math.cos(radians) * distance, math.sin(radians) * distance, extent * 0.85)
        )
        camera.rotation_euler = (centre - camera.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = str(out_dir / f"{index:02d}.png")
        bpy.ops.render.render(write_still=True)
        log(f"rendered {scene.render.filepath}")


if __name__ == "__main__":
    main()
