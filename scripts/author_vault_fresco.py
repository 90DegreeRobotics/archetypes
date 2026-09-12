"""Author the Inner Castle vault: a saucer dome carrying a painted ceiling fresco.

Second module of the Blender architecture kit. It replaces the procedural `Cone` the vault was
made of, which was wrong twice over: a cone's UV unwrap runs around its lateral surface, so a
circular painting mapped onto it would smear into a spiral, and the 24 separate rib bars cut
straight across the painting.

A ceiling fresco needs two things this module provides:

1. **A dome, not a cone.** A spherical cap with base radius 114m and a 30m rise. The sphere it
   is cut from has radius `(R^2 + h^2) / 2h = (114^2 + 30^2) / 60 = 231.6m`, centred 201.6m
   below the springing plane. That is a shallow saucer dome — the proportion Baroque ceilings
   are painted on, and the reason the picture stays readable from the floor instead of
   foreshortening away.

2. **Polar UVs projected from below.** Every vertex takes `u = 0.5 + x / 2R`, `v = 0.5 + y / 2R`,
   which is a straight orthographic projection of the disc onto the image. The painting's
   circular border lands exactly on the dome's springing circle and its centre burst lands
   exactly on the crown, with no seam, because there is no wrap.

The hall has no shadow casting and the vault sits 96m up, so the fresco also carries a modest
emissive term. Without it a painted ceiling in a dim rotunda is simply black.

Run headless:

  blender --background --python scripts/author_vault_fresco.py -- \
      --export assets/scenes/vault_fresco.glb --no-render
"""

from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

# --- contract with castle.rs -----------------------------------------------------------
BASE_RADIUS = 114.0      # castle_inner_face()
VAULT_RISE = 30.0        # VAULT_RISE
TEXTURE = Path("assets/textures/vault_fresco.png")

RINGS = 28
SEGMENTS = 96
EMISSIVE_STRENGTH = 0.45
TRIANGLE_LIMIT = 7000

# The painting is a circle inscribed in a square PNG, and the pixels outside it are fully
# transparent — which in an unpremultiplied PNG means their RGB is undefined garbage. Sampling
# exactly at UV radius 0.5 therefore drags that garbage onto the springing line as coloured
# speckle. Pulling the sample circle just inside the painted edge removes it; the cost is the
# outermost 2.5% of the gold border, which is a plain repeating band.
UV_INSET = 0.975

SPHERE_RADIUS = (BASE_RADIUS * BASE_RADIUS + VAULT_RISE * VAULT_RISE) / (2.0 * VAULT_RISE)
SPHERE_CENTRE_Z = VAULT_RISE - SPHERE_RADIUS


def log(*parts: object) -> None:
    print("[vault-fresco]", *parts, flush=True)


def parse_args() -> argparse.Namespace:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    parser = argparse.ArgumentParser()
    parser.add_argument("--export", default="assets/scenes/vault_fresco.glb")
    parser.add_argument("--texture", default=str(TEXTURE))
    parser.add_argument("--render", default="artifacts/visual-proof/vault_fresco_blender.png")
    parser.add_argument("--no-render", action="store_true")
    return parser.parse_args(argv)


def dome_height(radius: float) -> float:
    """Height of the spherical cap at a given distance from its axis."""
    return math.sqrt(max(0.0, SPHERE_RADIUS * SPHERE_RADIUS - radius * radius)) + SPHERE_CENTRE_Z


def build_dome() -> bpy.types.Object:
    verts: list[tuple[float, float, float]] = [(0.0, 0.0, dome_height(0.0))]
    uvs_by_vert: list[tuple[float, float]] = [(0.5, 0.5)]

    for ring in range(1, RINGS + 1):
        radius = BASE_RADIUS * ring / RINGS
        height = dome_height(radius)
        for segment in range(SEGMENTS):
            angle = math.tau * segment / SEGMENTS
            x = radius * math.cos(angle)
            y = radius * math.sin(angle)
            verts.append((x, y, height))
            # Orthographic projection of the disc onto the image: no wrap, so no seam.
            uvs_by_vert.append((
                0.5 + UV_INSET * x / (2.0 * BASE_RADIUS),
                0.5 + UV_INSET * y / (2.0 * BASE_RADIUS),
            ))

    def index(ring: int, segment: int) -> int:
        return 1 + (ring - 1) * SEGMENTS + (segment % SEGMENTS)

    faces: list[tuple[int, ...]] = []
    # Wound so the surface faces down into the hall, which is the only side anyone sees.
    for segment in range(SEGMENTS):
        faces.append((0, index(1, segment + 1), index(1, segment)))
    for ring in range(1, RINGS):
        for segment in range(SEGMENTS):
            faces.append((
                index(ring, segment),
                index(ring, segment + 1),
                index(ring + 1, segment + 1),
                index(ring + 1, segment),
            ))

    mesh = bpy.data.meshes.new("Vault_Fresco")
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    mesh.update()

    uv_layer = mesh.uv_layers.new(name="UVMap")
    for loop in mesh.loops:
        uv_layer.data[loop.index].uv = uvs_by_vert[loop.vertex_index]

    obj = bpy.data.objects.new("Vault_Fresco", mesh)
    bpy.context.collection.objects.link(obj)
    mesh.shade_smooth()
    return obj


