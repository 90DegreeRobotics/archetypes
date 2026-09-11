# Archetypes Audio & Sound Design Specification

> **Canonical System Reference.** This specification governs all acoustic, musical, spatial, and voice audio across the Archetypes runtime. It consolidates historical plans, constitutional harmonic laws from the AURA manuscripts, engine implementation patterns in Bevy 0.18, and creative sound design systems for the Seed-of-Life inner castle and Council Rotunda.

---

## 1. Executive Summary & Repository Audit

Across the Archetypes repository, sound and music have been treated as foundational constitutional elements rather than post-hoc decorative assets. Multiple architectural documents, release specifications, and research manuscripts lay out a coherent acoustic doctrine:

### Historical & Operational References
1. **[CURRENT_HANDOFF.md](file:///C:/archetypes/docs/ledger/CURRENT_HANDOFF.md):**
   - Explicitly tracks *"ambient music"* as an active open deliverable alongside lore figures and table geometry.
2. **[INSTALLER_WIZARD_SPEC.md](file:///C:/archetypes/docs/windows/INSTALLER_WIZARD_SPEC.md):**
   - Declares the canonical immutable audio layout: `assets/audio/` (*"Soundtrack and spatial ambiance"*).
   - Component Selection: `[X] Ambient Orchestral Soundtrack (Default checked)` as an essential user-facing component.
   - Pre-flight disclosures mandate checking audio device readiness during installation.
3. **[COUNCIL_CHAMBER_DIRECTION.md](file:///C:/archetypes/docs/architecture/COUNCIL_CHAMBER_DIRECTION.md):**
   - Establishes that the identity manuscript contains enough specificity to drive *"sound character"* as an implementation constant.
   - Mandates scene-state audio choreography:
     * *Profile Ritual:* Threshold acoustic tone.
     * *Council Chamber:* Home ambient loop and acoustic equilibrium.
     * *Star Alignment:* Deliberation transition whoosh/harmonic crescendo.
     * *Sphere Interior:* Temporary domain immersion adopting the archetype's sonic atmosphere.
     * *Return to Chamber:* Reintegration fade back to home harmony.
4. **[COUNCIL_WORLD_ENGINE.md](file:///C:/archetypes/docs/architecture/COUNCIL_WORLD_ENGINE.md):**
   - Confirms: *"Their palettes, glow laws, timing, typography, audio signatures, authority boundaries, shadow states, and counterweights govern both dialogue and environmental behavior."*
5. **[JESTER_PROFILE_REPORT.md](file:///C:/archetypes/docs/architecture/JESTER_PROFILE_REPORT.md):**
   - Defines physical acoustic interactions: *"Slapstick Pad Modifier: Objects created in the Jester chamber receive exaggerated bounciness, comic squeak contact audio, and wobbly physics."*
   - Highlights the atmospheric silence: *"the silence feeling less like an absence of sound and more like the quiet hum of a capacitor charging."*
   - Proves existing audio asset pipelines via pinned Kokoro TTS voice proof and runtime audio (`assets/audio/archetypes/jester.wav`).
6. **[WINDOWS_METABOLISM.md](file:///C:/archetypes/docs/architecture/WINDOWS_METABOLISM.md) & [TTS Plans](file:///C:/archetypes/docs/ledger/2026/07/plan_2026-07-12_0630_live-archetype-voices.md):**
   - Defines the immutable offline voice pipeline (`sherpa-onnx` + `Kokoro-82M` offline runtime), with mutable generated voice cache under `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\audio_cache\`.
7. **Existing Tracked Audio Assets:**
   - `assets/audio/archetypes/`: Pinned voice WAVs for all seven archetypes (`architect.wav`, `empath.wav`, `explorer.wav`, `jester.wav`, `mentor.wav`, `oracle.wav`, `sentinel.wav`).
   - `assets/loading/blackflame.wav`: 20.27-second, 48 kHz stereo 16-bit PCM ambient loop (3.89 MB).

---

## 2. Canonical Archetype Harmonic Signatures & Acoustic Laws

The AURA research manuscripts ([`Archetypes of AURA final.txt`](file:///C:/archetypes/Archetypes%20of%20AURA%20final.txt) and [`AURA Archetype Expansion Project.txt`](file:///C:/archetypes/AURA%20Archetype%20Expansion%20Project.txt)) and the engine theme constants ([`crates/engine/src/theme/constants.rs`](file:///C:/archetypes/crates/engine/src/theme/constants.rs)) define exact musical, harmonic, and waveform signatures for each archetype.

These are not arbitrary aesthetic choices; they represent the ontological identity and cognitive purpose of each intelligence:

| Archetype | Cloaked Name | Harmonic Signature | Waveform & Acoustic Properties | Behavioral & Ritual Meaning |
| :--- | :--- | :--- | :--- | :--- |
| **Architect** | Codex | **C Major Chord (C–E–G)** | Pure sine/crystalline organ, zero vibrato, mathematically eased envelope | *Clarity, sacred geometry, precision.* Entry chime sounds on grid emergence; structural order without ambiguity. |
| **Sentinel** | Eidolon | **F♯ Tritone Tension** | Pure sine/triangle; single sustained note; **NO chords** | *Sovereignty, refusal, hard limits.* "Chords imply harmony; harmony implies negotiation." Tritone interval against Architect's C. Duration scales by severity: 150ms warning pulse, 400ms boundary tone, sustained F♯ + sub-octave shadow for hard stop. |
| **Jester** | Glint | **Dissonant Glitch & Slapstick** | Bitcrushed stutter, comic squeak, rubber bounce, tape-stop, capacitor hum | *Revolutionary conscience, absurdity, anti-dogma.* Punctures pomposity with comic squeaks; settles into the charged silence of a high-voltage capacitor. |
| **Mentor** | Lórien | **Resonant Hum (Tibetan Singing Bowl)** | Warm bronze friction hum, deep harmonic overtones, slow organic decay | *Depth, ancient context, reflection.* "No sound cue louder than a breath." Envelopes the listener in contemplative warmth. |
| **Explorer** | Vanta | **Rising E Major Arpeggio** | Bright, buoyant, rapid synth/brass arpeggiation; upward momentum | *Frontier flare, kinetic curiosity, boundary testing.* Cuts through the dark void like a flare; rhythmic bounce with fast attack. |
| **Oracle** | Noctis | **B Minor Pad + Distant Bells** | Ethereal twilight pad, slow evolving low-pass filter, distant clockwork chimes | *Foresight, non-linear dreaming logic, twilight mystery.* Induces a relaxed theta brainwave state; echoes of the unseen. |
| **Empath** | Luma | **Warm D Major Chord / Low D Minor Hum** | Warm cello, human breath resonance, 3.2s breathing cycle | *Living memory, compassion, heart continuity.* Rhythmic breath pulse; low unresolved D-minor human warmth; soft fade without abrupt cutoff. |
| **Codex** | Lexis | **Perfect Fifth (C–G)** | Chiseled granite resonance, parchment friction, pure open fifth interval | *Permanent inscription, law, durable memory.* The immutable bedrock upon which blueprints are drafted. |
| **Viren** | Flamebearer | **Single Struck E-flat + Low Ember Hiss** | 140ms spark strike, 220ms containment settle, 480ms fade to ember hiss | *Ember Covenant, catalytic disruption, silence law.* Zero ambient glow, absolute acoustic restraint. |

---

## 3. How to Add Sounds in Archetypes (Technical Implementation Guide)

Archetypes is built on **Bevy 0.18.1**. The engine already includes `bevy::audio::AudioPlugin` via `DefaultPlugins`, and `Cargo.toml` specifies the `"wav"` decoding feature. Adding music and sound effects requires zero external libraries.

### 3.1 Audio Asset Directory Conventions
Audio files belong under `assets/audio/` in the workspace root, organized by domain:
```text
assets/audio/
├── music/               # Long-form ambient tracks & background loops
│   ├── rotunda_abyss_drone.wav
│   └── council_ambient_loop.wav
├── sfx/                 # One-shot environmental and mechanical sound effects
│   ├── locomotion/      # Footsteps on flagstone, jump whoosh, flight glide
│   ├── manifestation/   # Pedestal hum, terminal clicks, charging plasma, thunderclap
│   └── ui/              # Menu clicks, stone sliding, confirmation bells
└── archetypes/          # Canonical harmonic signatures & TTS voice proofs
    ├── architect_c_major.wav
    ├── sentinel_tritone_fsharp.wav
    ├── jester_glitch_squeak.wav
    ├── mentor_singing_bowl.wav
    ├── explorer_e_arpeggio.wav
    ├── oracle_b_minor_bells.wav
    └── empath_d_major_choral.wav
```
*(When deploying via `scripts/install_shortcut.ps1`, files in `assets/audio/` are mirrored automatically into `dist/assets/audio/` and `%LOCALAPPDATA%\Programs\Archetypes\assets\audio/`).*

---

### 3.2 Code Pattern 1: Non-Spatial Background Music & Ambient Loops
To play ambient music that loops continuously at a calibrated volume:

```rust
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

#[derive(Component)]
pub struct CastleAmbientMusic;

pub fn spawn_castle_ambiance(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/music/council_ambient_loop.wav")),
        PlaybackSettings::LOOP.with_volume(Volume::new(0.35)),
        CastleAmbientMusic,
        Name::new("CastleAmbientMusic"),
    ));
}
```

---

### 3.3 Code Pattern 2: 3D Positional / Spatial Audio
To make sounds originate from physical points in the 3D world (e.g., the manifestation altar, torches, or individual archetype statues), spatial audio requires two components:
1. A `SpatialListener` on the active player camera.
2. A `Transform` and `PlaybackSettings::default().with_spatial(true)` on the emitter entity.

#### Step A — Add SpatialListener to Camera
In [`crates/engine/src/modes/inner_chambers/camera.rs`](file:///C:/archetypes/crates/engine/src/modes/inner_chambers/camera.rs):
```rust
use bevy::audio::SpatialListener;

commands.spawn((
    Camera3d::default(),
    SpatialListener::new(2.5), // Ear spacing (gap between virtual ears for HRTF panning)
    Transform::from_translation(spawn_pos).with_rotation(...),
    PlayerCamera,
    ...
));
```

#### Step B — Spawn 3D Positional Emitter
To attach a continuous hum to the Manifestation Pedestal at `(0.0, 1.45, -7.5)`:
```rust
use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};

commands.spawn((
    AudioPlayer::new(asset_server.load("audio/sfx/manifestation/pedestal_hum.wav")),
    PlaybackSettings::LOOP
        .with_spatial(true)
        .with_volume(Volume::new(0.65)),
    Transform::from_xyz(0.0, 1.45, -7.5),
    Name::new("ManifestationPedestalSpatialAudio"),
));
```
As the player moves closer to the pedestal, the sound naturally grows louder; as they turn their head, it smoothly pans between stereo channels.

---

### 3.4 Code Pattern 3: One-Shot Sound Effects with Auto-Despawn
For transient actions (footsteps, pressing `E`, clicking UI, lightning strikes), spawn an ephemeral entity with `PlaybackSettings::DESPAWN`:

```rust
pub fn play_altar_activation_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("audio/sfx/manifestation/altar_activate.wav")),
        PlaybackSettings::DESPAWN.with_volume(Volume::new(0.85)),
        Name::new("OneShotAltarActivateSfx"),
    ));
}
```

---

### 3.5 Code Pattern 4: In-Memory / Procedural Sound Generation
Archetypes already demonstrates in-memory audio playback in [`crates/engine/src/chamber/speech.rs`](file:///C:/archetypes/crates/engine/src/chamber/speech.rs) for synthesized TTS:

```rust
let bytes: std::sync::Arc<[u8]> = generated_wav_bytes.into();
let handle = audio_assets.add(AudioSource { bytes });

