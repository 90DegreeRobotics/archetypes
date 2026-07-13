"""Prepare the operator-authored blackflame loading veil for Bevy.

Bevy 0.18 has no native MP4 player. This build-time conversion preserves the supplied
motion as a compact JPEG frame sequence that the engine loops while the chamber loads.
Requires `imageio-ffmpeg` only when regenerating; no video sidecar is required at runtime.
"""

from pathlib import Path
import subprocess

import imageio_ffmpeg

SOURCE = Path(r"C:\Users\m\Videos\blackflame.mp4")
OUTPUT = Path(r"C:\archetypes\assets\loading\blackflame")


def main():
    if not SOURCE.is_file():
        raise SystemExit(f"missing authored loading video: {SOURCE}")
    OUTPUT.mkdir(parents=True, exist_ok=True)
    command = [
        imageio_ffmpeg.get_ffmpeg_exe(), "-y", "-i", str(SOURCE),
        "-vf", "fps=8,scale=960:-2", "-q:v", "4",
        str(OUTPUT / "frame_%03d.jpg"),
    ]
    subprocess.run(command, check=True)
    subprocess.run([
        imageio_ffmpeg.get_ffmpeg_exe(), "-y", "-i", str(SOURCE),
        "-vn", "-acodec", "pcm_s16le", "-ar", "48000", "-ac", "2",
        str(OUTPUT.parent / "blackflame.wav"),
    ], check=True)
    frames = sorted(OUTPUT.glob("frame_*.jpg"))
    if len(frames) < 8:
        raise SystemExit(f"conversion produced only {len(frames)} frames")
    print(f"prepared {len(frames)} loading frames and audio in {OUTPUT.parent}")


if __name__ == "__main__":
    main()
