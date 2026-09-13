"""Turn a static humanoid character GLB into a skinned, animated game avatar.

Usage (headless):
  blender -b --factory-startup -P scripts/rig_character_avatar.py -- <in.glb> <out.glb> <out.blend> <proof_dir>

Generic humanoid logic only: joints come from measured horizontal cross-sections of the mesh
(arm span band, leg split, neck waist), never from a per-character table. Expects a T/A-pose
mesh standing on Z=0 facing -Y, which is what the glTF importer produces for our Meshy exports.

Weights: bone-heat fails on dense, non-manifold generated meshes, so heat weights are solved on
a voxel-remeshed watertight proxy and transferred onto the decimated, UV-preserving game mesh.
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Quaternion, Vector

TARGET_TRIS = 45_000
PROXY_VOXEL = 0.012
FPS = 30


def args():
    argv = sys.argv[sys.argv.index("--") + 1:]
    if len(argv) != 4:
        raise SystemExit("expected: <in.glb> <out.glb> <out.blend> <proof_dir>")
    return argv


def log(*parts):
    print("RIG", *parts, flush=True)


def import_mesh(path):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    obj.parent = None
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    for o in list(bpy.context.scene.objects):
        if o.type == "EMPTY":
            bpy.data.objects.remove(o)
    obj.name = "Avatar_Mesh"
    return obj


def clean_and_decimate(obj):
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
    bm.to_mesh(obj.data)
    bm.free()
    tris = sum(len(p.vertices) - 2 for p in obj.data.polygons)
    if tris > TARGET_TRIS:
        mod = obj.modifiers.new("Decimate", "DECIMATE")
        mod.ratio = TARGET_TRIS / tris
        bpy.ops.object.modifier_apply(modifier=mod.name)
    after = sum(len(p.vertices) - 2 for p in obj.data.polygons)
    log("decimate", tris, "->", after)
    return after


def measure(obj):
    """Joint landmarks from horizontal slices."""
    vs = [v.co.copy() for v in obj.data.vertices]
    height = max(v.z for v in vs)
    bands = 60
    rows = []
    for i in range(bands):
        z0, z1 = height * i / bands, height * (i + 1) / bands
        band = [v for v in vs if z0 <= v.z < z1]
        if not band:
            rows.append(None)
            continue
        xs = sorted(v.x for v in band)
        segs, start, prev = [], xs[0], xs[0]
        for x in xs[1:]:
            if x - prev > 0.02:
                segs.append((start, prev))
                start = x
            prev = x
        segs.append((start, prev))
        rows.append(((z0 + z1) / 2, max(abs(xs[0]), abs(xs[-1])), segs))

    upper = [r for r in rows if r and r[0] > height * 0.55]
    arm_row = max(upper, key=lambda r: r[1])
    arm_z, hand_x = arm_row[0], arm_row[1]

    # Leg split: highest low band whose occupancy has a gap straddling x=0.
    crotch_z = height * 0.47
    for r in rows:
        if r and r[0] < height * 0.6:
            if any(a < -0.01 for a, b in r[2]) and not any(a < 0 < b for a, b in r[2]):
                crotch_z = r[0]

    # Torso width at chest just below the arm band sets the shoulder joint.
    chest_rows = [r for r in rows if r and arm_z - 0.20 < r[0] < arm_z - 0.08]
    torso_half = min(max(abs(a), abs(b)) for r in chest_rows for a, b in r[2]
                     if a < 0 < b) if chest_rows else height * 0.1

    # Neck: narrowest central segment between arm band and top.
    neck_z = arm_z + (height - arm_z) * 0.35
    best = 9.0
    for r in rows:
        if r and arm_z + 0.06 < r[0] < height - 0.15:
            centre = [b - a for a, b in r[2] if a <= 0 <= b]
            if centre and centre[0] < best:
                best, neck_z = centre[0], r[0]

    leg_x = []
    for r in rows:
        if r and height * 0.2 < r[0] < crotch_z - 0.05:
            leg_x += [(a + b) / 2 for a, b in r[2] if a > 0]
    leg_x = sum(leg_x) / len(leg_x) if leg_x else height * 0.06

    toe_y = min(v.y for v in vs if v.z < height * 0.05)
    m = dict(height=height, arm_z=arm_z, hand_x=hand_x, crotch_z=crotch_z,
             shoulder_x=torso_half * 0.95, neck_z=neck_z, leg_x=leg_x, toe_y=toe_y)
    log("landmarks", {k: round(v, 3) for k, v in m.items()})
    return m


def build_armature(m):
    h = m["height"]
    arm = bpy.data.armatures.new("Avatar_Armature")
    rig = bpy.data.objects.new("Avatar_Rig", arm)
    bpy.context.scene.collection.objects.link(rig)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.mode_set(mode="EDIT")
    eb = arm.edit_bones

    def bone(name, head, tail, parent=None, connect=False, deform=True):
        b = eb.new(name)
        b.head, b.tail = Vector(head), Vector(tail)
        b.roll = 0.0
        b.use_deform = deform
        if parent:
            b.parent = eb[parent]
            b.use_connect = connect
        return b

    hips_z = m["crotch_z"] + 0.06
    spine_z = hips_z + (m["arm_z"] - hips_z) * 0.33
    chest_z = hips_z + (m["arm_z"] - hips_z) * 0.66
    neck_z, head_z = m["neck_z"] - 0.04, m["neck_z"] + 0.05
    ax, az = m["hand_x"], m["arm_z"]
    sx = m["shoulder_x"]
    elbow_x = sx + (ax - sx) * 0.45
    wrist_x = sx + (ax - sx) * 0.80
    lx = m["leg_x"]
    knee_z, ankle_z = m["crotch_z"] * 0.55, h * 0.06

    bone("root", (0, 0, 0), (0, 0.3, 0), deform=False)
    bone("hips", (0, 0, hips_z), (0, 0, spine_z), "root")
    bone("spine", (0, 0, spine_z), (0, 0, chest_z), "hips", True)
    bone("chest", (0, 0, chest_z), (0, 0, neck_z), "spine", True)
    bone("neck", (0, 0, neck_z), (0, 0, head_z), "chest", True)
    bone("head", (0, 0, head_z), (0, 0, h), "neck", True)
    for side, s in (("L", 1.0), ("R", -1.0)):
        bone(f"clavicle.{side}", (s * 0.04, 0, az), (s * sx, 0, az), "chest")
        bone(f"upper_arm.{side}", (s * sx, 0, az), (s * elbow_x, 0, az), f"clavicle.{side}", True)
        bone(f"forearm.{side}", (s * elbow_x, 0, az), (s * wrist_x, 0, az), f"upper_arm.{side}", True)
        bone(f"hand.{side}", (s * wrist_x, 0, az), (s * ax, 0, az), f"forearm.{side}", True)
        bone(f"thigh.{side}", (s * lx, 0, hips_z - 0.04), (s * lx, 0, knee_z), "hips")
        bone(f"shin.{side}", (s * lx, 0, knee_z), (s * lx, 0, ankle_z), f"thigh.{side}", True)
        bone(f"foot.{side}", (s * lx, 0, ankle_z), (s * lx, m["toe_y"] * 0.5, 0.02), f"shin.{side}", True)
        bone(f"toe.{side}", (s * lx, m["toe_y"] * 0.5, 0.02), (s * lx, m["toe_y"], 0.02), f"foot.{side}", True)
    bpy.ops.object.mode_set(mode="OBJECT")
    log("bones", len(arm.bones))
    return rig


def solve_weights(mesh, rig):
    proxy = mesh.copy()
    proxy.data = mesh.data.copy()
    proxy.name = "Avatar_WeightProxy"
    bpy.context.scene.collection.objects.link(proxy)
    rem = proxy.modifiers.new("Remesh", "REMESH")
    rem.mode = "VOXEL"
    rem.voxel_size = PROXY_VOXEL
    bpy.context.view_layer.objects.active = proxy
    bpy.ops.object.modifier_apply(modifier=rem.name)
    log("proxy verts", len(proxy.data.vertices))

    bpy.ops.object.select_all(action="DESELECT")
    proxy.select_set(True)
    rig.select_set(True)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    unweighted = sum(1 for v in proxy.data.vertices if not any(g.weight > 0 for g in v.groups))
    log("proxy unweighted", unweighted, "of", len(proxy.data.vertices))

    for name in [b.name for b in rig.data.bones if b.use_deform]:
        if name not in mesh.vertex_groups:
            mesh.vertex_groups.new(name=name)
    dt = mesh.modifiers.new("WeightTransfer", "DATA_TRANSFER")
    dt.object = proxy
    dt.use_vert_data = True
    dt.data_types_verts = {"VGROUP_WEIGHTS"}
    dt.vert_mapping = "POLYINTERP_NEAREST"
    dt.layers_vgroup_select_src = "ALL"
    dt.layers_vgroup_select_dst = "NAME"
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
    bpy.ops.object.modifier_apply(modifier=dt.name)

    bpy.ops.object.mode_set(mode="WEIGHT_PAINT")
    bpy.ops.object.vertex_group_limit_total(group_select_mode="ALL", limit=4)
    bpy.ops.object.vertex_group_normalize_all(group_select_mode="ALL", lock_active=False)
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.data.objects.remove(proxy)

    unweighted = sum(1 for v in mesh.data.vertices if not any(g.weight > 0 for g in v.groups))
    log("game mesh unweighted", unweighted, "of", len(mesh.data.vertices))
    mesh.parent = rig
    mod = mesh.modifiers.new("Armature", "ARMATURE")
    mod.object = rig
    return unweighted


def world_rot(rig, bone_name, axis, degrees):
    """Rotation about a world axis expressed in the bone's rest frame."""
    local = rig.data.bones[bone_name].matrix_local.to_3x3().inverted() @ Vector(axis)
    return Quaternion(local.normalized(), math.radians(degrees))