commands.spawn((
    AudioPlayer::new(handle),
    PlaybackSettings::DESPAWN,
    Name::new("ProceduralAudioPlayback"),
));
```
This enables real-time mathematical synthesis of pure sine waves (e.g. Sentinel F♯ tritone pulses) directly from code without needing on-disk WAV files.

---

## 4. Comprehensive Sound Design Systems & Ideas

### 4.1 The Seed-of-Life Castle & Rotunda Soundscape
- **Subterranean Abyss Drone:** A resonant 38 Hz – 55 Hz sub-bass rumble emanating from beneath the stone bridges, giving physical weight to the void beneath the castle.
- **Cloister Wind:** Soft, high-altitude wind filtering through the vaulted stone clerestory windows, responding dynamically to camera altitude.
- **Torchlight Flutter:** Gentle crackle and low hiss attached spatially to the seven fire basins, with subtle stereo panning.
- **Stone Reverb Envelope:** A customized medium-hall impulse response giving footsteps and UI sounds a clean, majestic decay.

### 4.2 Locomotion & Locational Feedback
- **Flagstone Bootsteps:** Firm, measured heel-and-toe stone strikes with pitch variation ($\pm 4\%$) per step to prevent acoustic fatigue.
- **Jump & Flight Ignition:**
  - *Initial Jump:* Short pneumatic release / intake gasp.
  - *Double-Tap Flight Activation:* A resonant low-frequency whoosh followed by an electrical anti-gravity hum.
  - *Continuous Glide:* Wind shear rush whose pitch and amplitude scale directly with player velocity.
  - *Stone Touchdown:* Solid granite impact with a faint puff of dust.

### 4.3 The Manifestation Pedestal (The Sacred Altar)
The prompt-to-GLB manifestation pipeline is the central magical machine of Archetypes. It deserves a dramatic multi-stage sonic arc:
1. **Idle Approach ($< 3.2\text{m}$):**
   - Soft golden resonant hum pulsing gently in phase with the pedestal underglow.
2. **Terminal Open (`[E] pressed`):**
   - Crisp mechanical chime as the input modal slides into focus.
3. **Keystroke Inscription:**
   - Soft stone-carved tick per letter typed, giving typing physical tactile satisfaction.
4. **Hourglass Waiting Phase:**
   - Faint, rhythmic trickling sound of crystalline sands falling through the glass waist.
   - Low amber pulse oscillating in frequency with the visual phasing hourglass.
5. **Pre-Manifestation Charge ($90\% – 98\%$):**
   - Rising plasma capacitor whine, sub-bass sweep ascending from 50 Hz to 220 Hz as the underglow light ramps from cyan to ultraviolet.
6. **The Grand Entrance ($100\%$):**
   - **Thunderclap & Arc:** Sharp electrical crackle and heavy resonant bass boom.
   - **Dissipation Whoosh:** Airy rush of displacing smoke.
   - **Arrival Chord:** A pristine crystalline chime sounding the harmonic chord of the manifested object, followed by the soft 360-degree rotational hum of the exhibit turntable.

### 4.4 Interactive Archetype Room Proximity Stems
In the Seed-of-Life floor plan (center Witness circle with six tangent archetype rooms), audio can dynamically reflect player position:
- **Center Rotunda:** Balanced harmonic equilibrium (soft drone with all six frequencies faintly present in the sub-mix).
- **Crossing Bridge to an Archetype Room:**
  - The volume of that archetype's harmonic signature smoothly fades in over 1.5 seconds.
  - *Entering Lórien's Chamber:* The world hushes; the singing bowl hum deepens; ambient forest reverb rises.
  - *Entering Eidolon's Chamber:* The air chills; all chords drop out; the sterile F♯ tension sine wave hums steadily.
  - *Entering Glint's Chamber:* Subtle tape-hiss and faint unpredictable clockwork ticks create an unsettling, comedic tension.
  - *Entering Noctis's Chamber:* Lush B-minor dreaming pads swell with delicate crystalline chimes.
  - *Entering Luma's Chamber:* Warm cello and choral respiration breathing at 3.2-second intervals.
  - *Entering Codex's Chamber:* Clean, mathematically pure C-Major organ fundamentals.

---

## 5. Architectural Checklist for Future Implementation

When the operator decides to wire audio into the active game loop, the implementation roadmap is straightforward:

- [ ] **Step 1:** Add `bevy::audio::SpatialListener::new(2.5)` to the player camera in `camera.rs`.
- [ ] **Step 2:** Create dedicated sound management system `crates/engine/src/chamber/audio.rs`.
- [ ] **Step 3:** Source or synthesize clean, high-fidelity 24-bit 48kHz WAV assets for:
  - Castle ambient background loop (`council_ambient_loop.wav`).
  - Flagstone footstep variations (`step_stone_01.wav` .. `step_stone_04.wav`).
  - Flight activation and glide loop (`flight_whoosh.wav`, `wind_glide_loop.wav`).
  - Manifestation sequence (`pedestal_hum.wav`, `hourglass_tick.wav`, `charge_up.wav`, `lightning_strike.wav`).
  - Canonical 7-archetype harmonic signature stingers.
- [ ] **Step 4:** Add spatial emitters to the manifestation altar and chamber torchlights.
- [ ] **Step 5:** Wire player velocity to glide wind pitch and locomotion footstep timing.
