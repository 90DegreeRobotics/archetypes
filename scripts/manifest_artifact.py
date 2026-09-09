"""
Headless Chronos Manifestation Pipeline for Archetypes.
Synthesizes, grounds, and exports game-ready 3D artifacts live based on player prompts.
Executed via Blender 4.5 background mode.
"""

import os
import sys
import math
import argparse
import bpy
from mathutils import Vector, Euler

def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)

def create_material(name, base_color, metallic=0.85, roughness=0.25, emissive_color=(0,0,0,1), emissive_strength=0.0):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (*base_color[:3], 1.0)
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
        if "Emission Color" in bsdf.inputs:
            bsdf.inputs["Emission Color"].default_value = (*emissive_color[:3], 1.0)
            bsdf.inputs["Emission Strength"].default_value = emissive_strength
    return mat

def build_artifact_mesh(prompt_text):
    p = prompt_text.lower()
    
    gold_mat = create_material("ChronosGold", (0.95, 0.78, 0.28), metallic=0.92, roughness=0.20)
    crystal_mat = create_material("ChronosCrystal", (0.35, 0.85, 1.0), metallic=0.1, roughness=0.10, emissive_color=(0.3, 0.8, 1.0, 1.0), emissive_strength=1.5)
    obsidian_mat = create_material("ChronosObsidian", (0.05, 0.05, 0.08), metallic=0.7, roughness=0.15)
    ruby_mat = create_material("ChronosRuby", (0.95, 0.15, 0.25), metallic=0.3, roughness=0.15, emissive_color=(0.9, 0.1, 0.2, 1.0), emissive_strength=1.8)
    emerald_mat = create_material("ChronosEmerald", (0.15, 0.90, 0.45), metallic=0.3, roughness=0.15, emissive_color=(0.1, 0.9, 0.4, 1.0), emissive_strength=1.8)

    created_objs = []

    if any(k in p for k in ["chalice", "goblet", "cup", "grail"]):
        # Ornate Goblet / Chalice
        bpy.ops.mesh.primitive_cylinder_add(radius=0.38, depth=0.08, location=(0, 0, 0.04))
        base = bpy.context.active_object
        base.data.materials.append(gold_mat)
        created_objs.append(base)

        bpy.ops.mesh.primitive_cylinder_add(radius=0.10, depth=0.45, location=(0, 0, 0.30))
        stem = bpy.context.active_object
        stem.data.materials.append(obsidian_mat)
        created_objs.append(stem)

        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.16, location=(0, 0, 0.30))
        gem = bpy.context.active_object
        gem.data.materials.append(crystal_mat)
        created_objs.append(gem)

        bpy.ops.mesh.primitive_cylinder_add(radius=0.36, depth=0.42, location=(0, 0, 0.70))
        bowl = bpy.context.active_object
        bowl.data.materials.append(gold_mat)
        created_objs.append(bowl)

        bpy.ops.mesh.primitive_torus_add(major_radius=0.38, minor_radius=0.035, location=(0, 0, 0.91))
        rim = bpy.context.active_object
        rim.data.materials.append(crystal_mat)
        created_objs.append(rim)

    elif any(k in p for k in ["dagger", "sword", "blade", "knife"]):
        # Ornate Dagger / Blade
        bpy.ops.mesh.primitive_cylinder_add(radius=0.08, depth=0.12, location=(0, 0, 0.06))
        pommel = bpy.context.active_object
        pommel.data.materials.append(ruby_mat)
        created_objs.append(pommel)

        bpy.ops.mesh.primitive_cylinder_add(radius=0.055, depth=0.32, location=(0, 0, 0.28))
        hilt = bpy.context.active_object
        hilt.data.materials.append(obsidian_mat)
        created_objs.append(hilt)

        bpy.ops.mesh.primitive_cube_add(size=0.15, location=(0, 0, 0.46))
        guard = bpy.context.active_object
        guard.scale = (3.2, 0.5, 0.5)
        guard.data.materials.append(gold_mat)
        created_objs.append(guard)

        bpy.ops.mesh.primitive_cone_add(radius1=0.12, depth=0.85, location=(0, 0, 0.90))
        blade = bpy.context.active_object
        blade.scale = (1.0, 0.22, 1.0)
        blade.data.materials.append(crystal_mat)
        created_objs.append(blade)

    elif any(k in p for k in ["astrolabe", "mechanism", "clock", "compass", "dial", "device"]):
        # Concentric Geared Astrolabe
        bpy.ops.mesh.primitive_torus_add(major_radius=0.55, minor_radius=0.04, location=(0, 0, 0.60))
        ring1 = bpy.context.active_object
        ring1.data.materials.append(gold_mat)
        created_objs.append(ring1)

        bpy.ops.mesh.primitive_torus_add(major_radius=0.42, minor_radius=0.035, location=(0, 0, 0.60))
        ring2 = bpy.context.active_object
        ring2.rotation_euler = (math.radians(35), math.radians(45), 0)
        ring2.data.materials.append(crystal_mat)
        created_objs.append(ring2)

        bpy.ops.mesh.primitive_cylinder_add(radius=0.28, depth=0.05, location=(0, 0, 0.60))
        plate = bpy.context.active_object
        plate.data.materials.append(obsidian_mat)
        created_objs.append(plate)

        bpy.ops.mesh.primitive_cylinder_add(radius=0.35, depth=0.15, location=(0, 0, 0.08))
        ped = bpy.context.active_object
        ped.data.materials.append(gold_mat)
        created_objs.append(ped)

        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.12, location=(0, 0, 0.60))
        core = bpy.context.active_object
        core.data.materials.append(ruby_mat)
        created_objs.append(core)

    elif any(k in p for k in ["crown", "tiara", "circlet", "diadem"]):
        # Sacred Royal Crown
        bpy.ops.mesh.primitive_torus_add(major_radius=0.42, minor_radius=0.05, location=(0, 0, 0.25))
        base_ring = bpy.context.active_object
        base_ring.data.materials.append(gold_mat)
        created_objs.append(base_ring)

        num_spikes = 7
        for i in range(num_spikes):
            angle = (2 * math.pi / num_spikes) * i
            x = 0.42 * math.cos(angle)
            y = 0.42 * math.sin(angle)
            bpy.ops.mesh.primitive_cone_add(radius1=0.07, depth=0.32, location=(x, y, 0.44))
            spike = bpy.context.active_object
            spike.data.materials.append(gold_mat)
            created_objs.append(spike)

            bpy.ops.mesh.primitive_uv_sphere_add(radius=0.05, location=(x, y, 0.62))
            jewel = bpy.context.active_object
            jewel.data.materials.append(ruby_mat if i % 2 == 0 else crystal_mat)
            created_objs.append(jewel)

    elif any(k in p for k in ["tree", "pine", "plant", "forest"]):
        # Mystic Evergreen Tree
        bpy.ops.mesh.primitive_cylinder_add(radius=0.12, depth=0.35, location=(0, 0, 0.18))
        trunk = bpy.context.active_object
        trunk.data.materials.append(obsidian_mat)
        created_objs.append(trunk)

        for tier in range(3):
            z_pos = 0.40 + tier * 0.28
            r = 0.50 - tier * 0.12
            bpy.ops.mesh.primitive_cone_add(radius1=r, depth=0.38, location=(0, 0, z_pos))
            foliage = bpy.context.active_object
            foliage.data.materials.append(emerald_mat)
            created_objs.append(foliage)

    elif any(k in p for k in ["buddha", "statue", "monk", "deity", "idol", "figure", "meditation"]):
        # Sacred Meditative Buddha Statue with Lotus Throne & Halo
        # 1. Lotus Throne Base
        bpy.ops.mesh.primitive_cylinder_add(radius=0.45, depth=0.10, location=(0, 0, 0.05))
        ped_base = bpy.context.active_object
        ped_base.data.materials.append(obsidian_mat)
        created_objs.append(ped_base)

        bpy.ops.mesh.primitive_torus_add(major_radius=0.42, minor_radius=0.06, location=(0, 0, 0.12))
        lotus_tier = bpy.context.active_object
        lotus_tier.data.materials.append(gold_mat)
        created_objs.append(lotus_tier)

        # Petals around the lotus throne
        num_petals = 12
        for i in range(num_petals):
            angle = (2 * math.pi / num_petals) * i
            x = 0.40 * math.cos(angle)
            y = 0.40 * math.sin(angle)
            bpy.ops.mesh.primitive_cone_add(radius1=0.08, depth=0.14, location=(x, y, 0.15))
            petal = bpy.context.active_object
            petal.rotation_euler = (0.35 * math.sin(angle), -0.35 * math.cos(angle), angle)
            petal.data.materials.append(gold_mat)
            created_objs.append(petal)

        # 2. Seated Lotus Posture (Crossed Legs)
        bpy.ops.mesh.primitive_cylinder_add(radius=0.35, depth=0.14, location=(0, 0, 0.24))
        legs = bpy.context.active_object
        legs.scale = (1.2, 0.85, 1.0)
        legs.data.materials.append(gold_mat)
        created_objs.append(legs)

        # 3. Torso with Robes
        bpy.ops.mesh.primitive_cone_add(radius1=0.24, radius2=0.16, depth=0.36, location=(0, 0, 0.48))
        torso = bpy.context.active_object
        torso.scale = (1.0, 0.8, 1.0)
        torso.data.materials.append(gold_mat)
        created_objs.append(torso)

        # 4. Shoulders & Draped Arms
        bpy.ops.mesh.primitive_torus_add(major_radius=0.22, minor_radius=0.06, location=(0, -0.02, 0.38))
        arms = bpy.context.active_object
        arms.scale = (1.1, 0.7, 1.0)
        arms.data.materials.append(gold_mat)
        created_objs.append(arms)

        # 5. Serene Head & Ushnisha
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.13, location=(0, 0, 0.73))
        head = bpy.context.active_object
        head.scale = (0.9, 0.95, 1.1)
        head.data.materials.append(gold_mat)
        created_objs.append(head)

        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.05, location=(0, 0, 0.88))
        ushnisha = bpy.context.active_object
        ushnisha.data.materials.append(gold_mat)
        created_objs.append(ushnisha)

        # 6. Radiant Aureole / Halo Disc
        bpy.ops.mesh.primitive_cylinder_add(radius=0.36, depth=0.03, location=(0, 0.08, 0.72))
        halo = bpy.context.active_object
        halo.rotation_euler = (math.radians(90), 0, 0)
        halo.data.materials.append(crystal_mat)
        created_objs.append(halo)

        bpy.ops.mesh.primitive_torus_add(major_radius=0.37, minor_radius=0.02, location=(0, 0.08, 0.72))
        halo_ring = bpy.context.active_object
        halo_ring.rotation_euler = (math.radians(90), 0, 0)
        halo_ring.data.materials.append(ruby_mat)
        created_objs.append(halo_ring)

    else:
        # Sacred Stellar Reliquary / Crystalline Monolith
        bpy.ops.mesh.primitive_cylinder_add(radius=0.40, depth=0.12, location=(0, 0, 0.06))
        base = bpy.context.active_object
        base.data.materials.append(obsidian_mat)
        created_objs.append(base)

        bpy.ops.mesh.primitive_torus_add(major_radius=0.42, minor_radius=0.03, location=(0, 0, 0.13))
        gold_trim = bpy.context.active_object
        gold_trim.data.materials.append(gold_mat)
        created_objs.append(gold_trim)

        # Stellated Octahedron / Crystal Cluster
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=0.36, location=(0, 0, 0.58))
        crystal = bpy.context.active_object
        crystal.scale = (0.85, 0.85, 1.45)
        crystal.data.materials.append(crystal_mat)
        created_objs.append(crystal)

        # Orbiting sacred ring
        bpy.ops.mesh.primitive_torus_add(major_radius=0.48, minor_radius=0.025, location=(0, 0, 0.58))
        ring = bpy.context.active_object
        ring.rotation_euler = (math.radians(45), math.radians(25), 0)
        ring.data.materials.append(gold_mat)
        created_objs.append(ring)

        # Core glowing jewel
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.15, location=(0, 0, 0.58))
        core = bpy.context.active_object
        core.data.materials.append(ruby_mat)
        created_objs.append(core)

    # Join all into one unified entity
    bpy.ops.object.select_all(action='DESELECT')
    for o in created_objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = created_objs[0]
    bpy.ops.object.join()
    final_obj = bpy.context.view_layer.objects.active
    final_obj.name = "ManifestedArtifactMesh"

    # Apply all transforms
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    # Ground feet flush to Z=0 and center horizontally at (0, 0)
    bbox = [final_obj.matrix_world @ Vector(c) for c in final_obj.bound_box]
    min_x = min(c.x for c in bbox)
    max_x = max(c.x for c in bbox)
    min_y = min(c.y for c in bbox)
    max_y = max(c.y for c in bbox)
    min_z = min(c.z for c in bbox)
    max_z = max(c.z for c in bbox)

    center_x = (min_x + max_x) / 2.0
    center_y = (min_y + max_y) / 2.0
    final_obj.location.x -= center_x
    final_obj.location.y -= center_y
    final_obj.location.z -= min_z
    bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)

    return final_obj

def export_manifested_artifact(prompt, output_glb):
    clear_scene()
    print(f"Manifesting artifact for prompt: '{prompt}'")
    obj = build_artifact_mesh(prompt)
    
    os.makedirs(os.path.dirname(os.path.abspath(output_glb)), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=output_glb,
        export_format='GLB',
        use_selection=True,
        export_apply=True,
        export_yup=True,
        export_materials='EXPORT',
        export_image_format='AUTO'
    )
    print(f"Successfully manifested GLB to: {output_glb} ({os.path.getsize(output_glb)} bytes)")

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("--prompt", type=str, default="a sacred crystalline reliquary")
    parser.add_argument("--output", type=str, default=r"C:\archetypes\assets\scenes\manifested_artifact.glb")
    
    if "--" in sys.argv:
        args_to_parse = sys.argv[sys.argv.index("--") + 1:]
    else:
        args_to_parse = sys.argv[1:]
    args = parser.parse_args(args_to_parse)
    
    export_manifested_artifact(args.prompt, args.output)
