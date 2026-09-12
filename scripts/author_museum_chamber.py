"""Author one walkable museum chamber for the Inner Castle wall.

Twelve of the ground gallery's seventy-two arches are real openings rather than blind panels.
This is what is behind them: a room cut 9m into the 12m wall, the same clear width as the arch
in front of it, roofed by a barrel vault that springs from the same 5.5m line as that arch — so
the vault outside and the vault inside are one continuous piece of geometry rather than two
ideas meeting at a seam.

Why 9m and not 12m: the wall is load-bearing to the fiction as well as to the geometry. Cutting
all the way through would leave piers standing between holes and the building would read as
scaffolding. Three metres of masonry stay behind every chamber.

Geometry contract with `crates/engine/src/modes/inner_chambers/castle.rs`:

  clear opening width   7.7452 m   (arcade_bay_width() - ARCADE_PIER_WIDTH)
  chamber depth         9.0 m      (MUSEUM_CHAMBER_DEPTH)
  springing line        5.5 m      (the arch outside springs from the same height)
  wall thickness       12.0 m      (CASTLE_WALL_THICKNESS)

Local frame, matching `author_arcade_bay.py` exactly so the engine can place both with the same
transform:

  +X  along the wall, chamber centred on X = 0
  +Y  radially into the wall; the opening is at Y = 0 and the chamber runs to Y = +9
  +Z  up, 0 at this gallery's deck

The arcade bay projects toward the hall in -Y from the same origin, so bay and chamber meet at
Y = 0 with no gap and no overlap.

**Hanging positions are not authored here.** `castle.rs` computes them, because `castle.rs` is
the only place dimensions live in this repo and a position baked into a GLB would be a second
source of truth that could drift from the collision that has to agree with it.

Run headless:

  blender --background --python scripts/author_museum_chamber.py -- \
      --export assets/scenes/museum_chamber.glb --no-render
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

import bmesh
import bpy
from mathutils import Vector

# --- contract with castle.rs -----------------------------------------------------------
BAYS_PER_LEVEL = 72
WALL_FACE_RADIUS = 114.0
PIER_WIDTH = 2.2
BAY_WIDTH = 2.0 * WALL_FACE_RADIUS * math.sin(math.pi / BAYS_PER_LEVEL)
OPENING_WIDTH = BAY_WIDTH - PIER_WIDTH
CHAMBER_DEPTH = 9.0
WALL_THICKNESS = 12.0
SPRING_LINE = 5.5

# Must match `STONE_TILE_METRES` in the other kit scripts and `stone.rs`.
STONE_TILE_METRES = 4.0

HALF_WIDTH = OPENING_WIDTH * 0.5
VAULT_RADIUS = HALF_WIDTH
VAULT_CROWN = SPRING_LINE + VAULT_RADIUS
FLOOR_THICKNESS = 0.45
WALL_THICK = 0.5
VAULT_SHELL = 0.45
VAULT_SEGMENTS = 24

TRIANGLE_LIMIT = 6000


def log(*parts: object) -> None:
    print("[museum-chamber]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default="assets/scenes/museum_chamber.glb")
    parser.add_argument(
        "--render", default="artifacts/visual-proof/museum_chamber_blender.png"
    )
    parser.add_argument("--no-render", action="store_true")
    return parser.parse_args(argv)


def stone_material(name: str, base: tuple[float, float, float], roughness: float):
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    principled = material.node_tree.nodes["Principled BSDF"]
    principled.inputs["Base Color"].default_value = (*base, 1.0)
    principled.inputs["Roughness"].default_value = roughness
    principled.inputs["Metallic"].default_value = 0.0
    return material


def add_box(name, material, centre, size):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=centre)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(material)
    return obj


def add_barrel_vault(name, material):
    """A half-cylinder shell running along +Y: the chamber's roof.

    Modelled as a real shell with thickness rather than a single-sided surface, so the vault
    has an underside a player standing inside actually sees lit, and an outside that meets the
    masonry above it.
    """
    mesh = bpy.data.meshes.new(name)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)

    inner = VAULT_RADIUS
    outer = VAULT_RADIUS + VAULT_SHELL
    y_front = 0.0
    y_back = CHAMBER_DEPTH

    verts: list[tuple[float, float, float]] = []
    for index in range(VAULT_SEGMENTS + 1):
        angle = math.pi * index / VAULT_SEGMENTS
        cos_a, sin_a = math.cos(angle), math.sin(angle)
        for y in (y_front, y_back):
            # Angle runs from +X round to -X, so the arc spans the opening and rises to the
            # crown at its midpoint.
            verts.append((inner * cos_a, y, SPRING_LINE + inner * sin_a))
            verts.append((outer * cos_a, y, SPRING_LINE + outer * sin_a))

    def ring(index: int) -> tuple[int, int, int, int]:
        base = index * 4
        return base, base + 1, base + 2, base + 3

    faces: list[tuple[int, ...]] = []
    for index in range(VAULT_SEGMENTS):
        a_if, a_of, a_ib, a_ob = ring(index)
        b_if, b_of, b_ib, b_ob = ring(index + 1)
        faces.append((a_if, b_if, b_of, a_of))  # front edge
        faces.append((a_ib, a_ob, b_ob, b_ib))  # back edge
        faces.append((a_if, a_ib, b_ib, b_if))  # intrados: the ceiling seen from inside
        faces.append((a_of, b_of, b_ob, a_ob))  # extrados
    first = ring(0)
    last = ring(VAULT_SEGMENTS)
    faces.append((first[0], first[1], first[3], first[2]))
    faces.append((last[0], last[2], last[3], last[1]))

    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    mesh.update()
    obj.data.materials.append(material)

    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    return obj


def bevel_and_unwrap(obj, width: float = 0.05) -> None:
    bpy.context.view_layer.objects.active = obj
    modifier = obj.modifiers.new(name="Bevel", type="BEVEL")
    modifier.width = width
    modifier.segments = 2
    modifier.limit_method = "ANGLE"
    modifier.angle_limit = math.radians(35.0)
    bpy.ops.object.modifier_apply(modifier=modifier.name)

    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    # World-scaled, same as every other kit module: one block of stone is one size everywhere
    # in the building, including inside a chamber where the player stands closest to it.
    bpy.ops.uv.cube_project(cube_size=STONE_TILE_METRES)
    bpy.ops.object.mode_set(mode="OBJECT")
    obj.select_set(False)


def build_chamber() -> list:
    limestone = stone_material("ChamberLimestone", (0.62, 0.58, 0.51), 0.86)
    trim = stone_material("ChamberTrim", (0.47, 0.42, 0.35), 0.78)
    # The hanging wall is deliberately a shade flatter and cooler: a painting hung on the same
    # bright limestone as everything else has nothing to sit against.
    hanging = stone_material("ChamberHangingWall", (0.40, 0.385, 0.36), 0.92)

    built = []
    mid_depth = CHAMBER_DEPTH * 0.5

    # Floor slab, its top face exactly on the deck plane so there is no step at the threshold.
    built.append(
        add_box(
            "Chamber_Floor",
            trim,
            (0.0, mid_depth, -FLOOR_THICKNESS * 0.5),
            (OPENING_WIDTH, CHAMBER_DEPTH, FLOOR_THICKNESS),
        )
    )

    # Side walls, outside the clear opening so the interior width stays the arch's width.
    for side, label in ((-1.0, "Left"), (1.0, "Right")):
        built.append(
            add_box(
                f"Chamber_SideWall_{label}",
                hanging,
                (side * (HALF_WIDTH + WALL_THICK * 0.5), mid_depth, SPRING_LINE * 0.5),
                (WALL_THICK, CHAMBER_DEPTH, SPRING_LINE),
            )
        )
        # A plinth course along the foot of each hanging wall, so a frame has something to sit
        # above rather than floating on a blank slab.
        built.append(
            add_box(
                f"Chamber_Plinth_{label}",
                trim,
                (side * (HALF_WIDTH - 0.12), mid_depth, 0.55),
                (0.30, CHAMBER_DEPTH, 1.10),
            )
        )

    built.append(
        add_box(
            "Chamber_BackWall",
            hanging,
            (0.0, CHAMBER_DEPTH + WALL_THICK * 0.5, VAULT_CROWN * 0.5),
            (OPENING_WIDTH + WALL_THICK * 2.0, WALL_THICK, VAULT_CROWN),
        )
    )
    built.append(
        add_box(
            "Chamber_BackPlinth",
            trim,
            (0.0, CHAMBER_DEPTH - 0.12, 0.55),
            (OPENING_WIDTH, 0.30, 1.10),
        )
    )

    built.append(add_barrel_vault("Chamber_Vault", limestone))

    # Impost band at the springing line, running the depth of both walls. It is what makes the
    # vault read as resting on the walls instead of merging into them.
    for side, label in ((-1.0, "Left"), (1.0, "Right")):
        built.append(
            add_box(
                f"Chamber_Impost_{label}",
                trim,
                (side * (HALF_WIDTH - 0.10), mid_depth, SPRING_LINE - 0.16),
                (0.36, CHAMBER_DEPTH, 0.32),
            )
        )

    for obj in built:
        bevel_and_unwrap(obj)

    log(
        "built",
        f"opening={OPENING_WIDTH:.4f}",
        f"depth={CHAMBER_DEPTH}",
        f"spring={SPRING_LINE}",
        f"crown={VAULT_CROWN:.4f}",
        f"objects={len(built)}",
    )
    return built


def export(export_path: Path, render_path: Path | None) -> None:
    export_path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=str(export_path),
        export_format="GLB",
        use_selection=False,
        export_lights=False,
        export_cameras=False,
        export_apply=True,
        export_yup=True,
    )
    log(f"exported={export_path}")

    if render_path is not None:
        render_path.parent.mkdir(parents=True, exist_ok=True)
        bpy.context.scene.render.engine = "BLENDER_EEVEE_NEXT"
        bpy.context.scene.render.resolution_x = 1280
        bpy.context.scene.render.resolution_y = 1280
        bpy.context.scene.render.filepath = str(render_path)
        bpy.ops.render.render(write_still=True)
        log(f"rendered={render_path}")


def triangle_count() -> int:
    return sum(
        len(polygon.vertices) - 2
        for obj in bpy.context.scene.objects
        if obj.type == "MESH"
        for polygon in obj.data.polygons
    )


def verify_export(path: Path) -> None:
    """Re-import what was written and check it against the contract, not against intent."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    names = {obj.name.split(".")[0] for obj in meshes}
    required = {
        "Chamber_Floor",
        "Chamber_SideWall_Left",
        "Chamber_SideWall_Right",
        "Chamber_BackWall",
        "Chamber_Vault",
    }
    missing = sorted(required - names)
    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    triangles = triangle_count()
    uv_missing = [obj.name for obj in meshes if not obj.data.uv_layers]

    def named(stem: str):
        return [obj for obj in meshes if obj.name.split(".")[0] == stem]

    def extent(objects, axis: int) -> tuple[float, float]:
        points = [obj.matrix_world @ Vector(c) for obj in objects for c in obj.bound_box]
        values = [p[axis] for p in points]
        return min(values), max(values)

    floor_x_low, floor_x_high = extent(named("Chamber_Floor"), 0)
    floor_width = floor_x_high - floor_x_low
    all_y_low, all_y_high = extent(meshes, 1)
    all_z_low, all_z_high = extent(meshes, 2)
    vault_z_low, vault_z_high = extent(named("Chamber_Vault"), 2)

    log(
        "verify",
        f"meshes={len(meshes)}",
        f"triangles={triangles}",
        f"cameras={len(cameras)}",
        f"uv_missing={uv_missing}",
        f"floor_width={floor_width:.4f}",
        f"depth_y=({all_y_low:.3f},{all_y_high:.3f})",
        f"height_z=({all_z_low:.3f},{all_z_high:.3f})",
        f"vault_spring={vault_z_low:.3f}",
        f"vault_crown={vault_z_high:.3f}",
        f"missing={missing}",
    )

    if missing:
        raise RuntimeError(f"museum chamber export missing nodes: {missing}")
    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if uv_missing:
        raise RuntimeError(f"every chamber mesh needs UVs for stone; missing on {uv_missing}")
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")

    # The interior must be exactly as wide as the arch in front of it, or the player walks
    # through an opening into a room of a different size.
    if abs(floor_width - OPENING_WIDTH) > 0.02:
        raise RuntimeError(
            f"chamber floor is {floor_width:.4f}m wide against a {OPENING_WIDTH:.4f}m arch "
            "opening"
        )

    # It must not cut through the wall.
    if all_y_high > WALL_THICKNESS - 2.5:
        raise RuntimeError(
            f"chamber reaches {all_y_high:.3f}m into a {WALL_THICKNESS}m wall, leaving less "
            "than 2.5m of masonry behind it"
        )
    if all_y_low < -0.01:
        raise RuntimeError(
            f"chamber projects {-all_y_low:.3f}m out into the hall, through its own arch"
        )

    # The vault must spring from the same line as the arch outside, or the ceiling inside and
    # the arch head outside are two different pieces of architecture meeting at the threshold.
    if abs(vault_z_low - SPRING_LINE) > 0.05:
        raise RuntimeError(
            f"vault springs at {vault_z_low:.3f}m, not the arch's {SPRING_LINE}m"
        )
    if abs(vault_z_high - (VAULT_CROWN + VAULT_SHELL)) > 0.05:
        raise RuntimeError(
            f"vault crowns at {vault_z_high:.3f}m, not the expected "
            f"{VAULT_CROWN + VAULT_SHELL:.3f}m"
        )

    # Standing room. A 3.25m eye height needs the springing line well clear of it.
    if SPRING_LINE < 3.6:
        raise RuntimeError(
            f"a {SPRING_LINE}m springing line puts the vault in the player's face"
        )


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    render_path = None if args.no_render else Path(args.render).resolve()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_chamber()
    export(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
