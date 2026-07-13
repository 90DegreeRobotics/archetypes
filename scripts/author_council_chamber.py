"""Author the Lane B circular council chamber and export the runtime GLB.

Run:
    blender --background --python scripts/author_council_chamber.py

The script imports the current runtime chamber, preserves the named council
contract nodes, removes only the old shell/star geometry, authors an open
torch-lit arch ring, exports ``assets/scenes/uiscene1.glb``, then re-imports
the GLB and verifies the contract.
"""

from __future__ import annotations

import argparse
import math
from pathlib import Path

import bpy
from mathutils import Vector


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SCENE = ROOT / "assets" / "scenes" / "uiscene1.glb"
ARCHETYPES = ("Architect", "Sentinel", "Jester", "Mentor", "Explorer", "Oracle", "Empath")
TRIANGLE_LIMIT = 150_000

FLOOR_CENTER_Z = -5.25
FLOOR_DEPTH = 0.44
FLOOR_TOP_Z = FLOOR_CENTER_Z + FLOOR_DEPTH / 2.0
COLUMN_RADIUS = 12.0
COLUMN_HEIGHT = 6.0
COLUMN_TOP_Z = FLOOR_TOP_Z + COLUMN_HEIGHT


def parse_args() -> argparse.Namespace:
    argv = []
    if "--" in __import__("sys").argv:
        sys_argv = __import__("sys").argv
        argv = sys_argv[sys_argv.index("--") + 1 :]
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", default=str(DEFAULT_SCENE))
    parser.add_argument("--export", default=str(DEFAULT_SCENE))
    return parser.parse_args(argv)


def log(*args) -> None:
    print("LANE_B_CHAMBER:", *args)


def principled(material: bpy.types.Material) -> bpy.types.Node:
    material.use_nodes = True
    return next(node for node in material.node_tree.nodes if node.type == "BSDF_PRINCIPLED")


def emission_input_name(node: bpy.types.Node) -> str:
    return "Emission Color" if "Emission Color" in node.inputs else "Emission"


def material(name: str, base, metallic: float, roughness: float, emission=None, strength: float = 0.0):
    mat = bpy.data.materials.new(name)
    node = principled(mat)
    node.inputs["Base Color"].default_value = base
    node.inputs["Metallic"].default_value = metallic
    node.inputs["Roughness"].default_value = roughness
    if emission is not None:
        node.inputs[emission_input_name(node)].default_value = emission
        if "Emission Strength" in node.inputs:
            node.inputs["Emission Strength"].default_value = strength
    mat.diffuse_color = base
    return mat


def link_to_collection(obj: bpy.types.Object, collection: bpy.types.Collection) -> bpy.types.Object:
    if obj.name not in collection.objects:
        collection.objects.link(obj)
    return obj


def cylinder_between(
    name: str,
    start,
    end,
    radius: float,
    mat: bpy.types.Material,
    collection: bpy.types.Collection,
    vertices: int = 16,
) -> bpy.types.Object:
    start_vec = Vector(start)
    end_vec = Vector(end)
    delta = end_vec - start_vec
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=vertices,
        radius=radius,
        depth=delta.length,
        location=(start_vec + end_vec) / 2.0,
    )
    obj = bpy.context.object
    obj.name = name
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = delta.to_track_quat("Z", "Y")
    obj.data.materials.append(mat)
    return link_to_collection(obj, collection)


def add_cylinder(
    name: str,
    radius: float,
    depth: float,
    location,
    mat: bpy.types.Material,
    collection: bpy.types.Collection,
    vertices: int,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=location)
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    return link_to_collection(obj, collection)