def key_pose(rig, frame, pose):
    for pb in rig.pose.bones:
        pb.rotation_mode = "QUATERNION"
        pb.rotation_quaternion = Quaternion()
    for name, rotations in pose.items():
        q = Quaternion()
        for axis, deg in rotations:
            q = world_rot(rig, name, axis, deg) @ q
        rig.pose.bones[name].rotation_quaternion = q
    for pb in rig.pose.bones:
        pb.keyframe_insert("rotation_quaternion", frame=frame)


def arms_down(extra=0.0):
    # Positive rotation about world -Y... sides mirror, so the sign flips per side.
    return {"upper_arm.L": [((0, 1, 0), 68 + extra)], "upper_arm.R": [((0, 1, 0), -68 - extra)],
            "forearm.L": [((0, 0, 1), 8)], "forearm.R": [((0, 0, 1), -8)]}


def author_actions(rig):
    rig.animation_data_create()
    actions = {}

    idle = bpy.data.actions.new("Idle")
    idle.use_fake_user = True
    rig.animation_data.action = idle
    for frame, t in ((1, 0.0), (31, 1.0), (61, 0.0), (91, -1.0), (121, 0.0)):
        pose = arms_down(2.0 * t)
        pose["chest"] = [((1, 0, 0), 2.0 * t)]
        pose["head"] = [((0, 0, 1), 5.0 * t), ((1, 0, 0), -1.5 * abs(t))]
        pose["hips"] = [((0, 1, 0), 1.2 * t)]
        key_pose(rig, frame, pose)
    actions["Idle"] = idle

    wave = bpy.data.actions.new("Wave")
    wave.use_fake_user = True
    rig.animation_data.action = wave
    for frame, t in ((1, 0.0), (16, 1.0), (31, -1.0), (46, 1.0), (61, 0.0)):
        pose = arms_down()
        if frame in (1, 61):
            key_pose(rig, frame, pose)
            continue
        pose["upper_arm.R"] = [((0, 1, 0), 55)]
        pose["forearm.R"] = [((0, 1, 0), 60 + 25 * t)]
        pose["head"] = [((0, 0, 1), -8)]
        key_pose(rig, frame, pose)
    actions["Wave"] = wave

    for name, act in actions.items():
        track = rig.animation_data.nla_tracks.new()
        track.name = name
        track.strips.new(name, 1, act)
        track.mute = True
    rig.animation_data.action = idle
    bpy.context.scene.render.fps = FPS
    return actions


