# Plan: live archetype voices — 2026-07-12 06:30

## Status
COMPLETED

## Goal
Turn the proven Kokoro/sherpa-onnx audition into a real operator-facing capability: typed Witness offerings remain unchanged, while the focused archetype synthesizes and plays its own offline voice in the live Bevy chamber through a declared, readiness-probed Windows runtime.

## Steps
### Step 1 — Establish the Windows voice dependency contract
- [x] Action: Extend the canonical dependency manifest and bootstrap/readiness lane for pinned sherpa-onnx runtime and Kokoro model artifacts, hashes, immutable install paths, and mutable generated-audio cache.
- Files touched: `scripts/dependencies.json`, bootstrap/runtime path modules, tests, and Windows Metabolism documentation.
- Expected outcome: No ad hoc PATH dependency, manual model prerequisite, or undeclared sidecar.

### Step 2 — Build the non-blocking Bevy speech subsystem
- [x] Action: Add a data-driven `SpeechPlugin`, seven-speaker manifest, background synthesis worker, WAV handoff, Bevy audio playback, visible error state, and silent-mode fallback.
- Files touched: engine speech module, chamber/plugin wiring, Cargo dependencies only if required.
- Expected outcome: TTS never blocks the render thread and only one foreground archetype speaks at a time.

### Step 3 — Wire the ritual to voice
- [x] Action: Emit focused-archetype speech requests from real chamber transitions and verdict content while retaining keyboard interaction and deterministic capture behavior.
- Files touched: ritual/state integration and focused tests.
- Expected outcome: A typed offering causes the Architect to speak an appropriate line in the live chamber.

### Step 4 — Prove, document, and publish
- [x] Action: Run focused tests, full `cargo test`, installed-path/readiness checks, fresh engine build, and live audio proof; reconcile README/STATUS; commit and push `main`.
- Files touched: proof artifacts, documentation, and this plan.
- Expected outcome: The archetypes audibly speak through a reproducible local-first path, with honest failure behavior and clean repository state.

## Verification

- `cargo test`: 11 passed, 0 failed after adding speech tests and WAV support.
- `cargo build -p engine`: passed immediately before live proof.
- `setup_windows.ps1 -PreflightOnly` passed against a proof installed layout: Ollama, pinned sherpa-onnx executable, and pinned Kokoro model readiness paths all resolved.
- Live deterministic run reached `FocusArchetype`; the frame visibly reports `Voice: Architect speaking` and is preserved as `artifacts/audio-proof/kokoro/live-architect-focus.png`.
- The same run reached `WitnessVerdict` and produced `artifacts/audio-proof/kokoro/live-architect-verdict.wav` (808,552 bytes), validated as RIFF/WAVE before Bevy playback.
- Seven packaged signature WAVs provide immediate focus speech without synthesis delay.
- Dynamic capture-mode verdict synthesis took 58.07 seconds while eight large screenshots were competing for resources. This remains an explicit optimization gap; warm standalone short-line synthesis was 2.47-3.92 seconds.
