"""Author the game's sound effects from first principles.

"Free sound effects" is answered by **owning them**. Every cue here is synthesised from noise,
decaying partials and filtered transients — no sample pack, no licence to check, no attribution
to carry, nothing that has to be torn out later if a licence turns out to be encumbered. Same
reasoning as the stone tiles: see `memory: build-it-dont-wait-for-a-clean-license`.

Everything is deterministic. Each cue seeds its own generator, so re-running this script writes
byte-identical files and a sound never silently changes under a build.

Run:  python scripts/author_sound_effects.py
"""

from __future__ import annotations

import argparse
import json
import math
import struct
import wave
from dataclasses import dataclass
from pathlib import Path

import numpy as np

OUT_DIR = Path("assets/audio/sfx")
RATE = 44_100

# The engine mixes these under its own master and SFX sliders, so they are authored at a
# consistent headroom rather than each one being normalised to full scale — a footstep and a
# manifestation flourish should not arrive at the same loudness.
PEAK = 0.86


def log(*parts: object) -> None:
    print("[sfx]", *parts, flush=True)


# ---------------------------------------------------------------------------------------
# Synthesis primitives
# ---------------------------------------------------------------------------------------

def t(seconds: float) -> np.ndarray:
    return np.linspace(0.0, seconds, int(RATE * seconds), endpoint=False)


def env_ad(times: np.ndarray, attack: float, decay: float, curve: float = 2.2) -> np.ndarray:
    """Attack-decay envelope. Percussive cues are almost entirely this shape."""
    out = np.zeros_like(times)
    rise = times < attack
    if attack > 0:
        out[rise] = times[rise] / attack
    fall = ~rise
    span = max(decay, 1e-6)
    out[fall] = np.clip(1.0 - (times[fall] - attack) / span, 0.0, 1.0) ** curve
    return out


def sine(times: np.ndarray, freq, phase: float = 0.0) -> np.ndarray:
    """A sine whose frequency may itself be an array, integrated so a sweep stays continuous.

    Multiplying `freq * t` for a varying frequency produces a phase discontinuity and an
    audible click; integrating the instantaneous frequency is what makes a glide glide.
    """
    if np.isscalar(freq):
        return np.sin(2.0 * math.pi * float(freq) * times + phase)
    dt = 1.0 / RATE
    return np.sin(2.0 * math.pi * np.cumsum(np.asarray(freq) * dt) + phase)


def noise(times: np.ndarray, rng: np.random.Generator) -> np.ndarray:
    return rng.uniform(-1.0, 1.0, times.shape)


def lowpass(signal: np.ndarray, cutoff: float) -> np.ndarray:
    """One-pole lowpass. Crude and exactly right for shaping noise into a material."""
    alpha = 1.0 - math.exp(-2.0 * math.pi * cutoff / RATE)
    out = np.empty_like(signal)
    carry = 0.0
    for index, sample in enumerate(signal):
        carry += alpha * (sample - carry)
        out[index] = carry
    return out


def highpass(signal: np.ndarray, cutoff: float) -> np.ndarray:
    return signal - lowpass(signal, cutoff)


def bandpass(signal: np.ndarray, low: float, high: float) -> np.ndarray:
    return highpass(lowpass(signal, high), low)


def normalise(signal: np.ndarray, peak: float = PEAK) -> np.ndarray:
    loudest = float(np.max(np.abs(signal)))
    if loudest < 1e-9:
        return signal
    return signal * (peak / loudest)