def fresco_material(texture_path: Path) -> bpy.types.Material:
    material = bpy.data.materials.new("VaultFresco")
    material.use_nodes = True
    # Left double sided on purpose: a one-sided ceiling disappears the moment the camera
    # crosses above it, and free flight can reach the wall head.
    material.use_backface_culling = False

    tree = material.node_tree
    principled = tree.nodes["Principled BSDF"]
    image_node = tree.nodes.new("ShaderNodeTexImage")
    image_node.image = bpy.data.images.load(str(texture_path.resolve()))
    image_node.location = (-420, 0)

    tree.links.new(image_node.outputs["Color"], principled.inputs["Base Color"])
    tree.links.new(image_node.outputs["Color"], principled.inputs["Emission Color"])
    principled.inputs["Emission Strength"].default_value = EMISSIVE_STRENGTH
    principled.inputs["Roughness"].default_value = 0.92
    principled.inputs["Metallic"].default_value = 0.0
    return material


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
        max(0, len(poly.vertices) - 2)
        for obj in bpy.context.scene.objects
        if obj.type == "MESH"
        for poly in obj.data.polygons
    )


def verify_export(path: Path) -> None:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(path))

    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    cameras = [obj for obj in bpy.context.scene.objects if obj.type == "CAMERA"]
    triangles = triangle_count()

    if not meshes:
        raise RuntimeError("vault export produced no mesh")
    dome = meshes[0]

    textured = [
        node
        for material_slot in dome.material_slots
        if material_slot.material and material_slot.material.use_nodes
        for node in material_slot.material.node_tree.nodes
        if node.type == "TEX_IMAGE" and node.image is not None
    ]

    low = [float("inf")] * 3
    high = [float("-inf")] * 3
    for obj in meshes:
        for corner in obj.bound_box:
            point = obj.matrix_world @ Vector(corner)
            for axis in range(3):
                low[axis] = min(low[axis], point[axis])
                high[axis] = max(high[axis], point[axis])

    span_x = high[0] - low[0]
    span_y = high[1] - low[1]
    rise = high[2] - low[2]

    log(
        "verify",
        f"meshes={len(meshes)}",
        f"triangles={triangles}",
        f"cameras={len(cameras)}",
        f"uv_layers={len(dome.data.uv_layers)}",
        f"textures={len(textured)}",
        f"span_x={span_x:.3f}",
        f"span_y={span_y:.3f}",
        f"rise={rise:.3f}",
        f"base_z={low[2]:.3f}",
    )

    if cameras:
        raise RuntimeError(f"runtime GLB must not export cameras; found {len(cameras)}")
    if not dome.data.uv_layers:
        raise RuntimeError("the fresco needs UVs or the painting cannot be placed at all")
    if not textured:
        raise RuntimeError("the fresco material carries no image texture")
    # Nothing may sample at or beyond the painted circle's edge, or the transparent corners
    # of the PNG bleed back onto the springing line.
    uv_layer = dome.data.uv_layers.active
    furthest = max(
        math.hypot(datum.uv[0] - 0.5, datum.uv[1] - 0.5) for datum in uv_layer.data
    )
    log(f"furthest_uv_radius={furthest:.4f}")
    if furthest > 0.495:
        raise RuntimeError(
            f"UVs reach radius {furthest:.4f}; that samples the PNG's transparent corners"
        )
    if triangles > TRIANGLE_LIMIT:
        raise RuntimeError(f"triangle count {triangles} exceeds limit {TRIANGLE_LIMIT}")
    # The dome has to meet the wall head exactly: too small and daylight shows at the join,
    # too large and it buries itself in the masonry.
    for label, span in (("x", span_x), ("y", span_y)):
        if abs(span - 2.0 * BASE_RADIUS) > 0.6:
            raise RuntimeError(
                f"dome {label} span {span:.3f} does not match the {2.0 * BASE_RADIUS}m springing circle"
            )
    if abs(rise - VAULT_RISE) > 0.05:
        raise RuntimeError(f"dome rise {rise:.3f} does not match the {VAULT_RISE}m vault rise")
    if abs(low[2]) > 0.05:
        raise RuntimeError(f"dome springs from z={low[2]:.3f}, not from its own base plane")


def main() -> None:
    args = parse_args()
    export_path = Path(args.export).resolve()
    texture_path = Path(args.texture)
    if not texture_path.exists():
        raise SystemExit(f"fresco texture not found: {texture_path}")
    render_path = None if args.no_render else Path(args.render).resolve()

    bpy.ops.wm.read_factory_settings(use_empty=True)
    dome = build_dome()
    dome.data.materials.append(fresco_material(texture_path))
    log(
        "built",
        f"sphere_radius={SPHERE_RADIUS:.3f}",
        f"centre_z={SPHERE_CENTRE_Z:.3f}",
        f"rings={RINGS}",
        f"segments={SEGMENTS}",
    )
    export(export_path, render_path)
    verify_export(export_path)


if __name__ == "__main__":
    main()
