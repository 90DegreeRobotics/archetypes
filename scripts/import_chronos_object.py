"""Generic, no-recipe Chronos Object-mode OBJ -> GLB handoff for Archetypes."""
import argparse
import os
import sys
import bpy
from mathutils import Vector

MAX_GAME_TRIANGLES = 75_000
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
    for polygon in obj.data.polygons:
        polygon.use_smooth = True
    obj.data.calc_loop_triangles()
    game_triangles = len(obj.data.loop_triangles)
    print(
        f"[archetypes-import] generic triangle budget: "
        f"{source_triangles} -> {game_triangles} (limit {MAX_GAME_TRIANGLES}); "
        f"mesh_repaired={repaired_mesh}"
    )
    os.makedirs(os.path.dirname(os.path.abspath(ns.output)), exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=ns.output, export_format="GLB", use_selection=True, export_apply=True, export_yup=True)
    if not os.path.isfile(ns.output) or os.path.getsize(ns.output) < 1024:
        raise RuntimeError("GLB export was absent or implausibly small")

if __name__ == "__main__":
    main()
