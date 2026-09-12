"""Seed one real manifested object into the player's library, for capture proof.

The carry / place / duplicate mechanics need something to carry. A live manifestation takes
minutes and needs the whole Chronos2 pipeline up, which makes it a poor gate for a mechanic that
has nothing to do with generation. This stages **a real artifact from a real Chronos2 run** —
the GLB already produced and converted by `import_chronos_object.py` — into the content-addressed
library, and records one placement so the object is standing in the hall when the game boots.

It writes through the same ledger the game writes: `services/artifacts.rs` reads these files, so
what the capture proves is the real persistence path, not a fixture.

Run:  python scripts/seed_object_demo.py
      python scripts/seed_object_demo.py --clear
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import time
from pathlib import Path

ARTIFACT_ID = "seeded_probe"

# Mirrors `services::paths::app_data_root`.
def app_data_root() -> Path:
    local = os.environ.get("LOCALAPPDATA")
    if not local:
        raise SystemExit("LOCALAPPDATA is unset; cannot locate the product data root")
    # Note the trailing "data": `services::paths::app_data_root` ends there, and the config
    # tree is its sibling. Mirroring only "NeuroCognica/Archetypes" writes a ledger the game
    # never reads, which looks exactly like a placement system that does not work.
    return Path(local) / "NeuroCognica" / "Archetypes" / "data"


def library_root() -> Path:
    return app_data_root() / "artifacts"


def asset_roots() -> list[Path]:
    """Everywhere the game might read its assets from: the repo, `dist`, and the install."""
    roots = [Path("assets"), Path("dist/assets")]
    local = os.environ.get("LOCALAPPDATA")
    if local:
        roots.append(Path(local) / "Programs" / "Archetypes" / "assets")
    return [root for root in roots if root.is_dir()]


def log(*parts: object) -> None:
    print("[seed-object]", *parts, flush=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--clear", action="store_true", help="remove the seeded rows and files")
    parser.add_argument("--id", default=ARTIFACT_ID)
    args = parser.parse_args()

    root = library_root()
    library = root / "library.jsonl"
    placements = root / "placements.jsonl"

    if args.clear:
        for path in (library, placements):
            if path.is_file():
                path.unlink()
                log(f"removed {path}")
        for assets in asset_roots():
            candidate = assets / "manifested" / f"{args.id}.glb"
            if candidate.is_file():
                candidate.unlink()
                log(f"removed {candidate}")
        return

    source = Path("assets/scenes/manifested_artifact.glb")
    if not source.is_file():
        raise SystemExit(
            f"{source} is missing. It is written by a real manifestation, or by running "
            "import_chronos_object.py over a Chronos2 bundle's engine_mesh."
        )

    staged = 0
    for assets in asset_roots():
        target = assets / "manifested" / f"{args.id}.glb"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        staged += 1
        log(f"staged {target}")
    if staged == 0:
        raise SystemExit("found no assets directory to stage into")

    root.mkdir(parents=True, exist_ok=True)
    stamp = str(int(time.time()))

    with library.open("a", encoding="utf-8") as handle:
        handle.write(
            json.dumps(
                {
                    "id": args.id,
                    "asset": f"manifested/{args.id}.glb",
                    "prompt": "a stone lantern",
                    "created": stamp,
                }
            )
            + "\n"
        )

    # One object standing on the Council floor, a few metres in from where the walk starts.
    with placements.open("a", encoding="utf-8") as handle:
        handle.write(
            json.dumps(
                {
                    "kind": "Placed",
                    "placement": f"seed_{stamp}",
                    "artifact": args.id,
                    "asset": f"manifested/{args.id}.glb",
                    "position": [0.0, 0.4, 6.5],
                    "yaw": 0.0,
                    "scale": 1.0,
                    "at": stamp,
                }
            )
            + "\n"
        )

    log(f"library: {library}")
    log(f"placements: {placements}")
    log("one object standing at (0.0, 0.4, 6.5) on the Council floor")


if __name__ == "__main__":
    main()
