"""Author original seamless stone tile sets for the Inner Castle kit.

The castle's Blender modules are all UV-unwrapped and all flat colour. That is the whole of
the "it looks fake" problem: an untextured pier is a shape, not masonry, and albedo alone will
not fix it either — a flat photograph of stone on a flat pier still reads flat, because nothing
on the surface responds to the light. Every tile written here therefore ships **albedo, normal
and roughness** together.

Why authored and not sourced
----------------------------
The operator's reference sheet is one 1254x1254 image holding all nine stone types, so each
tile is roughly 378px. On a 9.9m arcade bay that is about 38 px/m, which reads as mush. It is
an excellent *look* reference and an unusable *asset*. Per `memory: build-it-dont-wait-for-a-
clean-license`, when the available material is unusable that closes "adopt", not the
capability — so these are authored from scratch, seamless by construction, and owned outright.

Seamless by construction, not by healing
----------------------------------------
Every generator here is periodic on purpose rather than blurred at the edges afterwards:

- Value noise interpolates a random lattice with wrapped indices, so octave N tiles exactly.
- Worley cells measure distance with wrapped deltas (`d = min(|d|, 1 - |d|)`), so a cell that
  leaves the right edge re-enters on the left.
- Block courses divide the tile into a whole number of blocks, so no block is cut in half.

This matters at this scale: the wall is 716m around, so a tile that repeats every 4m is drawn
179 times in a single ring. A seam would read as a stripe down the whole building.

Physical scale
--------------
Each tile covers `TILE_METRES` square of real wall. The kit modules must use world-scaled UVs
(a cube projection sized in metres) for that to hold — `smart_project` packs islands into unit
space per object and has no physical scale, so the same stone would appear at one size on a
pier and another on a stair tread.

Run:  python scripts/author_stone_tiles.py
      python scripts/author_stone_tiles.py --resolution 2048 --only weathered_limestone_ashlar
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np
from PIL import Image

OUT_DIR = Path("assets/textures/stone")

# One tile covers this much real wall. 4m against a 0.9m block gives roughly four courses of
# masonry per repeat, which is enough variation that the eye does not lock onto the period.
TILE_METRES = 4.0
DEFAULT_RESOLUTION = 1024

# Matches the normal-map convention already shipped by `world.rs::chamber_floor_textures()`:
# nx from the left/right height difference, ny from the up/down difference, nz = 1, then
# packed to 0..1. Keeping one convention across the codebase means a light that reads correctly
# on the chamber floor reads correctly on the walls.
NORMAL_STRENGTH_DEFAULT = 1.65


def log(*parts: object) -> None:
    print("[stone-tiles]", *parts, flush=True)


# ---------------------------------------------------------------------------------------
# Periodic noise primitives
# ---------------------------------------------------------------------------------------

def _smoothstep(t: np.ndarray) -> np.ndarray:
    return t * t * (3.0 - 2.0 * t)


def periodic_value_noise(resolution: int, cells: int, rng: np.random.Generator) -> np.ndarray:
    """Value noise on a `cells` x `cells` lattice, wrapped, sampled at `resolution`.

    Wrapping is what makes it tile: the lattice index is taken modulo `cells`, so the sample
    at u=1 reads the same lattice corner as the sample at u=0.
    """
    lattice = rng.random((cells, cells), dtype=np.float64)

    coords = np.arange(resolution, dtype=np.float64) / resolution * cells
    cell_index = np.floor(coords).astype(np.int64)
    frac = _smoothstep(coords - cell_index)

    i0 = cell_index % cells
    i1 = (cell_index + 1) % cells

    # Bilinear over the wrapped lattice, rows then columns.
    top = lattice[np.ix_(i0, i0)] * (1 - frac)[None, :] + lattice[np.ix_(i0, i1)] * frac[None, :]
    bottom = lattice[np.ix_(i1, i0)] * (1 - frac)[None, :] + lattice[np.ix_(i1, i1)] * frac[None, :]
    return top * (1 - frac)[:, None] + bottom * frac[:, None]


def periodic_fbm(
    resolution: int,
    base_cells: int,
    octaves: int,
    rng: np.random.Generator,
    gain: float = 0.5,
) -> np.ndarray:
    """Fractal sum of periodic value noise. Each octave doubles the lattice, so each octave
    tiles on its own and therefore so does the sum."""
    total = np.zeros((resolution, resolution), dtype=np.float64)
    amplitude = 1.0
    normalisation = 0.0
    cells = base_cells
    for _ in range(octaves):
        # A lattice finer than the image cannot be resolved, so stop rather than alias.
        if cells > resolution:
            break
        total += periodic_value_noise(resolution, cells, rng) * amplitude
        normalisation += amplitude
        amplitude *= gain
        cells *= 2
    return total / max(normalisation, 1e-9)


def periodic_worley(
    resolution: int, seed_count: int, rng: np.random.Generator
) -> tuple[np.ndarray, np.ndarray]:
    """Worley/Voronoi cells that wrap.

    Returns `(cell_id, edge_distance)` where `edge_distance` is the gap between the nearest and
    second-nearest seed — small near a cell boundary, which is exactly where mortar goes.

    Distances use wrapped deltas, so a cell whose seed sits near the right edge owns pixels on
    the left edge too. That is what makes the rubble tile without a visible join.
    """
    seeds = rng.random((seed_count, 2), dtype=np.float64)

    axis = (np.arange(resolution, dtype=np.float64) + 0.5) / resolution
    grid_y, grid_x = np.meshgrid(axis, axis, indexing="ij")

    nearest = np.full((resolution, resolution), np.inf)
    second = np.full((resolution, resolution), np.inf)
    cell_id = np.zeros((resolution, resolution), dtype=np.int64)

    for index, (seed_y, seed_x) in enumerate(seeds):
        dy = np.abs(grid_y - seed_y)
        dx = np.abs(grid_x - seed_x)
        dy = np.minimum(dy, 1.0 - dy)
        dx = np.minimum(dx, 1.0 - dx)
        distance = np.sqrt(dy * dy + dx * dx)

        closer = distance < nearest
        second = np.where(closer, nearest, np.minimum(second, distance))
        cell_id = np.where(closer, index, cell_id)
        nearest = np.where(closer, distance, nearest)

    return cell_id, second - nearest


def periodic_courses(
    resolution: int, columns: int, rows: int, stagger: float, rng: np.random.Generator
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Staggered ashlar courses that tile.

    `columns` and `rows` are whole numbers of blocks across the tile, so no block is cut by the
    tile edge. Alternate courses shift by `stagger` of a block, which is how masonry is
    actually laid and what stops the joints lining up into a visible grid.

    Returns `(block_id, distance_to_joint, per_pixel_block_random)`.
    """
    axis = (np.arange(resolution, dtype=np.float64) + 0.5) / resolution
    grid_y, grid_x = np.meshgrid(axis, axis, indexing="ij")

    course = np.floor(grid_y * rows).astype(np.int64)
    shifted_x = (grid_x + np.where(course % 2 == 1, stagger, 0.0)) % 1.0

    column = np.floor(shifted_x * columns).astype(np.int64)
    block_id = course * columns + column

    # Distance to the nearest joint, in block-local units, so a wide block and a tall block
    # both get a mortar line of the same physical width.
    within_x = shifted_x * columns - column
    within_y = grid_y * rows - course
    edge_x = np.minimum(within_x, 1.0 - within_x) / columns
    edge_y = np.minimum(within_y, 1.0 - within_y) / rows
    joint_distance = np.minimum(edge_x, edge_y)

    block_random = rng.random(rows * columns)
    return block_id, joint_distance, block_random[block_id % block_random.size]


