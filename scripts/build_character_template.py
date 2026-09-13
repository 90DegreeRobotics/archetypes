"""
Blender 4.5 Python Script: Build Reusable Character Authoring Template & Scene
Executed via headless Blender:
& "C:\Program Files\Blender Foundation\Blender 4.5\blender.exe" -b -P scripts/build_character_template.py
"""

import sys
import os
import math
from pathlib import Path
import bpy
from mathutils import Vector, Euler

REPO_ROOT = Path(r"C:\archetypes")
CHAR_DIR = REPO_ROOT / "assets" / "authoring" / "characters" / "nebula_jester"
TEMPLATE_DIR = REPO_ROOT / "assets" / "authoring" / "characters" / "template"
PROOF_DIR = REPO_ROOT / "artifacts" / "visual-proof" / "character_setup"

REF_ALPHA_PATH = CHAR_DIR / "reference_front_alpha.png"
OUTPUT_BLEND = CHAR_DIR / "nebula_jester_modeling.blend"
OUTPUT_TEMPLATE_BLEND = TEMPLATE_DIR / "character_template.blend"
PREVIEW_IMG = PROOF_DIR / "viewport_preview.png"

def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    for c in list(bpy.data.collections):
        bpy.data.collections.remove(c)

def setup_units():
    scene = bpy.context.scene
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1.0
    scene.unit_settings.length_unit = 'METERS'

def get_or_create_collection(name, parent_coll=None):
    if name in bpy.data.collections:
        coll = bpy.data.collections[name]
    else:
        coll = bpy.data.collections.new(name)
        if parent_coll:
            parent_coll.children.link(coll)
        else:
            bpy.context.scene.collection.children.link(coll)
    return coll

def create_clay_material():
    mat = bpy.data.materials.new("MAT_Modeling_Clay")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    nodes.clear()
    output = nodes.new('ShaderNodeOutputMaterial')
    output.location = (300, 0)
    principled = nodes.new('ShaderNodeBsdfPrincipled')
    principled.location = (0, 0)
    principled.inputs['Base Color'].default_value = (0.78, 0.76, 0.74, 1.0)
    principled.inputs['Roughness'].default_value = 0.40
    principled.inputs['Metallic'].default_value = 0.0
    mat.node_tree.links.new(principled.outputs['BSDF'], output.inputs['Surface'])
    return mat

def create_reference_plane(ref_coll, image_path):
    print(f"Loading reference image: {image_path}")
    if not image_path.exists():
        print(f"Warning: {image_path} does not exist!")
        return None

    img = bpy.data.images.load(str(image_path))

    # Total character height in image = 1462px -> 1.95m.
    # Image aspect: 1024 x 1536.
    img_height = 1.95 * (1536.0 / 1462.0)  # ~2.0487m
    img_width = img_height * (1024.0 / 1536.0)  # ~1.3658m

    plane_z = img_height / 2.0 - 0.0467
    plane_x = 0.0086
    plane_y = 0.04

    # 1. Textured Mesh Plane (visible in viewport rendered & material preview)
    bpy.ops.mesh.primitive_plane_add(size=1.0, location=(plane_x, plane_y, plane_z))
    plane = bpy.context.active_object
    plane.name = "REF_Front_Image_Plane"
    plane.scale = (img_width, img_height, 1.0)
    plane.rotation_euler = (math.radians(90), 0, 0)
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    ref_coll.objects.link(plane)
    bpy.context.scene.collection.objects.unlink(plane)

    mat = bpy.data.materials.new("MAT_Reference_Front")
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    nodes.clear()
    out_node = nodes.new('ShaderNodeOutputMaterial')
    out_node.location = (400, 0)
    bsdf = nodes.new('ShaderNodeBsdfPrincipled')
    bsdf.location = (100, 0)
    tex_node = nodes.new('ShaderNodeTexImage')
    tex_node.location = (-250, 0)
    tex_node.image = img

    mat.node_tree.links.new(tex_node.outputs['Color'], bsdf.inputs['Base Color'])
    mat.node_tree.links.new(tex_node.outputs['Alpha'], bsdf.inputs['Alpha'])
    mat.node_tree.links.new(bsdf.outputs['BSDF'], out_node.inputs['Surface'])

    bsdf.inputs['Roughness'].default_value = 1.0
    bsdf.inputs['Specular IOR Level'].default_value = 0.0
    plane.data.materials.append(mat)

    # 2. Reference Empty (visible in wireframe/solid orthographic view)
    empty = bpy.data.objects.new("REF_Front_Empty", None)
    empty.empty_display_type = 'IMAGE'
    empty.data = img
    empty.empty_display_size = img_height
    empty.rotation_euler = (math.radians(90), 0, 0)
    empty.location = (plane_x, plane_y - 0.01, plane_z)
    empty.color[3] = 0.70
    empty.show_empty_image_orthographic = True
    empty.show_empty_image_perspective = True
    ref_coll.objects.link(empty)

    return plane

