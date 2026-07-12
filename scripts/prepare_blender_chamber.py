"""Normalize the authoritative Council Chamber scene and export its runtime GLB."""

from __future__ import annotations

import argparse
from pathlib import Path

import bpy
from mathutils import Vector


STATIC_CHAMBER_NODES = (
    "Star_Tetra_A",
    "Star_Tetra_B",
    "Witness",
    "Sentinel",
    "Architect",
    "Mentor",
    "Explorer",
    "Oracle",
    "Empath",
    "Jester",
    "Reflection_Plane",
)


def parse_args() -> argparse.Namespace:
    argv = []
    if "--" in __import__("sys").argv:
        argv = __import__("sys").argv[__import__("sys").argv.index("--") + 1 :]
    parser = argparse.ArgumentParser()
    parser.add_argument("--blend", required=True)
    parser.add_argument("--export", action="append", required=True)
    return parser.parse_args(argv)


def parent_preserving_world(child: bpy.types.Object, parent: bpy.types.Object) -> None:
    if child.parent == parent:
        return
    world = child.matrix_world.copy()
    child.parent = parent
    child.matrix_world = world


def main() -> None:
    args = parse_args()
    blend_path = Path(args.blend).resolve()

    root = bpy.data.objects.get("CouncilChamber")
    if root is None:
        root = bpy.data.objects.new("CouncilChamber", None)
        bpy.context.scene.collection.objects.link(root)
    root["runtime_role"] = "chamber_root"

    for name in STATIC_CHAMBER_NODES:
        obj = bpy.data.objects.get(name)
        if obj is None:
            raise RuntimeError(f"Required chamber object is missing: {name}")
        parent_preserving_world(obj, root)
        obj["runtime_role"] = name.lower()

    cinematic_camera = (
        bpy.data.objects.get("Witness_CinematicCamera")
        or bpy.data.objects.get("Witness_Camera")
        or bpy.data.objects.get("Camera")
    )
    if cinematic_camera is None or cinematic_camera.type != "CAMERA":
        raise RuntimeError("The authoritative Witness camera is missing")
    cinematic_camera.name = "Witness_CinematicCamera"
    cinematic_camera.data.name = "Witness_CinematicCamera"
    cinematic_camera["runtime_role"] = "witness_cinematic_camera"

    camera = bpy.data.objects.get("Witness_Camera")
    if camera is None:
        camera_data = cinematic_camera.data.copy()
        camera = bpy.data.objects.new("Witness_Camera", camera_data)
        bpy.context.scene.collection.objects.link(camera)
    # The working .blend owns the approved Witness framing. Export must preserve
    # that authored camera instead of replacing it with a generic fallback view.
    camera.animation_data_clear()
    camera["runtime_role"] = "witness_camera"
    bpy.context.scene.camera = camera

    for light in (obj for obj in bpy.context.scene.objects if obj.type == "LIGHT"):
        if light.data.type == "SUN":
            light.data.energy = 2.5
        elif light.data.type == "POINT":
            light.data.energy = 250.0

    track = bpy.data.objects.get("Camera_Track")
    if track is not None:
        track["runtime_role"] = "camera_track"

    bpy.context.scene.frame_start = 0
    bpy.context.scene.frame_end = 240
    # Preserve the authored working frame; panel animation is exported below.
    bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))

    for export_path_value in args.export:
        export_path = Path(export_path_value).resolve()
        export_path.parent.mkdir(parents=True, exist_ok=True)
        is_runtime_export = export_path.parent.name == "scenes"
        bpy.ops.export_scene.gltf(
            filepath=str(export_path),
            export_format="GLB",
            use_selection=False,
            # The camera is authored in the working .blend and is part of the
            # scene contract even when the current engine supplies its own view.
            export_cameras=True,
            export_lights=True,
            export_animations=True,
            export_nla_strips=True,
            export_apply=True,  # bake the star-tetrahedron Wireframe modifier (black edges)
            export_yup=True,
        )
        print(f"EXPORTED {export_path}")


if __name__ == "__main__":
    main()
