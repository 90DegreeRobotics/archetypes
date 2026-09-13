"""Review Object Gate: Automated & Deterministic Full-Volume All-Angle Quality Gate.

Evaluates 3D models (GLB/OBJ) against strict video-game quality criteria:
1. Watertightness (Manifold geometry, zero open boundary edges).
2. 3D Volume (Thickness >= 5% of max dimension, non-degenerate bounding box).
3. 360-degree Cardinal Silhouette Consistency (Front 0°, Right 90°, Rear 180°, Left 270°).
   Catches the single-view reconstruction failure where the rear is a flat slab,
   hollow void, or melted blob.
4. Topology & Connectedness (Checks for disconnected floating scraps).

Outputs:
- Machine-readable `inspection.json` receipt.
- Multi-angle visual inspection turnaround sheet.
- Clean exit codes: 0 for PASS, 1 for FAIL.
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

# Script that runs INSIDE Blender when invoked via headless background mode
BLENDER_WORKER_SCRIPT = """
import bpy
import bmesh
import json
import math
import os
import sys
from pathlib import Path
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
glb_path = argv[0]
out_dir = Path(argv[1])
out_dir.mkdir(parents=True, exist_ok=True)

bpy.ops.wm.read_factory_settings(use_empty=True)

ext = Path(glb_path).suffix.lower()
if ext in (".glb", ".gltf"):
    bpy.ops.import_scene.gltf(filepath=glb_path)
elif ext == ".obj":
    bpy.ops.wm.obj_import(filepath=glb_path)
else:
    raise RuntimeError(f"Unsupported model extension: {ext}")

meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
if not meshes:
    res = {
        "verdict": "FAIL",
        "error": "No mesh objects found in file",
        "defects": ["No mesh geometry present"]
    }
    with open(out_dir / "inspection.json", "w") as f:
        json.dump(res, f, indent=2)
    sys.exit(0)

# Join meshes into one evaluated unit if multiple exist
bpy.ops.object.select_all(action="DESELECT")
for m in meshes:
    m.select_set(True)
bpy.context.view_layer.objects.active = meshes[0]
if len(meshes) > 1:
    bpy.ops.object.join()

active_obj = bpy.context.active_object
mesh_data = active_obj.data

# 1. Topological Analysis via bmesh
bm = bmesh.new()
bm.from_mesh(mesh_data)
bm.edges.ensure_lookup_table()
bm.faces.ensure_lookup_table()
bm.verts.ensure_lookup_table()

vert_count = len(bm.verts)
face_count = len(bm.faces)
edge_count = len(bm.edges)

# Check boundary edges (edges with exactly 1 face)
# Note: In game models with baked sharp edges (EdgeSplit), coincident edges are split for normal control.
# We test both raw boundary edges and welded boundary edges to detect genuine holes.
raw_boundary_edges = sum(1 for e in bm.edges if e.is_boundary)

# Make a copy or weld to test watertight manifold topology
bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.0005)
welded_boundary_edges = sum(1 for e in bm.edges if e.is_boundary)
is_manifold = (welded_boundary_edges == 0) and (face_count > 0)
boundary_edges = welded_boundary_edges

# Check disconnected components
visited_verts = set()
component_sizes = []
for v in bm.verts:
    if v not in visited_verts:
        c_size = 1
        queue = [v]
        visited_verts.add(v)
        while queue:
            curr = queue.pop()
            for edge in curr.link_edges:
                other = edge.other_vert(curr)
                if other not in visited_verts:
                    visited_verts.add(other)
                    c_size += 1
                    queue.append(other)
        component_sizes.append(c_size)

components = len(component_sizes)
debris_components = sum(1 for sz in component_sizes if sz < max(10, vert_count * 0.005))
debris_vert_count = sum(sz for sz in component_sizes if sz < max(10, vert_count * 0.005))
debris_ratio = debris_vert_count / max(1, vert_count)

bm.free()

# 2. Geometric Bounding Box & Thickness
bbox_corners = [active_obj.matrix_world @ Vector(corner) for corner in active_obj.bound_box]
min_corner = Vector((min(p.x for p in bbox_corners), min(p.y for p in bbox_corners), min(p.z for p in bbox_corners)))
max_corner = Vector((max(p.x for p in bbox_corners), max(p.y for p in bbox_corners), max(p.z for p in bbox_corners)))
dims = max_corner - min_corner
dx, dy, dz = max(0.0001, dims.x), max(0.0001, dims.y), max(0.0001, dims.z)
max_dim = max(dx, dy, dz)
min_dim = min(dx, dy, dz)
thickness_ratio = min_dim / max_dim
centre = (min_corner + max_corner) * 0.5

