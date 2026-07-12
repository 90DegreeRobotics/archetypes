"""Generate circular face-crop avatars for each archetype from the aura portraits.

Each council portrait is a full figure with the head at top-centre; we crop an
upper-centre square around the head and apply a circular alpha mask, producing
`assets/avatars/<archetype>.png` for use in the speaking bubbles.
"""

import os
from PIL import Image, ImageDraw

SRC = r"C:\archetypes\assets\aura"
OUT = r"C:\archetypes\assets\avatars"
ARCHETYPES = ["architect", "sentinel", "jester", "mentor", "explorer", "oracle", "empath"]
SIZE = 256

os.makedirs(OUT, exist_ok=True)
for name in ARCHETYPES:
    path = os.path.join(SRC, name + ".png")
    if not os.path.isfile(path):
        print("MISSING", name)
        continue
    im = Image.open(path).convert("RGBA")
    w, h = im.size
    side = int(0.44 * w)
    cx, cy = int(0.5 * w), int(0.16 * h)
    left = min(max(cx - side // 2, 0), w - side)
    top = min(max(cy - side // 2, 0), h - side)
    box = (left, top, left + side, top + side)
    face = im.crop(box).resize((SIZE, SIZE), Image.LANCZOS)
    mask = Image.new("L", (SIZE, SIZE), 0)
    ImageDraw.Draw(mask).ellipse((0, 0, SIZE, SIZE), fill=255)
    face.putalpha(mask)
    face.save(os.path.join(OUT, name + ".png"))
    print("wrote", name, "box", box, "of", (w, h))
