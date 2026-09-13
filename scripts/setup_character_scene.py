"""
Setup Character Authoring Template & Scene for Blender 4.5
Creates a reusable character modeling and rigging environment:
- Reference plane / empty aligned to front orthographic view
- Humanoid blockout meshes with active Mirror modifiers
- Biped humanoid armature with X-Axis mirror editing
- Studio lighting, ground plane, and front orthographic camera
"""

import os
import math
from pathlib import Path
from PIL import Image
import numpy as np

REPO_ROOT = Path(r"C:\archetypes")
BLENDER_EXE = Path(r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe")

SRC_IMG = Path(r"C:\Users\m\Downloads\ChatGPT Image Sep 9, 2026, 11_49_42 AM.png")
CHAR_DIR = REPO_ROOT / "assets" / "authoring" / "characters" / "nebula_jester"
TEMPLATE_DIR = REPO_ROOT / "assets" / "authoring" / "characters" / "template"
PROOF_DIR = REPO_ROOT / "artifacts" / "visual-proof" / "character_setup"

def prepare_reference_images():
    CHAR_DIR.mkdir(parents=True, exist_ok=True)
    TEMPLATE_DIR.mkdir(parents=True, exist_ok=True)
    PROOF_DIR.mkdir(parents=True, exist_ok=True)

    dst_raw = CHAR_DIR / "reference_front.png"
    dst_alpha = CHAR_DIR / "reference_front_alpha.png"

    print(f"Preparing reference image from {SRC_IMG}...")
    img = Image.open(SRC_IMG).convert("RGBA")
    img.save(dst_raw)

    arr = np.array(img).astype(np.float32)
    r = arr[:, :, 0]
    g = arr[:, :, 1]
    b = arr[:, :, 2]
    a = arr[:, :, 3]

    # Chroma key green detection
    # High green, significantly higher than red and blue
    max_rb = np.maximum(r, b)
    is_green = (g > 120) & (g > max_rb * 1.25)

    # Soft alpha feathering based on green excess
    excess = np.clip((g - max_rb) / 60.0, 0.0, 1.0)
    new_a = np.where(is_green, (1.0 - excess) * 255.0, a)

    # Spill suppression: remove green hue on edges
    new_g = np.where(g > max_rb, max_rb, g)

    arr[:, :, 1] = new_g
    arr[:, :, 3] = new_a
    arr = np.clip(arr, 0, 255).astype(np.uint8)

    alpha_img = Image.fromarray(arr, mode="RGBA")
    alpha_img.save(dst_alpha)
    print(f"Saved: {dst_raw}")
    print(f"Saved: {dst_alpha}")
    return dst_alpha

if __name__ == "__main__":
    prepare_reference_images()
