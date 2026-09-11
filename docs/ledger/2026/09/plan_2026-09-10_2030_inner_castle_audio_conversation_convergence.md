# Plan: Inner Castle Audio, Conversation, and Mode Convergence — 2026-09-10 20:30

## Status
IN-PROGRESS

## Goal

Turn Inner Chambers into the eventual primary game shell without pretending that a
background loop or a proximity prompt is a complete system. This plan defines three
connected deliveries: an authored, local-first soundscape; a player-to-archetype
conversation loop entered in-world; and a migration architecture through which the
current standalone modes become castle-located systems rather than menu destinations.

The product outcome is a player who walks the castle, hears where they are, approaches
an embodiment, presses `E`, and uses the existing typed conversation capability in a
diegetic encounter. Push-to-talk speech input is an optional local enhancement with a
visible availability state, not a substitute for a working keyboard path.

## Audit baseline: facts, not assumptions

- **Implemented audio:** `crates/engine/src/chamber/speech.rs` generates council and
  verdict speech through the local Kokoro/sherpa-onnx worker and plays the resulting WAV
  through Bevy `AudioPlayer`. This is one-shot TTS, not environmental audio.
- **Implemented assets:** `assets/audio/archetypes/*.wav` contains seven pinned voice
  assets; `assets/loading/blackflame.wav` exists but is deliberately not part of current
  startup playback.
- **Not implemented:** no `assets/audio/music/`, no `assets/audio/sfx/`, no ambient
  player, no positional emitter, no `SpatialListener`, no footsteps, no manifestation
  sound, no music transition system, and no audio settings surface.
- **Existing chat:** `crates/engine/src/modes/standard_mecha/mod.rs` owns the playable
  typed archetype chat, local Ollama request flow, durable history, response UI, and
  optional artifact reveal. Its session and bridge types are private to that mode, so
  Inner Chambers cannot honestly “reuse it” without extracting a shared service/UI
  contract first.
- **Input conflict:** Inner Chambers already uses `E` for manifestation and extraction.
  A character interaction system must resolve one focused target before it opens any UI;
  adding another `just_pressed(KeyE)` system would create non-deterministic behavior.
- **Speech recognition:** no STT engine, model, dependency declaration, microphone
  permission/readiness check, or runtime route currently exists. STT is unbuilt.

## Product principles

1. Sound must communicate place, distance, state, and consequence. It must not become an
   unbroken wallpaper layer that buries TTS or makes the castle feel like a generic trailer.
2. Every generated or recorded asset has provenance, license, source session, format, loop
   points, and loudness metadata. Do not install mystery audio from the web.
3. Typed conversation ships before STT. If the microphone, model, or transcription worker
   fails, the player can still engage every archetype with the keyboard.
4. One interaction resolver owns `E`; systems register targets and priorities rather than
   competing for raw key events.
5. A standalone mode is not retired until its equivalent castle activation is wired,
   end-to-end tested, persisted correctly, and reachable from normal Inner Chambers play.
6. Preserve local-first operation. Ambient audio plays from installed assets; inference stays
   local; transcripts and optional recordings live only under LocalAppData.

## Asset contract

All immutable authored audio belongs under `assets/audio/`; mutable user audio and STT cache
belong under `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\audio_cache\` or a new scoped
subdirectory. Do not place generated captures in `assets/`.

```text
assets/audio/
  music/
    inner_castle_base_loop.wav
    inner_castle_base_loop.json
    seven_heavens_overlook_loop.wav
    seven_heavens_overlook_loop.json
  ambience/
    abyss_low_air_loop.wav
    castle_torch_crackle_loop.wav
    archetypes/
      architect_proximity_loop.wav
      sentinel_proximity_loop.wav
      explorer_proximity_loop.wav
      empath_proximity_loop.wav
      mentor_proximity_loop.wav
      oracle_proximity_loop.wav
  sfx/
    footsteps/stone_01.wav ... stone_08.wav
    footsteps/bridge_01.wav ... bridge_06.wav
    manifestation/idle_hum_loop.wav
    manifestation/input_rune.wav
    manifestation/charge_rise.wav
    manifestation/arrival_chord.wav
    ui/interact_focus.wav
    ui/interact_open.wav
    ui/interact_close.wav
  archetypes/                         # existing voice assets remain here
