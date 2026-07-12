"""Author the enclosed Council Temple in an isolated Blender working copy.

Run with Blender against ``uiscene1.codex-temple.blend``. The script refuses to
touch the original source filename and refuses to stack a second overhaul onto
an already-authored copy.
"""

from __future__ import annotations

import math
from pathlib import Path

import bpy
from mathutils import Vector


ROOT = Path(__file__).resolve().parents[1]
WORKING_NAME = "uiscene1.codex-temple.blend"
ARCHETYPES = {
    "Architect": ((0.18, 0.72, 1.00, 1.0), "architect.png", "architect-icon.png"),
    "Sentinel": ((0.16, 0.88, 0.78, 1.0), "sentinel.png", "sentinel-icon.png"),
    "Jester": ((1.00, 0.28, 0.58, 1.0), "jester.png", "jester-icon.png"),
    "Mentor": ((1.00, 0.68, 0.20, 1.0), "mentor.png", "mentor-icon.png"),
    "Explorer": ((0.38, 0.56, 1.00, 1.0), "explorer.png", "explorer-icon.png"),
    "Oracle": ((0.68, 0.34, 1.00, 1.0), "oracle.png", "oracle-icon.png"),
    "Empath": ((1.00, 0.38, 0.42, 1.0), "empath.png", "empath-icon.png"),
}


def principled(material: bpy.types.Material) -> bpy.types.Node:
    material.use_nodes = True
    return material.node_tree.nodes.get("Principled BSDF")


def make_surface(name: str, color, metallic=0.0, roughness=0.5, emission=None, strength=0.0):
    material = bpy.data.materials.new(name)
    node = principled(material)
    node.inputs["Base Color"].default_value = color
    node.inputs["Metallic"].default_value = metallic
    node.inputs["Roughness"].default_value = roughness
    if emission:
        node.inputs["Emission Color"].default_value = emission
        node.inputs["Emission Strength"].default_value = strength
    return material


def make_glass(name: str, tint, alpha=0.26, transmission=0.92, roughness=0.12):
    material = bpy.data.materials.new(name)
    node = principled(material)
    node.inputs["Base Color"].default_value = tint
    node.inputs["Roughness"].default_value = roughness
    node.inputs["Metallic"].default_value = 0.06
    node.inputs["Transmission Weight"].default_value = transmission
    node.inputs["IOR"].default_value = 1.45
    node.inputs["Alpha"].default_value = alpha
    material.surface_render_method = "DITHERED"
    material.use_transparency_overlap = False
    material.diffuse_color = (*tint[:3], alpha)
    return material


def make_image_material(name: str, image_path: Path):
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    material.use_backface_culling = False
    nodes = material.node_tree.nodes
    links = material.node_tree.links
    for node in list(nodes):
        nodes.remove(node)
    output = nodes.new("ShaderNodeOutputMaterial")
    shader = nodes.new("ShaderNodeBsdfPrincipled")
    texture = nodes.new("ShaderNodeTexImage")
    texture.image = bpy.data.images.load(str(image_path), check_existing=True)
    shader.inputs["Roughness"].default_value = 0.32
    shader.inputs["Metallic"].default_value = 0.08
    shader.inputs["Emission Strength"].default_value = 0.18
    links.new(texture.outputs["Color"], shader.inputs["Base Color"])
    links.new(texture.outputs["Color"], shader.inputs["Emission Color"])
    links.new(shader.outputs["BSDF"], output.inputs["Surface"])
    return material


def link_to_collection(obj, collection):
    for current in list(obj.users_collection):
        current.objects.unlink(obj)
    collection.objects.link(obj)


def add_cube(name, location, scale, material, collection, bevel=0.0):
    bpy.ops.mesh.primitive_cube_add(location=location)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel:
        modifier = obj.modifiers.new("Temple edge softening", "BEVEL")
        modifier.width = bevel
        modifier.segments = 3
    obj.data.materials.append(material)
    link_to_collection(obj, collection)
    return obj


def add_torus(name, location, major_radius, minor_radius, material, collection, rotation=(0, 0, 0)):
    bpy.ops.mesh.primitive_torus_add(
        major_radius=major_radius,
        minor_radius=minor_radius,
        major_segments=64,
        minor_segments=10,
        location=location,
        rotation=rotation,
    )
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(material)
    link_to_collection(obj, collection)
    return obj


