"""Prepare the portal table for real-time use.

Imports the raw AI export of the golden clock-table, decimates it to a real-time
triangle budget, upgrades its single baked-albedo material to metallic gold with an
emissive map (so the engraved geometry and portal glow), and adds a `Stargate_Portal`
disc at the table's centre carrying an emissive vortex texture for the engine to spin.
Exports a lean `assets/scenes/table.glb`.

Run: blender --background --python scripts/prepare_table.py
"""

import math
import bpy

SRC = r"C:\Users\m\Desktop\table.glb"
OUT = r"C:\archetypes\assets\scenes\table.glb"
TEXTURES = r"C:\archetypes\assets\textures\table"
EMISSIVE = TEXTURES + r"\table_emissive.png"
VORTEX = TEXTURES + r"\portal_vortex.png"
TARGET_TRIS = 120000
PORTAL_RADIUS = 0.62
PORTAL_TOP_Z = 0.30  # Blender Z-up; table top ~0.28


def log(*a):
    print("NCTABLE:", *a)


def emission_input_name(bsdf):
    return "Emission Color" if "Emission Color" in bsdf.inputs else "Emission"


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=SRC)

    table = next(o for o in bpy.data.objects if o.type == "MESH")
    tris = len(table.data.polygons)
    log("imported", table.name, "polys", tris)

    # --- decimate to budget ---
    ratio = min(1.0, TARGET_TRIS / max(1, tris))
    bpy.context.view_layer.objects.active = table
    table.select_set(True)
    mod = table.modifiers.new("Decimate", "DECIMATE")
    mod.decimate_type = "COLLAPSE"
    mod.ratio = ratio
    bpy.ops.object.modifier_apply(modifier=mod.name)
    log("decimated ratio", round(ratio, 4), "-> polys", len(table.data.polygons))

    # --- upgrade the table material: metallic gold + emissive glow ---
    mat = table.data.materials[0]
    mat.use_nodes = True
    bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Metallic"].default_value = 0.55
    bsdf.inputs["Roughness"].default_value = 0.35
    em_img = bpy.data.images.load(EMISSIVE)
    em_node = mat.node_tree.nodes.new("ShaderNodeTexImage")
    em_node.image = em_img
    mat.node_tree.links.new(em_node.outputs["Color"], bsdf.inputs[emission_input_name(bsdf)])
    if "Emission Strength" in bsdf.inputs:
        bsdf.inputs["Emission Strength"].default_value = 2.2
    log("material upgraded", mat.name)

    # --- stargate portal disc at the table centre ---
    bpy.ops.mesh.primitive_circle_add(
        vertices=72, radius=PORTAL_RADIUS, fill_type="NGON", location=(0, 0, PORTAL_TOP_Z)
    )
    disc = bpy.context.active_object
    disc.name = "Stargate_Portal"
    me = disc.data
    me.uv_layers.new(name="UVMap")
    uvl = me.uv_layers[0].data
    for poly in me.polygons:
        for li in poly.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            uvl[li].uv = (0.5 + co.x / (2 * PORTAL_RADIUS), 0.5 + co.y / (2 * PORTAL_RADIUS))

    pmat = bpy.data.materials.new("Stargate_Portal_Mat")
    pmat.use_nodes = True
    pmat.blend_method = "BLEND"
    pbsdf = next(n for n in pmat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    pbsdf.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1.0)
    vtx = bpy.data.images.load(VORTEX)
    vnode = pmat.node_tree.nodes.new("ShaderNodeTexImage")
    vnode.image = vtx
    pmat.node_tree.links.new(vnode.outputs["Color"], pbsdf.inputs[emission_input_name(pbsdf)])
    pmat.node_tree.links.new(vnode.outputs["Alpha"], pbsdf.inputs["Alpha"])
    if "Emission Strength" in pbsdf.inputs:
        pbsdf.inputs["Emission Strength"].default_value = 3.0
    disc.data.materials.append(pmat)
    log("portal disc added")

    bpy.ops.export_scene.gltf(
        filepath=OUT,
        export_format="GLB",
        use_selection=False,
        export_apply=True,
        export_yup=True,
    )
    log("exported", OUT)


main()
