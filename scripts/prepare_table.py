"""Prepare the portal table for real-time use.

Preserves the raw AI export as a hidden reference, then authors a clean real-time
council table from deterministic geometry: a dark circular top, restrained gold
rims, cyan inlay contained inside the tabletop, a living `Stargate_Portal`, and
four supported legs. Only the clean authored collection is exported.

Run: blender --background --python scripts/prepare_table.py
"""

import math
import bpy
from mathutils import Vector

SRC = r"C:\Users\m\Desktop\table.glb"
OUT = r"C:\archetypes\assets\scenes\table.glb"
TEXTURES = r"C:\archetypes\assets\textures\table"
EMISSIVE = TEXTURES + r"\table_emissive.png"
VORTEX = TEXTURES + r"\portal_vortex_v2.png"
PORTAL_RADIUS = 0.31
PORTAL_TOP_Z = 0.30  # Blender Z-up; table top ~0.28


def log(*a):
    print("NCTABLE:", *a)


def emission_input_name(bsdf):
    return "Emission Color" if "Emission Color" in bsdf.inputs else "Emission"


def cylinder_between(name, start, end, radius, material):
    start, end = Vector(start), Vector(end)
    delta = end - start
    bpy.ops.mesh.primitive_cylinder_add(vertices=24, radius=radius, depth=delta.length,
                                        location=(start + end) / 2)
    obj = bpy.context.active_object
    obj.name = name
    obj.rotation_mode = "QUATERNION"
    obj.rotation_quaternion = delta.to_track_quat("Z", "Y")
    obj.data.materials.append(material)
    return obj


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=SRC)
    for obj in bpy.context.scene.objects:
        obj.name = f"Legacy_Raw_Table_Source_{obj.name}"
        obj.hide_render = True
        obj.hide_viewport = True
        obj.select_set(False)
    log("raw AI table preserved as hidden, non-exported reference")

    def material(name, color, metallic, roughness, emission=None, strength=0.0):
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        bsdf.inputs["Base Color"].default_value = color
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
        if emission:
            bsdf.inputs[emission_input_name(bsdf)].default_value = emission
            bsdf.inputs["Emission Strength"].default_value = strength
        return mat

    gold = material("Council_Gold", (0.34, 0.16, 0.025, 1), 0.92, 0.2)
    dark = material("Council_Dark_Steel", (0.012, 0.018, 0.028, 1), 0.72, 0.28)
    cyan = material("Council_Cyan_Inlay", (0.0, 0.08, 0.12, 1), 0.3, 0.16,
                    (0.02, 0.62, 1.0, 1), 4.0)
    authored = []

    def cylinder(name, radius, depth, z, mat, vertices=96):
        bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=(0, 0, z))
        obj = bpy.context.active_object; obj.name = name; obj.data.materials.append(mat); authored.append(obj)
        return obj

    def torus(name, major, minor, z, mat):
        bpy.ops.mesh.primitive_torus_add(major_radius=major, minor_radius=minor,
                                        major_segments=96, minor_segments=12, location=(0, 0, z))
        obj = bpy.context.active_object; obj.name = name; obj.data.materials.append(mat); authored.append(obj)
        return obj

    cylinder("Council_Table_Top", 0.95, 0.12, 0.20, dark)
    torus("Council_Table_Outer_Rim", 0.87, 0.085, 0.27, gold)
    torus("Council_Table_Inner_Rim", 0.38, 0.035, 0.292, gold)
    for radius in (0.48, 0.63, 0.76):
        torus(f"Council_Geometry_Ring_{int(radius * 100)}", radius, 0.009, 0.294, cyan)
    for index in range(12):
        angle = math.tau * index / 12
        start = (0.39 * math.cos(angle), 0.39 * math.sin(angle), 0.294)
        end = (0.75 * math.cos(angle), 0.75 * math.sin(angle), 0.294)
        authored.append(cylinder_between(f"Council_Geometry_Ray_{index + 1}", start, end, 0.006, cyan))

    # Physical support and four splayed, readable legs.
    authored.append(cylinder_between("Table_Pedestal", (0, 0, -0.78), (0, 0, 0.14), 0.14, dark))
    torus("Table_Underside_Brace", 0.34, 0.055, -0.18, gold)
    for i in range(4):
        angle = math.radians(45 + i * 90)
        foot = (0.72 * math.cos(angle), 0.72 * math.sin(angle), -0.88)
        hip = (0.42 * math.cos(angle), 0.42 * math.sin(angle), 0.10)
        authored.append(cylinder_between(f"Table_Leg_{i + 1}", foot, hip, 0.07, gold))
        bpy.ops.mesh.primitive_cube_add(size=1, location=foot)
        foot_obj = bpy.context.active_object; foot_obj.name = f"Table_Foot_{i + 1}"
        foot_obj.scale = (0.18, 0.12, 0.07); foot_obj.rotation_euler[2] = angle
        foot_obj.data.materials.append(dark); authored.append(foot_obj)
    log("pedestal and four legs added")

    # --- stargate portal disc at the table centre ---
    bpy.ops.mesh.primitive_circle_add(
        vertices=72, radius=PORTAL_RADIUS, fill_type="NGON", location=(0, 0, PORTAL_TOP_Z)
    )
    disc = bpy.context.active_object
    disc.name = "Stargate_Portal"
    authored.append(disc)
    me = disc.data
    me.uv_layers.new(name="UVMap")
    uvl = me.uv_layers[0].data
    for poly in me.polygons:
        for li in poly.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            uvl[li].uv = (0.5 + co.x / (2 * PORTAL_RADIUS), 0.5 + co.y / (2 * PORTAL_RADIUS))

    pmat = bpy.data.materials.new("Stargate_Portal_Mat")
    pmat.use_nodes = True
    pbsdf = next(n for n in pmat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    pbsdf.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1.0)
    vtx = bpy.data.images.load(VORTEX)
    vnode = pmat.node_tree.nodes.new("ShaderNodeTexImage")
    vnode.image = vtx
    pmat.node_tree.links.new(vnode.outputs["Color"], pbsdf.inputs[emission_input_name(pbsdf)])
    pmat.node_tree.links.new(vnode.outputs["Color"], pbsdf.inputs["Base Color"])
    if "Emission Strength" in pbsdf.inputs:
        pbsdf.inputs["Emission Strength"].default_value = 3.0
    disc.data.materials.append(pmat)
    log("portal disc added")

    bpy.ops.object.select_all(action="DESELECT")
    for obj in authored:
        obj.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=OUT,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_yup=True,
    )
    log("exported", OUT)


main()