def aim_at(obj, target):
    obj.rotation_euler = (Vector(target) - obj.location).to_track_quat("-Z", "Y").to_euler()


def author_panel(archetype, sphere, art_name, icon_name, collection):
    root = bpy.data.objects.new(f"{archetype}_PanelSpinner", None)
    root.location = sphere.location.copy()
    collection.objects.link(root)
    root.rotation_euler = (Vector((0.0, 0.0, 1.0)) - root.location).to_track_quat("Z", "Y").to_euler()

    for face, filename, rotation in (
        ("Icon", icon_name, (0.0, 0.0, 0.0)),
        ("Portrait", art_name, (math.pi, 0.0, 0.0)),
    ):
        bpy.ops.mesh.primitive_plane_add(size=1.16, location=(0.0, 0.0, 0.0))
        panel = bpy.context.object
        panel.name = f"{archetype}_{face}_Panel"
        panel.parent = root
        panel.location = (0.0, 0.0, 0.025 if face == "Icon" else -0.025)
        panel.rotation_euler = rotation
        panel.data.materials.append(
            make_image_material(
                f"{archetype}_{face}_Art",
                ROOT / "assets" / ("icons" if face == "Icon" else "aura") / filename,
            )
        )
        bevel = panel.modifiers.new("Panel bevel", "BEVEL")
        bevel.width = 0.045
        bevel.segments = 3
        link_to_collection(panel, collection)

    root.rotation_mode = "XYZ"
    root.keyframe_insert(data_path="rotation_euler", frame=1)
    root.rotation_euler.rotate_axis("Y", math.tau)
    root.keyframe_insert(data_path="rotation_euler", frame=360)
    action = root.animation_data.action
    for curve in action.fcurves:
        for point in curve.keyframe_points:
            point.interpolation = "LINEAR"


