"""Generic, no-recipe Chronos Object-mode OBJ -> GLB handoff for Archetypes."""
import argparse
import math
import os
import sys
import bpy
from mathutils import Vector

MAX_GAME_TRIANGLES = 75_000

# Faces meeting at more than this angle keep a hard edge; anything shallower is smoothed.
# 30 degrees is the usual hard-surface threshold: it holds a box corner crisp while letting a
# cylinder or an organic curve read as continuous.
SHARP_EDGE_DEGREES = 30.0
CHRONOS_FORWARD_AXIS = "NEGATIVE_Y"
CHRONOS_UP_AXIS = "Z"

def main():
    args = argparse.ArgumentParser()
    args.add_argument("--input", required=True)
    args.add_argument("--output", required=True)
    ns = args.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else None)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    # Chronos/TripoSR declares Z-up in triposr_artifact.json. Blender's OBJ
    # importer defaults to Y-up; accepting that default rotates every generated
    # subject onto its side before the later glTF Y-up conversion. Import in the
    # producer's declared basis so the GLB exporter performs the one intended
    # Z-up -> Y-up conversion.
    bpy.ops.wm.obj_import(
        filepath=ns.input,
        forward_axis=CHRONOS_FORWARD_AXIS,
        up_axis=CHRONOS_UP_AXIS,
    )
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    if len(meshes) != 1:
        raise RuntimeError(f"expected exactly one Chronos source mesh, found {len(meshes)}")
    obj = meshes[0]
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bounds = [obj.matrix_world @ Vector(c) for c in obj.bound_box]
    lo = Vector((min(p.x for p in bounds), min(p.y for p in bounds), min(p.z for p in bounds)))
    hi = Vector((max(p.x for p in bounds), max(p.y for p in bounds), max(p.z for p in bounds)))
    obj.location -= Vector(((lo.x + hi.x) / 2, (lo.y + hi.y) / 2, lo.z))
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)
    if max(hi - lo) <= 0:
        raise RuntimeError("Chronos source mesh has empty bounds")
    obj.scale *= 1.4 / max(hi - lo)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.calc_loop_triangles()
    source_triangles = len(obj.data.loop_triangles)
    if source_triangles > MAX_GAME_TRIANGLES:
        modifier = obj.modifiers.new(name="GameTriangleBudget", type="DECIMATE")
        modifier.decimate_type = "COLLAPSE"
        modifier.ratio = MAX_GAME_TRIANGLES / source_triangles
        modifier.use_collapse_triangulate = True
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        obj.data.calc_loop_triangles()
    repaired_mesh = obj.data.validate(verbose=True, clean_customdata=False)
    obj.data.update(calc_edges=True, calc_edges_loose=True)

    # Smooth the curves, keep the corners.
    #
    # Every polygon used to be forced smooth, which rounds off every hard edge in the mesh - a
    # plinth, a camera body, a machined rim all came out looking melted, on top of whatever
    # softness marching cubes had already introduced. Smoothing everything and smoothing
    # nothing are both wrong; the rule is the angle between faces.
    #
    # EdgeSplit rather than Blender's auto-smooth modifier because this has to survive a glTF
    # export: splitting the geometry at sharp edges bakes the discontinuity into the mesh
    # itself, so the normals are correct in any engine that loads it, with no dependence on
    # smoothing metadata the format may not carry.
    for polygon in obj.data.polygons:
        polygon.use_smooth = True
    split = obj.modifiers.new(name="SharpEdges", type="EDGE_SPLIT")
    split.split_angle = math.radians(SHARP_EDGE_DEGREES)
    split.use_edge_angle = True
    split.use_edge_sharp = True
    bpy.ops.object.modifier_apply(modifier=split.name)

    # An explicit material carrying the reconstruction's own vertex colour.
    #
    # The export previously carried `COLOR_0` but no material at all, leaving every engine to
    # invent one. The colour itself round-trips correctly - the exporter converts TripoSR's
    # sRGB vertex colours to linear, which is what glTF and Bevy expect - but whether it is
    # *used* was left to whatever default the consumer picked. This states it.
    material = bpy.data.materials.new("ChronosArtifact")
    material.use_nodes = True
    tree = material.node_tree
    principled = tree.nodes["Principled BSDF"]
    colours = obj.data.color_attributes
    if colours:
        attribute = tree.nodes.new("ShaderNodeVertexColor")
        attribute.layer_name = colours[0].name
        tree.links.new(attribute.outputs["Color"], principled.inputs["Base Color"])
    # TripoSR recovers colour only - there is no roughness or metalness in the reconstruction,
    # and guessing them from the subject would be a recipe. A single plausible dielectric
    # setting applies to everything equally and lets the colour do the work.
    principled.inputs["Roughness"].default_value = 0.55
    principled.inputs["Metallic"].default_value = 0.0
    obj.data.materials.clear()
    obj.data.materials.append(material)

    obj.data.calc_loop_triangles()
    game_triangles = len(obj.data.loop_triangles)
    print(
        f"[archetypes-import] sharp-edge threshold {SHARP_EDGE_DEGREES} deg; "
        f"vertex colour {'carried' if colours else 'absent'}"
    )
    print(
        f"[archetypes-import] generic triangle budget: "
        f"{source_triangles} -> {game_triangles} (limit {MAX_GAME_TRIANGLES}); "
        f"mesh_repaired={repaired_mesh}"
    )
    os.makedirs(os.path.dirname(os.path.abspath(ns.output)), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=ns.output, export_format="GLB", use_selection=True, export_apply=True, export_yup=True)
    if not os.path.isfile(ns.output) or os.path.getsize(ns.output) < 1024:
        raise RuntimeError("GLB export was absent or implausibly small")

    # Reviewed full-volume quality gate receipt
    import bmesh
    import json
    out_dir = os.path.dirname(os.path.abspath(ns.output))
    inspection_path = os.path.join(out_dir, "inspection.json")

    defects = []
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.edges.ensure_lookup_table()
    edge_count = len(bm.edges)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.0005)
    boundary_edges = sum(1 for e in bm.edges if e.is_boundary)
    boundary_ratio = boundary_edges / max(1, edge_count)
    if boundary_ratio > 0.005:
        defects.append(f"Non-manifold mesh: {boundary_edges} open boundary edges ({boundary_ratio:.2%})")

    dims = hi - lo
    min_dim = min(dims.x, dims.y, dims.z)
    max_dim = max(dims.x, dims.y, dims.z)
    thickness_ratio = min_dim / max(0.0001, max_dim)
    if thickness_ratio < 0.05:
        defects.append(f"Degenerate 2D flat slab: thickness ratio {thickness_ratio:.4f} < 0.05")

    bm.free()

    verdict = "PASS" if len(defects) == 0 else "FAIL"
    receipt = {
        "verdict": verdict,
        "model_path": ns.output,
        "metrics": {
            "vertices": len(obj.data.vertices),
            "triangles": game_triangles,
            "boundary_edges": boundary_edges,
            "thickness_ratio": round(thickness_ratio, 4)
        },
        "defects": defects
    }
    with open(inspection_path, "w", encoding="utf-8") as f:
        json.dump(receipt, f, indent=2)
    print(f"[archetypes-import] review gate verdict: {verdict}")

if __name__ == "__main__":
    main()