def add_mirror_and_apply_origin(obj):
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    mod = obj.modifiers.new(name="Mirror_X", type='MIRROR')
    mod.use_axis[0] = True
    mod.use_clip = True
    mod.merge_threshold = 0.002
    return mod

def build_segmented_tube(name, rings, num_sides=8):
    verts = []
    faces = []

    for r_idx, (center, radius, normal) in enumerate(rings):
        c = Vector(center)
        norm = Vector(normal).normalized()
        up = Vector((0, 0, 1))
        if abs(norm.dot(up)) > 0.95:
            up = Vector((0, 1, 0))
        tangent = norm.cross(up).normalized()
        bitangent = norm.cross(tangent).normalized()

        for s in range(num_sides):
            angle = 2.0 * math.pi * s / num_sides
            v = c + tangent * (radius * math.cos(angle)) + bitangent * (radius * math.sin(angle))
            verts.append(v)

        if r_idx > 0:
            offset_prev = (r_idx - 1) * num_sides
            offset_curr = r_idx * num_sides
            for s in range(num_sides):
                next_s = (s + 1) % num_sides
                faces.append([
                    offset_prev + s,
                    offset_prev + next_s,
                    offset_curr + next_s,
                    offset_curr + s
                ])

    mesh_data = bpy.data.meshes.new(name + "_Mesh")
    mesh_data.from_pydata(verts, [], faces)
    mesh_data.update()
    obj = bpy.data.objects.new(name, mesh_data)
    bpy.context.scene.collection.objects.link(obj)
    return obj

