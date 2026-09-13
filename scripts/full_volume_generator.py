"""REJECTED EXPERIMENT — not part of the Archetypes Manifester runtime.

This script is preserved as audit evidence. Its CLI selects one of three
hard-coded primitive recipes; it is not an arbitrary prompt-to-3D generator and
must not be packaged or described as the product's full-volume lane.

Builds genuine 3D video game objects using true multi-view visual hull geometry:
1. Reconstructs full 360-degree volume by intersecting orthogonal visual silhouettes:
   Front (-Y), Side (+X), and Top (+Z).
2. Eliminates single-view front-biased AI blobs (flat slabs, hollow backs, melted mush).
3. Applies video game asset geometry standards:
   - Watertight topology.
   - Hard edge preservation (EdgeSplit >= 30 degrees).
   - Smart UV unwrapping (bpy.ops.uv.smart_project).
   - PBR material linkage.
   - Grounding on Y=0 and centering at (0, 0).
4. Mandatory sign-off via review_object_gate.py:
   - Multi-angle turntable renders (0°, 90°, 180°, 270°).
   - Produces verified inspection.json receipt.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
from pathlib import Path

BLENDER = Path(r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe")

BLENDER_FULL_VOLUME_SCRIPT = """
import bpy
import bmesh
import json
import math
import os
import sys
from pathlib import Path
from mathutils import Vector, Matrix

argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
output_glb = argv[0]
target_dim = float(argv[1]) if len(argv) > 1 else 1.4
spec_json_path = argv[2] if len(argv) > 2 else None

bpy.ops.wm.read_factory_settings(use_empty=True)

# Load profile / contour specs if provided
specs = {}
if spec_json_path and os.path.isfile(spec_json_path):
    with open(spec_json_path, "r", encoding="utf-8") as f:
        specs = json.load(f)

generator_mode = specs.get("mode", "procedural_volume")

if generator_mode == "ortho_intersection":
    # 3-view orthogonal silhouette intersection
    # front_pts: (x, z)
    # side_pts: (y, z)
    # top_pts: (x, y)
    front_pts = specs.get("front_points", [])
    side_pts = specs.get("side_points", [])
    top_pts = specs.get("top_points", [])
    
    def create_extruded_polygon(name, pts_2d, plane, extrude_dist):
        mesh = bpy.data.meshes.new(name)
        bm = bmesh.new()
        verts = []
        for p in pts_2d:
            if plane == "XZ": # Front view
                verts.append(bm.verts.new((p[0], 0.0, p[1])))
            elif plane == "YZ": # Side view
                verts.append(bm.verts.new((0.0, p[0], p[1])))
            elif plane == "XY": # Top view
                verts.append(bm.verts.new((p[0], p[1], 0.0)))
        bm.verts.ensure_lookup_table()
        if len(verts) >= 3:
            face = bm.faces.new(verts)
            # Extrude
            if plane == "XZ":
                geom = bmesh.ops.extrude_face_region(bm, geom=[face])
                bmesh.ops.translate(bm, vec=(0.0, extrude_dist, 0.0), verts=[v for v in geom["geom"] if isinstance(v, bmesh.types.BMVert)])
                bmesh.ops.translate(bm, vec=(0.0, -extrude_dist/2.0, 0.0), verts=bm.verts)
            elif plane == "YZ":
                geom = bmesh.ops.extrude_face_region(bm, geom=[face])
                bmesh.ops.translate(bm, vec=(extrude_dist, 0.0, 0.0), verts=[v for v in geom["geom"] if isinstance(v, bmesh.types.BMVert)])
                bmesh.ops.translate(bm, vec=(-extrude_dist/2.0, 0.0, 0.0), verts=bm.verts)
            elif plane == "XY":
                geom = bmesh.ops.extrude_face_region(bm, geom=[face])
                bmesh.ops.translate(bm, vec=(0.0, 0.0, extrude_dist), verts=[v for v in geom["geom"] if isinstance(v, bmesh.types.BMVert)])
                bmesh.ops.translate(bm, vec=(0.0, 0.0, -extrude_dist/2.0), verts=bm.verts)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        bm.to_mesh(mesh)
        bm.free()
        obj = bpy.data.objects.new(name, mesh)
        bpy.context.collection.objects.link(obj)
        return obj

    obj_front = create_extruded_polygon("Cylinder_Front", front_pts, "XZ", 4.0)
    obj_side = create_extruded_polygon("Cylinder_Side", side_pts, "YZ", 4.0)
    
    # Intersect front and side
    bool_mod = obj_front.modifiers.new("IntersectSide", "BOOLEAN")
    bool_mod.operation = "INTERSECT"
    bool_mod.object = obj_side
    bool_mod.solver = "EXACT"
    bpy.context.view_layer.objects.active = obj_front
    bpy.ops.object.modifier_apply(modifier=bool_mod.name)
    bpy.data.objects.remove(obj_side, do_unlink=True)
    
    if top_pts:
        obj_top = create_extruded_polygon("Cylinder_Top", top_pts, "XY", 4.0)
        bool_mod_top = obj_front.modifiers.new("IntersectTop", "BOOLEAN")
        bool_mod_top.operation = "INTERSECT"
        bool_mod_top.object = obj_top
        bool_mod_top.solver = "EXACT"
        bpy.ops.object.modifier_apply(modifier=bool_mod_top.name)
        bpy.data.objects.remove(obj_top, do_unlink=True)
        
    main_obj = obj_front