```

Every audio sidecar JSON records: `source`, `creator`, `license`, `created_at`, `sample_rate`,
`bit_depth`, `channels`, `integrated_lufs`, `true_peak_dbtp`, `loop_start_samples`,
`loop_end_samples`, and a short intended-runtime note. Require 48 kHz WAV for source masters;
use 24-bit PCM for music/ambience masters and 24-bit mono WAV for short effects. Runtime may
derive a compressed delivery format only after a measured packaging/performance decision.

## Gemini music prompts

Generate tracks as original, non-derivative works. Do not request a named composer, franchise,
or living artist style. Ask for no spoken words: spoken language competes with council TTS.

### Prompt A — Inner Castle base layer

```text
Create an original 12-minute dark-ambient instrumental for a vast ancient stone castle
suspended over a luminous abyss. The piece must be designed as a seamless background loop,
not a trailer cue. Begin and end with compatible ambience and no hard final chord.

Palette: distant low male choir as texture only (no intelligible words), restrained bass
viol drones, bowed metal, sparse bronze singing-bowl harmonics, barely audible wind moving
through enormous vaulted corridors, and occasional far-off stone resonance. Tempo 58 BPM or
slower; no percussion groove; no heroic melody; no sudden stingers; no EDM, trap, pop, or
cinematic trailer impacts. Emotional arc: solemn curiosity, dignity, depth, and a sense of
an inhabited place with secrets.

Leave substantial headroom for spoken dialogue. Deliver the highest-quality lossless WAV
available, 48 kHz, and make the final 20 seconds compatible with the first 20 seconds for
crossfade looping. Provide a separate short description of loop points and instrumentation.
```

### Prompt B — Seven Heavens overlook layer

```text
Create an original 10-minute seamless ambient instrumental for walking high circular galleries
inside an enormous ancient castle, looking down over six distant chambers and bridges. This is
not combat music and not a fantasy anthem.

Palette: slow glass harmonics, quiet pipe-organ fundamentals, very distant wordless choir,
soft air, and occasional bell-like tones with long natural decay. The harmony should feel
uplifted and spacious without becoming cheerful or sentimental. Use an extremely slow evolving
texture; no beat, no drums, no recognizable melody, no vocals with words, no abrupt changes,
no final cadence. Preserve quiet intervals so footsteps, banners, dialogue, and environmental
effects can be heard.

