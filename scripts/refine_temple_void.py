"""Refine the isolated Council Chamber Blender copy and export its runtime GLB.

This script refuses the operator's original ``uiscene1.blend``. It preserves but
hides the oversized star halo and overhead vault so the authored lights and assets
read against an absolute black void.
"""

from pathlib import Path
import bpy

WORKING_NAME = "uiscene1.codex-temple.blend"
OUT = Path(r"C:\archetypes\assets\scenes\uiscene1.glb")
HIDDEN_FROM_RUNTIME = ("Temple_Star_Halo", "Temple_Ceiling_Ring", "Temple_Vault")

blend_path = Path(bpy.data.filepath)
if blend_path.name != WORKING_NAME:
    raise RuntimeError(f"Refusing to modify {blend_path}; expected isolated {WORKING_NAME}")

for name in HIDDEN_FROM_RUNTIME:
    obj = bpy.data.objects.get(name)
    if obj is None:
        raise RuntimeError(f"Required authored object is missing: {name}")
    obj.hide_render = True
    obj.hide_viewport = True

bpy.context.scene.world.color = (0.0, 0.0, 0.0)
bpy.ops.wm.save_as_mainfile(filepath=str(blend_path))
bpy.ops.object.select_all(action="DESELECT")
for obj in bpy.context.scene.objects:
    if obj.name not in HIDDEN_FROM_RUNTIME and not obj.hide_render:
        obj.select_set(True)
bpy.ops.export_scene.gltf(
    filepath=str(OUT),
    export_format="GLB",
    export_apply=True,
    export_yup=True,
    export_animations=False,
    use_selection=True,
)
print("REFINED", blend_path)
print("EXPORTED", OUT)