def create_blockout_geometry(blockout_coll, clay_mat):
    print("Building anatomically calibrated blockout geometry...")

    def register_mesh(obj, name, add_subsurf=True):
        obj.name = name
        obj.show_wire = True
        obj.show_all_edges = True
        if add_subsurf:
            sub = obj.modifiers.new(name="Subsurf", type='SUBSURF')
            sub.levels = 1
        add_mirror_and_apply_origin(obj)
        obj.data.materials.append(clay_mat)
        blockout_coll.objects.link(obj)
        if obj.name in bpy.context.scene.collection.objects:
            bpy.context.scene.collection.objects.unlink(obj)
        return obj

    # 1. Head (Chin 1.49m, Visor 1.63m, Cranium 1.76m)
    bpy.ops.mesh.primitive_cube_add(size=0.18, location=(0, 0, 1.63))
    head = bpy.context.active_object
    head.scale = (0.75, 0.95, 1.25)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    register_mesh(head, "Blockout_Head")

    # Neck
    bpy.ops.mesh.primitive_cylinder_add(radius=0.045, depth=0.12, location=(0, 0, 1.48))
    neck = bpy.context.active_object
    register_mesh(neck, "Blockout_Neck")

    # 2. Jester Horns (Embedded in skull at (0.07, 1.70) arching up to peak (0.33, 1.94) and down to bell (0.40, 1.66))
    horn_rings = [
        ((0.07, 0.0, 1.70), 0.046, (0.5, 0.0, 0.86)),
        ((0.18, 0.0, 1.88), 0.042, (0.85, 0.0, 0.52)),
        ((0.31, 0.0, 1.94), 0.036, (0.98, 0.0, -0.2)),
        ((0.38, 0.0, 1.82), 0.030, (0.4, 0.0, -0.9)),
        ((0.40, 0.0, 1.66), 0.022, (0.1, 0.0, -1.0))
    ]
    horn_obj = build_segmented_tube("Blockout_Jester_Horns", horn_rings, num_sides=8)
    register_mesh(horn_obj, "Blockout_Jester_Horns")

    # Horn bell tips
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.030, location=(0.405, 0.0, 1.65))
    bell = bpy.context.active_object
    register_mesh(bell, "Blockout_Horn_Bells", add_subsurf=False)

    # Shoulder Armor Caps (Left, mirrored)
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.072, location=(0.26, 0.0, 1.36))
    shoulder = bpy.context.active_object
    register_mesh(shoulder, "Blockout_Shoulder")

    # 3. Chest / Upper Torso (Reactor core center 1.24m, Shoulders 1.36m)
    bpy.ops.mesh.primitive_cube_add(size=0.20, location=(0, 0, 1.28))
    chest = bpy.context.active_object
    chest.scale = (1.30, 0.85, 1.25)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    register_mesh(chest, "Blockout_Chest")

    # 4. Abdomen / Waist (0.95m to 1.15m)
    bpy.ops.mesh.primitive_cube_add(size=0.16, location=(0, 0, 1.05))
    abdomen = bpy.context.active_object
    abdomen.scale = (0.90, 0.75, 0.90)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    register_mesh(abdomen, "Blockout_Abdomen")

    # 5. Pelvis / Hips (0.80m to 0.95m)
    bpy.ops.mesh.primitive_cube_add(size=0.18, location=(0, 0, 0.86))
    pelvis = bpy.context.active_object
    pelvis.scale = (1.05, 0.80, 0.85)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    register_mesh(pelvis, "Blockout_Pelvis")

    # 6. Upper Arm (Shoulder (0.26, 0, 1.36) to Elbow (0.37, 0, 1.10))
    arm_rings = [
        ((0.26, 0.0, 1.36), 0.062, (0.38, 0.0, -0.92)),
        ((0.315, 0.0, 1.23), 0.055, (0.38, 0.0, -0.92)),
        ((0.37, 0.0, 1.10), 0.048, (0.38, 0.0, -0.92))
    ]
    arm_up = build_segmented_tube("Blockout_Arm_Upper", arm_rings, num_sides=8)
    register_mesh(arm_up, "Blockout_Arm_Upper")

    # 7. Forearm (Elbow (0.37, 0, 1.10) to Wrist (0.38, 0, 0.82))
    forearm_rings = [
        ((0.37, 0.0, 1.10), 0.050, (0.04, 0.0, -0.99)),
        ((0.375, 0.0, 0.96), 0.046, (0.04, 0.0, -0.99)),
        ((0.38, 0.0, 0.82), 0.040, (0.04, 0.0, -0.99))
    ]
    forearm = build_segmented_tube("Blockout_Forearm", forearm_rings, num_sides=8)
    register_mesh(forearm, "Blockout_Forearm")

    # 8. Hand (Wrist (0.38, 0, 0.82) to Fingertips (0.40, 0, 0.64))
    hand_rings = [
        ((0.38, 0.0, 0.82), 0.038, (0.1, 0.0, -0.99)),
        ((0.39, 0.0, 0.73), 0.034, (0.1, 0.0, -0.99)),
        ((0.40, 0.0, 0.64), 0.024, (0.1, 0.0, -0.99))
    ]
    hand = build_segmented_tube("Blockout_Hand", hand_rings, num_sides=6)
    register_mesh(hand, "Blockout_Hand")

    # 9. Thigh (Hip (0.13, 0, 0.82) to Knee (0.17, 0, 0.48))
    thigh_rings = [
        ((0.13, 0.0, 0.82), 0.092, (0.12, 0.0, -0.99)),
        ((0.15, 0.0, 0.65), 0.084, (0.12, 0.0, -0.99)),
        ((0.17, 0.0, 0.48), 0.070, (0.12, 0.0, -0.99))
    ]
    thigh = build_segmented_tube("Blockout_Thigh", thigh_rings, num_sides=8)
    register_mesh(thigh, "Blockout_Thigh")

    # 10. Shin (Knee (0.17, 0, 0.48) to Ankle (0.18, 0, 0.11))
    shin_rings = [
        ((0.17, 0.0, 0.48), 0.070, (0.03, 0.0, -0.99)),
        ((0.175, 0.0, 0.30), 0.058, (0.03, 0.0, -0.99)),
        ((0.18, 0.0, 0.11), 0.048, (0.03, 0.0, -0.99))
    ]
    shin = build_segmented_tube("Blockout_Shin", shin_rings, num_sides=8)
    register_mesh(shin, "Blockout_Shin")

    # 11. Foot (Ankle (0.18, 0, 0.11) to Sole flush to 0.00m)
    bpy.ops.mesh.primitive_cube_add(size=0.10, location=(0.185, -0.04, 0.05))
    foot = bpy.context.active_object
    foot.scale = (0.9, 1.8, 0.5)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    register_mesh(foot, "Blockout_Foot")