Deliver highest-quality lossless WAV available at 48 kHz, with a loop-compatible opening and
ending and notes identifying a safe 15-second crossfade region.
```

### Acceptance test for generated music

Do not import the first attractive render. In a DAW, crossfade the declared end-to-start window
for 30 seconds, listen on speakers and headphones, and reject it if a pulse, click, harmonic
jump, intelligible lyric, or loudness jump appears. Measure the loop at approximately -24 LUFS
integrated and below -2 dBTP before in-game attenuation; this leaves real room for TTS.

## Sound-effect production plan

### Foley: footsteps and architecture

Record original foley with a phone or field recorder at 48 kHz, then clean and export masters
in a DAW. A stone footstep library needs at least eight non-identical takes: heel/weight shift,
full boot impact, scuff, and grit for both dry flagstone and bridge stone. Do not create one
footstep and pitch-shift it into eight fake variations.

Record on slate, ceramic tile over sand, concrete pavers, or a stone garden path. Place the
microphone 0.5–1.5 m away to avoid handling noise; record 10–20 isolated hits per surface. Trim
silence, apply only corrective high-pass filtering, remove clicks manually, normalize each one
shot to a conservative peak, and retain natural transient differences. The game randomizes
selection and applies only a small gain/pitch range (for example ±1.5 dB and ±3%), never enough
to make a footstep cartoonish.

### Manifestation sound family

Build the manifester as layers, not one giant generic magic WAV:

1. `idle_hum_loop`: quiet low electrical/stone resonance; 20–40 seconds; seamless loop.
2. `input_rune`: a tactile short glass/metal confirmation when a typed character is accepted;
   not played for every key if that becomes noisy—rate-limit it.
3. `charge_rise`: 2–5 second rising harmonic texture while a real generation job transitions
   from queued to rendering.
4. `arrival_chord`: one restrained harmonic arrival when an artifact is actually ready; never
   play it for a failed job.
5. `failure_dissolve`: soft downward de-tune/noise release for a real failure state, paired with
   visible text. It must not imply an artifact was made.

Create these from original recordings plus synthesis in a DAW: bowed cymbal or singing-bowl
scrape for harmonic partials, low filtered noise for abyss air, and a recorded stone tap for
physical contact. Keep source stems and a README describing the processing chain. If using a
commercial/CC0 pack, store its license and exact source URL in the asset sidecar; reject
no-attribution, unclear-license, or ripped game audio.

### Future castle emitters

- Torch: a low-volume crackle loop, one randomized ember pop every 8–30 seconds.
- Abyss: non-musical low air and distant sub-harmonic stone resonance, spatially strongest near
  bridge edges and lower levels.
- Banners: quiet cloth movement driven by nearby wind zones; no perpetual loud flapping.
- Shelves/art: silent by default. An artifact earns an emitter only when it has a narrative or
  interactive reason.

## Runtime architecture

### Audio service

Create `crates/engine/src/chamber/audio.rs` as a mode-neutral plugin. It owns a named bus model:
`Music`, `Ambience`, `Sfx`, `Voice`, and `Ui`, with persisted user volume settings. Do not allow
individual modes to spawn untracked infinite `AudioPlayer` entities. The service must own:

- explicit handles and lifecycle components for loops;
- crossfades measured in seconds, not a hard stop/start;
- ducking: active TTS reduces Music and Ambience but not critical SFX;
- source availability/error state in the in-game settings panel;
- deterministic cleanup when a mode exits;
- testable events such as `AudioCue::ManifestationReady` rather than direct sound spawns in
  gameplay code.

Add `SpatialListener` to the actual Inner Chambers player camera only after listener behavior is
tested from the installed build. Attach positional loops to world entities, not absolute global
coordinates, so a migrated castle layout does not detach sound from architecture.

### Archetype conversation service

Extract reusable types from `standard_mecha` into a new neutral module, for example
`crates/engine/src/chamber/archetype_conversation.rs`. Do not import `standard_mecha::mod.rs`
into Inner Chambers or duplicate its private `StandardMechaSession` logic.

The extracted contract owns:

- selected `Archetype` and persona/system prompt;
- typed draft, cursor, transcript, request-in-flight status, and response/error payload;
- local Ollama request worker and cancellation/timeout policy;
- durable transcript record with mode/location/encounter metadata;
- optional artifact rendering only when explicitly requested by the player and supported by the
  current encounter; chat must not silently generate images;
- a UI adapter interface so Standard Mecha and Inner Chambers can render different frames over
  the same conversation state.

The Inner Chambers overlay is a modal but diegetic encounter panel: the archetype name, chamber
title, transcript, text input, service status, and clear close/return instruction. Player motion
is paused while the panel owns keyboard input. On close, camera control returns to the exact
world position and encounter target. A failed Ollama request leaves the typed message visible
and reports the failure; it does not fabricate an archetype answer.

### One interaction resolver

Introduce `Interactable` data with `kind`, `priority`, `range`, `prompt`, and an entity reference.
Each frame, `InteractionFocusSystem` chooses exactly one eligible target from player distance and
view direction, then renders one proximity prompt. Suggested deterministic priority:

1. Active modal controls (close/submit/cancel).
2. A focused archetype embodiment in direct range and view.
3. A focused mini-game/device entrance.
4. Manifestation altar.
5. Extraction/truth node.

`E` sends one `InteractRequested { target }` event. Individual systems consume typed target
events, never raw `KeyE`. This resolves the existing manifestation/extraction conflict and makes
future castle systems composable. Interaction prompt text must say what will happen, for example
`[E] Speak with Architect — typed conversation` rather than vague “interact.”

### Local STT, after typed chat works

STT is a push-to-talk enhancement. Do not auto-record, continually stream the microphone, or
send audio to cloud services. Use a hold-to-talk binding such as `V`; `E` remains engagement.

Evaluate local engines on the Forge before adoption:

- **whisper.cpp**: benchmark a small English model for latency, CPU use, executable license, and
  clean subprocess cancellation.
- **sherpa-onnx offline ASR**: evaluate because the project already packages sherpa-onnx for TTS,
  but treat ASR model/runtime requirements as independent until verified.

The STT worker writes temporary WAV only under LocalAppData, sends it to the local engine, inserts
the transcript into the existing typed draft for player review, then deletes the temporary audio
on success/failure unless the user explicitly enables a retained diagnostic recording. The player
presses Enter to send; STT never automatically transmits words to an archetype. The UI must show
`Microphone ready`, `Listening`, `Transcribing locally`, `Review before send`, or a specific
failure. Add an audio-device selection/settings surface before treating microphone support as
shipped.

## Folding modes into the castle

The final product direction is one Inner Chambers entry point, but the migration is by system
adapter—not by deleting proven standalone modes early.

| Current surface | Castle activation | Return contract | Retirement gate |
|---|---|---|---|
| Standard Mecha typed archetype chat | Approach an archetype embodiment and press `E` | Close returns player to that embodiment | Shared conversation service passes transcript/history/UI/runtime tests in both surfaces |
| Oracle Riddle | Oracle observatory device or table | Solve/abandon returns to Observatory | Scoring, ledger proof, artifact path, and difficulty UI work in-world |
| Manifestation | Council/manifestation workshop | Generated artifact returns to its display point | Existing receipt, progress, failure, and object pipeline remain end-to-end valid |
| Living Engine | A dedicated engine/observatory installation | Session ends at that installation | Simulation state, controls, and accessibility parity are proven |
| Future mini-games | A named physical device/door/table with clear affordance | Always returns to original world transform | Each mini-game owns an explicit state, save record, exit path, and test witness |

Mini-games should be architecturally earned: Sentinel has discipline/trial systems, Oracle has
pattern/riddle systems, Explorer has navigation/discovery systems, Empath has relation/listening
systems, Mentor has archive/reflection systems, Architect has construction/blueprint systems, and
Jester has controlled disruption/play systems. Do not ship labels as games. Each requires a
concrete loop, failure/exit behavior, reward or reflection output, persistence rule, and a
castle-location reason to exist.

## Delivery sequence and verification

### Phase 1 — Audio intake and proof
- [ ] Create the asset directories and sidecar schema.
- [ ] Receive/import one original castle base loop and one original Seven Heavens loop.
- [ ] Verify format, loop crossfade, loudness, provenance, packaging path, and installed playback.
- [ ] Add volume settings and TTS ducking before adding more than two loops.

### Phase 2 — First physical sound slice
- [ ] Add stone footsteps, one abyss emitter, one torch emitter, and manifestation idle/arrival.
- [ ] Add one player `SpatialListener`, move around emitters, and capture/listen from the installed
  Desktop/Taskbar build.
- [ ] Verify no duplicate loops after mode transitions and no audio plays for failed manifestation.

### Phase 3 — Typed archetype encounter
- [x] Extract the shared persona/request contract from Standard Mecha into
  `services/archetype_conversation.rs`; both Standard Mecha and Inner Chambers now use the
  same canonical local-Ollama persona contract, with no automatic image-generation side effect
  in the castle path. `cargo test --workspace` passed 119/119 on 2026-09-10.
- [ ] Replace all competing raw-`E` paths with the planned typed `Interactable` resolver. The
  first encounter uses a range-gated embodiment focus; manifestation and truth-node input are
  not yet migrated, so the global resolver is not claimed complete.
- [x] Add six embodied typed encounters in Inner Chambers. The modal panel pauses locomotion,
  writes local JSONL transcript records under LocalAppData plus hash-chained ledger metadata,
  issues a local Ollama request, and exposes a specific failure without fabricating a reply.
- [ ] Prove from the installed Desktop/Taskbar product that `E` opens only the focused target,
  player movement pauses, local Ollama failures remain
  visible, close restores play, and a transcript persists with encounter metadata.
- [ ] Expand to all archetypes only after the first one passes the installed witness.

### Phase 4 — Optional local STT spike
- [ ] Benchmark whisper.cpp and sherpa-onnx ASR on the RTX 3060/32 GB Forge using the same short
  microphone corpus; record latency, CPU/GPU/VRAM, accuracy, license, device behavior, and cancel
  behavior.
- [ ] Adopt one only if it inserts reviewable text into typed chat reliably and does not impair TTS,
  rendering, or local privacy. Otherwise ship typed chat and record STT as blocked/deferred.

### Phase 5 — Castle mode convergence
- [ ] Move one current mode at a time behind a physical castle activation point.
- [ ] Preserve the standalone mode until in-world parity is independently verified.
- [ ] Retire menu entry only after migration proof, documentation, installed launcher test, and
  explicit operator acceptance.

## Definition of done for the first real slice

The first delivery is not “audio assets exist” or “a panel opens.” It is complete only when a
player launches from the pinned/desktop product, enters Inner Chambers, walks to one embodiment,
sees one truthful `[E] Speak with …` prompt, opens a typed local conversation, sends and receives
or visibly fails a local response, closes back to the exact location, hears an original properly
looped castle bed at a non-intrusive level, and can observe an actual manifestation/ambient
emitter behavior without duplicate playback. The relevant automated tests, audio metadata checks,
installed build, and recorded visual/audio witness must be attached before commit and push.

## Current implementation checkpoint — 2026-09-10 22:20

The typed encounter code is compiled and the complete Rust suite passed (`119 passed; 0 failed`).
`install_shortcut.ps1` built the release binaries but did **not** complete the installed-product
copy: a running process held
`%LOCALAPPDATA%\Programs\Archetypes\assets\mecha\uxbacklayer.png` open. No commit or push is
authorized by the repository delivery rules until that lock is cleared and the installed build is
visually exercised. Audio, STT, the global interaction resolver, and mode convergence are also
still unimplemented; the plan remains IN-PROGRESS.