# ---------------------------------------------------------------------------------------
# Maps
# ---------------------------------------------------------------------------------------

def height_to_normal(height: np.ndarray, strength: float) -> np.ndarray:
    """Tangent-space normal from a height field, wrapped so the normal map tiles too.

    `np.roll` is what keeps the seam closed: the gradient at column 0 is taken against the last
    column, not against a clamped edge, so the lighting matches across the join.
    """
    left = np.roll(height, 1, axis=1)
    right = np.roll(height, -1, axis=1)
    up = np.roll(height, 1, axis=0)
    down = np.roll(height, -1, axis=0)

    nx = (left - right) * strength
    ny = (up - down) * strength
    nz = np.ones_like(height)

    length = np.sqrt(nx * nx + ny * ny + nz * nz)
    return np.dstack((nx / length, ny / length, nz / length))


def to_srgb_bytes(channel: np.ndarray) -> np.ndarray:
    return np.clip(channel * 255.0, 0, 255).astype(np.uint8)


# ---------------------------------------------------------------------------------------
# Stone definitions
# ---------------------------------------------------------------------------------------

@dataclass
class Stone:
    name: str
    description: str
    base_colour: tuple[float, float, float]
    # Block layout across one tile; `None` means rubble rather than coursed masonry.
    courses: tuple[int, int] | None
    seed: int
    mortar_metres: float = 0.035
    mortar_depth: float = 0.55
    mortar_colour: tuple[float, float, float] = (0.44, 0.42, 0.38)
    block_variation: float = 0.10
    grain_cells: int = 24
    grain_octaves: int = 5
    grain_amount: float = 0.16
    relief: float = 1.0
    roughness_range: tuple[float, float] = (0.72, 0.94)
    rubble_seeds: int = 0
    chip_amount: float = 0.0
    extra: dict = field(default_factory=dict)