def create_humanoid_armature(rig_coll):
    print("Building humanoid biped animation rig...")
    arm_data = bpy.data.armatures.new("Armature_Humanoid_Biped")
    arm_obj = bpy.data.objects.new("Rig_Humanoid_Biped", arm_data)
    rig_coll.objects.link(arm_obj)

    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode='EDIT')

    edit_bones = arm_data.edit_bones

    # Root
    root = edit_bones.new("root")
    root.head = Vector((0, 0, 0))
    root.tail = Vector((0, 0.15, 0))

    # Hips / Pelvis
    hips = edit_bones.new("hips")
    hips.head = Vector((0, 0, 0.80))
    hips.tail = Vector((0, 0, 0.95))
    hips.parent = root

    # Spine
    spine = edit_bones.new("spine")
    spine.head = Vector((0, 0, 0.95))
    spine.tail = Vector((0, 0, 1.18))
    spine.parent = hips

    # Chest
    chest = edit_bones.new("chest")
    chest.head = Vector((0, 0, 1.18))
    chest.tail = Vector((0, 0, 1.42))
    chest.parent = spine

    # Neck
    neck = edit_bones.new("neck")
    neck.head = Vector((0, 0, 1.42))
    neck.tail = Vector((0, 0, 1.52))
    neck.parent = chest

    # Head
    head = edit_bones.new("head")
    head.head = Vector((0, 0, 1.52))
    head.tail = Vector((0, 0, 1.76))
    head.parent = neck

    def add_mirrored_limb(name_base, p_head, p_tail, parent_name):
        b_l = edit_bones.new(f"{name_base}.L")
        b_l.head = Vector(p_head)
        b_l.tail = Vector(p_tail)
        if parent_name:
            b_l.parent = edit_bones[parent_name]

        b_r = edit_bones.new(f"{name_base}.R")
        b_r.head = Vector((-p_head[0], p_head[1], p_head[2]))
        b_r.tail = Vector((-p_tail[0], p_tail[1], p_tail[2]))
        if parent_name:
            r_parent = parent_name.replace(".L", ".R") if ".L" in parent_name else parent_name
            b_r.parent = edit_bones[r_parent]

    # Arms
    add_mirrored_limb("clavicle", (0.04, 0.0, 1.40), (0.24, 0.0, 1.38), "chest")
    add_mirrored_limb("upper_arm", (0.26, 0.0, 1.36), (0.37, 0.0, 1.10), "clavicle.L")
    add_mirrored_limb("forearm", (0.37, 0.0, 1.10), (0.38, 0.0, 0.82), "upper_arm.L")
    add_mirrored_limb("hand", (0.38, 0.0, 0.82), (0.40, 0.0, 0.64), "forearm.L")

    # Legs
    add_mirrored_limb("thigh", (0.13, 0.0, 0.82), (0.17, 0.0, 0.48), "hips")
    add_mirrored_limb("shin", (0.17, 0.0, 0.48), (0.18, 0.0, 0.11), "thigh.L")
    add_mirrored_limb("foot", (0.18, 0.0, 0.11), (0.19, -0.15, 0.02), "shin.L")
    add_mirrored_limb("toe", (0.19, -0.15, 0.02), (0.19, -0.26, 0.0), "foot.L")

    # Jester Horns
    add_mirrored_limb("horn_base", (0.11, 0.0, 1.70), (0.33, 0.0, 1.94), "head")
    add_mirrored_limb("horn_tip", (0.33, 0.0, 1.94), (0.40, 0.0, 1.66), "horn_base.L")

    bpy.ops.object.mode_set(mode='OBJECT')

    arm_data.use_mirror_x = True
    arm_data.display_type = 'OCTAHEDRAL'
    arm_obj.show_in_front = True

    return arm_obj

