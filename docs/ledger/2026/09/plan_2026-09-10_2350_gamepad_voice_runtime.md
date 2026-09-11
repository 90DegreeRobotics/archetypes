# Plan: Gamepad and Voice Runtime — 2026-09-10 23:50

## Status
IN-PROGRESS

## Goal

Replace the unimplemented claims in the Xbox/gamepad guide with a real local input and
voice path: Xbox-compatible gamepad movement/interact/settings controls, persisted settings,
push-to-talk local speech recognition, and spoken Inner Castle archetype replies. This work
does not treat a static controller mapping table, a TTS executable on disk, or an unavailable
microphone as a shipped feature.

## Audit corrections

- Bevy 0.18 supplies an entity-based `Gamepad` component, but the guide's exact button/axis
  snippets and assumptions must be compiled against the pinned crate before adoption.
- The described settings mixer and apply-on-close surface do not exist in the current game.
- Existing Kokoro/sherpa TTS is real only for Council/Verdict pathways; Inner Castle responses
  presently remain text-only.
- No local ASR binary, model, microphone capture path, or STT readiness contract exists.

## Steps

### Step 1 — Verify platform contracts and installed capability
- [ ] Action: Verify Bevy's actual gamepad APIs, available Windows input devices, current TTS
  runtime, and a local ASR candidate/license/package contract.
- Expected outcome: No code is based on invented Bevy names or an assumed microphone engine.

### Step 2 — Implement action settings and gamepad controls
- [ ] Action: Add a persisted, local settings resource; deadzone/look controls; gamepad
  locomotion/look/interact/cancel/menu bindings; and a visible settings surface.
- Expected outcome: Xbox/XInput input reaches the same semantic actions as keyboard input.

### Step 3 — Make Inner Castle voice bidirectional
- [ ] Action: Route successful Inner Castle replies into the existing local Kokoro/sherpa worker;
  add explicit voice status and volume control.
- Expected outcome: A typed or spoken player turn receives an audible, local archetype reply.

### Step 4 — Implement local push-to-talk STT
- [ ] Action: Install/package one verified offline ASR runtime and model, capture only while held,
  transcribe locally, insert reviewable text into the encounter draft, and delete temporary audio.
- Expected outcome: `V` hold-to-talk is a real local recording/transcription route, never
  cloud-streaming or automatic-send behavior.

### Step 5 — Verify, package, commit, and push
- [ ] Action: Run device-aware tests, live microphone/TTS checks, full Rust suite, installed
  launcher staging, visual/audio witness, then commit and push.
- Expected outcome: Buyer-reachable controls and voice behavior with concrete evidence.
