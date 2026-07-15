"""Author the lore-compliant council chamber and export the runtime GLB.

Run headless:
    blender --background --python scripts/author_lore_chamber.py

Run in Blender UI:
    Scripting tab -> open this file -> Run Script

The script is intentionally self-contained and idempotent. It rebuilds the
CouncilChamber collection, uses only GLB-safe mesh/material/light data, and
exports assets/scenes/lore_chamber.glb by default. Blender cameras stay in the
authoring scene for proof renders, but are not exported into the runtime GLB;
Bevy owns the single live camera so desktop launch never races an imported view.
"""

from __future__ import annotations

import argparse
import math
from pathlib import Path
from typing import Iterable

import bmesh
import bpy
from mathutils import Vector


ROOT = Path(__file__).resolve().parents[1]
SCENE_COLL = "CouncilChamber"
DEFAULT_EXPORT = ROOT / "assets" / "scenes" / "lore_chamber.glb"
DEFAULT_RENDER = ROOT / "artifacts" / "visual-proof" / "lore-chamber-2026-07-15_0539" / "blender_lore_chamber.png"

CHAMBER_R_IN = 10.0
WALL_THICK = 0.6
WALL_H = 8.0
COLUMN_R_POS = 8.8
TABLE_R = 2.2
TABLE_TOP_Z = 0.98
CHAIR_R_POS = 3.55
TRIANGLE_LIMIT = 240_000

# Canon pulled from crates/engine/src/theme/constants.rs. Codex/Lexis and Viren
# are real theme nodes, but they are not one of the seven council seats.
ARCHETYPES = [
    {
        "id": "Architect",
        "theme": "Luminous Blueprint",
        "role": "structure",
        "color": (59 / 255, 130 / 255, 246 / 255),
        "secondary": (147 / 255, 197 / 255, 253 / 255),
        "harmonic": "C Major Chord",
    },
    {
        "id": "Sentinel",
        "theme": "Null Aegis",
        "role": "protection",
        "color": (122 / 255, 162 / 255, 247 / 255),
        "secondary": (255 / 255, 95 / 255, 86 / 255),
        "harmonic": "F# Tritone Tension",
    },
    {
        "id": "Jester",
        "theme": "Mulligan Engine",
        "role": "disruption",
        "color": (67 / 255, 56 / 255, 202 / 255),
        "secondary": (185 / 255, 28 / 255, 28 / 255),
        "harmonic": "Dissonant Glitch",
    },
    {
        "id": "Mentor",
        "theme": "Ancient Resonance",
        "role": "meaning",
        "color": (0 / 255, 150 / 255, 136 / 255),
        "secondary": (128 / 255, 203 / 255, 196 / 255),
        "harmonic": "Resonant Hum (Tibetan Bowl)",
    },
    {
        "id": "Explorer",
        "theme": "Frontier Flare",
        "role": "exploration",
        "color": (255 / 255, 136 / 255, 0 / 255),
        "secondary": (255 / 255, 213 / 255, 79 / 255),
        "harmonic": "Rising E Major Arpeggio",
    },
    {
        "id": "Oracle",
        "theme": "Noctis Veil",
        "role": "foresight",
        "color": (92 / 255, 77 / 255, 125 / 255),
        "secondary": (149 / 255, 117 / 255, 205 / 255),
        "harmonic": "B Minor Pad + Distant Bells",
    },
    {
        "id": "Empath",
        "theme": "Luma Resonance",
        "role": "memory",
        "color": (247 / 255, 202 / 255, 201 / 255),
        "secondary": (240 / 255, 98 / 255, 146 / 255),
        "harmonic": "Warm D Major + Ambient Choral",
    },
]


def parse_args() -> argparse.Namespace:
    argv = []
    if "--" in __import__("sys").argv:
        sys_argv = __import__("sys").argv
        argv = sys_argv[sys_argv.index("--") + 1 :]
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default=str(DEFAULT_EXPORT))
    parser.add_argument("--render", default=str(DEFAULT_RENDER))
    parser.add_argument("--no-render", action="store_true")
    return parser.parse_args(argv)


def log(*parts: object) -> None:
    print("LORE_CHAMBER:", *parts)


