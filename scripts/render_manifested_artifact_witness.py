"""Render the actual game GLB for manifestation import visual verification."""
import argparse
import math
import os
import sys

import bpy
from mathutils import Vector


def aim(obj, target):
    obj.rotation_euler = (Vector(target) - obj.location).to_track_quat("-Z", "Y").to_euler()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=args.input)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError("witness GLB contains no mesh")

    bpy.ops.mesh.primitive_cylinder_add(vertices=96, radius=1.05, depth=0.12, location=(0, 0, -0.06))
    pedestal = bpy.context.object
    pedestal.name = "WitnessPedestal"
    material = bpy.data.materials.new("WitnessObsidian")
    material.diffuse_color = (0.035, 0.055, 0.085, 1.0)
    material.metallic = 0.35
    material.roughness = 0.24
    pedestal.data.materials.append(material)

    bpy.ops.object.camera_add(location=(3.2, -5.0, 2.5))
    camera = bpy.context.object
    aim(camera, (0, 0, 0.68))
    camera.data.lens = 58
    bpy.context.scene.camera = camera

    for name, location, energy, size, color in (
        ("WarmKey", (-2.8, -3.0, 4.6), 1050, 3.0, (1.0, 0.86, 0.70)),
        ("NeutralFill", (3.2, -1.5, 2.6), 650, 2.5, (0.76, 0.86, 1.0)),
        ("Rim", (0.2, 3.0, 3.2), 800, 2.0, (0.60, 0.78, 1.0)),
    ):
        data = bpy.data.lights.new(name, "AREA")
        data.energy = energy
        data.shape = "DISK"
        data.size = size
        data.color = color
        light = bpy.data.objects.new(name, data)
        bpy.context.collection.objects.link(light)
        light.location = location
        aim(light, (0, 0, 0.65))

    world = bpy.data.worlds.new("WitnessWorld")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.015, 0.022, 0.04, 1.0)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.22
    bpy.context.scene.world = world

    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE_NEXT"
    scene.render.resolution_x = 768
    scene.render.resolution_y = 768
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.render.filepath = os.path.abspath(args.output)
    os.makedirs(os.path.dirname(scene.render.filepath), exist_ok=True)
    bpy.ops.render.render(write_still=True)
    print(f"MANIFESTATION_WITNESS_PASS meshes={len(meshes)} output={scene.render.filepath}")


if __name__ == "__main__":
    main()
