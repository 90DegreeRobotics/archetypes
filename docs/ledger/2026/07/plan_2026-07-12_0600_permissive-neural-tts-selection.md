# Plan: permissive neural TTS selection — 2026-07-12 06:00

## Status
COMPLETED

## Goal
Rule out ZeroVOX, demote speech-to-text, and identify the fastest viable path to seven natural offline archetype voices on Windows using a permissively licensed neural TTS runtime and models that can satisfy Archetypes Windows Metabolism.

## Steps
### Step 1 — Search the current permissive neural field
- [x] Action: Evaluate current offline neural TTS engines and model families using upstream license, Windows runtime, voice inventory, latency, footprint, and maintenance evidence.
- Files touched: This plan only.
- Expected outcome: Shortlist that excludes GPL, non-commercial, research-only, Linux-only, and classical robotic engines.

### Step 2 — Verify the leading runtime on the Forge
- [x] Action: Inspect available Windows artifacts and APIs and run a local synthesis spike with the strongest candidate.
- Files touched: Designated proof/runtime scratch paths only unless implementation is authorized by evidence.
- Expected outcome: A heard or mechanically verified voice candidate rather than a paper recommendation.

### Step 3 — Select and define the TTS implementation lane
- [x] Action: Record the chosen engine, models, seven-voice mapping strategy, installer contract, Bevy bridge, proof gates, and rejected alternatives.
- Files touched: Architecture/status documentation and implementation files if the candidate passes.
- Expected outcome: Immediate build direction centered on hearing the archetypes, with keyboard input preserved.

## Selection

Use **Kokoro-82M through sherpa-onnx** as the Archetypes TTS lane.

- Kokoro model: Apache-2.0.
- sherpa-onnx runtime: Apache-2.0, native Windows x64, offline, CPU-capable, and exposes Rust/C/C++ APIs.
- Runtime proof used official `sherpa-onnx v1.13.4` Windows x64 shared MD Release and official `kokoro-en-v0_19` model bundle.
- Runtime archive SHA-256: `D4DACC8BE5AFE03F22ADE4D50CFD587C03A625EACA8C41F2D99A24D3DB463EAB`.
- Model archive SHA-256: `912804855A04745FA77A30BE545B3F9A5D15C4D66DB00B88CBCD4921DF605AC7`.
- No Python, CUDA, cloud service, GPL runtime, or voice cloning is required.

## Forge Proof

Seven 24 kHz mono PCM WAVs were synthesized locally with four CPU threads and preserved in `artifacts/audio-proof/kokoro/`.

| Archetype | Kokoro speaker | Speaker ID | Synthesis seconds | Audio seconds | RTF |
|---|---|---:|---:|---:|---:|
| Architect | `bm_george` | 9 | 5.168 | 4.986 | 1.036 cold |
| Sentinel | `bm_lewis` | 10 | 2.934 | 4.112 | 0.714 |
| Mentor | `am_adam` | 5 | 2.473 | 3.732 | 0.663 |
| Explorer | `af_sky` | 4 | 2.667 | 4.078 | 0.654 |
| Oracle | `bf_emma` | 7 | 2.528 | 3.719 | 0.680 |
| Empath | `af_sarah` | 3 | 3.565 | 4.086 | 0.873 |
| Jester | `af_nicole` | 2 | 3.915 | 5.308 | 0.738 |

The mapping is a first audition slate, not immutable canon. The operator must approve voices by ear before it becomes the shipped manifest.

## Implementation Contract

1. Keep typing as the Witness input; STT is out of the critical path.
2. Declare sherpa-onnx runtime files and Kokoro model assets with hashes in `scripts/dependencies.json`.
3. Install immutable runtime/model payload under `%ProgramFiles%\Archetypes`, not LocalAppData; reserve LocalAppData for generated WAV cache and manifests.
4. Add launcher readiness probes for executable/library load, model presence, hash, and one short synthesis.
5. Add a Bevy `SpeechPlugin` whose worker owns the native TTS runtime off the render thread and returns WAV bytes for `AudioSource` playback.
6. Trigger one foreground archetype voice at a time from chamber state and `CurrentFocus`; preserve keyboard and silent-mode fallbacks.
7. End-to-end proof must type an offering, enter focus/verdict state, generate a non-empty WAV, play it in the live chamber, and verify the selected archetype speaker ID.

## Rejected

- ZeroVOX: Linux/alpha/distribution stop.
- Piper: maintained GPLv3 lane and legacy-runtime ambiguity.
- XTTS: non-commercial model license.
- MaryTTS/Flite/Mimic: permissive but below the required voice quality.