def main():
    blend_path = Path(bpy.data.filepath)
    if blend_path.name != WORKING_NAME:
        raise RuntimeError(f"Refusing to modify {blend_path}; expected isolated {WORKING_NAME}")
    if bpy.data.collections.get("Temple_Overhaul"):
        raise RuntimeError("Temple_Overhaul already exists; refusing to stack duplicate geometry")

    scene = bpy.context.scene
    temple = bpy.data.collections.new("Temple_Overhaul")
    scene.collection.children.link(temple)

    stone = make_surface("Temple Basalt", (0.018, 0.022, 0.032, 1.0), metallic=0.18, roughness=0.42)
    stone_relief = make_surface("Temple Relief", (0.055, 0.064, 0.085, 1.0), metallic=0.3, roughness=0.34)
    gold = make_surface(
        "Temple Warm Metal", (0.24, 0.12, 0.025, 1.0), metallic=0.88, roughness=0.2,
        emission=(1.0, 0.27, 0.035, 1.0), strength=1.1,
    )
    cyan = make_surface(
        "Temple Cyan Light", (0.01, 0.16, 0.2, 1.0), metallic=0.3, roughness=0.16,
        emission=(0.02, 0.62, 1.0, 1.0), strength=3.0,
    )

    # Closed floor, faceted walls, overhead vault, and restrained architectural rhythm.
    bpy.ops.mesh.primitive_cylinder_add(vertices=64, radius=17.0, depth=0.5, location=(0, 0, -5.25))
    floor = bpy.context.object
    floor.name = "Temple_Floor"
    floor.data.materials.append(stone)
    link_to_collection(floor, temple)

    for index in range(16):
        angle = math.tau * index / 16
        x, y = 16.2 * math.cos(angle), 16.2 * math.sin(angle)
        pillar = add_cube(
            f"Temple_Pillar_{index + 1:02d}", (x, y, 2.0), (0.52, 0.82, 7.2),
            stone_relief, temple, bevel=0.18,
        )
        pillar.rotation_euler[2] = angle
        wall = add_cube(
            f"Temple_Wall_{index + 1:02d}",
            (15.25 * math.cos(angle + math.pi / 16), 15.25 * math.sin(angle + math.pi / 16), 2.0),
            (3.12, 0.34, 7.2), stone, temple, bevel=0.12,
        )
        wall.rotation_euler[2] = angle + math.pi / 16 + math.pi / 2

    bpy.ops.mesh.primitive_uv_sphere_add(segments=64, ring_count=32, radius=16.6, location=(0, 0, 4.2))
    vault = bpy.context.object
    vault.name = "Temple_Vault"
    vault.scale.z = 0.52
    vault.data.materials.append(stone)
    link_to_collection(vault, temple)

    add_torus("Temple_Floor_Ring", (0, 0, -4.93), 7.4, 0.075, cyan, temple)
    add_torus("Temple_Star_Halo", (0, 0, 0), 5.4, 0.055, gold, temple, rotation=(math.pi / 2, 0, 0))
    add_torus("Temple_Ceiling_Ring", (0, 0, 8.8), 10.8, 0.11, gold, temple)

    # The star becomes two legible panes of complementary sanctified glass.
    star_a = bpy.data.objects["Star_Tetra_A"]
    star_b = bpy.data.objects["Star_Tetra_B"]
    for obj, tint in ((star_a, (0.12, 0.62, 0.95, 1.0)), (star_b, (0.84, 0.28, 0.68, 1.0))):
        obj.data.materials.clear()
        obj.data.materials.append(make_glass(f"{obj.name}_TempleGlass", tint, alpha=0.22, transmission=0.96, roughness=0.08))
        obj.data.materials.append(gold)

    # Larger, readable vessels and the previously untouched identity art.
    for archetype, (color, art_name, icon_name) in ARCHETYPES.items():
        sphere = bpy.data.objects[archetype]
        sphere.scale *= 2.45
        sphere.data.materials.clear()
        sphere.data.materials.append(make_glass(f"{archetype}_VesselGlass", color, alpha=0.2, transmission=0.88, roughness=0.1))
        author_panel(archetype, sphere, art_name, icon_name, temple)

    # A stronger central altar makes the star feel suspended inside a room, not lost in a void.
    bpy.ops.mesh.primitive_cylinder_add(vertices=64, radius=4.4, depth=0.65, location=(0, 0, -4.72))
    dais = bpy.context.object
    dais.name = "Council_Dais"
    dais.data.materials.append(stone_relief)
    link_to_collection(dais, temple)
    add_torus("Council_Dais_Light", (0, 0, -4.35), 3.55, 0.12, cyan, temple)

    # Preserve breathing room between the temple shell and the suspended council.
    for obj in temple.objects:
        if obj.name.startswith("Temple_"):
            obj.location.x *= 1.38
            obj.location.y *= 1.38
            obj.scale.x *= 1.38
            obj.scale.y *= 1.38

    # Replace the flat two-light setup with motivated warm/cool temple lighting.
    for light in [obj for obj in scene.objects if obj.type == "LIGHT"]:
        light.hide_render = True
    for index, (location, color, energy) in enumerate((
        ((0, -7, 7), (0.16, 0.48, 1.0), 1300),
        ((-8, 5, 1), (1.0, 0.18, 0.06), 1000),
        ((8, 5, 1), (0.58, 0.16, 1.0), 900),
        ((0, 0, -3), (0.05, 0.52, 1.0), 700),
    )):
        data = bpy.data.lights.new(f"Temple_Light_{index + 1}", "POINT")
        data.energy = energy
        data.color = color
        data.shadow_soft_size = 4.0
        light = bpy.data.objects.new(data.name, data)
        light.location = location
        aim_at(light, (0, 0, 0))
        temple.objects.link(light)

    camera = bpy.data.objects["Witness_Camera"]
    camera.location = (8.0, -12.0, 6.2)
    camera.data.lens = 34.0
    aim_at(camera, (0.0, 0.0, -0.15))
    scene.camera = camera
    scene.render.engine = "BLENDER_EEVEE_NEXT"
    scene.render.resolution_x = 1600
    scene.render.resolution_y = 1000
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.film_transparent = False
    scene.world.color = (0.002, 0.003, 0.008)
    scene.frame_start = 1
    scene.frame_end = 360
    scene.frame_set(40)

    bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))
    print(f"AUTHORED {blend_path}")


if __name__ == "__main__":
    main()
