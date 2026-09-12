"""A measurement loop for manifested object quality.

The operator asked for objects that look real, and for random subjects to be tried and tuned
against. That needs a loop, not one-offs: generate a spread of subjects once, then re-render the
*same* meshes after each change to the import so the comparison isolates the change from
generation variance.

Two phases, deliberately separate:

  python scripts/manifest_quality_lab.py --generate 6
      Runs Chronos2 first-light for each test prompt and keeps the bundles. Slow (minutes each).

  python scripts/manifest_quality_lab.py --render
      Imports every kept bundle's mesh through `scripts/import_chronos_object.py`, renders it
      from three angles, and writes one contact sheet per subject plus a summary. Fast, and
      re-runnable after every tuning change.

**The prompts are test inputs, not recipes.** Nothing in the pipeline may branch on subject
matter — see `memory: no-recipes-is-the-north-star`. They are chosen only to span the kinds of
thing that break reconstruction differently: hard-edged, organic, thin-limbed, transparent,
reflective. If a fix only helps one of them, it is a recipe and does not ship.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

CHRONOS = Path(r"C:\chronos2\target\release\chronos.exe")
BLENDER = Path(r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe")
LAB = Path("artifacts/quality-lab")
BUNDLES = LAB / "bundles"
SHEETS = LAB / "sheets"

# Chosen to break differently, not to be catered to.
TEST_PROMPTS = [
    # Carried over from the first live in-game run, kept as a fixed before/after control.
    ("baseline_relic", "sacred celestial relic"),
    ("hard_edge", "a brass astrolabe"),
    ("organic", "a ceramic owl figurine"),
    ("thin_limbs", "a wrought iron candelabra"),
    ("transparent", "a cut crystal decanter"),
    ("reflective", "a polished copper kettle"),
    ("rough_stone", "a carved granite lion head"),
    ("fabric", "a folded wool blanket"),
    ("machined", "a vintage film camera"),
]

REFERENCE_CKPT = "Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors"


def log(*parts: object) -> None:
    print("[quality-lab]", *parts, flush=True)


def generate(count: int, timeout_secs: int) -> None:
    BUNDLES.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ)
    env.setdefault("CHRONOS_FORGE_REFERENCE_CKPT", REFERENCE_CKPT)
    env.setdefault("CHRONOS_FLUX_PROFILE", "lowvram")

    for label, prompt in TEST_PROMPTS[:count]:
        out = BUNDLES / label
        if (out / "engine_mesh").is_dir():
            log(f"{label}: already generated, skipping")
            continue
        if out.exists():
            shutil.rmtree(out, ignore_errors=True)

        log(f"{label}: generating '{prompt}' ...")
        started = time.time()
        result = subprocess.run(
            [
                str(CHRONOS), "first-light",
                "--prompt", prompt,
                "--out-dir", str(out.resolve()),
                "--geometry-forge", "--void",
            ],
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout_secs,
        )
        elapsed = time.time() - started
        ok = (out / "engine_mesh").is_dir()
        log(f"{label}: {'ok' if ok else 'FAILED'} in {elapsed:.0f}s (exit {result.returncode})")
        if not ok:
            tail = (result.stdout or "")[-600:] + (result.stderr or "")[-600:]
            (out if out.exists() else LAB).mkdir(parents=True, exist_ok=True)
            (LAB / f"{label}_failure.txt").write_text(tail, encoding="utf-8")


def mesh_for(bundle: Path) -> Path | None:
    candidate = bundle / "engine_mesh" / "0" / "mesh.obj"
    return candidate if candidate.is_file() else None


def render_bundle(bundle: Path, label: str) -> dict:
    """Import one mesh through the real importer, then render it from three angles."""
    mesh = mesh_for(bundle)
    if mesh is None:
        return {"label": label, "status": "no mesh"}

    glb = LAB / "glb" / f"{label}.glb"
    glb.parent.mkdir(parents=True, exist_ok=True)
    imported = subprocess.run(
        [
            str(BLENDER), "-b", "--factory-startup",
            "-P", "scripts/import_chronos_object.py", "--",
            "--input", str(mesh.resolve()),
            "--output", str(glb.resolve()),
        ],
        capture_output=True, text=True, timeout=900,
    )
    if not glb.is_file():
        return {
            "label": label,
            "status": "import failed",
            "detail": (imported.stdout + imported.stderr)[-500:],
        }

    renders = LAB / "renders" / label
    renders.mkdir(parents=True, exist_ok=True)
    rendered = subprocess.run(
        [
            str(BLENDER), "-b", "--factory-startup",
            "-P", "scripts/render_object_turntable.py", "--",
            "--glb", str(glb.resolve()),
            "--out-dir", str(renders.resolve()),
        ],
        capture_output=True, text=True, timeout=900,
    )

    stats = {}
    artifact = bundle / "engine_mesh" / "triposr_artifact.json"
    if artifact.is_file():
        try:
            doc = json.loads(artifact.read_text(encoding="utf-8", errors="replace"))
            stats = {
                "subject_coverage": doc.get("preparation", {}).get("subject_coverage"),
                "vertices": doc.get("mesh", {}).get("vertices"),
                "faces": doc.get("mesh", {}).get("faces"),
                "mc_resolution": doc.get("timing", {}).get("mc_resolution"),
            }
        except Exception:
            pass

    frames = sorted(renders.glob("*.png"))
    return {
        "label": label,
        "status": "ok" if frames else "render failed",
        "frames": [str(f) for f in frames],
        "glb_bytes": glb.stat().st_size,
        "render_log": (rendered.stdout + rendered.stderr)[-400:] if not frames else "",
        **stats,
    }


def contact_sheet(bundle: Path, label: str, result: dict) -> Path | None:
    from PIL import Image, ImageDraw, ImageFont

    cells = []
    for name, path in (
        ("reference", bundle / "reference_input.png"),
        ("fed to TripoSR", bundle / "engine_mesh" / "prepared_input.png"),
    ):
        if path.is_file():
            cells.append((name, Image.open(path).convert("RGB")))
    for index, frame in enumerate(result.get("frames", [])):
        cells.append((f"mesh {index + 1}", Image.open(frame).convert("RGB")))

    if not cells:
        return None

    size = 440
    pad = 26
    sheet = Image.new("RGB", (size * len(cells), size + pad), (18, 18, 18))
    draw = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype(r"C:\Windows\Fonts\consola.ttf", 16)
    except Exception:
        font = ImageFont.load_default()

    for index, (name, image) in enumerate(cells):
        sheet.paste(image.resize((size, size), Image.LANCZOS), (index * size, pad))
        draw.text((index * size + 8, 5), name, font=font, fill=(210, 210, 210))

    caption = (
        f"{label}   coverage={result.get('subject_coverage')}   "
        f"faces={result.get('faces')}   mc={result.get('mc_resolution')}"
    )
    draw.text((8, size + 4), caption, font=font, fill=(150, 150, 150))

    SHEETS.mkdir(parents=True, exist_ok=True)
    out = SHEETS / f"{label}.png"
    sheet.save(out)
    return out


def render_all() -> None:
    results = []
    for label, _prompt in TEST_PROMPTS:
        bundle = BUNDLES / label
        if not bundle.is_dir():
            continue
        log(f"{label}: importing and rendering ...")
        result = render_bundle(bundle, label)
        sheet = contact_sheet(bundle, label, result)
        if sheet:
            result["sheet"] = str(sheet)
        results.append(result)
        log(f"  {result['status']}  coverage={result.get('subject_coverage')} faces={result.get('faces')}")

    LAB.mkdir(parents=True, exist_ok=True)
    (LAB / "summary.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
    log(f"summary: {LAB / 'summary.json'}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generate", type=int, default=0, help="generate this many subjects")
    parser.add_argument("--render", action="store_true", help="import and render what exists")
    parser.add_argument("--timeout", type=int, default=1200)
    args = parser.parse_args()

    if args.generate:
        generate(args.generate, args.timeout)
    if args.render or not args.generate:
        render_all()


if __name__ == "__main__":
    main()
