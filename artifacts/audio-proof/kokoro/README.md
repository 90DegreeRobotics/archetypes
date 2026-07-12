# Kokoro Archetype Voice Audition

These WAV files are proof outputs generated locally on the Forge. They are not a final casting decision and are not yet wired into the application.

- Runtime: sherpa-onnx v1.13.4 Windows x64 shared MD Release (Apache-2.0)
- Model: kokoro-en-v0_19, derived from Kokoro-82M (Apache-2.0)
- Audio: 24 kHz, mono, 16-bit PCM WAV
- Inference: CPU, four threads, fully offline

| File | Speaker ID | Kokoro speaker |
|---|---:|---|
| `architect.wav` | 9 | `bm_george` |
| `sentinel.wav` | 10 | `bm_lewis` |
| `mentor.wav` | 5 | `am_adam` |
| `explorer.wav` | 4 | `af_sky` |
| `oracle.wav` | 7 | `bf_emma` |
| `empath.wav` | 3 | `af_sarah` |
| `jester.wav` | 2 | `af_nicole` |

The runtime and model archives remain outside the repository in the designated LocalAppData proof area. They must not become product dependencies until declared, installed, and readiness-probed through Windows Metabolism.
