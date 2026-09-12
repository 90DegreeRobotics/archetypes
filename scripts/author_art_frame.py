"""Author the picture frame hung in every museum chamber.

One module, instanced once per hanging position. The artwork and the placard are applied as
textures at runtime from `assets/museum/manifest.json`, so sixty hangings cost one 4:3 frame
asset rather than sixty baked ones.

Local frame (Blender, Z up), chosen so the engine can place it with a single yaw:

  +X  frame width
  +Z  up; the canvas centre sits on Z = 0, so the engine places it at the hanging height
  -Y  the direction it faces; the wall is at Y = 0 and the frame projects into the room

`export_yup=True` turns Blender's -Y into Bevy's +Z, and `Quat::from_rotation_y(yaw)` sends
local +Z to `(sin yaw, 0, cos yaw)` — so `castle::museum_hanging_positions` hands back a yaw
whose +Z is the wall's inward normal and the picture faces the room.

**The canvas and the placard must not be cube-projected.** They are created with clean 0..1
UVs, which is what maps an image onto them 1:1. Running the kit's world-scaled unwrap over them
would tile a painting across its own frame.

Run headless:

  blender --background --python scripts/author_art_frame.py -- \
      --export assets/scenes/art_frame.glb --no-render
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

# --- contract with the staging script and castle.rs ------------------------------------
# `scripts/stage_museum_art.py` mounts every work onto a 4:3 plate, so the canvas is 4:3 and
# no painting is stretched to fit.
CANVAS_WIDTH = 1.60
CANVAS_HEIGHT = CANVAS_WIDTH * 3.0 / 4.0

MOULDING_WIDTH = 0.10
MOULDING_DEPTH = 0.09
CANVAS_STANDOFF = 0.045      # how far the picture plane sits off the wall
BACKING_DEPTH = 0.04

# The placard image is 512x168, so 3.048:1.
PLACARD_WIDTH = 0.55
PLACARD_HEIGHT = PLACARD_WIDTH * 168.0 / 512.0
PLACARD_DROP = 0.30          # below the frame's lower edge, to the placard's centre

# Eight bevelled parts. A frame is small and close to the eye, so the bevels earn their cost.
TRIANGLE_LIMIT = 900


def log(*parts: object) -> None:
    print("[art-frame]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default="assets/scenes/art_frame.glb")
    parser.add_argument("--render", default="artifacts/visual-proof/art_frame_blender.png")
    parser.add_argument("--no-render", action="store_true")
    return parser.parse_args(argv)


def plain_material(name: str, base, roughness: float, metallic: float = 0.0):
    material = bpy.data.materials.new(name)
    material.use_nodes = True
    principled = material.node_tree.nodes["Principled BSDF"]
    principled.inputs["Base Color"].default_value = (*base, 1.0)
    principled.inputs["Roughness"].default_value = roughness
    principled.inputs["Metallic"].default_value = metallic
    return material


def add_box(name, material, centre, size):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=centre)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(material)
    return obj


def add_facing_plane(name, material, centre, width, height):
    """A plane in the XZ plane facing -Y, with clean 0..1 UVs.

    Blender's plane is created in XY facing +Z, so it is rotated -90 degrees about X to stand
    up and face -Y. The rotation is applied, which leaves the UVs untouched.
    """
    bpy.ops.mesh.primitive_plane_add(size=1.0, location=centre)
    obj = bpy.context.active_object
    obj.name = name
    # +90, not -90. `R_x(-90)` sends the plane's +Z normal to +Y, which points *into* the wall
    # — the picture then faces the masonry, gets backface-culled, and every frame in the museum
    # shows its own dark backing board instead of a painting. `R_x(+90)` sends +Z to -Y, out
    # into the room.
    obj.rotation_euler = (math.radians(90.0), 0.0, 0.0)
    # Scale the plane's own Y, which the -90 degree rotation about X turns into Z. Scaling Z
    # here does nothing at all: a plane has no Z extent to scale, so the height silently stayed
    # 1.0 and the canvas came out 1.6 x 1.0 instead of 4:3.
    obj.scale = (width, height, 1.0)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    obj.data.materials.append(material)
    return obj


def bevel(obj, width: float = 0.012) -> None:
    bpy.context.view_layer.objects.active = obj
    modifier = obj.modifiers.new(name="Bevel", type="BEVEL")
    modifier.width = width
    modifier.segments = 2
    modifier.limit_method = "ANGLE"
    modifier.angle_limit = math.radians(35.0)
    bpy.ops.object.modifier_apply(modifier=modifier.name)

    # Box UVs for the moulding only. A frame is small enough that one projection is fine, and
    # it must not disturb the canvas or the placard.
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.cube_project(cube_size=1.0)
    bpy.ops.object.mode_set(mode="OBJECT")
    obj.select_set(False)


def build_frame() -> list:
    gilt = plain_material("FrameGilt", (0.44, 0.35, 0.19), 0.34, metallic=0.62)
    backing = plain_material("FrameBacking", (0.07, 0.065, 0.06), 0.90)
    plate = plain_material("FramePlacardPlate", (0.14, 0.13, 0.12), 0.62, metallic=0.30)
    # White, non-emissive: the artwork's colour comes entirely from the texture, and the
    # chamber's own warm light is what reveals it. An emissive painting is a lightbox, not a
    # painting.
    canvas_mat = plain_material("FrameCanvas", (1.0, 1.0, 1.0), 0.72)
    placard_mat = plain_material("FramePlacard", (1.0, 1.0, 1.0), 0.58)

    built = []
    half_w = CANVAS_WIDTH * 0.5
    half_h = CANVAS_HEIGHT * 0.5
    outer_w = half_w + MOULDING_WIDTH
    outer_h = half_h + MOULDING_WIDTH
    moulding_y = -MOULDING_DEPTH * 0.5

    # Four sides of moulding rather than one ring, so the mitres read and the bevel catches
    # light along each run.
    built.append(add_box("Frame_Moulding_Top", gilt,
                         (0.0, moulding_y, half_h + MOULDING_WIDTH * 0.5),
                         (outer_w * 2.0, MOULDING_DEPTH, MOULDING_WIDTH)))
    built.append(add_box("Frame_Moulding_Bottom", gilt,
                         (0.0, moulding_y, -(half_h + MOULDING_WIDTH * 0.5)),
                         (outer_w * 2.0, MOULDING_DEPTH, MOULDING_WIDTH)))
    built.append(add_box("Frame_Moulding_Left", gilt,
                         (-(half_w + MOULDING_WIDTH * 0.5), moulding_y, 0.0),
                         (MOULDING_WIDTH, MOULDING_DEPTH, CANVAS_HEIGHT)))
    built.append(add_box("Frame_Moulding_Right", gilt,
                         (half_w + MOULDING_WIDTH * 0.5, moulding_y, 0.0),
                         (MOULDING_WIDTH, MOULDING_DEPTH, CANVAS_HEIGHT)))

    # A dark backing board behind the picture, so the frame is not a window onto the wall.
    built.append(add_box("Frame_Backing", backing,
                         (0.0, -BACKING_DEPTH * 0.5, 0.0),
                         (CANVAS_WIDTH, BACKING_DEPTH, CANVAS_HEIGHT)))

    built.append(add_box("Frame_PlacardPlate", plate,
                         (0.0, -0.022, -(outer_h + PLACARD_DROP)),
                         (PLACARD_WIDTH + 0.05, 0.044, PLACARD_HEIGHT + 0.05)))

    for obj in built:
        bevel(obj)

    # These two carry images and keep their own UVs. Added after the bevel loop so nothing
    # re-projects them.
    canvas = add_facing_plane(
        "Frame_Canvas", canvas_mat, (0.0, -CANVAS_STANDOFF, 0.0), CANVAS_WIDTH, CANVAS_HEIGHT
    )
    placard = add_facing_plane(
        "Frame_Placard",
        placard_mat,
        (0.0, -0.046, -(outer_h + PLACARD_DROP)),
        PLACARD_WIDTH,
        PLACARD_HEIGHT,
    )
    built.extend([canvas, placard])

    log(
        "built",
        f"canvas={CANVAS_WIDTH}x{CANVAS_HEIGHT:.3f}",
        f"placard={PLACARD_WIDTH}x{PLACARD_HEIGHT:.3f}",
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
        bpy.context.scene.render.resolution_x = 1024
        bpy.context.scene.render.resolution_y = 1024
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
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    names = {obj.name.split(".")[0] for obj in meshes}
    required = {"Frame_Canvas", "Frame_Placard", "Frame_Moulding_Top", "Frame_Backing"}
    missing = sorted(required - names)
    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    triangles = triangle_count()
    uv_missing = [obj.name for obj in meshes if not obj.data.uv_layers]

    def named(stem: str):
        return [obj for obj in meshes if obj.name.split(".")[0] == stem]

    def uv_bounds(stem: str):
        us, vs = [], []
        for obj in named(stem):
            for loop in obj.data.uv_layers[0].data:
                us.append(loop.uv.x)
                vs.append(loop.uv.y)
        return (min(us), max(us), min(vs), max(vs)) if us else (0, 0, 0, 0)

    canvas_pts = [obj.matrix_world @ Vector(c) for obj in named("Frame_Canvas") for c in obj.bound_box]
    canvas_w = max(p.x for p in canvas_pts) - min(p.x for p in canvas_pts)
    canvas_h = max(p.z for p in canvas_pts) - min(p.z for p in canvas_pts)
    canvas_uv = uv_bounds("Frame_Canvas")
    placard_uv = uv_bounds("Frame_Placard")
    all_y = [
        (obj.matrix_world @ Vector(c)).y for obj in meshes for c in obj.bound_box
    ]

    log(
        "verify",
        f"meshes={len(meshes)}",
        f"triangles={triangles}",
        f"cameras={len(cameras)}",
        f"uv_missing={uv_missing}",
        f"canvas={canvas_w:.3f}x{canvas_h:.3f}",
        f"canvas_uv={tuple(round(v, 3) for v in canvas_uv)}",
        f"placard_uv={tuple(round(v, 3) for v in placard_uv)}",
        f"depth_y=({min(all_y):.3f},{max(all_y):.3f})",
        f"missing={missing}",
    )

    if missing:
        raise RuntimeError(f"art frame export missing nodes: {missing}")
    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if uv_missing:
        raise RuntimeError(f"missing UVs on {uv_missing}")
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")

    if abs(canvas_w / canvas_h - 4.0 / 3.0) > 0.01:
        raise RuntimeError(
            f"canvas is {canvas_w:.3f}x{canvas_h:.3f}, not the 4:3 the staging script mounts "
            "every work onto — paintings would be stretched"
        )

    # The whole reason these two planes skip the kit unwrap: an image maps onto them 1:1 only
    # if their UVs span exactly the unit square.
    for label, bounds in (("canvas", canvas_uv), ("placard", placard_uv)):
        low_u, high_u, low_v, high_v = bounds
        if abs(low_u) > 0.001 or abs(high_u - 1.0) > 0.001 or abs(low_v) > 0.001 or abs(high_v - 1.0) > 0.001:
            raise RuntimeError(
                f"{label} UVs span ({low_u:.3f}..{high_u:.3f}, {low_v:.3f}..{high_v:.3f}), not "
                "0..1 — the image would tile or crop instead of mapping onto it once"
            )

    # The frame hangs on a wall at Y = 0 and projects into the room. Anything at positive Y is
    # inside the masonry.
    if max(all_y) > 0.001:
        raise RuntimeError(f"frame reaches {max(all_y):.3f} into the wall it hangs on")

    # The two image planes must face out of the wall. A plane facing the wrong way is not a
    # subtle defect: it is backface-culled, so every painting in the museum is invisible and
    # the frame shows its own backing board. Checked by the polygon's own normal rather than by
    # the rotation that was asked for.
    for stem in ("Frame_Canvas", "Frame_Placard"):
        for obj in named(stem):
            for polygon in obj.data.polygons:
                world_normal = (obj.matrix_world.to_3x3() @ polygon.normal).normalized()
                if world_normal.y > -0.99:
                    raise RuntimeError(
                        f"{stem} faces {tuple(round(v, 3) for v in world_normal)}; it must face "
                        "(0, -1, 0), out of the wall into the room"
                    )


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    render_path = None if args.no_render else Path(args.render).resolve()
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_frame()
    export(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