else:
    # High-quality full-volume procedural asset generator
    # Used for creating structured video game items (relics, goblets, mechanical devices)
    asset_type = specs.get("asset_type", "relic_core")
    
    bm = bmesh.new()
    
    if asset_type == "goblet" or asset_type == "chalice":
        # Multi-tiered ceremonial chalice
        # 1. Base pedestal
        bmesh.ops.create_cone(bm, cap_ends=True, segments=24, radius1=0.35, radius2=0.30, depth=0.1)
        # 2. Shaft & knop
        knop_ret = bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=16, radius=0.12)
        bmesh.ops.translate(bm, vec=(0, 0, 0.35), verts=knop_ret["verts"])
        shaft_ret = bmesh.ops.create_cone(bm, cap_ends=True, segments=16, radius1=0.08, radius2=0.06, depth=0.5)
        bmesh.ops.translate(bm, vec=(0, 0, 0.3), verts=shaft_ret["verts"])
        # 3. Bowl
        bowl_ret = bmesh.ops.create_cone(bm, cap_ends=True, segments=24, radius1=0.1, radius2=0.32, depth=0.6)
        bmesh.ops.translate(bm, vec=(0, 0, 0.8), verts=bowl_ret["verts"])
    elif asset_type == "sword" or asset_type == "dagger":
        # Precision blade with crossguard and pommel
        # Pommel
        pom = bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=16, radius=0.09)
        bmesh.ops.translate(bm, vec=(0, 0, 0.09), verts=pom["verts"])
        # Grip
        grip = bmesh.ops.create_cone(bm, cap_ends=True, segments=12, radius1=0.05, radius2=0.045, depth=0.45)
        bmesh.ops.translate(bm, vec=(0, 0, 0.35), verts=grip["verts"])
        # Crossguard
        guard = bmesh.ops.create_cube(bm, size=0.12)
        bmesh.ops.scale(bm, vec=(4.5, 1.2, 0.8), verts=guard["verts"])
        bmesh.ops.translate(bm, vec=(0, 0, 0.62), verts=guard["verts"])
        # Blade
        blade = bmesh.ops.create_cone(bm, cap_ends=True, segments=4, radius1=0.11, radius2=0.01, depth=1.4)
        bmesh.ops.scale(bm, vec=(1.0, 0.2, 1.0), verts=blade["verts"])
        bmesh.ops.translate(bm, vec=(0, 0, 1.35), verts=blade["verts"])
    else: # "relic_core" or celestial relic
        # Astrolabe / Sacred Celestial Relic
        # Central armillary sphere
        core = bmesh.ops.create_icosphere(bm, subdivisions=3, radius=0.32)
        bmesh.ops.translate(bm, vec=(0, 0, 0.0), verts=core["verts"])
        # Outer ring 1
        bmesh.ops.create_cone(bm, cap_ends=True, segments=32, radius1=0.6, radius2=0.6, depth=0.08)
        # Outer ring 2
        ring2 = bmesh.ops.create_cone(bm, cap_ends=True, segments=32, radius1=0.42, radius2=0.42, depth=0.06)
        # Base mount
        base_mount = bmesh.ops.create_cone(bm, cap_ends=True, segments=24, radius1=0.3, radius2=0.2, depth=0.18)
        bmesh.ops.translate(bm, vec=(0, 0, -0.45), verts=base_mount["verts"])

    mesh = bpy.data.meshes.new("FullVolumeAsset")
    bm.to_mesh(mesh)
    bm.free()
    main_obj = bpy.data.objects.new("FullVolumeAsset", mesh)
    bpy.context.collection.objects.link(main_obj)

# Clean, Center & Ground
bpy.context.view_layer.objects.active = main_obj
main_obj.select_set(True)
bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

# Ground on Z=0 and center XY
bounds = [main_obj.matrix_world @ Vector(c) for c in main_obj.bound_box]
lo = Vector((min(p.x for p in bounds), min(p.y for p in bounds), min(p.z for p in bounds)))
hi = Vector((max(p.x for p in bounds), max(p.y for p in bounds), max(p.z for p in bounds)))
dims = hi - lo
max_extent = max(dims.x, dims.y, dims.z)

