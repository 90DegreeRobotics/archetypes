# Plan: local voice architecture audit — 2026-07-12 05:30

## Status
COMPLETED

## Goal
Audit the proposed Whisper.cpp and Piper speech architecture against the actual Archetypes repository, the attached external analysis, Bevy 0.18.1 compatibility, licensing, model availability, Windows installation requirements, and the real operator-facing ritual loop before selecting an implementation lane.

## Steps
### Step 1 — Read the external proposal and repository seams
- [x] Action: Inspect the attached report and current engine, dependency manifest, Windows bootstrap, audio dependencies, and ritual state flow.
- Files touched: This plan only.
- Expected outcome: Evidence-backed map of what is real, missing, compatible, and risky.

### Step 2 — Verify current upstream options
- [x] Action: Check official crate, Whisper.cpp, Piper, voice-model, licensing, and runtime-distribution facts that can drift.
- Files touched: None.
- Expected outcome: A current recommendation grounded in primary sources rather than consultant assertions.

### Step 3 — Deliver the implementation verdict
- [x] Action: Reconcile the proposals into the smallest honest offline STT/TTS architecture for the Forge and identify any decisions that require operator authority.
- Files touched: This plan and architecture/status documentation only if implementation is authorized.
- Expected outcome: Clear build order, model/runtime choices, risks, and proof gates without premature code changes.

## Verdict

- STT: use CPAL/WASAPI capture and Whisper.cpp off the Bevy main thread, initially push-to-talk rather than heuristic silence detection. Keep keyboard input as an accessibility and proof fallback.
- Do not pin the consultant-proposed old crate versions. `whisper-rs` moved from its archived GitHub repository to Codeberg, and current CPAL is newer than the proposed `0.15`.
- TTS: Piper remains technically suitable, but the maintained upstream is now GPLv3 and Python-first; the old standalone MIT-labelled executable is archived and carries unresolved distribution/licensing ambiguity around its espeak-ng dependency. Legal/distribution posture requires operator authority before bundling either lane.
- Voice model identifiers and licenses must be validated one model at a time from each model card. The proposed `amy-high` and `alba-high` identifiers are not valid current entries; Alba is currently medium quality.
- Bevy 0.18.1 can play an in-memory encoded WAV through `AudioSource { bytes }`, but raw Piper PCM is not directly a Bevy `AudioSource`; the worker must emit or wrap a supported file format.
- Windows Metabolism requires the speech runtime, models, hashes, license metadata, install path, and readiness probes to be declared in `scripts/dependencies.json` and exercised by launcher/bootstrap proof. A sidecar copied ad hoc into LocalAppData is forbidden.
