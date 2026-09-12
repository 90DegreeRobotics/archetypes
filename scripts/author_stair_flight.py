"""Author one flight of the Inner Castle perimeter ascent.

Third module of the Blender architecture kit. It replaces 36 loose `Cuboid` treads per flight
with a single built flight: stepped treads, a solid raked soffit under them, a parapet on both
sides with a coping rail, and the landing at the head.

The procedural version it replaces had three faults that no material change could fix:

* its open side was a raw stepped silhouette with nothing behind it, so the stair read as a comb
  of floating slabs rather than a mass of masonry;
* it carried a balustrade on one side only, while a flight rising through the void between two
  gallery decks is exposed on both;
* every tread was a separate box, so nothing could run continuously along the flight — a
  stringer, a parapet and a handrail all need to.

A flight is therefore the repeat unit, not a tread. Two instances make a storey.

Geometry contract with `crates/engine/src/modes/inner_chambers/castle.rs`:

  centreline radius   108.0 m   (STAIR_CENTRE_RADIUS)
  stair width           8.0 m   (STAIR_WIDTH)
  risers per flight      36     (STAIR_FLIGHT_RISERS)
  riser               0.16667 m (12.0 m storey / 72 risers)
  tread                0.30 m   (STAIR_TREAD)
  landing at head      3.0 m    (STAIR_LANDING_LENGTH)
  run                 13.8 m    -> sweep 13.8 / 108 = 7.32 degrees
  rise                 6.0 m

The flight is genuinely curved, built at the real 108m radius rather than modelled straight and
bent at runtime. Local frame matches the arcade bay so the same placement yaw serves both:

  +X  along the wall tangent at the flight's foot
  +Y  radially outward
  +Z  up, 0 at the height the first tread rises from

Run headless:

  blender --background --python scripts/author_stair_flight.py -- \
      --export assets/scenes/stair_flight.glb --no-render
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
CENTRE_RADIUS = 108.0
STAIR_WIDTH = 8.0
FLIGHT_RISERS = 36
STOREY_RISE = 12.0
RISERS_PER_STOREY = 72
RISER = STOREY_RISE / RISERS_PER_STOREY
TREAD = 0.30
LANDING = 3.0

FLIGHT_GOING = FLIGHT_RISERS * TREAD
FLIGHT_RUN = FLIGHT_GOING + LANDING
FLIGHT_RISE = FLIGHT_RISERS * RISER

# --- flight construction ----------------------------------------------------------------
SOFFIT_DEPTH = 1.0          # vertical mass under the treads
PARAPET_HEIGHT = 1.10       # above the nosing line
PARAPET_THICKNESS = 0.36
COPING_HEIGHT = 0.16
COPING_OVERHANG = 0.07
NOSING = 0.03               # tread overhang, so each step casts its own edge line
TRIANGLE_LIMIT = 9000

HALF_WIDTH = STAIR_WIDTH * 0.5


def log(*parts: object) -> None:
    print("[stair-flight]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default="assets/scenes/stair_flight.glb")
    parser.add_argument("--render", default="artifacts/visual-proof/stair_flight_blender.png")
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


def at(run: float, offset: float, height: float) -> tuple[float, float, float]:
    """Place a point by run along the arc, radial offset from the centreline, and height.

    The flight curves, so a point's position is found on the circle rather than on a straight
    axis: the arc centre sits at local (0, -R, 0) and the module origin is on the circle.
    """
    angle = run / CENTRE_RADIUS
    radius = CENTRE_RADIUS + offset
    x = radius * math.sin(angle)
    y = radius * math.cos(angle) - CENTRE_RADIUS
    return (x, y, height)


def build_profile_solid(
    name: str,
    material: bpy.types.Material,
    profile: list[tuple[float, float]],
    offset_inner: float,
    offset_outer: float,
) -> bpy.types.Object:
    """Sweep a closed (run, height) profile across a radial band, following the arc."""
    verts: list[tuple[float, float, float]] = []
    for run, height in profile:
        verts.append(at(run, offset_inner, height))
    for run, height in profile:
        verts.append(at(run, offset_outer, height))

    count = len(profile)
    faces: list[tuple[int, ...]] = []
    for index in range(count):
        nxt = (index + 1) % count
        faces.append((index, nxt, count + nxt, count + index))
    faces.append(tuple(range(count - 1, -1, -1)))
    faces.append(tuple(range(count, count * 2)))

    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    mesh.update()

    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)

    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    return obj


def step_profile() -> list[tuple[float, float]]:
    """The staircase itself: risers, treads with a nosing, the landing, then the soffit back."""
    profile: list[tuple[float, float]] = [(0.0, 0.0)]
    for step in range(FLIGHT_RISERS):
        run = step * TREAD
        top = (step + 1) * RISER
        # Riser face, then the tread, overhanging very slightly so the edge reads as a line.
        profile.append((run - NOSING if step else 0.0, top))
        profile.append((run + TREAD, top))
    # Landing at the head of the flight.
    profile.append((FLIGHT_GOING + LANDING, FLIGHT_RISE))
    # Down the back and home along the raked underside.
    profile.append((FLIGHT_GOING + LANDING, FLIGHT_RISE - SOFFIT_DEPTH))
    profile.append((0.0, -SOFFIT_DEPTH))
    return profile


def parapet_profile() -> list[tuple[float, float]]:
    """A raked wall following the nosing line, level over the landing."""
    rake = RISER / TREAD
    bottom_start = 0.0
    bottom_end = FLIGHT_GOING * rake
    return [
        (0.0, bottom_start),
        (FLIGHT_GOING, bottom_end),
        (FLIGHT_GOING + LANDING, FLIGHT_RISE),
        (FLIGHT_GOING + LANDING, FLIGHT_RISE + PARAPET_HEIGHT),
        (FLIGHT_GOING, bottom_end + PARAPET_HEIGHT),
        (0.0, bottom_start + PARAPET_HEIGHT),
    ]


def coping_profile() -> list[tuple[float, float]]:
    rake = RISER / TREAD
    top = FLIGHT_GOING * rake + PARAPET_HEIGHT
    return [
        (0.0, PARAPET_HEIGHT),
        (FLIGHT_GOING, top),
        (FLIGHT_GOING + LANDING, FLIGHT_RISE + PARAPET_HEIGHT),
        (FLIGHT_GOING + LANDING, FLIGHT_RISE + PARAPET_HEIGHT + COPING_HEIGHT),
        (FLIGHT_GOING, top + COPING_HEIGHT),
        (0.0, PARAPET_HEIGHT + COPING_HEIGHT),
    ]


def unwrap(obj: bpy.types.Object) -> None:
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=math.radians(66.0), island_margin=0.02)
    bpy.ops.object.mode_set(mode="OBJECT")
    obj.select_set(False)


def build_flight() -> list[bpy.types.Object]:
    limestone = stone_material("ArcadeLimestone", (0.62, 0.58, 0.51), 0.86)
    trim = stone_material("ArcadeTrim", (0.47, 0.42, 0.35), 0.78)

    built = [
        build_profile_solid(
            "Stair_Steps", limestone, step_profile(), -HALF_WIDTH, HALF_WIDTH
        )
    ]

    # A parapet on each side. A flight rising through the void between two decks is open on
    # both, which the procedural version only acknowledged on one.
    for side, label in ((-1.0, "Inner"), (1.0, "Outer")):
        edge = side * HALF_WIDTH
        near = edge - side * PARAPET_THICKNESS
        built.append(
            build_profile_solid(
                f"Stair_Parapet{label}", limestone, parapet_profile(),
                min(edge, near), max(edge, near),
            )
        )
        built.append(
            build_profile_solid(
                f"Stair_Coping{label}", trim, coping_profile(),
                min(edge, near) - COPING_OVERHANG, max(edge, near) + COPING_OVERHANG,
            )
        )

    for obj in built:
        unwrap(obj)
    return built


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
        bpy.context.scene.render.resolution_x = 1024
        bpy.context.scene.render.resolution_y = 1024
        bpy.context.scene.render.filepath = str(render_path)
        bpy.ops.render.render(write_still=True)
        log(f"rendered={render_path}")


def verify_export(path: Path) -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    names = {obj.name.split(".")[0] for obj in meshes}
    required = {
        "Stair_Steps",
        "Stair_ParapetInner", "Stair_CopingInner",
        "Stair_ParapetOuter", "Stair_CopingOuter",
    }
    missing = sorted(required - names)
    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    uv_missing = [obj.name for obj in meshes if not obj.data.uv_layers]
    triangles = triangle_count()

    def extent(objects: list[bpy.types.Object], axis: int) -> tuple[float, float]:
        low, high = float("inf"), float("-inf")
        for obj in objects:
            for corner in obj.bound_box:
                value = (obj.matrix_world @ Vector(corner))[axis]
                low, high = min(low, value), max(high, value)
        return low, high

    steps = [obj for obj in meshes if obj.name.split(".")[0] == "Stair_Steps"]
    _, step_top = extent(steps, 2)

    # Radial occupancy has to be measured from the arc centre, not from a bounding box: the
    # flight curves 7.32 degrees, so its Y extent also contains the arc's own 0.88m of inward
    # drift and a box measurement reports the module as a metre wider than it is.
    inner_r, outer_r = float("inf"), float("-inf")
    for obj in meshes:
        for vertex in obj.data.vertices:
            point = obj.matrix_world @ vertex.co
            radius = math.hypot(point.x, point.y + CENTRE_RADIUS)
            inner_r, outer_r = min(inner_r, radius), max(outer_r, radius)
    width = outer_r - inner_r

    log(
        "verify",
        f"meshes={len(meshes)}",
        f"triangles={triangles}",
        f"cameras={len(cameras)}",
        f"uv_missing={uv_missing}",
        f"step_top={step_top:.4f}",
        f"radial_band=({inner_r:.3f},{outer_r:.3f})",
        f"radial_width={width:.3f}",
        f"missing={missing}",
    )

    if missing:
        raise RuntimeError(f"stair flight export missing nodes: {missing}")
    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if uv_missing:
        raise RuntimeError(f"every flight mesh needs UVs for later texturing; missing on {uv_missing}")
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")
    # The walking surface the engine computes and the geometry a player sees have to agree: the
    # head of the flight is exactly its rise above the foot, or the last step is a trip hazard
    # into the gallery deck.
    if abs(step_top - FLIGHT_RISE) > 0.001:
        raise RuntimeError(f"flight tops out at {step_top:.4f}, not its {FLIGHT_RISE}m rise")
    # Parapets sit inside the stair band; anything wider fouls the gallery deck edge, and the
    # collision in castle.rs only treats this band as stair surface.
    limit = STAIR_WIDTH + 2.0 * COPING_OVERHANG + 0.01
    if width > limit:
        raise RuntimeError(f"flight occupies {width:.3f}m of radius, past the {STAIR_WIDTH}m stair band")
    if inner_r < CENTRE_RADIUS - STAIR_WIDTH * 0.5 - COPING_OVERHANG - 0.01:
        raise RuntimeError(f"flight reaches in to r={inner_r:.3f}, past the stair band")
    if outer_r > CENTRE_RADIUS + STAIR_WIDTH * 0.5 + COPING_OVERHANG + 0.01:
        raise RuntimeError(f"flight reaches out to r={outer_r:.3f}, past the stair band")


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    render_path = None if args.no_render else Path(args.render).resolve()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    built = build_flight()
    log(
        "built",
        f"risers={FLIGHT_RISERS}",
        f"riser={RISER:.5f}",
        f"going={FLIGHT_GOING}",
        f"run={FLIGHT_RUN}",
        f"rise={FLIGHT_RISE}",
        f"sweep_deg={math.degrees(FLIGHT_RUN / CENTRE_RADIUS):.3f}",
        f"objects={len(built)}",
    )
    export(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