def get_collection() -> bpy.types.Collection:
    old = bpy.data.collections.get(SCENE_COLL)
    if old:
        for obj in list(old.objects):
            bpy.data.objects.remove(obj, do_unlink=True)
        bpy.data.collections.remove(old)
    coll = bpy.data.collections.new(SCENE_COLL)
    bpy.context.scene.collection.children.link(coll)
    return coll


def set_active_collection(coll: bpy.types.Collection) -> None:
    def find(layer_collection: bpy.types.LayerCollection):
        if layer_collection.collection == coll:
            return layer_collection
        for child in layer_collection.children:
            result = find(child)
            if result:
                return result
        return None

    layer_collection = find(bpy.context.view_layer.layer_collection)
    if layer_collection:
        bpy.context.view_layer.active_layer_collection = layer_collection


def set_input(bsdf: bpy.types.Node, names: Iterable[str], value) -> None:
    for name in names:
        socket = bsdf.inputs.get(name)
        if socket is not None:
            socket.default_value = value
            return


def make_mat(
    name: str,
    color,
    rough: float = 0.8,
    metal: float = 0.0,
    emit=None,
    emit_str: float = 0.0,
) -> bpy.types.Material:
    material = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    material.use_nodes = True
    nodes = material.node_tree.nodes
    nodes.clear()
    out = nodes.new("ShaderNodeOutputMaterial")
    bsdf = nodes.new("ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metal
    if emit is not None:
        set_input(bsdf, ("Emission Color", "Emission"), (*emit, 1.0))
        set_input(bsdf, ("Emission Strength",), emit_str)
    material.node_tree.links.new(bsdf.outputs[0], out.inputs[0])
    material.diffuse_color = (*color, 1.0)
    return material


def assign(obj: bpy.types.Object, material: bpy.types.Material) -> None:
    obj.data.materials.clear()
    obj.data.materials.append(material)


def smooth(obj: bpy.types.Object) -> None:
    for polygon in obj.data.polygons:
        polygon.use_smooth = True


def active() -> bpy.types.Object:
    return bpy.context.active_object


def join_objects(objs: list[bpy.types.Object], name: str) -> bpy.types.Object:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objs:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    result = bpy.context.active_object
    result.name = name
    return result


def apply_boolean(target: bpy.types.Object, cutter: bpy.types.Object, op: str = "DIFFERENCE") -> None:
    modifier = target.modifiers.new("bool", "BOOLEAN")
    modifier.operation = op
    modifier.object = cutter
    modifier.solver = "EXACT"
    bpy.context.view_layer.objects.active = target
    target.select_set(True)
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    bpy.data.objects.remove(cutter, do_unlink=True)


def look_at(obj: bpy.types.Object, target) -> None:
    direction = (Vector(target) - obj.location).normalized()
    obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def to_world(base, angle: float, lx: float, ly: float, lz: float):
    theta = angle - math.pi / 2
    return (
        base[0] + lx * math.cos(theta) - ly * math.sin(theta),
        base[1] + lx * math.sin(theta) + ly * math.cos(theta),
        lz,
    ), theta


def add_cube(name: str, dims, loc, rot_z: float = 0.0, mat: bpy.types.Material | None = None):
    bpy.ops.mesh.primitive_cube_add(size=1, location=loc)
    obj = active()
    obj.name = name
    obj.scale = dims
    obj.rotation_euler = (0, 0, rot_z)
    if mat:
        assign(obj, mat)
    return obj


def add_point_light(
    coll: bpy.types.Collection,
    name: str,
    loc,
    power: float,
    color,
    soft: float = 0.1,
) -> bpy.types.Object:
    light_data = bpy.data.lights.new(name, "POINT")
    light_data.energy = power
    light_data.color = color
    light_data.shadow_soft_size = soft
    obj = bpy.data.objects.new(name, light_data)
    coll.objects.link(obj)
    obj.location = loc
    return obj


def add_torus(name: str, loc, major: float, minor: float, mat: bpy.types.Material, rotate=(0, 0, 0)):
    bpy.ops.mesh.primitive_torus_add(
        location=loc,
        major_radius=major,
        minor_radius=minor,
        major_segments=72,
        minor_segments=8,
        rotation=rotate,
    )
    obj = active()
    obj.name = name
    assign(obj, mat)
    smooth(obj)
    return obj


def tag_lore(obj: bpy.types.Object, archetype: dict[str, object]) -> None:
    obj["archetype"] = archetype["id"]
    obj["theme"] = archetype["theme"]
    obj["role"] = archetype["role"]
    obj["harmonic_signature"] = archetype["harmonic"]


def add_sigil_orb(coll, archetype: dict[str, object], base, angle: float, materials: dict[str, bpy.types.Material]) -> None:
    color = archetype["color"]
    secondary = archetype["secondary"]
    arch_id = archetype["id"]
    orb_mat = make_mat(f"LC_Orb_{arch_id}_Canon", color, emit=color, emit_str=7.0)
    sigil_mat = make_mat(f"LC_Sigil_{arch_id}_Canon", secondary, rough=0.42, metal=0.35, emit=secondary, emit_str=4.5)

    loc, theta = to_world(base, angle, 0.0, -0.17, 1.02)
    bpy.ops.mesh.primitive_uv_sphere_add(segments=24, ring_count=12, radius=0.065, location=loc)
    orb = active()
    orb.name = f"LC_Orb_{arch_id}"
    assign(orb, orb_mat)
    smooth(orb)
    tag_lore(orb, archetype)

    # Small, readable mesh sigil tokens. They are intentionally symbolic stand-ins
    # rather than fake final portraits: geometry survives GLB export.
    if arch_id == "Architect":
        add_torus(f"LC_Sigil_{arch_id}_BlueprintRing", (loc[0], loc[1], loc[2]), 0.13, 0.006, sigil_mat, (math.pi / 2, 0, 0))
        add_torus(f"LC_Sigil_{arch_id}_BlueprintAxis", (loc[0], loc[1], loc[2]), 0.13, 0.006, sigil_mat, (0, math.pi / 2, 0))
    elif arch_id == "Sentinel":
        shield = add_cube(f"LC_Sigil_{arch_id}_AegisBar", (0.20, 0.018, 0.09), (loc[0], loc[1], loc[2]), theta, sigil_mat)
        tag_lore(shield, archetype)
    elif arch_id == "Jester":
        a = add_cube(f"LC_Sigil_{arch_id}_FaultA", (0.20, 0.014, 0.035), (loc[0], loc[1], loc[2] + 0.02), theta + 0.55, sigil_mat)
        b = add_cube(f"LC_Sigil_{arch_id}_FaultB", (0.20, 0.014, 0.035), (loc[0], loc[1], loc[2] - 0.02), theta - 0.55, sigil_mat)
        tag_lore(a, archetype)
        tag_lore(b, archetype)
    elif arch_id == "Mentor":
        add_torus(f"LC_Sigil_{arch_id}_ResonanceBowl", (loc[0], loc[1], loc[2] - 0.035), 0.12, 0.007, sigil_mat)
    elif arch_id == "Explorer":
        bpy.ops.mesh.primitive_cone_add(vertices=3, radius1=0.12, depth=0.20, location=(loc[0], loc[1], loc[2] + 0.01), rotation=(math.pi / 2, 0, theta))
        arrow = active()
        arrow.name = f"LC_Sigil_{arch_id}_FrontierArrow"
        assign(arrow, sigil_mat)
        tag_lore(arrow, archetype)
    elif arch_id == "Oracle":
        add_torus(f"LC_Sigil_{arch_id}_Eye", (loc[0], loc[1], loc[2]), 0.12, 0.006, sigil_mat, (math.pi / 2, 0, 0))
        bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=8, radius=0.035, location=(loc[0], loc[1], loc[2]))
        eye = active()
        eye.name = f"LC_Sigil_{arch_id}_Pupil"
        assign(eye, sigil_mat)
        tag_lore(eye, archetype)
    elif arch_id == "Empath":
        bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=8, radius=0.055, location=(loc[0] - 0.045, loc[1], loc[2] + 0.03))
        left = active()
        bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=8, radius=0.055, location=(loc[0] + 0.045, loc[1], loc[2] + 0.03))
        right = active()
        bpy.ops.mesh.primitive_cone_add(vertices=24, radius1=0.085, radius2=0.0, depth=0.12, location=(loc[0], loc[1], loc[2] - 0.045), rotation=(math.pi, 0, 0))
        point = active()
        for obj in (left, right, point):
            obj.name = f"LC_Sigil_{arch_id}_HeartPart"
            assign(obj, sigil_mat)
            smooth(obj)
            tag_lore(obj, archetype)

    add_point_light(coll, f"LC_OrbLight_{arch_id}", loc, 35.0, color, soft=0.22)