def declick(signal: np.ndarray, millis: float = 4.0) -> np.ndarray:
    """Fade the first and last few milliseconds.

    A waveform that starts or ends off zero is a step, and a step is a click. Every cue gets
    this whether or not its envelope already looks like it reaches zero.
    """
    span = max(1, int(RATE * millis / 1000.0))
    span = min(span, len(signal) // 2)
    if span <= 1:
        return signal
    ramp = np.linspace(0.0, 1.0, span)
    out = signal.copy()
    out[:span] *= ramp
    out[-span:] *= ramp[::-1]
    return out


def write_wav(path: Path, signal: np.ndarray) -> float:
    path.parent.mkdir(parents=True, exist_ok=True)
    samples = np.clip(declick(signal), -1.0, 1.0)
    pcm = (samples * 32767.0).astype(np.int16)
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(RATE)
        handle.writeframes(pcm.tobytes())
    return len(samples) / RATE


# ---------------------------------------------------------------------------------------
# Cues
# ---------------------------------------------------------------------------------------

def footstep(seed: int) -> np.ndarray:
    """A boot on dressed stone: a hard transient, a short body, no ring.

    Stone has almost no sustain, which is what separates it from wood or metal. The variation
    between the four is in the body's centre frequency and the transient's sharpness, so a run
    does not sound like one sample repeating.
    """
    rng = np.random.default_rng(seed)
    times = t(0.16)
    centre = 520.0 + rng.uniform(-130.0, 170.0)

    body = bandpass(noise(times, rng), centre * 0.55, centre * 2.6)
    body *= env_ad(times, 0.002, 0.075, curve=2.6)

    # The click of the heel, above the body and much shorter.
    tick = highpass(noise(times, rng), 2600.0) * env_ad(times, 0.0006, 0.016, curve=3.4)

    # A little low weight so a step has mass in a big hall.
    thump = sine(times, 92.0 + rng.uniform(-12.0, 12.0)) * env_ad(times, 0.003, 0.055, curve=3.0)

    return normalise(body * 0.75 + tick * 0.5 + thump * 0.35, PEAK * 0.55)


def jump() -> np.ndarray:
    rng = np.random.default_rng(2001)
    times = t(0.26)
    sweep = np.linspace(230.0, 430.0, times.size)
    tone = sine(times, sweep) * env_ad(times, 0.004, 0.16, curve=2.0)
    cloth = bandpass(noise(times, rng), 700.0, 3800.0) * env_ad(times, 0.001, 0.05, curve=3.0)
    return normalise(tone * 0.6 + cloth * 0.35, PEAK * 0.6)


def land() -> np.ndarray:
    rng = np.random.default_rng(2002)
    times = t(0.34)
    thud = sine(times, np.linspace(150.0, 62.0, times.size))
    thud *= env_ad(times, 0.003, 0.20, curve=2.4)
    grit = bandpass(noise(times, rng), 320.0, 2400.0) * env_ad(times, 0.001, 0.09, curve=3.0)
    return normalise(thud * 0.85 + grit * 0.4, PEAK * 0.72)


def pick_up() -> np.ndarray:
    """Rising two-partial ping: something leaving the ground and coming to hand."""
    times = t(0.30)
    sweep = np.linspace(430.0, 780.0, times.size)
    tone = sine(times, sweep) * 0.7 + sine(times, sweep * 2.0) * 0.22
    return normalise(tone * env_ad(times, 0.004, 0.24, curve=1.9), PEAK * 0.62)


def place() -> np.ndarray:
    """The inverse of pick up: a settling fall, then a soft stone contact."""
    rng = np.random.default_rng(2003)
    times = t(0.32)
    sweep = np.linspace(700.0, 330.0, times.size)
    tone = sine(times, sweep) * env_ad(times, 0.004, 0.17, curve=2.1)
    contact = bandpass(noise(times, rng), 260.0, 1900.0) * env_ad(times, 0.0, 0.06, curve=3.2)
    # Delay the contact so it lands at the bottom of the fall rather than with it.
    offset = int(RATE * 0.10)
    shifted = np.zeros_like(contact)
    shifted[offset:] = contact[: contact.size - offset]
    return normalise(tone * 0.6 + shifted * 0.55, PEAK * 0.62)


def duplicate() -> np.ndarray:
    """Two pings a fourth apart: one thing becoming two, said plainly."""
    times = t(0.42)
    first = sine(times, 620.0) * env_ad(times, 0.003, 0.13, curve=2.2)
    second = sine(times, 830.0) * env_ad(times, 0.003, 0.15, curve=2.2)
    offset = int(RATE * 0.11)
    delayed = np.zeros_like(second)
    delayed[offset:] = second[: second.size - offset]
    shimmer = sine(times, 1660.0) * env_ad(times, 0.004, 0.09, curve=3.0) * 0.18
    return normalise(first * 0.6 + delayed * 0.6 + shimmer, PEAK * 0.6)


def swing() -> np.ndarray:
    """A held object cutting the air: a band of noise that rises and falls as it passes.

    No impact. Nothing in the castle is breakable, and pretending otherwise with a thud would
    be a sound effect making a promise the game does not keep.
    """
    rng = np.random.default_rng(2006)
    times = t(0.34)
    # The band sweeps up and back down, which is what makes a pass read as passing rather than
    # as a burst of static.
    centre = 900.0 + 1500.0 * np.sin(np.pi * np.clip(times / 0.30, 0.0, 1.0))
    air = noise(times, rng)
    swept = bandpass(air, 380.0, 4200.0)
    shaped = swept * (centre / centre.max())
    body = shaped * env_ad(times, 0.020, 0.26, curve=1.7)
    return normalise(body, PEAK * 0.5)


def altar_charge() -> np.ndarray:
    """A rising hum under the manifestation prompt. Long, quiet, and not a musical note."""
    rng = np.random.default_rng(2004)
    times = t(1.60)
    base = np.linspace(74.0, 138.0, times.size)
    hum = sine(times, base) * 0.5 + sine(times, base * 1.5) * 0.25 + sine(times, base * 2.01) * 0.14
    air = lowpass(noise(times, rng), 900.0) * 0.16
    swell = np.clip(times / 1.1, 0.0, 1.0) ** 1.6
    tail = np.clip(1.0 - (times - 1.2) / 0.4, 0.0, 1.0)
    return normalise((hum + air) * swell * tail, PEAK * 0.45)


def manifest_success() -> np.ndarray:
    """A bright arrival. Three partials in just intonation so it reads as a chord, not a beep."""
    times = t(1.30)
    root = 294.0
    chord = (
        sine(times, root) * 0.5
        + sine(times, root * 1.5) * 0.34
        + sine(times, root * 2.0) * 0.26
        + sine(times, root * 3.0) * 0.12
    )
    body = chord * env_ad(times, 0.012, 1.15, curve=1.5)
    sparkle = sine(times, np.linspace(1800.0, 2900.0, times.size))
    sparkle *= env_ad(times, 0.006, 0.30, curve=2.6) * 0.2
    return normalise(body + sparkle, PEAK * 0.78)


def manifest_failure() -> np.ndarray:
    """A falling, detuned pair. Unpleasant on purpose, but short."""
    rng = np.random.default_rng(2005)
    times = t(0.85)
    a = sine(times, np.linspace(300.0, 128.0, times.size))
    b = sine(times, np.linspace(311.0, 131.0, times.size))
    buzz = bandpass(noise(times, rng), 180.0, 1300.0) * 0.22
    return normalise((a * 0.44 + b * 0.44 + buzz) * env_ad(times, 0.006, 0.72, curve=1.7), PEAK * 0.66)


def menu_move() -> np.ndarray:
    times = t(0.075)
    return normalise(sine(times, 1180.0) * env_ad(times, 0.001, 0.055, curve=2.8), PEAK * 0.40)


def menu_adjust() -> np.ndarray:
    times = t(0.065)
    return normalise(sine(times, 1560.0) * env_ad(times, 0.001, 0.045, curve=3.0), PEAK * 0.34)


def menu_confirm() -> np.ndarray:
    times = t(0.24)
    low = sine(times, 660.0) * env_ad(times, 0.002, 0.09, curve=2.4)
    high = sine(times, 990.0) * env_ad(times, 0.002, 0.14, curve=2.2)
    offset = int(RATE * 0.055)
    delayed = np.zeros_like(high)
    delayed[offset:] = high[: high.size - offset]
    return normalise(low * 0.6 + delayed * 0.6, PEAK * 0.5)


@dataclass
class Cue:
    name: str
    build: object
    description: str


CUES: list[Cue] = [
    Cue("footstep_stone_1", lambda: footstep(1101), "Boot on dressed stone, variation 1"),
    Cue("footstep_stone_2", lambda: footstep(1102), "Boot on dressed stone, variation 2"),
    Cue("footstep_stone_3", lambda: footstep(1103), "Boot on dressed stone, variation 3"),
    Cue("footstep_stone_4", lambda: footstep(1104), "Boot on dressed stone, variation 4"),
    Cue("jump", jump, "Push off the floor"),
    Cue("land", land, "Landing weight and grit"),
    Cue("pick_up", pick_up, "An object taken into the hand"),
    Cue("place", place, "An object set down on the floor"),
    Cue("duplicate", duplicate, "An object copied: one thing becoming two"),
    Cue("swing", swing, "A held object cutting the air - motion only, no impact"),
    Cue("altar_charge", altar_charge, "Rising hum while the altar takes a prompt"),
    Cue("manifest_success", manifest_success, "A work arrives on the altar"),
    Cue("manifest_failure", manifest_failure, "The pipeline refused or failed"),
    Cue("menu_move", menu_move, "Menu selection moved"),
    Cue("menu_adjust", menu_adjust, "Slider stepped"),
    Cue("menu_confirm", menu_confirm, "Menu row confirmed"),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default=str(OUT_DIR))
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    out_dir = Path(args.out)

    manifest = {
        "schema": "archetypes.sfx.v1",
        "rate": RATE,
        "note": (
            "Authored from noise, decaying partials and filtered transients. No sample pack, "
            "no licence, no attribution. Deterministic: re-running writes identical files."
        ),
        "cues": {},
    }

    for cue in CUES:
        signal = cue.build()
        path = out_dir / f"{cue.name}.wav"
        seconds = write_wav(path, signal)
        peak = float(np.max(np.abs(signal)))
        manifest["cues"][cue.name] = {
            "file": f"{cue.name}.wav",
            "description": cue.description,
            "seconds": round(seconds, 4),
            "peak": round(peak, 4),
        }
        log(f"  {cue.name}: {seconds:.3f}s peak {peak:.3f}")

    (out_dir / "sfx_manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    log(f"wrote {len(CUES)} cues to {out_dir}")


if __name__ == "__main__":
    main()
