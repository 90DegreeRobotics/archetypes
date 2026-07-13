"""Prepare the Lane C Flower-of-Life portal table for real-time use.

Preserves the raw AI export as a hidden reference, then exports only a clean
authored table: a dark circular top, thick gilded glyph rim, cyan Flower-of-Life
inlay, substantial brass/steel legs, and the approved `Stargate_Portal` disc.

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
TRIANGLE_LIMIT = 200_000


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


def triangle_count():
    return sum(
        len(poly.vertices) - 2
        for obj in bpy.context.scene.objects
        if obj.type == "MESH"
        for poly in obj.data.polygons
    )


def scene_bounds():
    points = []
    for obj in bpy.context.scene.objects:
        if obj.type != "MESH":
            continue
        points.extend(obj.matrix_world @ Vector(corner) for corner in obj.bound_box)
    if not points:
        return None
    mins = Vector((min(p.x for p in points), min(p.y for p in points), min(p.z for p in points)))
    maxs = Vector((max(p.x for p in points), max(p.y for p in points), max(p.z for p in points)))
    return mins, maxs


def verify_export(path):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    names = {obj.name for obj in bpy.context.scene.objects}
    triangles = triangle_count()
    bounds = scene_bounds()
    if "Stargate_Portal" not in names:
        raise RuntimeError("Stargate_Portal missing from table export")
    if triangles >= TRIANGLE_LIMIT:
        raise RuntimeError(f"table export has {triangles} triangles; limit is {TRIANGLE_LIMIT}")
    if bounds is None:
        raise RuntimeError("table export has no mesh bounds")
    mins, maxs = bounds
    log(
        "VERIFY",
        "Stargate_Portal=present",
        f"triangles={triangles}",
        "bbox_min=({:.3f},{:.3f},{:.3f})".format(mins.x, mins.y, mins.z),
        "bbox_max=({:.3f},{:.3f},{:.3f})".format(maxs.x, maxs.y, maxs.z),
    )


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

    gold = material("Council_Gilded_Brass", (0.78, 0.47, 0.13, 1), 0.92, 0.2)
    dark = material("Council_Polished_Black_Top", (0.012, 0.017, 0.027, 1), 0.05, 0.4)
    steel = material("Council_Dark_Steel", (0.020, 0.026, 0.036, 1), 0.7, 0.35)
    cyan = material("Council_Cyan_Inlay", (0.0, 0.08, 0.12, 1), 0.15, 0.16,
                    (0.02, 0.72, 1.0, 1), 8.0)
    glow = material("Council_Underglow_Cyan", (0.0, 0.035, 0.06, 1), 0.0, 0.25,
                    (0.02, 0.52, 1.0, 1), 5.5)
    authored = []

    def cylinder(name, radius, depth, z, mat, vertices=96):
        bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=(0, 0, z))
        obj = bpy.context.active_object; obj.name = name; obj.data.materials.append(mat); authored.append(obj)
        return obj

    def torus(name, major, minor, z, mat, major_segments=128, minor_segments=10, location=(0, 0)):
        bpy.ops.mesh.primitive_torus_add(major_radius=major, minor_radius=minor,
                                        major_segments=major_segments, minor_segments=minor_segments,
                                        location=(location[0], location[1], z))
        obj = bpy.context.active_object; obj.name = name; obj.data.materials.append(mat); authored.append(obj)
        return obj

    cylinder("Council_Table_Top", 0.98, 0.14, 0.20, dark, vertices=128)
    torus("Council_Table_Outer_Gilded_Rim", 0.89, 0.095, 0.282, gold, 144, 12)
    torus("Council_Table_Inner_Portal_Rim", 0.34, 0.030, 0.296, gold, 96, 8)
    torus("Council_Table_Underglow_Ring", 0.58, 0.014, 0.102, glow, 112, 8)

    # Plasma-etched Flower-of-Life: centre circle, six around it, then the outer
    # twelve-circle ring. The portal sits above the centre and remains visible.
    flower_radius = 0.185
    flower_centers = [(0.0, 0.0)]
    flower_centers.extend(
        (flower_radius * math.cos(math.tau * i / 6), flower_radius * math.sin(math.tau * i / 6))
        for i in range(6)
    )
    flower_centers.extend(
        (2 * flower_radius * math.cos(math.tau * i / 12), 2 * flower_radius * math.sin(math.tau * i / 12))
        for i in range(12)
    )
    for index, center in enumerate(flower_centers, start=1):
        torus(
            f"Council_Flower_Of_Life_Circle_{index:02d}",
            flower_radius,
            0.0055,
            0.286,
            cyan,
            96,
            6,
            center,
        )
    for radius in (0.54, 0.68, 0.79):
        torus(f"Council_Astrolabe_Guide_Ring_{int(radius * 100)}", radius, 0.005, 0.288, cyan, 128, 6)

    # Dark inset glyph blocks on the thick gold rim make it read engraved rather
    # than as a plain torus.
    for index in range(40):
        angle = math.tau * index / 40
        radius = 0.895
        bpy.ops.mesh.primitive_cube_add(
            size=1,
            location=(radius * math.cos(angle), radius * math.sin(angle), 0.366),
            rotation=(0, 0, angle),
        )
        glyph = bpy.context.active_object
        glyph.name = f"Council_Rim_Engraved_Glyph_{index + 1:02d}"
        glyph.scale = (0.012 if index % 3 else 0.017, 0.050, 0.006)
        glyph.data.materials.append(steel)
        authored.append(glyph)

    # Physical support and four splayed, readable legs.
    cylinder("Table_Central_Brass_Pedestal", 0.18, 0.56, -0.19, gold, vertices=48)
    authored.append(cylinder_between("Table_Dark_Core_Spine", (0, 0, -0.80), (0, 0, 0.14), 0.105, steel))
    torus("Table_Underside_Brass_Brace", 0.38, 0.060, -0.02, gold, 96, 10)
    torus("Table_Lower_Brass_Brace", 0.25, 0.045, -0.55, gold, 80, 8)
    for i in range(4):
        angle = math.radians(45 + i * 90)
        foot = (0.74 * math.cos(angle), 0.74 * math.sin(angle), -0.88)
        knee = (0.54 * math.cos(angle), 0.54 * math.sin(angle), -0.38)
        hip = (0.32 * math.cos(angle), 0.32 * math.sin(angle), 0.08)
        authored.append(cylinder_between(f"Table_Splayed_Brass_Leg_{i + 1}_Upper", hip, knee, 0.070, gold))
        authored.append(cylinder_between(f"Table_Splayed_Brass_Leg_{i + 1}_Lower", knee, foot, 0.082, gold))
        authored.append(cylinder_between(
            f"Table_Dark_Steel_Crossbrace_{i + 1}",
            (-0.20 * math.sin(angle), 0.20 * math.cos(angle), -0.30),
            (0.46 * math.cos(angle), 0.46 * math.sin(angle), -0.46),
            0.030,
            steel,
        ))
        bpy.ops.mesh.primitive_cube_add(size=1, location=foot)
        foot_obj = bpy.context.active_object; foot_obj.name = f"Table_Foot_{i + 1}"
        foot_obj.scale = (0.24, 0.15, 0.075); foot_obj.rotation_euler[2] = angle
        foot_obj.data.materials.append(steel); authored.append(foot_obj)
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
    verify_export(OUT)


main()