def build_scene(export_path: Path, render_path: Path | None) -> None:
    try:
        bpy.ops.object.mode_set(mode="OBJECT")
    except RuntimeError:
        pass

    coll = get_collection()
    set_active_collection(coll)

    materials = {
        "ground": make_mat("LC_Ground", (0.025, 0.023, 0.021), rough=0.96),
        "floor": make_mat("LC_Floor", (0.13, 0.12, 0.105), rough=0.66),
        "wall": make_mat("LC_Wall", (0.145, 0.125, 0.105), rough=0.9),
        "panel": make_mat("LC_FrescoRelief", (0.19, 0.16, 0.125), rough=0.93),
        "dome": make_mat("LC_Dome", (0.075, 0.070, 0.064), rough=0.92),
        "column": make_mat("LC_Column", (0.17, 0.15, 0.12), rough=0.84),
        "stone": make_mat("LC_ThroneStone", (0.135, 0.125, 0.115), rough=0.78),
        "wood": make_mat("LC_DarkWood", (0.12, 0.07, 0.04), rough=0.72),
        "gold": make_mat("LC_SoftGold", (0.84, 0.63, 0.22), rough=0.3, metal=1.0),
        "body": make_mat("LC_WitnessBody", (0.075, 0.080, 0.095), rough=0.42, metal=0.4),
        "flame": make_mat("LC_Flame", (1.0, 0.48, 0.10), emit=(1.0, 0.45, 0.12), emit_str=14.0),
        "seed": make_mat("LC_SeedGlow", (0.16, 0.62, 1.0), emit=(0.20, 0.65, 1.0), emit_str=6.0),
        "ringglow": make_mat("LC_RingGlow", (0.12, 0.42, 0.82), emit=(0.15, 0.50, 1.0), emit_str=2.2),
        "city": make_mat("LC_CityGlow", (0.36, 0.52, 0.86), emit=(0.32, 0.54, 1.0), emit_str=1.35),
        "skyline": make_mat("LC_CitySilhouette", (0.018, 0.021, 0.032), rough=0.82),
    }

    bpy.ops.mesh.primitive_plane_add(size=80, location=(0, 0, -0.05))
    ground = active()
    ground.name = "LC_GroundPlane_80m"
    assign(ground, materials["ground"])

    bpy.ops.mesh.primitive_cylinder_add(vertices=128, radius=CHAMBER_R_IN + WALL_THICK, depth=0.4, location=(0, 0, -0.2))
    floor = active()
    floor.name = "LC_FloorSlab_21m"
    assign(floor, materials["floor"])
    smooth(floor)

    bpy.ops.mesh.primitive_cylinder_add(vertices=128, radius=CHAMBER_R_IN, depth=WALL_H, location=(0, 0, WALL_H / 2), end_fill_type="NOTHING")
    wall = active()
    wall.name = "LC_SolidRingWall_21m"
    assign(wall, materials["wall"])
    smooth(wall)
    solid = wall.modifiers.new("solid_wall_thickness_0p6m", "SOLIDIFY")
    solid.thickness = WALL_THICK
    solid.offset = 1.0
    bpy.context.view_layer.objects.active = wall
    bpy.ops.object.modifier_apply(modifier=solid.name)

    bpy.ops.mesh.primitive_cube_add(size=1, location=(0, CHAMBER_R_IN + WALL_THICK / 2, 1.9))
    cut_box = active()
    cut_box.name = "LC_WindowCut_Rect"
    cut_box.scale = (2.4, 2.5, 3.2)
    apply_boolean(wall, cut_box)
    bpy.ops.mesh.primitive_cylinder_add(vertices=64, radius=1.2, depth=2.5, location=(0, CHAMBER_R_IN + WALL_THICK / 2, 3.5), rotation=(math.pi / 2, 0, 0))
    cut_cyl = active()
    cut_cyl.name = "LC_WindowCut_Arch"
    apply_boolean(wall, cut_cyl)

    mesh = bpy.data.meshes.new("LC_DomeMesh")
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=72, v_segments=36, radius=CHAMBER_R_IN + WALL_THICK)
    below = [vertex for vertex in bm.verts if vertex.co.z < -1e-4]
    bmesh.ops.delete(bm, geom=below, context="VERTS")
    bm.to_mesh(mesh)
    bm.free()
    dome = bpy.data.objects.new("LC_HemisphericalDome", mesh)
    coll.objects.link(dome)
    dome.location = (0, 0, WALL_H)
    dome.scale = (1.0, 1.0, 0.72)
    assign(dome, materials["dome"])
    smooth(dome)
    dome_solid = dome.modifiers.new("dome_shell_thickness", "SOLIDIFY")
    dome_solid.thickness = 0.18
    dome_solid.offset = 0.0
    bpy.context.view_layer.objects.active = dome
    bpy.ops.object.modifier_apply(modifier=dome_solid.name)

    for index in range(8):
        angle = -math.pi / 2 + math.pi / 8 + index * math.pi / 4
        cx, cy = COLUMN_R_POS * math.cos(angle), COLUMN_R_POS * math.sin(angle)
        bpy.ops.mesh.primitive_cylinder_add(vertices=28, radius=0.33, depth=7.5, location=(cx, cy, 3.75))
        column = active()
        column.name = f"LC_Column_{index:02d}"
        assign(column, materials["column"])
        smooth(column)

        bpy.ops.mesh.primitive_cylinder_add(vertices=28, radius=0.45, depth=0.3, location=(cx, cy, 0.15))
        base = active()
        base.name = f"LC_ColumnBase_{index:02d}"
        assign(base, materials["column"])
        smooth(base)

        add_cube(f"LC_ColumnCapital_{index:02d}", (0.85, 0.85, 0.4), (cx, cy, 7.7), rot_z=angle, mat=materials["column"])

        torch_radius = COLUMN_R_POS - 0.45
        tx, ty = torch_radius * math.cos(angle), torch_radius * math.sin(angle)
        bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=0.03, depth=0.35, location=(tx, ty, 3.05))
        stick = active()
        stick.name = f"LC_TorchStick_{index:02d}"
        assign(stick, materials["wood"])
        bpy.ops.mesh.primitive_cone_add(vertices=16, radius1=0.08, radius2=0.0, depth=0.28, location=(tx, ty, 3.36))
        flame = active()
        flame.name = f"LC_TorchFlame_{index:02d}"
        assign(flame, materials["flame"])
        smooth(flame)
        add_point_light(coll, f"LC_TorchLight_{index:02d}", (tx * 0.98, ty * 0.98, 3.48), 120, (1.0, 0.55, 0.22), soft=0.12)

    for index in range(8):
        angle = -math.pi / 2 + index * math.pi / 4
        if abs(angle - math.pi / 2) < 1e-3:
            continue
        px, py = (CHAMBER_R_IN - 0.06) * math.cos(angle), (CHAMBER_R_IN - 0.06) * math.sin(angle)
        relief = add_cube(f"LC_ReliefPanel_{index:02d}", (2.35, 0.10, 2.85), (px, py, 3.6), rot_z=angle + math.pi / 2, mat=materials["panel"])
        relief["runtime_role"] = "fresco_stand_in"

    bpy.ops.mesh.primitive_cylinder_add(vertices=72, radius=0.55, depth=TABLE_TOP_Z - 0.12, location=(0, 0, (TABLE_TOP_Z - 0.12) / 2))
    pedestal = active()
    pedestal.name = "LC_TablePedestal"
    assign(pedestal, materials["stone"])
    smooth(pedestal)
    bpy.ops.mesh.primitive_cylinder_add(vertices=128, radius=TABLE_R, depth=0.12, location=(0, 0, TABLE_TOP_Z - 0.06))
    top = active()
    top.name = "LC_TableTop_SeedOfLife"
    assign(top, materials["stone"])
    smooth(top)
    add_torus("LC_TableGoldRim", (0, 0, TABLE_TOP_Z), TABLE_R, 0.045, materials["gold"])

    seed_radius = 0.72
    seed_centers = [(0.0, 0.0)] + [
        (seed_radius * math.cos(index * math.pi / 3), seed_radius * math.sin(index * math.pi / 3))
        for index in range(6)
    ]
    for index, (sx, sy) in enumerate(seed_centers):
        add_torus(f"LC_SeedOfLife_Circle_{index}", (sx, sy, TABLE_TOP_Z + 0.012), seed_radius, 0.014, materials["seed"])
    add_torus("LC_SeedOfLife_Boundary", (0, 0, TABLE_TOP_Z + 0.012), 2 * seed_radius, 0.016, materials["gold"])
    add_torus("LC_WitnessFloorRing", (0, 0, 0.012), 3.9, 0.020, materials["ringglow"])

    for index, archetype in enumerate(ARCHETYPES):
        angle = -math.pi / 2 + (index + 1) * math.pi / 4
        base = (CHAIR_R_POS * math.cos(angle), CHAIR_R_POS * math.sin(angle), 0.0)
        arch_id = archetype["id"]

        throne_parts = []
        for name, dims, local in [
            ("plinth", (0.80, 0.78, 0.30), (0.0, 0.00, 0.15)),
            ("seat", (0.72, 0.70, 0.12), (0.0, 0.02, 0.51)),
            ("back", (0.72, 0.14, 1.70), (0.0, 0.32, 1.35)),
            ("arm_l", (0.12, 0.60, 0.55), (-0.41, 0.00, 0.62)),
            ("arm_r", (0.12, 0.60, 0.55), (0.41, 0.00, 0.62)),
        ]:
            loc, theta = to_world(base, angle, *local)
            throne_parts.append(add_cube(f"tmp_{arch_id}_{name}", dims, loc, rot_z=theta, mat=materials["stone"]))
        throne = join_objects(throne_parts, f"LC_Throne_{arch_id}")
        tag_lore(throne, archetype)

        body_parts = []
        loc, _ = to_world(base, angle, 0.0, 0.05, 0.88)
        bpy.ops.mesh.primitive_cylinder_add(vertices=20, radius=0.17, depth=0.60, location=loc)
        torso = active()
        assign(torso, materials["body"])
        smooth(torso)
        body_parts.append(torso)
        loc, _ = to_world(base, angle, 0.0, 0.03, 1.32)
        bpy.ops.mesh.primitive_uv_sphere_add(segments=20, ring_count=12, radius=0.13, location=loc)
        head = active()
        assign(head, materials["body"])
        smooth(head)
        body_parts.append(head)
        loc, theta = to_world(base, angle, 0.0, -0.22, 0.60)
        body_parts.append(add_cube(f"tmp_{arch_id}_thighs", (0.32, 0.42, 0.13), loc, rot_z=theta, mat=materials["body"]))
        loc, theta = to_world(base, angle, 0.0, -0.42, 0.32)
        body_parts.append(add_cube(f"tmp_{arch_id}_shins", (0.30, 0.12, 0.50), loc, rot_z=theta, mat=materials["body"]))
        figure = join_objects(body_parts, f"LC_Fig_{arch_id}")
        tag_lore(figure, archetype)

        add_sigil_orb(coll, archetype, base, angle, materials)

    bpy.ops.mesh.primitive_plane_add(size=2, location=(0, 14.5, 4.0), rotation=(math.pi / 2, 0, 0))
    backdrop = active()
    backdrop.name = "LC_CityGlowBackdrop_Window"
    backdrop.scale = (5.5, 4.0, 1.0)
    assign(backdrop, materials["city"])

    skyline = [
        (-1.85, 14.38, 2.70, 0.42, 0.08, 1.35),
        (-1.30, 14.37, 2.50, 0.32, 0.08, 0.95),
        (-0.70, 14.36, 2.82, 0.44, 0.08, 1.60),
        (-0.12, 14.35, 2.42, 0.28, 0.08, 0.78),
        (0.43, 14.36, 2.65, 0.36, 0.08, 1.22),
        (1.05, 14.37, 2.45, 0.30, 0.08, 0.88),
        (1.62, 14.38, 2.76, 0.48, 0.08, 1.45),
    ]
    for index, (x, y, z, sx, sy, sz) in enumerate(skyline):
        add_cube(
            f"LC_CitySilhouette_{index:02d}",
            (sx, sy, sz),
            (x, y, z),
            mat=materials["skyline"],
        )

    sun_data = bpy.data.lights.new("LC_WindowSun", "SUN")
    sun_data.energy = 2.8
    sun_data.color = (0.75, 0.85, 1.0)
    sun_data.angle = 0.02
    sun = bpy.data.objects.new("LC_WindowSun", sun_data)
    coll.objects.link(sun)
    sun.location = (0, 13.0, 9.0)
    look_at(sun, (0, 0, 1.0))

    add_point_light(coll, "LC_TableFill", (0, 0, 5.0), 210, (0.55, 0.70, 1.0), soft=0.6)

    cam_data = bpy.data.cameras.new("Witness_Camera")
    cam_data.lens = 32
    cam_data.clip_end = 120
    cam = bpy.data.objects.new("Witness_Camera", cam_data)
    coll.objects.link(cam)
    cam.location = (0, -7.4, 1.85)
    look_at(cam, (0, 0, 1.05))
    bpy.context.scene.camera = cam

    high_data = bpy.data.cameras.new("LC_Cam_High")
    high_data.lens = 24
    high_data.clip_end = 160
    high = bpy.data.objects.new("LC_Cam_High", high_data)
    coll.objects.link(high)
    high.location = (6.5, -6.8, 6.2)
    look_at(high, (0, 0, 1.0))

    world = bpy.data.worlds.get("LC_World") or bpy.data.worlds.new("LC_World")
    world.use_nodes = True
    background = world.node_tree.nodes.get("Background")
    if background:
        background.inputs[0].default_value = (0.004, 0.006, 0.012, 1.0)
        background.inputs[1].default_value = 1.0
    bpy.context.scene.world = world

    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.samples = 64
    bpy.context.scene.render.resolution_x = 1920
    bpy.context.scene.render.resolution_y = 1080
    bpy.context.scene.view_settings.view_transform = "Filmic"
    bpy.context.scene.view_settings.look = "Medium High Contrast"
    bpy.context.scene.view_settings.exposure = 0
    bpy.context.scene.view_settings.gamma = 1

    export_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(export_path),
        export_format="GLB",
        use_selection=False,
        export_lights=True,
        export_cameras=False,
        export_apply=True,
        export_yup=True,
    )
    log(f"exported={export_path}")

    if render_path is not None:
        render_path.parent.mkdir(parents=True, exist_ok=True)
        bpy.context.scene.render.filepath = str(render_path)
        bpy.ops.render.render(write_still=True)
        log(f"rendered={render_path}")


