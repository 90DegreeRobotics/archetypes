"""Author the ornate Council astrolabe table and export the runtime GLB.

Reference: a round council table with a gilded rim of ancient glyphs, an integrated
plasma-etched geometric grid top, and an ornate brass + dark-steel leg assembly
(an arcade of curved ribs and pointed arches around a central turned pedestal),
sitting on the chamber floor.

Geometry contract (measured against the chamber + engine):
- Chamber floor top is Blender z = -5.03.
- Engine loads the table at scale 5, y = -1.1 (Blender-Z -> engine-Y), so feet at
  local z = -0.786 land exactly on the floor (-0.786*5 - 1.1 = -5.03).
- Node `Stargate_Portal` must be present; triangles < 200k.

Run:  blender --background --python scripts/prepare_table.py
Optional preview render:  ... -- --preview <path.png>
"""

import math
import sys
import bmesh
import bpy
from mathutils import Vector

OUT = r"C:\archetypes\assets\scenes\table.glb"
VORTEX = r"C:\archetypes\assets\textures\table\portal_vortex_v2.png"
TRIANGLE_LIMIT = 200_000

TOP_Z = 0.28          # top surface height (local); world ~ +0.3
FOOT_Z = -0.786       # feet height (local) -> sits on chamber floor
TOP_RADIUS = 0.95
PORTAL_RADIUS = 0.72
PORTAL_TOP_Z = 0.30

authored: list = []


def log(*a):
    print("NCTABLE:", *a)


def emission_input(bsdf):
    return "Emission Color" if "Emission Color" in bsdf.inputs else "Emission"


def material(name, color, metallic, roughness, emission=None, strength=0.0):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    bsdf.inputs["Base Color"].default_value = color
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    if emission is not None:
        bsdf.inputs[emission_input(bsdf)].default_value = emission
        if "Emission Strength" in bsdf.inputs:
            bsdf.inputs["Emission Strength"].default_value = strength
    mat.diffuse_color = color  # so the Workbench preview shows real colours
    return mat


def keep(obj, mat=None):
    if mat is not None:
        obj.data.materials.append(mat)
    authored.append(obj)
    return obj


def cylinder(name, radius, depth, z, mat, verts=64):
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=radius, depth=depth, location=(0, 0, z))
    o = bpy.context.active_object
    o.name = name
    return keep(o, mat)


def torus(name, major, minor, z, mat, seg=128, minor_seg=12, flatten=1.0, loc=(0, 0)):
    bpy.ops.mesh.primitive_torus_add(major_radius=major, minor_radius=minor,
                                     major_segments=seg, minor_segments=minor_seg,
                                     location=(loc[0], loc[1], z))
    o = bpy.context.active_object
    o.name = name
    if flatten != 1.0:
        o.scale.z = flatten
    return keep(o, mat)


def tube_between(name, start, end, radius, mat, verts=16):
    start, end = Vector(start), Vector(end)
    delta = end - start
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=radius, depth=delta.length,
                                        location=(start + end) / 2)
    o = bpy.context.active_object
    o.name = name
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = delta.to_track_quat("Z", "Y")
    return keep(o, mat)


def curved_tube(name, points, radius, mat, bevel_res=4):
    """A smooth beveled tube through the given control points (ornate legs/arches)."""
    curve = bpy.data.curves.new(name, "CURVE")
    curve.dimensions = "3D"
    spline = curve.splines.new("BEZIER")
    spline.bezier_points.add(len(points) - 1)
    for bp, co in zip(spline.bezier_points, points):
        bp.co = Vector(co)
        bp.handle_left_type = "AUTO"
        bp.handle_right_type = "AUTO"
    curve.bevel_depth = radius
    curve.bevel_resolution = bevel_res
    o = bpy.data.objects.new(name, curve)
    bpy.context.scene.collection.objects.link(o)
    return keep(o, mat)


