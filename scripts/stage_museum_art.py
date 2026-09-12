"""Stage Chronos2 works into the repo as museum exhibits.

**The game must never read `C:\\chronos2` at runtime.** The launcher already treats Chronos as
optional, and a buyer's machine has no such directory — a museum that reads a sibling product's
working folder would be twelve empty rooms on every machine but this one. So this runs at
author time, copies what it selects into `assets/museum/`, and writes a manifest the engine
reads instead.

Two rules carried across from Chronos2's own museum concept
(`C:\\chronos2\\crates\\chronos_cli\\src\\gallery_exhibit.rs`) rather than rediscovered:

1. **`title_from_prompt`** — the placard title comes from the run's own `human_prompt.txt`,
   title-cased with display noise dropped.
2. **Runs whose own vista was `museum_wall` are excluded.** A photograph of a gallery wall must
   never be hung on a gallery wall.

And one rule from `docs/ledger/2026/09/plan_2026-09-11_2130_museum_arches.md`:

3. **A placard states what the bundle records, and nothing else.** Title from the prompt, date
   from the run directory name, provenance hash from the bundle's own manifest. If a bundle
   lacks a field the placard omits that line rather than filling it in. No invented artist, no
   invented year, no generated wall text.

Run:  python scripts/stage_museum_art.py
      python scripts/stage_museum_art.py --limit 60 --source C:\\chronos2\\out\\first_light
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

SOURCE = Path(r"C:\chronos2\out\first_light")
OUT_DIR = Path("assets/museum")

# Twelve chambers, five hangings each. `castle.rs` is the authority; this constant is checked
# against it by an engine test so the two cannot drift.
DEFAULT_LIMIT = 60

# A hung work is about 1.6m wide and the player can stand a metre from it, so more than this is
# texture memory spent on detail nobody can resolve. Sixty of them at 768px is the budget.
WORK_MAX_PIXELS = 768

# Every work is mounted onto the same 4:3 plate, the way a framer mats a print. The library's
# works are a mix of 16:9 and 4:3, and a single frame mesh showing both would either stretch
# some of them or need a frame per aspect ratio. Matting is what a gallery actually does, and
# it costs one fill instead of a second asset.
MOUNT_ASPECT = (4, 3)
MOUNT_COLOUR = (24, 23, 21)

PLACARD_SIZE = (512, 168)
PLACARD_PLATE = (28, 26, 24)
PLACARD_TITLE = (238, 230, 214)
PLACARD_BODY = (172, 164, 150)

# Windows ships these; the game's own UI is monospaced, so the placards match it.
FONT_CANDIDATES = [
    r"C:\Windows\Fonts\consolab.ttf",
    r"C:\Windows\Fonts\consola.ttf",
    r"C:\Windows\Fonts\segoeui.ttf",
]


def log(*parts: object) -> None:
    print("[museum-art]", *parts, flush=True)


@dataclass
class Exhibit:
    run: str
    title: str
    created: str
    image: str
    placard: str
    prompt: str
    integrity_hash: str | None


def title_from_prompt(prompt: str) -> str:
    """Ported from `gallery_exhibit.rs::title_from_prompt`, including its drop list.

    Kept identical on purpose: two products showing different titles for the same work would be
    two different claims about what the work is.
    """
    drop = {
        "a", "an", "the", "of", "museum", "gallery", "exhibit", "vista", "showroom", "painting",
        "canvas", "art", "image", "with", "at", "and", "over", "on", "in", "under", "above",
        "into", "by", "to", "from", "as", "before", "beside", "near", "through", "across",
    }
    words: list[str] = []
    current = ""
    for char in prompt:
        if char.isascii() and char.isalnum():
            current += char
        else:
            if current:
                words.append(current)
                current = ""
    if current:
        words.append(current)

    kept = [w for w in words if w.lower() not in drop][:4]
    if not kept:
        return "Untitled"
    return " ".join(w[0].upper() + w[1:].lower() for w in kept)


def is_run_dir(name: str) -> bool:
    """A real first-light run directory is `YYYYMMDD_HHMMSS`.

    Anything else under `out/first_light` is scratch — `test_pedestal`, `verify_blend` and so
    on — and must never be hung on a wall as if it were the operator's work.
    """
    return (
        len(name) == 15
        and name[8] == "_"
        and name[:8].isdigit()
        and name[9:].isdigit()
    )


def read_json(path: Path) -> dict:
    if not path.is_file():
        return {}
    try:
        loaded = json.loads(path.read_text(encoding="utf-8", errors="replace"))
    except Exception:
        return {}
    return loaded if isinstance(loaded, dict) else {}


def run_vista(bundle: Path) -> str | None:
    """Which vista a run produced.

    Chronos2's own museum code reads `scene.inspection.json`. In this library that file carries
    no `vista` key and the value actually lives in `manifest.json` under `render_context`, so
    both are consulted — reading only the documented location would silently exclude nothing.
    """
    manifest = read_json(bundle / "manifest.json")
    context = manifest.get("render_context")
    if isinstance(context, dict) and context.get("vista"):
        return str(context["vista"])
    inspection = read_json(bundle / "scene.inspection.json")
    if inspection.get("vista"):
        return str(inspection["vista"])
    return None


def is_museum_view(bundle: Path) -> bool:
    manifest = read_json(bundle / "manifest.json")
    context = manifest.get("render_context")
    return bool(isinstance(context, dict) and context.get("museum_view"))


def integrity_hash(bundle: Path) -> str | None:
    manifest = read_json(bundle / "manifest.json")
    value = manifest.get("final_png_hash")
    return str(value) if value else None


def created_date(run: str) -> str:
    return f"{run[0:4]}-{run[4:6]}-{run[6:8]}"


def source_image(bundle: Path) -> Path | None:
    """Chronos2's preference order: the generated artwork source first, the showroom hero last."""
    for candidate in (
        bundle / "canvas_source.png",
        bundle / "renders" / "reference_input.png",
        bundle / "reference_input.png",
        bundle / "renders" / "hero.png",
    ):
        if candidate.is_file():
            return candidate
    return None


