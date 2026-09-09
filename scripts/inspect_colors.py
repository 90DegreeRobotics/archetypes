import bpy
import numpy as np

files = {
    "aura": r"C:\archetypes\assets\scenes\aura.glb",
    "sentinel": r"C:\archetypes\assets\scenes\sentinel.glb",
    "oracle": r"C:\archetypes\assets\scenes\oracle.glb",
    "nebula_jester": r"C:\archetypes\assets\scenes\nebula_jester.glb",
    "empath": r"C:\archetypes\assets\scenes\empath.glb",
}

for name, path in files.items():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=path)
    print(f"\n==================== {name.upper()} ====================")
    for img in bpy.data.images:
        if "basecolor" in img.name.lower() or "image_0" in img.name.lower():
            # Get pixels
            px = np.array(img.pixels[:])
            px = px.reshape(-1, 4)
            # compute mean of non-black pixels
            mask = px[:, 3] > 0.1
            visible = px[mask]
            mean_rgb = visible[:, :3].mean(axis=0)
            p75 = np.percentile(visible[:, :3], 75, axis=0)
            p90 = np.percentile(visible[:, :3], 90, axis=0)
            print(f"Image: {img.name} ({img.size[0]}x{img.size[1]})")
            print(f"  Mean RGB: ({mean_rgb[0]:.2f}, {mean_rgb[1]:.2f}, {mean_rgb[2]:.2f})")
            print(f"  75th %:   ({p75[0]:.2f}, {p75[1]:.2f}, {p75[2]:.2f})")
            print(f"  90th %:   ({p90[0]:.2f}, {p90[1]:.2f}, {p90[2]:.2f})")
