import bpy

files = {
    "aura": r"C:\archetypes\assets\scenes\aura.glb",
    "sentinel": r"C:\archetypes\assets\scenes\sentinel.glb",
    "oracle": r"C:\archetypes\assets\scenes\oracle.glb",
    "nebula_jester": r"C:\archetypes\assets\scenes\nebula_jester.glb",
}

for name, path in files.items():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    print(f"\n--- {name.upper()} ---")
    for mat in bpy.data.materials:
        print(f"Material: {mat.name}")
        if mat.use_nodes:
            for node in mat.node_tree.nodes:
                if node.type == 'BSDF_PRINCIPLED':
                    base_col = node.inputs['Base Color'].default_value[:]
                    metallic = node.inputs['Metallic'].default_value
                    roughness = node.inputs['Roughness'].default_value
                    print(f"  Principled: BaseColor={base_col[:3]}, Metallic={metallic}, Roughness={roughness}")
                elif node.type == 'TEX_IMAGE':
                    img = node.image
                    if img:
                        print(f"  Image: {img.name}, size={img.size[:]}")