def load_font(size: int) -> ImageFont.FreeTypeFont:
    for path in FONT_CANDIDATES:
        if Path(path).is_file():
            try:
                return ImageFont.truetype(path, size)
            except Exception:
                continue
    return ImageFont.load_default()


def render_placard(exhibit_title: str, created: str, hash_value: str | None, out: Path) -> None:
    """Draws the placard as an image at author time.

    Bevy cannot put text on a wall in 3D, and the alternative — a text mesh per placard — is a
    lot of geometry for something the player reads from a metre away. Rendering it here also
    means the typography is decided once rather than by whatever font the runtime happens to
    have.
    """
    image = Image.new("RGB", PLACARD_SIZE, PLACARD_PLATE)
    draw = ImageDraw.Draw(image)

    # A hairline border, so the placard reads as a plate rather than as a hole in the wall.
    draw.rectangle(
        [(4, 4), (PLACARD_SIZE[0] - 5, PLACARD_SIZE[1] - 5)],
        outline=(70, 66, 60),
        width=2,
    )

    title_font = load_font(34)
    body_font = load_font(21)

    draw.text((26, 26), exhibit_title, font=title_font, fill=PLACARD_TITLE)

    # Only lines the bundle actually supports. A missing field omits its line.
    y = 78
    if created:
        draw.text((26, y), created, font=body_font, fill=PLACARD_BODY)
        y += 30
    if hash_value:
        draw.text(
            (26, y),
            f"Chronos2 provenance {hash_value[:12]}",
            font=body_font,
            fill=PLACARD_BODY,
        )

    out.parent.mkdir(parents=True, exist_ok=True)
    image.save(out)


def trim_to_content(image: Image.Image) -> Image.Image:
    """Crop away the render's empty backdrop.

    The Chronos2 showroom hero puts a small subject in the middle of a large flat backdrop.
    Mounted as-is the work arrives on a gallery wall as a postage stamp in a sea of grey, and
    the mat this script adds makes it worse. The backdrop is a near-uniform colour, so the
    subject is whatever differs from the corner pixel.

    Falls back to the whole image whenever the detection is not confident — a bad crop is an
    edit to someone's work, and doing nothing is the safer failure.
    """
    grey = image.convert("L")
    corner = grey.getpixel((1, 1))
    mask = grey.point(lambda v: 255 if abs(v - corner) > 12 else 0)
    box = mask.getbbox()
    if box is None:
        return image

    left, upper, right, lower = box
    width, height = right - left, lower - upper
    if width < image.width * 0.05 or height < image.height * 0.05:
        return image
    if width > image.width * 0.98 and height > image.height * 0.98:
        return image

    # Breathing room around the subject, in the spirit of a mount rather than a tight crop.
    margin = int(max(width, height) * 0.10)
    return image.crop(
        (
            max(0, left - margin),
            max(0, upper - margin),
            min(image.width, right + margin),
            min(image.height, lower + margin),
        )
    )


# Below this much variation across the trimmed image there is nothing in the picture. Some
# first-light runs produced an all-black frame or a flat backdrop with no subject; hanging one
# in a gallery is hanging an empty rectangle and calling it a work.
MIN_CONTENT_STDDEV = 9.0


def has_visible_content(image: Image.Image) -> bool:
    """Whether a trimmed render contains anything at all.

    Deliberately mechanical: a measure of variation, not a judgement about subject matter. It
    rejects a black frame and a blank backdrop; it says nothing about whether a picture is any
    good, which is not this script's business.
    """
    grey = image.convert("L")
    histogram = grey.histogram()
    total = sum(histogram)
    if total == 0:
        return False
    mean = sum(value * count for value, count in enumerate(histogram)) / total
    variance = sum(count * (value - mean) ** 2 for value, count in enumerate(histogram)) / total
    return variance ** 0.5 >= MIN_CONTENT_STDDEV