def geometric_grid(name, radius, z, rings, mat, thickness=0.006):
    """A triangulated geodesic disc rendered as a glowing wireframe = plasma grid."""
    bm = bmesh.new()
    center = bm.verts.new((0, 0, 0))
    ring_verts = [[center]]
    for r in range(1, rings + 1):
        rad = radius * r / rings
        n = 6 * r
        row = [bm.verts.new((rad * math.cos(math.tau * i / n), rad * math.sin(math.tau * i / n), 0))
               for i in range(n)]
        ring_verts.append(row)
    for r in range(1, rings + 1):
        inner, outer = ring_verts[r - 1], ring_verts[r]
        ni, no = len(inner), len(outer)
        for i in range(no):
            o0, o1 = outer[i], outer[(i + 1) % no]
            i0 = inner[((i * ni) // no) % ni]
            i1 = inner[(((i + 1) * ni) // no) % ni]
            for tri in ((o0, o1, i0), (o1, i1, i0)):
                if len({v.index for v in tri}) == 3:
                    try:
                        bm.faces.new(tri)
                    except ValueError:
                        pass
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    o = bpy.data.objects.new(name, mesh)
    o.location = (0, 0, z)
    bpy.context.scene.collection.objects.link(o)
    wf = o.modifiers.new("wire", "WIREFRAME")
    wf.thickness = thickness
    wf.use_replace = True
    return keep(o, mat)


def build_table():
    brass = material("Council_Gilded_Brass", (0.82, 0.52, 0.15, 1), 0.95, 0.26)
    brass_lit = material("Council_Brass_Bright", (0.92, 0.66, 0.26, 1), 0.95, 0.18)
    steel = material("Council_Dark_Steel", (0.028, 0.033, 0.045, 1), 0.78, 0.34)
    top_dark = material("Council_Table_Surface", (0.020, 0.030, 0.052, 1), 0.15, 0.42)
    cyan = material("Council_Plasma_Cyan", (0.0, 0.09, 0.14, 1), 0.1, 0.16, (0.03, 0.78, 1.0, 1), 9.0)
    underglow = material("Council_Underglow", (0.0, 0.05, 0.09, 1), 0.0, 0.3, (0.02, 0.55, 1.0, 1), 5.0)

    # --- table top slab + ornate gilded glyph rim ---
    cylinder("Council_Table_Top", TOP_RADIUS, 0.09, TOP_Z - 0.045, top_dark, verts=128)
    # Wide gilded rim: an outer rounded molding + a raised flat glyph band.
    torus("Council_Rim_Outer_Molding", TOP_RADIUS - 0.02, 0.055, TOP_Z, brass, 160, 14)
    torus("Council_Rim_Glyph_Band", TOP_RADIUS - 0.11, 0.075, TOP_Z + 0.006, brass, 160, 14, flatten=0.42)
    torus("Council_Rim_Inner_Molding", TOP_RADIUS - 0.20, 0.028, TOP_Z + 0.004, brass_lit, 128, 10)
    # Engraved glyph blocks recessed into the rim band.
    for i in range(48):
        a = math.tau * i / 48
        r = TOP_RADIUS - 0.11
        bpy.ops.mesh.primitive_cube_add(size=1, location=(r * math.cos(a), r * math.sin(a), TOP_Z + 0.02),
                                        rotation=(0, 0, a))
        g = bpy.context.active_object
        g.name = f"Council_Rim_Glyph_{i + 1:02d}"
        g.scale = (0.010 if i % 2 else 0.016, 0.052, 0.012)
        keep(g, steel)

    # The tabletop is left clean — a dark polished surface with the portal glowing
    # at its centre and a single subtle underglow ring. (The operator did not want a
    # white grid etched across the top.)
    torus("Council_Underglow_Ring", 0.60, 0.018, TOP_Z - 0.11, underglow, 128, 10)

    # --- apron skirt under the top ---
    cylinder("Council_Apron", TOP_RADIUS - 0.14, 0.13, TOP_Z - 0.155, steel, verts=96)
    torus("Council_Apron_Molding", TOP_RADIUS - 0.14, 0.03, TOP_Z - 0.10, brass, 128, 8)

    # --- central turned pedestal ---
    tube_between("Council_Pedestal_Shaft", (0, 0, -0.62), (0, 0, 0.12), 0.115, brass, verts=32)
    for zz in (-0.5, -0.22, 0.02):
        torus(f"Council_Pedestal_Ring_{int((zz+1)*100)}", 0.155, 0.035, zz, brass_lit, 48, 8)
    cylinder("Council_Pedestal_Cap", 0.20, 0.06, 0.12, brass, verts=40)
    # spreading base foot of the pedestal
    bpy.ops.mesh.primitive_cone_add(vertices=48, radius1=0.34, radius2=0.14, depth=0.14,
                                    location=(0, 0, -0.70))
    base_cone = bpy.context.active_object
    base_cone.name = "Council_Pedestal_Base"
    keep(base_cone, steel)

    # --- ornate arcade: curved brass ribs + pointed arches + base ring + feet ---
    n_legs = 8
    leg_r = 0.63          # radius of the vertical rib circle
    torus("Council_Base_Ring", leg_r + 0.02, 0.05, FOOT_Z + 0.05, brass, 128, 10)
    torus("Council_Base_Ring_Molding", leg_r + 0.02, 0.022, FOOT_Z + 0.11, brass_lit, 128, 8)
    leg_tops = []
    for i in range(n_legs):
        a = math.tau * i / n_legs
        cos, sin = math.cos(a), math.sin(a)
        leg_tops.append(a)
        curved_tube(
            f"Council_Leg_Rib_{i + 1}",
            [
                (0.44 * cos, 0.44 * sin, 0.13),          # meets apron/pedestal top
                (0.60 * cos, 0.60 * sin, -0.06),         # gentle outward set
                (leg_r * cos, leg_r * sin, -0.46),       # nearly vertical shaft
                (leg_r * cos, leg_r * sin, FOOT_Z + 0.04),  # straight down to the foot
            ],
            0.058,
            brass,
        )
        # splayed foot block on the floor
        bpy.ops.mesh.primitive_cube_add(size=1, location=(leg_r * cos, leg_r * sin, FOOT_Z + 0.02),
                                        rotation=(0, 0, a))
        foot = bpy.context.active_object
        foot.name = f"Council_Foot_{i + 1}"
        foot.scale = (0.11, 0.06, 0.032)
        keep(foot, steel)

    # pointed arches between adjacent ribs (openwork gothic arcade)
    for i in range(n_legs):
        a0 = leg_tops[i]
        amid = a0 + (math.tau / n_legs) / 2.0
        apex = (0.58 * math.cos(amid), 0.58 * math.sin(amid), 0.02)
        for side, aa in ((0, a0), (1, leg_tops[(i + 1) % n_legs])):
            spring = (leg_r * math.cos(aa), leg_r * math.sin(aa), -0.30)
            curved_tube(
                f"Council_Arch_{i + 1}_{side}",
                [spring, ((spring[0] + apex[0]) / 2, (spring[1] + apex[1]) / 2, -0.06), apex],
                0.026,
                brass_lit,
                bevel_res=3,
            )

    # --- Stargate_Portal: the living rotation spans the full inner tabletop ---
    bpy.ops.mesh.primitive_circle_add(vertices=72, radius=PORTAL_RADIUS, fill_type="NGON",
                                      location=(0, 0, PORTAL_TOP_Z))
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
    pbsdf = next(n for n in pmat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    pbsdf.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1.0)
    vnode = pmat.node_tree.nodes.new("ShaderNodeTexImage")
    vnode.image = bpy.data.images.load(VORTEX)
    pmat.node_tree.links.new(vnode.outputs["Color"], pbsdf.inputs[emission_input(pbsdf)])
    pmat.node_tree.links.new(vnode.outputs["Color"], pbsdf.inputs["Base Color"])
    if "Emission Strength" in pbsdf.inputs:
        pbsdf.inputs["Emission Strength"].default_value = 3.0
    disc.data.materials.append(pmat)
    disc.data.materials[0] = pmat
    authored.append(disc)
    disc.select_set(False)
    log("portal disc added")


def triangle_count():
    return sum(len(p.vertices) - 2 for o in bpy.context.scene.objects
               if o.type == "MESH" for p in o.data.polygons)


def export():
    bpy.ops.object.select_all(action="DESELECT")
    for o in authored:
        o.select_set(True)
    bpy.ops.export_scene.gltf(filepath=OUT, export_format="GLB", use_selection=True,
                              export_apply=True, export_yup=True)
    log("exported", OUT)


def verify():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=OUT)
    names = {o.name for o in bpy.context.scene.objects}
    tris = triangle_count()
    pts = [o.matrix_world @ Vector(c) for o in bpy.context.scene.objects
           if o.type == "MESH" for c in o.bound_box]
    zmin = min(p.z for p in pts)
    zmax = max(p.z for p in pts)
    if "Stargate_Portal" not in names:
        raise RuntimeError("Stargate_Portal missing")
    if tris >= TRIANGLE_LIMIT:
        raise RuntimeError(f"{tris} triangles >= {TRIANGLE_LIMIT}")
    log("VERIFY", "Stargate_Portal=present", f"triangles={tris}",
        f"z_min={zmin:.3f} (feet; target -0.786)", f"z_max={zmax:.3f}")


def render_preview(path):
    """Workbench 3/4 render over a ground plane so the model can be judged."""
    bpy.ops.mesh.primitive_plane_add(size=8, location=(0, 0, FOOT_Z))
    bpy.context.active_object.name = "PreviewGround"
    cam_data = bpy.data.cameras.new("PreviewCam")
    cam_data.lens = 42
    cam = bpy.data.objects.new("PreviewCam", cam_data)
    bpy.context.scene.collection.objects.link(cam)
    cam.location = (2.3, -2.3, 0.95)
    cam.rotation_euler = (Vector((0, 0, -0.15)) - cam.location).to_track_quat("-Z", "Y").to_euler()
    bpy.context.scene.camera = cam
    scene = bpy.context.scene
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.render.resolution_x = 1280
    scene.render.resolution_y = 960
    scene.render.filepath = path
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.color_type = "MATERIAL"
    shading.show_cavity = True
    shading.show_shadows = True
    bpy.ops.render.render(write_still=True)
    log("preview", path)


def main():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    build_table()
    log(f"triangles={triangle_count()}")
    export()
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if "--preview" in argv:
        render_preview(argv[argv.index("--preview") + 1])
    verify()


main()