STONES: list[Stone] = [
    Stone(
        name="dark_basalt_fortress",
        description="Dense dark basalt, tight joints, heavy blocks for the base of the wall.",
        base_colour=(0.152, 0.155, 0.168),
        courses=(4, 6),
        seed=101,
        mortar_metres=0.028,
        mortar_depth=0.70,
        mortar_colour=(0.115, 0.118, 0.128),
        block_variation=0.07,
        grain_cells=32,
        grain_amount=0.13,
        relief=1.15,
        roughness_range=(0.62, 0.84),
    ),
    Stone(
        name="rough_granite_block",
        description="Coarse speckled granite, deep pitting, wide blocks.",
        base_colour=(0.372, 0.356, 0.344),
        courses=(4, 5),
        seed=202,
        mortar_metres=0.040,
        mortar_depth=0.62,
        mortar_colour=(0.300, 0.292, 0.278),
        block_variation=0.11,
        grain_cells=48,
        grain_octaves=6,
        grain_amount=0.26,
        relief=1.30,
        roughness_range=(0.74, 0.96),
    ),
    Stone(
        name="old_mortared_fieldstone",
        description="Irregular field stones swimming in broad mortar beds.",
        base_colour=(0.398, 0.378, 0.344),
        courses=None,
        seed=303,
        rubble_seeds=52,
        mortar_metres=0.085,
        mortar_depth=0.78,
        mortar_colour=(0.520, 0.500, 0.462),
        block_variation=0.15,
        grain_cells=36,
        grain_amount=0.20,
        relief=1.25,
        roughness_range=(0.78, 0.97),
    ),
    Stone(
        name="chipped_rubble_stone",
        description="Broken rubble coursing, edges knocked off, shallow bedding.",
        base_colour=(0.430, 0.412, 0.386),
        courses=None,
        seed=404,
        rubble_seeds=86,
        mortar_metres=0.055,
        mortar_depth=0.66,
        mortar_colour=(0.468, 0.452, 0.424),
        block_variation=0.17,
        grain_cells=40,
        grain_amount=0.24,
        relief=1.20,
        chip_amount=0.35,
        roughness_range=(0.80, 0.98),
    ),
    Stone(
        name="weathered_limestone_ashlar",
        description="Fine pale ashlar, close joints — the storey the arcade was designed in.",
        base_colour=(0.628, 0.596, 0.534),
        courses=(4, 7),
        seed=505,
        mortar_metres=0.026,
        mortar_depth=0.50,
        mortar_colour=(0.556, 0.530, 0.480),
        block_variation=0.075,
        grain_cells=28,
        grain_amount=0.13,
        relief=0.90,
        roughness_range=(0.70, 0.88),
    ),
    Stone(
        name="sandstone_block",
        description="Warm sandstone with visible bedding lines running with the course.",
        base_colour=(0.652, 0.554, 0.422),
        courses=(3, 6),
        seed=606,
        mortar_metres=0.032,
        mortar_depth=0.52,
        mortar_colour=(0.590, 0.512, 0.404),
        block_variation=0.095,
        grain_cells=20,
        grain_amount=0.17,
        relief=1.00,
        roughness_range=(0.76, 0.93),
        extra={"bedding": 0.30},
    ),
    Stone(
        name="pale_limestone_light",
        description="The lightest ashlar, for the top storey where the wall meets the vault.",
        base_colour=(0.716, 0.688, 0.628),
        courses=(4, 7),
        seed=707,
        mortar_metres=0.024,
        mortar_depth=0.44,
        mortar_colour=(0.650, 0.624, 0.572),
        block_variation=0.065,
        grain_cells=26,
        grain_amount=0.11,
        relief=0.78,
        roughness_range=(0.66, 0.86),
    ),
    Stone(
        name="worn_courtyard_flagstone",
        description="Broad worn paving, dished centres, for floors rather than walls.",
        base_colour=(0.470, 0.452, 0.424),
        courses=(3, 3),
        seed=808,
        mortar_metres=0.045,
        mortar_depth=0.58,
        mortar_colour=(0.386, 0.372, 0.350),
        block_variation=0.12,
        grain_cells=22,
        grain_amount=0.15,
        relief=0.95,
        roughness_range=(0.72, 0.92),
        extra={"dish": 0.40},
    ),
]