def add_torus(
    name: str,
    major: float,
    minor: float,
    location,
    mat: bpy.types.Material,
    collection: bpy.types.Collection,
    major_segments: int = 96,
    minor_segments: int = 10,
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_torus_add(
        major_radius=major,
        minor_radius=minor,
        major_segments=major_segments,
        minor_segments=minor_segments,
        location=location,
    )
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    return link_to_collection(obj, collection)


def aim_at(obj: bpy.types.Object, target) -> None:
    obj.rotation_euler = (Vector(target) - obj.location).to_track_quat("-Z", "Y").to_euler()


def remove_old_shell_and_star() -> None:
    old_prefixes = (
        "Temple_",
        "Council_Dais",
        "Chamber_",
        "Torch_",
        "Column_",
        "Arch_",
        "Central_Dais",
    )
    removed = []
    for obj in list(bpy.context.scene.objects):
        if (
            obj.name.startswith(old_prefixes)
            or obj.name.startswith("Star_Tetra")
            or obj.name.startswith("Merkaba")
            or "Merkaba" in obj.name
        ):
            removed.append(obj.name)
            bpy.data.objects.remove(obj, do_unlink=True)
    log(f"removed old shell/star objects={len(removed)}")


def ensure_witness_camera() -> None:
    camera = bpy.data.objects.get("Witness_Camera")
    if camera is not None and camera.type == "CAMERA":
        bpy.context.scene.camera = camera
        return

    if camera is not None:
        bpy.data.objects.remove(camera, do_unlink=True)

    data = bpy.data.cameras.new("Witness_Camera")
    camera = bpy.data.objects.new("Witness_Camera", data)
    bpy.context.scene.collection.objects.link(camera)
    camera.location = (8.0, -12.0, 6.2)
    data.lens = 34.0
    data.clip_end = 120.0
    aim_at(camera, (0.0, 0.0, -0.15))
    bpy.context.scene.camera = camera
    log("created Witness_Camera")


def build_chamber() -> None:
    collection = bpy.data.collections.new("Lane_B_Council_Chamber")
    bpy.context.scene.collection.children.link(collection)

    stone = material("Chamber_Dark_Polished_Stone", (0.025, 0.030, 0.045, 1.0), 0.0, 0.70)
    relief = material("Chamber_Dark_Stone_Relief", (0.040, 0.047, 0.064, 1.0), 0.0, 0.62)
    brass = material("Chamber_Gilded_Brass", (0.86, 0.55, 0.16, 1.0), 0.90, 0.25)
    ember = material(
        "Chamber_Torch_Ember",
        (1.0, 0.30, 0.04, 1.0),
        0.0,
        0.35,
        emission=(1.0, 0.35, 0.08, 1.0),
        strength=9.0,
    )

    add_cylinder("Chamber_Circular_Floor", 14.0, FLOOR_DEPTH, (0, 0, FLOOR_CENTER_Z), stone, collection, 96)
    add_torus("Chamber_Floor_Gilded_Inlay", 12.35, 0.055, (0, 0, FLOOR_TOP_Z + 0.035), brass, collection)
    add_cylinder("Chamber_Central_Dais", 4.0, 0.62, (0, 0, FLOOR_TOP_Z + 0.31), relief, collection, 80)
    add_torus("Chamber_Central_Dais_Gold_Rim", 3.88, 0.085, (0, 0, FLOOR_TOP_Z + 0.64), brass, collection, 96, 8)

    column_points = []
    for index in range(10):
        angle = math.tau * index / 10.0
        x = COLUMN_RADIUS * math.cos(angle)
        y = COLUMN_RADIUS * math.sin(angle)
        column_points.append(Vector((x, y, COLUMN_TOP_Z)))

        add_cylinder(
            f"Chamber_Column_{index + 1:02d}",
            0.46,
            COLUMN_HEIGHT,
            (x, y, FLOOR_TOP_Z + COLUMN_HEIGHT / 2.0),
            relief,
            collection,
            18,
        )
        add_torus(
            f"Chamber_Column_{index + 1:02d}_Base_Band",
            0.46,
            0.055,
            (x, y, FLOOR_TOP_Z + 0.22),
            brass,
            collection,
            36,
            8,
        )
        add_torus(
            f"Chamber_Column_{index + 1:02d}_Capital_Band",
            0.48,
            0.065,
            (x, y, COLUMN_TOP_Z - 0.12),
            brass,
            collection,
            36,
            8,
        )

        bpy.ops.mesh.primitive_cone_add(
            vertices=14,
            radius1=0.18,
            radius2=0.035,
            depth=0.52,
            location=(x, y, COLUMN_TOP_Z + 0.45),
        )
        flame = bpy.context.object
        flame.name = f"Torch_Flame_{index + 1:02d}"
        flame.data.materials.append(ember)
        link_to_collection(flame, collection)

        light_data = bpy.data.lights.new(f"Torch_Point_{index + 1:02d}", "POINT")
        light_data.energy = 150.0
        light_data.color = (1.0, 0.55, 0.20)
        light_data.shadow_soft_size = 3.0
        light = bpy.data.objects.new(light_data.name, light_data)
        light.location = (x, y, COLUMN_TOP_Z + 0.72)
        collection.objects.link(light)

    for index in range(10):
        start = column_points[index]
        end = column_points[(index + 1) % 10]
        arch_points = []
        trim_points = []
        for step in range(9):
            t = step / 8.0
            point = start.lerp(end, t)
            point.z = COLUMN_TOP_Z + 3.25 * math.sin(math.pi * t)
            arch_points.append(point)
            inward = Vector((-point.x, -point.y, 0.0))
            if inward.length > 0.0:
                inward.normalize()
            trim_points.append(point + inward * 0.14 + Vector((0, 0, 0.13)))

        for step in range(len(arch_points) - 1):
            cylinder_between(
                f"Chamber_Arch_{index + 1:02d}_Stone_{step + 1:02d}",
                arch_points[step],
                arch_points[step + 1],
                0.30,
                relief,
                collection,
                14,
            )
            cylinder_between(
                f"Chamber_Arch_{index + 1:02d}_Gold_Trim_{step + 1:02d}",
                trim_points[step],
                trim_points[step + 1],
                0.080,
                brass,
                collection,
                12,
            )

    if bpy.context.scene.world is None:
        bpy.context.scene.world = bpy.data.worlds.new("Chamber_World")
    bpy.context.scene.world.color = (0.002, 0.003, 0.010)
    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end = 240


def triangle_count() -> int:
    return sum(
        len(poly.vertices) - 2
        for obj in bpy.context.scene.objects
        if obj.type == "MESH"
        for poly in obj.data.polygons
    )


def verify_export(path: Path) -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))
    names = {obj.name: obj for obj in bpy.context.scene.objects}
    missing = []

    for archetype in ARCHETYPES:
        for required in (
            archetype,
            f"{archetype}_PanelSpinner",
            f"{archetype}_Icon_Panel",
            f"{archetype}_Portrait_Panel",
        ):
            if required not in names:
                missing.append(required)

    camera = names.get("Witness_Camera")
    if camera is None or camera.type != "CAMERA":
        missing.append("Witness_Camera:CAMERA")

    point_lights = [
        obj for obj in bpy.context.scene.objects if obj.type == "LIGHT" and obj.data.type == "POINT"
    ]
    triangles = triangle_count()

    log(
        "VERIFY",
        f"vessels={sum(1 for name in ARCHETYPES if name in names)}",
        f"panel_nodes={sum(1 for name in names if name.endswith('_PanelSpinner') or name.endswith('_Icon_Panel') or name.endswith('_Portrait_Panel'))}",
        f"point_lights={len(point_lights)}",
        f"triangles={triangles}",
        f"missing={missing}",
    )
    if missing:
        raise RuntimeError(f"Chamber export contract missing nodes: {missing}")
    if len(point_lights) < 10:
        raise RuntimeError(f"Chamber export has {len(point_lights)} POINT lights; expected at least 10")
    if triangles >= TRIANGLE_LIMIT:
        raise RuntimeError(f"Chamber export has {triangles} triangles; limit is {TRIANGLE_LIMIT}")


def main() -> None:
    args = parse_args()
    input_path = Path(args.input).resolve()
    export_path = Path(args.export).resolve()
    if not input_path.exists():
        raise RuntimeError(f"Input chamber GLB does not exist: {input_path}")

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(input_path))
    remove_old_shell_and_star()
    ensure_witness_camera()
    build_chamber()

    export_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(export_path),
        export_format="GLB",
        use_selection=False,
        export_cameras=True,
        export_lights=True,
        export_apply=True,
        export_yup=True,
    )
    log(f"exported {export_path}")
    verify_export(export_path)


if __name__ == "__main__":
    main()