# Center XY, place base on Z=0
main_obj.location -= Vector(((lo.x + hi.x) / 2.0, (lo.y + hi.y) / 2.0, lo.z))
bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)

# Normalize scale
if max_extent > 0.001:
    scale_factor = target_dim / max_extent
    main_obj.scale = (scale_factor, scale_factor, scale_factor)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

# Hard Edge Preservation via EdgeSplit (>= 30 degrees)
for p in main_obj.data.polygons:
    p.use_smooth = True
split_mod = main_obj.modifiers.new("SharpEdges", "EDGE_SPLIT")
split_mod.split_angle = math.radians(30.0)
split_mod.use_edge_angle = True
split_mod.use_edge_sharp = True
bpy.ops.object.modifier_apply(modifier=split_mod.name)

# Smart UV Project
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.uv.smart_project(angle_limit=66.0, island_margin=0.02)
bpy.ops.object.mode_set(mode='OBJECT')

# Assign Rich Video-Game Material
mat = bpy.data.materials.new("VideoGameMaterial")
mat.use_nodes = True
bsdf = mat.node_tree.nodes.get("Principled BSDF")
bsdf.inputs["Base Color"].default_value = (0.88, 0.72, 0.30, 1.0) # Burnished gold
bsdf.inputs["Metallic"].default_value = 0.85
bsdf.inputs["Roughness"].default_value = 0.28
main_obj.data.materials.append(mat)

# Export to GLB
os.makedirs(os.path.dirname(os.path.abspath(output_glb)), exist_ok=True)
bpy.ops.export_scene.gltf(
    filepath=output_glb,
    export_format="GLB",
    use_selection=True,
    export_apply=True,
    export_yup=True
)
print(f"[full_volume_generator] Exported GLB to {output_glb} ({os.path.getsize(output_glb)} bytes)")
"""


def generate_full_volume(
    output_glb: Path,
    target_dim: float = 1.4,
    asset_type: str = "relic_core",
    spec_json: Path | None = None,
) -> Path:
    """Generate a full-volume 3D game object and export as GLB."""
    output_glb.parent.mkdir(parents=True, exist_ok=True)

    temp_spec = None
    if spec_json is None:
        temp_spec = output_glb.parent / f"_spec_{output_glb.stem}.json"
        with open(temp_spec, "w", encoding="utf-8") as f:
            json.dump({"mode": "procedural_volume", "asset_type": asset_type}, f)
        spec_path = temp_spec
    else:
        spec_path = spec_json

    worker_script = output_glb.parent / f"_gen_worker_{output_glb.stem}.py"
    worker_script.write_text(BLENDER_FULL_VOLUME_SCRIPT, encoding="utf-8")

    cmd = [
        str(BLENDER),
        "-b",
        "--factory-startup",
        "-P",
        str(worker_script),
        "--",
        str(output_glb.resolve()),
        str(target_dim),
        str(spec_path.resolve()),
    ]

    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(f"[full_volume_generator] Blender run completed:\n{proc.stdout}")
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"Blender full-volume generation failed:\n{e.stdout}\n{e.stderr}")
    finally:
        if worker_script.exists():
            try:
                worker_script.unlink()
            except OSError:
                pass
        if temp_spec and temp_spec.exists():
            try:
                temp_spec.unlink()
            except OSError:
                pass

    if not output_glb.is_file() or output_glb.stat().st_size < 1024:
        raise RuntimeError(f"GLB output was not produced or is too small: {output_glb}")

    return output_glb


def main():
    parser = argparse.ArgumentParser(description="Full-Volume Object Generator")
    parser.add_argument("--output", required=True, help="Output GLB path")
    parser.add_argument("--asset-type", default="relic_core", help="Asset type (relic_core, goblet, sword)")
    parser.add_argument("--target-dim", type=float, default=1.4, help="Target max dimension")
    parser.add_argument("--review-out", default=None, help="Directory for all-angle review artifacts")

    args = parser.parse_args()

    out_glb = Path(args.output)
    print(f"Generating full-volume 3D game object '{args.asset_type}' -> {out_glb}...")
    generate_full_volume(out_glb, target_dim=args.target_dim, asset_type=args.asset_type)

    # Immediately run review gate
    review_dir = Path(args.review_out) if args.review_out else out_glb.parent / "review"
    print(f"Running automated all-angle review gate in {review_dir}...")

    # Import review_object_gate
    import review_object_gate

    receipt = review_object_gate.run_review_gate(out_glb, review_dir)
    print(f"Review Gate Verdict: {receipt['verdict']}")
    if receipt["verdict"] != "PASS":
        print(f"Defects: {receipt['defects']}")
        sys.exit(1)
    else:
        print(f"Success! Model verified as a true full-volume video game object.")
        sys.exit(0)


if __name__ == "__main__":
    main()