def build_stone(stone: Stone, resolution: int) -> dict[str, np.ndarray]:
    rng = np.random.default_rng(stone.seed)

    # Mortar width is specified in metres, so a tile covering more wall gets a proportionally
    # narrower joint in UV space. Stating it in metres is what keeps a joint a joint at every
    # scale — the same mistake the castle floor made when it stated joints as angles.
    mortar_uv = stone.mortar_metres / TILE_METRES

    if stone.courses is not None:
        columns, rows = stone.courses
        block_id, joint_distance, block_random = periodic_courses(
            resolution, columns, rows, stagger=0.5, rng=rng
        )
    else:
        block_id, edge_distance = periodic_worley(resolution, stone.rubble_seeds, rng)
        joint_distance = edge_distance * 0.5
        block_random = rng.random(stone.rubble_seeds)[block_id % stone.rubble_seeds]

    # Mortar is a soft band around every joint rather than a hard line: real mortar has a
    # meniscus, and a hard step reads as a decal.
    mortar = 1.0 - np.clip(joint_distance / max(mortar_uv, 1e-6), 0.0, 1.0)
    mortar = _smoothstep(mortar)

    grain = periodic_fbm(resolution, stone.grain_cells, stone.grain_octaves, rng)
    fine = periodic_fbm(resolution, min(stone.grain_cells * 3, resolution // 2), 3, rng)

    # Weathering zones: a very low-frequency field that darkens or lightens whole patches of
    # wall at once. Without it every block carries an independent random tone and the result
    # reads as a checkerboard of tiles rather than as masonry — real walls stain in areas,
    # because water runs down them in areas.
    zones = periodic_fbm(resolution, 3, 3, rng)

    height = np.zeros((resolution, resolution), dtype=np.float64)
    height += (block_random - 0.5) * stone.block_variation          # block-to-block sit
    height += (grain - 0.5) * stone.grain_amount                    # face weathering
    height += (fine - 0.5) * stone.grain_amount * 0.35              # tooling marks

    if stone.chip_amount > 0.0:
        # Knock the arrises off: bias the height down near joints, more on some blocks than
        # others, so edges read as broken rather than bevelled uniformly.
        chip = _smoothstep(np.clip(joint_distance / (mortar_uv * 3.0), 0.0, 1.0))
        height -= (1.0 - chip) * stone.chip_amount * (0.4 + 0.6 * block_random)

    if "bedding" in stone.extra:
        # Sedimentary bedding lines run horizontally through the block, not across it.
        lines = np.sin(
            (np.arange(resolution)[:, None] / resolution) * np.pi * 2.0 * 18.0
            + grain * 6.0
        )
        height += lines * stone.extra["bedding"] * 0.05

    if "dish" in stone.extra:
        # Paving wears hollow in the middle where feet fall, not at the edges.
        dish = _smoothstep(np.clip(joint_distance / (mortar_uv * 8.0), 0.0, 1.0))
        height -= dish * stone.extra["dish"] * 0.10

    height -= mortar * stone.mortar_depth

    # Normalise so `relief` means the same thing for every stone regardless of how its layers
    # happened to sum.
    span = height.max() - height.min()
    if span > 1e-9:
        height = (height - height.min()) / span
    height *= stone.relief

    base = np.array(stone.base_colour, dtype=np.float64)
    mortar_rgb = np.array(stone.mortar_colour, dtype=np.float64)

    # Block-to-block tone, but pulled toward its neighbourhood by the weathering zone rather
    # than being independent per block.
    value = 1.0 + (block_random - 0.5) * stone.block_variation * 1.7
    value += (zones - 0.5) * stone.block_variation * 2.6
    value += (grain - 0.5) * stone.grain_amount * 1.6
    value += (fine - 0.5) * stone.grain_amount * 0.6

    albedo = base[None, None, :] * value[:, :, None]
    albedo = albedo * (1.0 - mortar[:, :, None]) + mortar_rgb[None, None, :] * mortar[:, :, None]
    albedo = np.clip(albedo, 0.0, 1.0)

    low, high = stone.roughness_range
    roughness = low + (high - low) * np.clip(grain * 0.6 + fine * 0.4, 0.0, 1.0)
    # Mortar is always rougher than the stone it beds.
    roughness = roughness * (1.0 - mortar) + np.minimum(high + 0.04, 1.0) * mortar

    normal = height_to_normal(height, NORMAL_STRENGTH_DEFAULT)

    return {"albedo": albedo, "normal": normal, "roughness": roughness, "height": height}


def write_stone(stone: Stone, maps: dict[str, np.ndarray], out_dir: Path) -> dict[str, str]:
    out_dir.mkdir(parents=True, exist_ok=True)
    written: dict[str, str] = {}

    albedo_path = out_dir / f"{stone.name}_albedo.png"
    Image.fromarray(to_srgb_bytes(maps["albedo"])).save(albedo_path)
    written["albedo"] = albedo_path.name

    normal_path = out_dir / f"{stone.name}_normal.png"
    packed = to_srgb_bytes(maps["normal"] * 0.5 + 0.5)
    Image.fromarray(packed).save(normal_path)
    written["normal"] = normal_path.name

    roughness_path = out_dir / f"{stone.name}_roughness.png"
    Image.fromarray(to_srgb_bytes(maps["roughness"])).save(roughness_path)
    written["roughness"] = roughness_path.name

    return written


def verify_seamless(stone: Stone, maps: dict[str, np.ndarray]) -> None:
    """Check the tile really wraps, rather than trusting that it was built to.

    Compares the first and last row/column against their wrapped neighbours. On a seamless
    tile the join is no more of a discontinuity than any other adjacent pair of rows, so the
    test is relative to the texture's own internal variation, not an absolute threshold.
    """
    albedo = maps["albedo"]

    def discontinuity(a: np.ndarray, b: np.ndarray) -> float:
        return float(np.abs(a - b).mean())

    seam_x = discontinuity(albedo[:, 0, :], albedo[:, -1, :])
    seam_y = discontinuity(albedo[0, :, :], albedo[-1, :, :])
    interior_x = discontinuity(albedo[:, 1:, :], albedo[:, :-1, :])
    interior_y = discontinuity(albedo[1:, :, :], albedo[:-1, :, :])

    # Allow the seam to be up to 3x a typical neighbouring-pixel step. Anything beyond that is
    # a visible stripe repeated 179 times around a 716m wall.
    for axis, seam, interior in (("x", seam_x, interior_x), ("y", seam_y, interior_y)):
        if seam > interior * 3.0 + 1e-6:
            raise RuntimeError(
                f"{stone.name}: {axis} seam step {seam:.5f} against an interior step of "
                f"{interior:.5f} — the tile does not wrap"
            )

    log(
        f"  seam_x={seam_x:.5f} (interior {interior_x:.5f})",
        f"seam_y={seam_y:.5f} (interior {interior_y:.5f})",
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default=str(OUT_DIR))
    parser.add_argument("--resolution", type=int, default=DEFAULT_RESOLUTION)
    parser.add_argument("--only", default=None, help="build a single stone by name")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    out_dir = Path(args.out)
    resolution = args.resolution

    selected = [s for s in STONES if args.only is None or s.name == args.only]
    if not selected:
        raise SystemExit(f"no stone named {args.only!r}; have {[s.name for s in STONES]}")

    manifest = {
        "tile_metres": TILE_METRES,
        "resolution": resolution,
        "normal_strength": NORMAL_STRENGTH_DEFAULT,
        "note": (
            "Authored originals, seamless by construction. UVs must be world-scaled: "
            "u = world_metres / tile_metres."
        ),
        "stones": {},
    }

    for stone in selected:
        log(f"building {stone.name} at {resolution}px covering {TILE_METRES}m")
        maps = build_stone(stone, resolution)
        verify_seamless(stone, maps)
        written = write_stone(stone, maps, out_dir)
        manifest["stones"][stone.name] = {
            "description": stone.description,
            "mortar_metres": stone.mortar_metres,
            **written,
        }
        log(f"  wrote {', '.join(written.values())}")

    manifest_path = out_dir / "stone_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    log(f"manifest={manifest_path}")


if __name__ == "__main__":
    main()