def setup_environment_and_camera(guide_coll):
    # Floor circle indicator
    bpy.ops.mesh.primitive_circle_add(radius=1.2, location=(0, 0, 0))
    circle = bpy.context.active_object
    circle.name = "Guide_Ground_Ring"
    guide_coll.objects.link(circle)
    bpy.context.scene.collection.objects.unlink(circle)

    # Front Orthographic Camera
    cam_data = bpy.data.cameras.new("Cam_Front_Ortho")
    cam_data.type = 'ORTHO'
    cam_data.ortho_scale = 2.4
    cam_obj = bpy.data.objects.new("Camera_Front_Ortho", cam_data)
    cam_obj.location = (0.0, -4.0, 1.0)
    cam_obj.rotation_euler = (math.radians(90), 0, 0)
    guide_coll.objects.link(cam_obj)
    bpy.context.scene.camera = cam_obj

    # Studio Lights
    def add_light(name, ltype, energy, loc, color=(1, 1, 1)):
        ldata = bpy.data.lights.new(name, ltype)
        ldata.energy = energy
        ldata.color = color
        lobj = bpy.data.objects.new(name, ldata)
        lobj.location = loc
        guide_coll.objects.link(lobj)
        return lobj

    add_light("Light_Key", 'AREA', 250, (2.5, -3.0, 2.5))
    add_light("Light_Fill", 'AREA', 80, (-2.5, -2.5, 1.5))
    add_light("Light_Rim", 'AREA', 180, (0.0, 2.5, 2.8))

    return cam_obj

def configure_render_and_preview(guide_coll, ortho_cam):
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_EEVEE_NEXT' if hasattr(bpy.types, 'RenderSettings') and 'BLENDER_EEVEE_NEXT' in [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties['engine'].enum_items] else 'BLENDER_EEVEE'
    scene.render.resolution_x = 1080
    scene.render.resolution_y = 1440

    # 1. Front Ortho render
    scene.camera = ortho_cam
    scene.render.filepath = str(PREVIEW_IMG)
    bpy.ops.render.render(write_still=True)
    print(f"Rendered viewport preview to: {PREVIEW_IMG}")

    # 2. 3/4 Perspective render
    cam_p_data = bpy.data.cameras.new("Cam_Persp_Proof")
    cam_p_obj = bpy.data.objects.new("Camera_Persp_Proof", cam_p_data)
    cam_p_obj.location = Vector((2.2, -3.4, 1.5))
    dir_vec = Vector((0.0, 0.0, 1.05)) - cam_p_obj.location
    cam_p_obj.rotation_euler = dir_vec.to_track_quat('-Z', 'Y').to_euler()
    guide_coll.objects.link(cam_p_obj)

    scene.camera = cam_p_obj
    scene.render.filepath = str(PROOF_DIR / "perspective_preview.png")
    bpy.ops.render.render(write_still=True)
    print(f"Rendered perspective preview to: {PROOF_DIR / 'perspective_preview.png'}")

    # Reset active camera to Front Ortho for modeling ergonomics
    scene.camera = ortho_cam

def main():
    clear_scene()
    setup_units()

    ref_coll = get_or_create_collection("01_Reference")
    blockout_coll = get_or_create_collection("02_Blockout_Mirrored")
    rig_coll = get_or_create_collection("03_Rig_Armature")
    guide_coll = get_or_create_collection("04_Guides_Studio")

    clay_mat = create_clay_material()

    # Load reference image
    create_reference_plane(ref_coll, REF_ALPHA_PATH)

    # Blockout meshes
    create_blockout_geometry(blockout_coll, clay_mat)

    # Humanoid Biped Rig
    create_humanoid_armature(rig_coll)

    # Camera & Lights
    cam_obj = setup_environment_and_camera(guide_coll)

    # Save Nebula Jester blend file
    OUTPUT_BLEND.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(OUTPUT_BLEND))
    print(f"Saved character scene to: {OUTPUT_BLEND}")

    # Also save as reusable template
    OUTPUT_TEMPLATE_BLEND.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(OUTPUT_TEMPLATE_BLEND))
    print(f"Saved reusable template to: {OUTPUT_TEMPLATE_BLEND}")

    # Render preview snapshot
    configure_render_and_preview(guide_coll, cam_obj)

if __name__ == "__main__":
    main()
