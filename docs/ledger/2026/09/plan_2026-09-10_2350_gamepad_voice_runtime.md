# Plan: Gamepad and Voice Runtime — 2026-09-10 23:50

## Status
IN-PROGRESS (Steps 1-2 landed 2026-09-11; Steps 3-5 remain)

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
- [x] Action: Verify Bevy's actual gamepad APIs, available Windows input devices, current TTS
  runtime, and a local ASR candidate/license/package contract.
- Verified against the pinned `bevy_input-0.18.1` source in the local cargo registry cache:
  gamepads are entity-based (`Component Gamepad` with `left_stick()`/`right_stick()`/
  `pressed()`/`just_pressed(GamepadButton)`), and `gilrs`/`bevy_gilrs` are already resolved
  in `Cargo.lock` (compiled in by default, no Cargo.toml feature change needed). A real
  physical controller is in fact connected on this workstation: the installed-build capture
  run below logged `gilrs::gamepad: No mapping found for UUID 0000...` /
  `bevy_input::gamepad: Gamepad 31v0 connected.` — proof this is a live device, not a
  hypothetical. TTS and ASR audit unchanged from the prior corrections in this file (Kokoro/
  sherpa real for Council/Verdict; no local ASR yet) — Steps 3-4 remain open below.

### Step 2 — Implement action settings and gamepad controls
- [x] Action: Add a persisted, local settings resource; deadzone/look controls; gamepad
  locomotion/look/interact/cancel/menu bindings.
- [ ] Action: Build a visible in-game Settings UI screen (sliders/menu) for these values.
- Landed: `services/settings.rs` (`GameSettings` resource: mouse sensitivity, gamepad look
  sensitivity, gamepad deadzone, four volume channels; clamped on load/save; atomic
  temp-then-rename write to `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json`;
  saves immediately whenever changed rather than a staged "Apply" screen — that staging UI is
  the still-open half of this step). `services/gamepad_input.rs` (radial deadzone rescale +
  cross-pad button/stick folding helpers, unit tested). Wired into
  `modes/inner_chambers/camera.rs` (left stick = move, right stick = look, South = jump/
  double-tap-fly/triple-tap-land alongside Space, RightTrigger2/LeftTrigger2 = flight ascend/
  descend alongside Space/Shift), `modes/inner_chambers/encounters.rs` and
  `modes/inner_chambers/manifestation.rs` (West = interact alongside E, East = cancel
  alongside Escape). Movement stays digital-direction (matches the pre-existing keyboard
  feel); analog throttle scaling is not implemented. No volume slider yet reaches an actual
  audio bus — the settings values exist and persist but nothing multiplies playback by them
  yet; that lands with Step 3's voice wiring.
- Verification: `cargo build -p engine` and `cargo test --workspace` both pass (108 engine +
  19 launcher + 5 windows_identity, 15 of the engine tests new). `scripts\install_shortcut.ps1`
  rebuilt release and restaged Desktop/Start Menu/Taskbar with SHA-256-verified binaries. A
  real `ARCHETYPES_INNER_CAPTURE=1` run of the installed `engine.exe` produced 8 fresh frames
  under `artifacts/visual-proof/gamepad-and-settings-2026-09-11/` with no crash, the connected
  gamepad recognized by gilrs, and the updated HUD legend ("WASD/L-Stick", "Space/A",
  "Esc/B", "Space/RT", "Shift/LT") rendering correctly. `%LOCALAPPDATA%\NeuroCognica\
  Archetypes\config\settings.json` was created for real with the expected default fields.
  This proves the input plumbing and persistence are real and non-crashing; it does not
  prove analog stick-to-camera motion, since no available tool here can move the physical
  controller's sticks — that needs the operator to hold the pad in a live (non-capture) run.

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
