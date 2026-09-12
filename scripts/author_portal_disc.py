"""Author the Council floor portal disc — the spinning vortex, and nothing else.

Operator directive, 2026-09-12: "the table goes away. only the animated spinning disk gets
put on the floor. the manifester sits on the floor in the middle of the spinning disk. there
is no table."

So this module exports exactly one node. The disc formerly lived inside `table.glb` as the
`Stargate_Portal` child of an ornate astrolabe table; the table was the part nobody asked for.
Authoring the disc standalone — rather than spawning `table.glb` and hiding its other meshes —
means nothing ships in the scene as an invisible mesh, and the module can be sized for the
floor instead of for a tabletop.

Contract with the engine:

- The node **must** be named `Stargate_Portal`. `crates/engine/src/chamber/portal.rs` binds by
  that exact name, spins it on its local Y at 0.16 rad/s and pulses its emissive every frame.
  That system is not modified by this module and must not need to be.
- The disc lies in the Blender XY plane (normal +Z). `export_yup=True` turns that into a
  horizontal disc with normal +Y in Bevy, which is the axis `rotate_local_y` spins about, so
  the vortex swirls in plane rather than tumbling.
- Origin is at the disc centre and at z=0, so the engine seats it by translation alone: place
  the root at the floor and the disc lies on the floor.

Radius is 3.6m against the manifestation pedestal's 1.20m base, leaving a 2.4m annulus of
visible vortex turning around the altar — the arrangement the operator asked for. The old
tabletop disc was 0.72m authored at 2.6x, so 1.87m effective; the floor inlay is nearly twice
that because a floor has room a tabletop did not.

Run headless:

  blender --background --python scripts/author_portal_disc.py -- \
      --export assets/scenes/portal_disc.glb --no-render
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

# --- contract with world.rs / portal.rs ------------------------------------------------
NODE_NAME = "Stargate_Portal"
DISC_RADIUS = 3.6
SEGMENTS = 144
TRIANGLE_LIMIT = 400

# The altar footprint the disc has to surround, from `manifestation.rs`'s base plinth.
ALTAR_BASE_RADIUS = 1.20

VORTEX_TEXTURE = Path("assets/textures/table/portal_vortex_v2.png")


def log(*parts: object) -> None:
    print("[portal-disc]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default="assets/scenes/portal_disc.glb")
    parser.add_argument("--render", default="artifacts/visual-proof/portal_disc_blender.png")
    parser.add_argument("--no-render", action="store_true")
    return parser.parse_args(argv)


def emission_input(bsdf: bpy.types.Node) -> str:
    return "Emission Color" if "Emission Color" in bsdf.inputs else "Emission"


def build_disc(vortex_path: Path) -> bpy.types.Object:
    bpy.ops.mesh.primitive_circle_add(
        vertices=SEGMENTS, radius=DISC_RADIUS, fill_type="NGON", location=(0.0, 0.0, 0.0)
    )
    disc = bpy.context.active_object
    disc.name = NODE_NAME

    # Polar UVs projected straight down the disc normal, so the vortex image lands centred
    # with its own centre on the disc's centre. A default circle unwrap would put the swirl's
    # eye somewhere off-axis and the spin would read as a wobble.
    mesh = disc.data
    mesh.uv_layers.new(name="UVMap")
    uv_layer = mesh.uv_layers[0].data
    for polygon in mesh.polygons:
        for loop_index in polygon.loop_indices:
            co = mesh.vertices[mesh.loops[loop_index].vertex_index].co
            uv_layer[loop_index].uv = (
                0.5 + co.x / (2.0 * DISC_RADIUS),
                0.5 + co.y / (2.0 * DISC_RADIUS),
            )

    material = bpy.data.materials.new(f"{NODE_NAME}_Mat")
    material.use_nodes = True
    bsdf = next(n for n in material.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.25
    bsdf.inputs["Metallic"].default_value = 0.0

    texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    texture.image = bpy.data.images.load(str(vortex_path))
    # Both inputs: the base colour carries the swirl when the engine's emissive pulse is at
    # its trough, and the emissive input is what `spin_portal` modulates each frame.
    material.node_tree.links.new(texture.outputs["Color"], bsdf.inputs["Base Color"])
    material.node_tree.links.new(texture.outputs["Color"], bsdf.inputs[emission_input(bsdf)])
    if "Emission Strength" in bsdf.inputs:
        bsdf.inputs["Emission Strength"].default_value = 3.0

    mesh.materials.append(material)
    log(
        "built",
        f"radius={DISC_RADIUS}",
        f"segments={SEGMENTS}",
        f"annulus_around_altar={DISC_RADIUS - ALTAR_BASE_RADIUS:.2f}m",
    )
    return disc


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
    """Re-import what was written and check it against the contract, not against intent."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    names = {obj.name.split(".")[0] for obj in meshes}
    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    triangles = triangle_count()
    uv_missing = [obj.name for obj in meshes if not obj.data.uv_layers]

    points = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    x_span = max(p.x for p in points) - min(p.x for p in points)
    y_span = max(p.y for p in points) - min(p.y for p in points)
    # Measured in Blender's frame, not Bevy's. `export_yup=True` writes the disc with its
    # normal on glTF +Y, and the importer converts that straight back to Blender +Z on the way
    # in, so a correct round trip lands flat in XY again with zero Z extent. Checking for a
    # flat Y here would fail on a perfectly good asset.
    z_span = max(p.z for p in points) - min(p.z for p in points)

    emissive = [
        slot.material.name
        for obj in meshes
        for slot in obj.material_slots
        if slot.material is not None
    ]

    log(
        "verify",
        f"objects={len(bpy.context.scene.objects)}",
        f"meshes={len(meshes)}",
        f"names={sorted(names)}",
        f"triangles={triangles}",
        f"cameras={len(cameras)}",
        f"uv_missing={uv_missing}",
        f"span_x={x_span:.3f}",
        f"span_y={y_span:.3f}",
        f"thickness_z={z_span:.4f}",
        f"materials={emissive}",
    )

    if NODE_NAME not in names:
        raise RuntimeError(
            f"{NODE_NAME} missing from export; chamber/portal.rs binds by that exact name and "
            "would silently animate nothing"
        )
    if len(meshes) != 1:
        raise RuntimeError(
            f"the operator asked for the disc alone; export carries {len(meshes)} meshes: "
            f"{sorted(names)}"
        )
    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if uv_missing:
        raise RuntimeError(f"the vortex needs UVs to map; missing on {uv_missing}")
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")
    if not emissive:
        raise RuntimeError("disc exported with no material, so there is no vortex to pulse")

    # The disc must lie flat in its own plane. If the circle were authored in the wrong plane,
    # or the export came through without the yup conversion, the thickness would land on a
    # different axis and the engine's in-plane spin would tumble the disc edge over edge.
    if z_span > 0.001:
        raise RuntimeError(
            f"disc is not flat after round trip (Z extent {z_span:.4f}); "
            "rotate_local_y would tumble it rather than swirl it"
        )
    for axis, span in (("X", x_span), ("Y", y_span)):
        if abs(span - 2.0 * DISC_RADIUS) > 0.02:
            raise RuntimeError(
                f"disc {axis} span {span:.3f} does not match the authored diameter "
                f"{2.0 * DISC_RADIUS:.3f}"
            )
    if DISC_RADIUS <= ALTAR_BASE_RADIUS:
        raise RuntimeError(
            f"disc radius {DISC_RADIUS} does not clear the {ALTAR_BASE_RADIUS}m altar base, so "
            "the altar would cover the vortex instead of standing in the middle of it"
        )


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    render_path = None if args.no_render else Path(args.render).resolve()
    vortex_path = VORTEX_TEXTURE.resolve()
    if not vortex_path.is_file():
        raise RuntimeError(f"vortex texture not found at {vortex_path}")

    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_disc(vortex_path)
    export(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