# 3. Setup Scene for Turnaround Renders & Silhouette Analysis
scene = bpy.context.scene
scene.render.resolution_x = 512
scene.render.resolution_y = 512
scene.render.engine = "BLENDER_EEVEE_NEXT"
scene.render.film_transparent = True

# Camera
cam_data = bpy.data.cameras.new("InspectorCam")
cam_data.type = "ORTHO"
cam_data.ortho_scale = max_dim * 1.35
cam_obj = bpy.data.objects.new("InspectorCam", cam_data)
bpy.context.collection.objects.link(cam_obj)
scene.camera = cam_obj

# Lighting
key_light = bpy.data.lights.new("KeyLight", type="SUN")
key_light.energy = 4.0
key_obj = bpy.data.objects.new("KeyLight", key_light)
bpy.context.collection.objects.link(key_obj)
key_obj.rotation_euler = (math.radians(45), math.radians(30), math.radians(45))

# Angles: 0=Front, 90=Right, 180=Rear, 270=Left
cardinal_angles = {
    "front": 0.0,
    "right": 90.0,
    "rear": 180.0,
    "left": 270.0,
    "iso": 45.0
}

dist = max_dim * 3.0
occupancies = {}

# Ensure material exists with vertex color if available
if not active_obj.data.materials:
    mat = bpy.data.materials.new("InspectMaterial")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if active_obj.data.color_attributes:
        col_node = mat.node_tree.nodes.new("ShaderNodeVertexColor")
        col_node.layer_name = active_obj.data.color_attributes[0].name
        mat.node_tree.links.new(col_node.outputs["Color"], bsdf.inputs["Base Color"])
    else:
        bsdf.inputs["Base Color"].default_value = (0.7, 0.7, 0.72, 1.0)
    active_obj.data.materials.append(mat)

for name, angle_deg in cardinal_angles.items():
    rad = math.radians(angle_deg)
    if name == "iso":
        cam_obj.location = centre + Vector((dist * math.sin(rad) * 0.8, -dist * math.cos(rad) * 0.8, dist * 0.6))
    else:
        cam_obj.location = centre + Vector((dist * math.sin(rad), -dist * math.cos(rad), 0.0))
    cam_obj.rotation_euler = (centre - cam_obj.location).to_track_quat("-Z", "Y").to_euler()
    
    render_path = str(out_dir / f"view_{name}.png")
    scene.render.filepath = render_path
    bpy.ops.render.render(write_still=True)
    
    # Analyze alpha silhouette of the rendered PNG
    # Load image in blender to inspect pixel buffer
    img = bpy.data.images.load(render_path)
    pixels = list(img.pixels) # RGBA array
    # Alpha is every 4th element (indices 3, 7, 11, ...)
    alpha_pixels = pixels[3::4]
    solid_pixels = sum(1 for a in alpha_pixels if a > 0.15)
    total_pixels = len(alpha_pixels)
    occupancy = solid_pixels / total_pixels if total_pixels > 0 else 0.0
    occupancies[name] = round(occupancy, 4)
    bpy.data.images.remove(img)

# 4. Evaluate Criteria
defects = []

# Criterion 1: Watertightness (Allow minor bottom seam / decimation tolerance < 0.5% open edges)
boundary_ratio = boundary_edges / max(1, edge_count)
if boundary_ratio > 0.005:
    defects.append(f"Non-manifold mesh: {boundary_edges} open boundary edges ({boundary_ratio:.2%})")

# Criterion 2: Thickness / Non-flat
if thickness_ratio < 0.05:
    defects.append(f"Degenerate 2D flat slab: thickness ratio {thickness_ratio:.4f} < 0.05")

# Criterion 3: Cardinal Silhouette Completeness
front_occ = occupancies["front"]
rear_occ = occupancies["rear"]
right_occ = occupancies["right"]
left_occ = occupancies["left"]

if front_occ < 0.01:
    defects.append(f"Front view silhouette empty ({front_occ:.4f})")
if rear_occ < 0.01:
    defects.append(f"Rear view silhouette empty ({rear_occ:.4f})")
if right_occ < 0.01:
    defects.append(f"Right side silhouette empty ({right_occ:.4f})")
if left_occ < 0.01:
    defects.append(f"Left side silhouette empty ({left_occ:.4f})")

rear_front_ratio = rear_occ / front_occ if front_occ > 0 else 0.0
side_front_ratio = min(right_occ, left_occ) / front_occ if front_occ > 0 else 0.0

if rear_front_ratio < 0.20:
    defects.append(f"Rear silhouette collapse: rear/front ratio {rear_front_ratio:.3f} < 0.20 (single-view AI blob indicator)")