def triangle_count() -> int:
    return sum(
        max(0, len(poly.vertices) - 2)
        for obj in bpy.context.scene.objects
        if obj.type == "MESH"
        for poly in obj.data.polygons
    )


def verify_export(path: Path) -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))
    names = {obj.name for obj in bpy.context.scene.objects}
    missing = []
    for archetype in ARCHETYPES:
        arch_id = archetype["id"]
        for required in (f"LC_Throne_{arch_id}", f"LC_Fig_{arch_id}", f"LC_Orb_{arch_id}"):
            if required not in names:
                missing.append(required)
    for required in ("LC_TableTop_SeedOfLife", "LC_SeedOfLife_Boundary", "LC_HemisphericalDome"):
        if required not in names:
            missing.append(required)

    lights = [obj for obj in bpy.context.scene.objects if obj.type == "LIGHT"]
    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    point_lights = [obj for obj in lights if obj.data.type == "POINT"]
    triangles = triangle_count()
    log(
        "verify",
        f"objects={len(bpy.context.scene.objects)}",
        f"meshes={sum(1 for obj in bpy.context.scene.objects if obj.type == 'MESH')}",
        f"lights={len(lights)}",
        f"cameras={len(cameras)}",
        f"point_lights={len(point_lights)}",
        f"triangles={triangles}",
        f"missing={missing}",
    )
    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if missing:
        raise RuntimeError(f"lore chamber export missing nodes: {missing}")
    if len(point_lights) < 16:
        raise RuntimeError(f"expected at least 16 point lights, found {len(point_lights)}")
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    render_path = None if args.no_render else Path(args.render).resolve()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_scene(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