def stage_work(source: Path, out: Path) -> tuple[int, int]:
    """Mount one work onto the common 4:3 plate and write it at the wall-panel size."""
    with Image.open(source) as opened:
        image = trim_to_content(opened.convert("RGB"))

    width = WORK_MAX_PIXELS
    height = width * MOUNT_ASPECT[1] // MOUNT_ASPECT[0]

    # Contain, never cover: cropping a work to fit a frame is an editorial act on someone
    # else's picture. The unused plate reads as a mat.
    fitted = image.copy()
    fitted.thumbnail((width - 36, height - 36), Image.LANCZOS)

    plate = Image.new("RGB", (width, height), MOUNT_COLOUR)
    plate.paste(fitted, ((width - fitted.width) // 2, (height - fitted.height) // 2))

    out.parent.mkdir(parents=True, exist_ok=True)
    plate.save(out)
    return plate.size


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", default=str(SOURCE))
    parser.add_argument("--out", default=str(OUT_DIR))
    parser.add_argument("--limit", type=int, default=DEFAULT_LIMIT)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source_root = Path(args.source)
    out_dir = Path(args.out)

    if not source_root.is_dir():
        raise SystemExit(
            f"Chronos2 library not found at {source_root}. Staging is an author-time step; "
            "the game never reads this path."
        )

    candidates = sorted(
        (d for d in source_root.iterdir() if d.is_dir() and is_run_dir(d.name)),
        key=lambda d: d.name,
    )
    log(f"run directories: {len(candidates)}")

    eligible: list[Path] = []
    excluded_museum = 0
    excluded_no_image = 0
    excluded_blank = 0
    for bundle in candidates:
        vista = run_vista(bundle)
        if vista == "museum_wall" or is_museum_view(bundle):
            excluded_museum += 1
            continue
        picture = source_image(bundle)
        if picture is None:
            excluded_no_image += 1
            continue
        try:
            with Image.open(picture) as opened:
                if not has_visible_content(trim_to_content(opened.convert("RGB"))):
                    excluded_blank += 1
                    continue
        except Exception:
            excluded_no_image += 1
            continue
        eligible.append(bundle)

    log(
        f"eligible: {len(eligible)}",
        f"excluded_museum_wall: {excluded_museum}",
        f"excluded_no_image: {excluded_no_image}",
        f"excluded_blank: {excluded_blank}",
    )

    selected = eligible[: args.limit]
    exhibits: list[Exhibit] = []

    for bundle in selected:
        prompt_file = bundle / "human_prompt.txt"
        prompt = (
            prompt_file.read_text(encoding="utf-8", errors="replace").strip()
            if prompt_file.is_file()
            else ""
        )
        title = title_from_prompt(prompt) if prompt else "Untitled"
        image_source = source_image(bundle)
        assert image_source is not None

        work_path = out_dir / "works" / f"{bundle.name}.png"
        placard_path = out_dir / "placards" / f"{bundle.name}.png"
        size = stage_work(image_source, work_path)
        hash_value = integrity_hash(bundle)
        render_placard(title, created_date(bundle.name), hash_value, placard_path)

        exhibits.append(
            Exhibit(
                run=bundle.name,
                title=title,
                created=created_date(bundle.name),
                image=f"works/{bundle.name}.png",
                placard=f"placards/{bundle.name}.png",
                prompt=prompt,
                integrity_hash=hash_value,
            )
        )
        log(f"  staged {bundle.name} '{title}' {size[0]}x{size[1]}")

    manifest = {
        "schema": "archetypes.museum.v1",
        "mount_aspect": list(MOUNT_ASPECT),
        "source": str(source_root),
        "staged_at_author_time": True,
        "note": (
            "Copied into the repo at author time. The game must never read the Chronos2 "
            "working directory at runtime; a buyer's machine does not have one."
        ),
        "eligible_runs": len(eligible),
        "excluded_museum_wall": excluded_museum,
        "excluded_blank_render": excluded_blank,
        "exhibits": [
            {
                "run": e.run,
                "title": e.title,
                "created": e.created,
                "image": e.image,
                "placard": e.placard,
                "prompt": e.prompt,
                **({"integrity_hash": e.integrity_hash} if e.integrity_hash else {}),
            }
            for e in exhibits
        ],
    }

    out_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = out_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    log(f"wrote {manifest_path} with {len(exhibits)} exhibits")

    placarded_with_hash = sum(1 for e in exhibits if e.integrity_hash)
    log(
        f"placards carrying a provenance hash: {placarded_with_hash} of {len(exhibits)} "
        "(the rest omit that line rather than inventing one)"
    )


if __name__ == "__main__":
    main()