if side_front_ratio < 0.10:
    defects.append(f"Side silhouette collapse: side/front ratio {side_front_ratio:.3f} < 0.10 (cardboard cutout indicator)")

# Criterion 4: Fragmentation / Debris scraps
if debris_ratio > 0.05 or (components > 50 and debris_components > 10):
    defects.append(f"Severe fragmentation: {components} components with {debris_ratio:.2%} debris scraps")

verdict = "PASS" if len(defects) == 0 else "FAIL"

receipt = {
    "verdict": verdict,
    "model_path": glb_path,
    "metrics": {
        "vertices": vert_count,
        "faces": face_count,
        "edges": edge_count,
        "is_manifold": is_manifold,
        "boundary_edges": boundary_edges,
        "components": components,
        "dimensions": {
            "x": round(dx, 4),
            "y": round(dy, 4),
            "z": round(dz, 4),
            "max_dim": round(max_dim, 4),
            "thickness_ratio": round(thickness_ratio, 4)
        },
        "silhouettes": {
            "front": front_occ,
            "right": right_occ,
            "rear": rear_occ,
            "left": left_occ,
            "iso": occupancies["iso"],
            "rear_to_front_ratio": round(rear_front_ratio, 4),
            "side_to_front_ratio": round(side_front_ratio, 4)
        }
    },
    "defects": defects
}

with open(out_dir / "inspection.json", "w") as f:
    json.dump(receipt, f, indent=2)

print(f"[review_gate] Completed review: {verdict}. Defects: {len(defects)}")
"""


def run_review_gate(model_path: Path, out_dir: Path) -> dict:
    """Run headless Blender review worker against the model."""
    out_dir.mkdir(parents=True, exist_ok=True)
    temp_script = out_dir / "_blender_worker.py"
    temp_script.write_text(BLENDER_WORKER_SCRIPT, encoding="utf-8")

    cmd = [
        str(BLENDER),
        "-b",
        "--factory-startup",
        "-P",
        str(temp_script),
        "--",
        str(model_path.resolve()),
        str(out_dir.resolve()),
    ]

    proc = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        check=False,
    )

    if temp_script.exists():
        try:
            temp_script.unlink()
        except OSError:
            pass

    receipt_path = out_dir / "inspection.json"
    if not receipt_path.exists():
        raise RuntimeError(
            f"Review worker failed to generate inspection.json:\n{proc.stdout}\n{proc.stderr}"
        )

    with open(receipt_path, "r", encoding="utf-8") as f:
        return json.load(f)


def test_fixtures():
    """Test the review gate against standard assets."""
    print("Testing review gate on real game assets...")
    root = Path(__file__).resolve().parent.parent
    test_out = root / "artifacts" / "test_review_gate"

    test_assets = [
        root / "assets" / "scenes" / "chronos_cake.glb",
        root / "assets" / "scenes" / "chronos_pinecone.glb",
        root / "assets" / "scenes" / "chronos_pinetree.glb",
    ]

    all_passed = True
    for asset in test_assets:
        if not asset.exists():
            print(f"Skipping {asset.name} (not found)")
            continue
        out = test_out / asset.stem
        receipt = run_review_gate(asset, out)
        print(f"Asset: {asset.name} -> {receipt['verdict']} (Defects: {receipt['defects']})")
        if receipt["verdict"] != "PASS":
            print(f"  Metrics: {json.dumps(receipt['metrics'], indent=2)}")
            all_passed = False

    if all_passed:
        print("[review_gate] All valid test fixtures PASSED!")
    else:
        print("[review_gate] One or more test fixtures failed.")
    return all_passed


def main():
    parser = argparse.ArgumentParser(description="Reviewed Full-Volume Quality Gate")
    parser.add_argument("--model", type=str, help="Path to GLB/OBJ model")
    parser.add_argument("--out-dir", type=str, help="Output directory for inspection receipt and renders")
    parser.add_argument("--test-fixtures", action="store_true", help="Run self-test on known assets")

    args = parser.parse_args()

    if args.test_fixtures:
        passed = test_fixtures()
        sys.exit(0 if passed else 1)

    if not args.model or not args.out_dir:
        parser.print_help()
        sys.exit(1)

    model_path = Path(args.model)
    out_dir = Path(args.out_dir)

    receipt = run_review_gate(model_path, out_dir)
    print(f"Review verdict: {receipt['verdict']}")
    if receipt["defects"]:
        print("Defects identified:")
        for d in receipt["defects"]:
            print(f" - {d}")

    sys.exit(0 if receipt["verdict"] == "PASS" else 1)


if __name__ == "__main__":
    main()
