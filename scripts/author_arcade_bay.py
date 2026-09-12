"""Author one arcade bay for the Inner Castle perimeter wall.

This is the first module of a Blender kit that replaces the procedural Bevy primitives the
castle galleries are currently built from. The engine keeps `castle.rs` as the placement
layer — it already computes every radius, level height and bearing — and this supplies the
thing that gets placed.

Why a kit at all: the current arcade is un-bevelled `Cuboid`s. A sharp 90 degree edge catches
no highlight, boxes have no UVs so no stone texture can sit on them, and an arcade is arches,
which cannot be made from boxes at all. Everything here is bevelled, UV'd and built to tile.

Geometry contract with `crates/engine/src/modes/inner_chambers/castle.rs`:

  bays per level     72
  wall face radius   114.0 m   (GALLERY_OUTER_RADIUS)
  storey height       12.0 m   (GALLERY_RISE)
  bay chord width    2 * 114 * sin(pi / 72) = 9.9448 m

Local frame (Blender, Z up), matching how the engine places it:

  +X  along the wall, bay centred on X = 0, spanning +/- width/2
  +Y  radially into the wall; the bay's back face sits on Y = 0 and it projects toward the
      hall in -Y, so placing the back face at radius 114 seats it on the wall's inner face
  +Z  up, 0 at this gallery's deck, 12 at the deck above

The pier sits on the bay's left edge so that arraying bays produces one solid pier per joint
rather than two half piers meeting in a seam.

Run headless:

  blender --background --python scripts/author_arcade_bay.py -- \
      --export assets/scenes/arcade_bay.glb --no-render
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
STOREY_HEIGHT = 12.0
BAY_WIDTH = 2.0 * WALL_FACE_RADIUS * math.sin(math.pi / BAYS_PER_LEVEL)

# --- bay proportions -------------------------------------------------------------------
PIER_WIDTH = 2.2
PIER_DEPTH = 1.2
PLINTH_HEIGHT = 0.9
CAPITAL_HEIGHT = 0.6
SPRING_LINE = 5.5           # height the arch springs from
ARCH_DEPTH = 1.45           # must exceed PIER_DEPTH or the arch hides behind the pier
ARCH_THICKNESS = 0.9        # voussoir depth, extrados minus intrados
RECESS_DEPTH = 0.9          # the blind panel sits well back, so the arch ring reads
CORNICE_BASE = 10.6
TRIANGLE_LIMIT = 8000

OPENING_WIDTH = BAY_WIDTH - PIER_WIDTH
ARCH_RADIUS = OPENING_WIDTH * 0.5
ARCH_CROWN = SPRING_LINE + ARCH_RADIUS + ARCH_THICKNESS + 0.35


def log(*parts: object) -> None:
    print("[arcade-bay]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default="assets/scenes/arcade_bay.glb")
    parser.add_argument("--render", default="artifacts/visual-proof/arcade_bay_blender.png")
    parser.add_argument("--no-render", action="store_true")
    return parser.parse_args(argv)


def stone_material(name: str, base: tuple[float, float, float], roughness: float) -> bpy.types.Material:
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    principled = material.node_tree.nodes["Principled BSDF"]
    principled.inputs["Base Color"].default_value = (*base, 1.0)
    principled.inputs["Roughness"].default_value = roughness
    principled.inputs["Metallic"].default_value = 0.0
    return material


def add_box(
    name: str,
    material: bpy.types.Material,
    centre: tuple[float, float, float],
    size: tuple[float, float, float],
) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=centre)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(material)
    return obj


def add_arch(
    name: str,
    material: bpy.types.Material,
    segments: int = 28,
    radius_offset: float = 0.0,
    thickness: float = ARCH_THICKNESS,
    depth: float = ARCH_DEPTH,
) -> bpy.types.Object:
    """A half-annulus prism: the real voussoir ring of a semicircular arch."""
    mesh = bpy.data.meshes.new(name)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)

    inner = ARCH_RADIUS + radius_offset
    outer = inner + thickness
    y_front = -depth
    y_back = 0.0

    verts: list[tuple[float, float, float]] = []
    for index in range(segments + 1):
        angle = math.pi * index / segments
        cos_a, sin_a = math.cos(angle), math.sin(angle)
        for y in (y_front, y_back):
            verts.append((inner * cos_a, y, SPRING_LINE + inner * sin_a))
            verts.append((outer * cos_a, y, SPRING_LINE + outer * sin_a))

    # Four vertices per ring: inner-front, outer-front, inner-back, outer-back.
    def ring(index: int) -> tuple[int, int, int, int]:
        base = index * 4
        return base, base + 1, base + 2, base + 3

    faces: list[tuple[int, ...]] = []
    for index in range(segments):
        a_if, a_of, a_ib, a_ob = ring(index)
        b_if, b_of, b_ib, b_ob = ring(index + 1)
        faces.append((a_if, b_if, b_of, a_of))  # front face
        faces.append((a_ib, a_ob, b_ob, b_ib))  # back face
        faces.append((a_if, a_ib, b_ib, b_if))  # intrados, the underside of the arch
        faces.append((a_of, b_of, b_ob, a_ob))  # extrados
    first = ring(0)
    last = ring(segments)
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


def bevel_and_unwrap(obj: bpy.types.Object, width: float = 0.05) -> None:
    """The single biggest visual difference from a primitive: an edge that catches light."""
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
    bpy.ops.uv.smart_project(angle_limit=math.radians(66.0), island_margin=0.02)
    bpy.ops.object.mode_set(mode="OBJECT")
    obj.select_set(False)


def build_bay() -> None:
    limestone = stone_material("ArcadeLimestone", (0.62, 0.58, 0.51), 0.86)
    trim = stone_material("ArcadeTrim", (0.47, 0.42, 0.35), 0.78)
    # Nothing in this hall casts shadows, so a recess cannot read by depth alone. The value
    # break has to be in the material or the arcade collapses into flat rectangles.
    shadowed = stone_material("ArcadeRecess", (0.16, 0.15, 0.14), 0.94)

    half = BAY_WIDTH * 0.5
    pier_x = -half  # on the bay's left edge, so arrayed bays share one solid pier

    built: list[bpy.types.Object] = []

    # Pier: plinth, shaft, capital.
    built.append(add_box(
        "Bay_PierPlinth", trim,
        (pier_x, -PIER_DEPTH * 0.5 - 0.2, PLINTH_HEIGHT * 0.5),
        (PIER_WIDTH + 0.45, PIER_DEPTH + 0.4, PLINTH_HEIGHT),
    ))
    shaft_height = SPRING_LINE - PLINTH_HEIGHT - CAPITAL_HEIGHT
    built.append(add_box(
        "Bay_PierShaft", limestone,
        (pier_x, -PIER_DEPTH * 0.5, PLINTH_HEIGHT + shaft_height * 0.5),
        (PIER_WIDTH, PIER_DEPTH, shaft_height),
    ))
    built.append(add_box(
        "Bay_PierCapital", trim,
        (pier_x, -PIER_DEPTH * 0.5 - 0.16, SPRING_LINE - CAPITAL_HEIGHT * 0.5),
        (PIER_WIDTH + 0.5, PIER_DEPTH + 0.32, CAPITAL_HEIGHT),
    ))

    # The arch itself, and the blind panel recessed behind it.
    built.append(add_arch("Bay_Arch", limestone))
    built.append(add_arch("Bay_Archivolt", trim, radius_offset=ARCH_THICKNESS, thickness=0.35,
                          depth=ARCH_DEPTH + 0.18))
    panel_height = SPRING_LINE + ARCH_RADIUS
    built.append(add_box(
        "Bay_BlindPanel", shadowed,
        (0.0, RECESS_DEPTH * 0.5, panel_height * 0.5),
        (OPENING_WIDTH, RECESS_DEPTH, panel_height),
    ))

    # Spandrel infill between the arch head and the cornice.
    spandrel_base = SPRING_LINE + ARCH_RADIUS * 0.35
    spandrel_height = CORNICE_BASE - spandrel_base
    built.append(add_box(
        "Bay_Spandrel", shadowed,
        (0.0, RECESS_DEPTH * 0.5, spandrel_base + spandrel_height * 0.5),
        (BAY_WIDTH, RECESS_DEPTH, spandrel_height),
    ))

    # String course and corona closing the storey.
    built.append(add_box(
        "Bay_StringCourse", trim,
        (0.0, -0.55, CORNICE_BASE + 0.275),
        (BAY_WIDTH, 1.5, 0.55),
    ))
    built.append(add_box(
        "Bay_Corona", trim,
        (0.0, -0.75, CORNICE_BASE + 0.55 + 0.175),
        (BAY_WIDTH, 1.9, 0.35),
    ))

    for obj in built:
        bevel_and_unwrap(obj)

    log(
        "built",
        f"bay_width={BAY_WIDTH:.4f}",
        f"opening={OPENING_WIDTH:.4f}",
        f"arch_radius={ARCH_RADIUS:.4f}",
        f"arch_crown={ARCH_CROWN:.4f}",
        f"objects={len(built)}",
    )


def triangle_count() -> int:
    return sum(
        max(0, len(poly.vertices) - 2)
        for obj in bpy.context.scene.objects
        if obj.type == "MESH"
        for poly in obj.data.polygons
    )


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


def verify_export(path: Path) -> None:
    """Re-import what was written and check it against the contract, not against intent."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    names = {obj.name.split(".")[0] for obj in meshes}
    required = {
        "Bay_PierPlinth", "Bay_PierShaft", "Bay_PierCapital",
        "Bay_Arch", "Bay_Archivolt", "Bay_BlindPanel", "Bay_Spandrel",
        "Bay_StringCourse", "Bay_Corona",
    }
    missing = sorted(required - names)

    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    triangles = triangle_count()
    uv_missing = [obj.name for obj in meshes if not obj.data.uv_layers]

    # A glTF re-import converts back to Blender's Z-up, so these are X along the wall,
    # Y radially, Z up — not the exported Y-up frame.
    def extent(objects: list[bpy.types.Object], axis: int) -> tuple[float, float]:
        low, high = float("inf"), float("-inf")
        for obj in objects:
            for corner in obj.bound_box:
                value = (obj.matrix_world @ Vector(corner))[axis]
                low, high = min(low, value), max(high, value)
        return low, high

    def named(prefix: str) -> list[bpy.types.Object]:
        return [obj for obj in meshes if obj.name.split(".")[0] == prefix]

    x_low, x_high = extent(meshes, 0)
    y_low, y_high = extent(meshes, 1)
    z_low, z_high = extent(meshes, 2)
    corona_low, corona_high = extent(named("Bay_Corona"), 0)
    corona_width = corona_high - corona_low
    pier = named("Bay_PierPlinth") + named("Bay_PierShaft") + named("Bay_PierCapital")
    pier_low, pier_high = extent(pier, 0)
    pier_width = pier_high - pier_low

    log(
        "verify",
        f"objects={len(bpy.context.scene.objects)}",
        f"meshes={len(meshes)}",
        f"triangles={triangles}",
        f"cameras={len(cameras)}",
        f"uv_missing={uv_missing}",
        f"bounds_x=({x_low:.3f},{x_high:.3f})",
        f"depth={y_high - y_low:.3f}",
        f"height={z_high - z_low:.3f}",
        f"corona_width={corona_width:.3f}",
        f"pier_width={pier_width:.3f}",
        f"missing={missing}",
    )

    if missing:
        raise RuntimeError(f"arcade bay export missing nodes: {missing}")
    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if uv_missing:
        raise RuntimeError(f"every bay mesh needs UVs for later texturing; missing on {uv_missing}")
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")

    # Tiling contract. The pier deliberately straddles the joint between neighbours, so the
    # bay's bounding box is wider than its repeat distance; what must match the chord is the
    # continuous run of cornice, and the pier must be narrow enough that arrayed piers do not
    # intersect each other.
    if abs(corona_width - BAY_WIDTH) > 0.15:
        raise RuntimeError(
            f"cornice run {corona_width:.3f} does not match the {BAY_WIDTH:.3f} chord, so bays "
            "will gap or overlap around the wall"
        )
    if pier_width >= BAY_WIDTH:
        raise RuntimeError(f"pier width {pier_width:.3f} would collide with the next bay's pier")
    if z_high - z_low > STOREY_HEIGHT:
        raise RuntimeError(f"bay height {z_high - z_low:.3f} exceeds the {STOREY_HEIGHT}m storey")
    if z_low < -0.05:
        raise RuntimeError(f"bay dips below its own deck at z={z_low:.3f}")


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    render_path = None if args.no_render else Path(args.render).resolve()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_bay()
    export(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