def render_proofs(rig, mesh, proof_dir):
    os.makedirs(proof_dir, exist_ok=True)
    scene = bpy.context.scene
    for engine in ("BLENDER_EEVEE_NEXT", "BLENDER_EEVEE"):
        try:
            scene.render.engine = engine
            break
        except TypeError:
            continue
    scene.render.resolution_x, scene.render.resolution_y = 720, 900
    world = bpy.data.worlds.new("ProofWorld")
    world.color = (0.02, 0.02, 0.03)
    scene.world = world
    cam_data = bpy.data.cameras.new("ProofCam")
    cam = bpy.data.objects.new("ProofCam", cam_data)
    scene.collection.objects.link(cam)
    h = max(v.co.z for v in mesh.data.vertices)
    cam.location = (1.9, -3.6, h * 0.62)
    direction = Vector((0, 0, h * 0.5)) - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    scene.camera = cam
    for name, loc, energy in (("Key", (2, -3, 3), 900), ("Fill", (-3, -2, 2), 400), ("Rim", (0, 3, 3), 600)):
        ld = bpy.data.lights.new(name, "AREA")
        ld.energy, ld.size = energy, 3
        lo = bpy.data.objects.new(name, ld)
        lo.location = loc
        lo.rotation_euler = (Vector((0, 0, 1)) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        scene.collection.objects.link(lo)

    shots = [("rest_tpose", None, 1), ("idle_f001", "Idle", 1), ("idle_f031", "Idle", 31),
             ("wave_f016", "Wave", 16), ("wave_f031", "Wave", 31)]
    for label, action, frame in shots:
        if action is None:
            rig.animation_data.action = None
            for pb in rig.pose.bones:
                pb.rotation_quaternion = Quaternion()
        else:
            rig.animation_data.action = bpy.data.actions[action]
        scene.frame_set(frame)
        scene.render.filepath = os.path.join(proof_dir, f"{label}.png")
        bpy.ops.render.render(write_still=True)
        log("proof", scene.render.filepath)
    rig.animation_data.action = bpy.data.actions["Idle"]
    scene.frame_set(1)


def export(rig, mesh, out_glb):
    bpy.ops.object.select_all(action="DESELECT")
    rig.select_set(True)
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = rig
    bpy.ops.export_scene.gltf(
        filepath=out_glb,
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_yup=True,
        export_skins=True,
        export_animations=True,
        export_animation_mode="ACTIONS",
        export_force_sampling=True,
        export_def_bones=True,
    )
    log("exported", out_glb, os.path.getsize(out_glb))


def main():
    in_glb, out_glb, out_blend, proof_dir = args()
    mesh = import_mesh(in_glb)
    clean_and_decimate(mesh)
    landmarks = measure(mesh)
    rig = build_armature(landmarks)
    unweighted = solve_weights(mesh, rig)
    if unweighted > len(mesh.data.vertices) * 0.01:
        raise SystemExit(f"weight transfer left {unweighted} vertices unweighted")
    author_actions(rig)
    os.makedirs(os.path.dirname(out_blend), exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=out_blend)
    export(rig, mesh, out_glb)
    render_proofs(rig, mesh, proof_dir)


main()
