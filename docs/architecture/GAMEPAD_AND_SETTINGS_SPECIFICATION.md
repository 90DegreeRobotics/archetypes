# Gamepad Wiring, Extensible Controls & Advanced Settings Specification

> **Canonical System Specification.** This document provides the comprehensive architectural blueprint for wiring USB gamepads—specifically the Microsoft Xbox One S Controller—into Archetypes, integrating an extensible action-binding framework, and delivering an advanced in-game Settings UI featuring independent audio volume mixing, look sensitivity tuning, and persistent "Apply-on-Close" storage adhering to Windows Metabolism.

---

## 1. Executive Vision: Tactile Sovereign Immersion

Until now, the player has interacted with the Council Chamber, flight locomotion, and manifestation mechanics through mouse and keyboard inputs. While precise for desktop development, mouse and keyboard create a cognitive barrier: digital on/off keypresses (`W, A, S, D`) lack the subtle spatial nuances of analog locomotion, and mouse looking can feel sharp or twitchy in a sacred architectural space.

Transitioning to an **Xbox One S Controller over USB** transforms the player experience:
1. **Analog Locomotion:** The player smoothly transitions from a contemplative stroll past the Council figures to a brisk sprint across the stone bridges based on subtle thumbstick deflection.
2. **Cinematic Camera Control:** Right thumbstick angular velocity enables sweeping, majestic camera pans around the high clerestory vaulted arches without sudden jerkiness.
3. **Ergonomic Flight & Manifestation:** Intuitive trigger-based ascending and descending, haptic and tactile button prompts, and effortless navigation between 3D exploration and menu interaction.
4. **Player Sovereignty Through Settings:** Full control over master volume, background orchestral music volume, motion sensitivity, and control remapping, persistently saved across sessions under `%LOCALAPPDATA%`.

---

## 2. Hardware & Driver Substrate (Windows, XInput & GilRs)

### 2.1 The USB Host Environment
The Microsoft Xbox One S Controller connects to the Windows host via standard USB (USB Type-A to Micro-USB / USB-C). On Windows 10 and 11, the controller is recognized natively by the operating system:
- **Device Protocol:** Standard XInput (Windows Gaming Input / HID).
- **USB Vendor ID (VID):** `0x045E` (Microsoft Corporation).
- **USB Product ID (PID):** `0x02EA` (Xbox One S Controller) or `0x0B12` (Xbox Wireless/Series Controller wired over USB).
- **Driver:** Handled automatically by the built-in Windows kernel driver (`xinput1_4.dll` / `windows.gaming.input.dll`). No third-party drivers or sidecar software are required.

### 2.2 Bevy 0.18 Engine Integration
Archetypes runs on **Bevy 0.18.1**. Under the hood, Bevy's `DefaultPlugins` includes `bevy_gilrs`, which uses the `gilrs` (Game Input Library for Rust) crate. On Windows, `gilrs` binds directly to XInput.

In Bevy 0.15+, gamepad handling was modernized into an **Entity-Component Architecture**:
- Connected controllers are no longer accessed through a global resource; **each connected gamepad is an individual Entity**.
- When an Xbox controller is plugged into USB, Bevy automatically spawns an Entity with the `Gamepad` component.
- If the controller is disconnected, the entity is despawned or marked inactive, providing seamless hot-plugging support.

```rust
use bevy::prelude::*;

fn gamepad_connection_monitor(
    mut connection_events: EventReader<GamepadConnectionEvent>,
) {
    for event in connection_events.read() {
        match &event.connection {
            GamepadConnection::Connected(info) => {
                println!("[Gamepad] Connected controller: {:?} (ID: {:?})", info.name, event.gamepad);
            }
            GamepadConnection::Disconnected => {
                println!("[Gamepad] Controller disconnected: {:?}", event.gamepad);
            }
        }
    }
}
```

---

## 3. Xbox One S Surface & Canonical Control Mapping

The Xbox One S controller provides two 2-axis analog thumbsticks, an 8-way directional pad, four primary action buttons, two digital shoulder bumpers, two analog triggers, and two central navigation buttons.

```text
               [ LB ] Left Bumper             [ RB ] Right Bumper
              [ LT ] Left Trigger           [ RT ] Right Trigger
                     (Analog)                      (Analog)
                    
         ( D-Pad )                 [ View ]   [ Menu ]                 ( Y )
            [▲]                                                         ▲
         [◄]   [►]                 [ Xbox / Guide ]              ( X ) ◄  ► ( B )
            [▼]                                                         ▼
                                                                       ( A )
               ( L3 ) Left Stick                        ( R3 ) Right Stick
               - Movement / Strafe                      - Pitch / Yaw Camera Look
               - Click: Sprint                          - Click: Center Camera / Reset
```

### 3.1 Dual-Mode Control Matrix

The Archetypes engine supports two distinct locomotion paradigms: **Ground Walking** and **Free Flight**. The controller maps naturally to both:

| Hardware Input | Bevy Gamepad Identifier | Ground Walking Mode | Free Flight Mode | In-Menu / Settings Mode |
| :--- | :--- | :--- | :--- | :--- |
| **Left Stick (X/Y)** | `GamepadAxis::LeftStickX/Y` | Analog Walk & Strafe | Horizontal Flight Movement | Navigate Sliders / Options |
| **Right Stick (X/Y)** | `GamepadAxis::RightStickX/Y` | Camera Look (Pitch & Yaw) | Camera Look (Pitch & Yaw) | — |
| **A Button** | `GamepadButton::South` | **Jump** (Double-tap: Engage Flight) | Ascent Burst | **Confirm / Select** |
| **B Button** | `GamepadButton::East` | Crouch / Cancel | Disengage Flight (Triple-tap) | **Cancel / Back / Close** |
| **X Button** | `GamepadButton::West` | **Interact / Manifest Pedestal** | Interact | Reset Slider to Default |
| **Y Button** | `GamepadButton::North` | Open Lore Codex / Inventory | Toggle Flight Hover | Quick-Save Options |
| **Left Trigger (LT)** | `GamepadButton::LeftTrigger2` | Walk Slowly (Stealth/Aim) | **Descend Vertically** | Jump to Previous Category |
| **Right Trigger (RT)** | `GamepadButton::RightTrigger2` | Sprint (Analog speed boost) | **Ascend Vertically** | Jump to Next Category |
| **Left Bumper (LB)** | `GamepadButton::LeftTrigger` | Cycle Council Archetype Left | Level Flight Pitch Horizon | Step Slider Left (-5%) |
| **Right Bumper (RB)** | `GamepadButton::RightTrigger` | Cycle Council Archetype Right | Fast Flight Afterburner | Step Slider Right (+5%) |
| **Left Stick Click (L3)**| `GamepadButton::LeftThumb` | Toggle Sprint | Toggle Flight Cruise Control | — |
| **Right Stick Click (R3)**| `GamepadButton::RightThumb` | Reset Camera to Horizon | Reset Flight Roll & Pitch | Reset All to Defaults |
| **D-Pad Up/Down** | `GamepadButton::DPadUp/Down` | Cycle Dialog / Camera Zoom | Fine Altitude Trim | **Navigate Menu Items Up/Down** |
| **D-Pad Left/Right** | `GamepadButton::DPadLeft/Right` | Cycle Altar Modes | Cycle Altar Modes | **Adjust Active Slider Value** |
| **Menu / Start** | `GamepadButton::Start` | **Open / Close Settings Menu** | **Open / Close Settings Menu** | **Apply Settings on Close** |
| **View / Back** | `GamepadButton::Select` | Toggle HUD / UI Visibility | Toggle Flight Speedometer | Discard Unsaved Changes |

---

## 4. Analog Locomotion & Smooth Camera Look Mathematics

Directly passing raw joystick values into camera orientation produces unplayable results: small physical imperfections in controller potentiometers cause **stick drift**, and linear axis response makes small aiming adjustments feel imprecise while full turns feel sluggish.

To deliver AAA-grade controller feel, the engine applies three mathematical stages: **Deadzone Calibration**, **Exponential Response Shaping**, and **Framerate-Independent Delta Integration**.

### 4.1 Radial Deadzone Filtering
Axial deadzones (checking X and Y independently) cause "stick snapping" along the cardinal axes. Instead, a true **radial deadzone** must be computed in Euclidean space:

$$r = \sqrt{x^2 + y^2}$$

If $r \le r_{\text{deadzone}}$, the input is exactly zero. Beyond the deadzone threshold, the input is normalized from $0.0$ to $1.0$:

$$\text{deflection} = \frac{r - r_{\text{deadzone}}}{1.0 - r_{\text{deadzone}}}$$

$$\hat{x} = \frac{x}{r} \cdot \text{deflection}, \quad \hat{y} = \frac{y}{r} \cdot \text{deflection}$$

*Standard calibrated deadzone for Xbox One S controllers is $r_{\text{deadzone}} = 0.12$ ($12\%$).*

### 4.2 Exponential Response Curve (The "Dual-Rate" Power Curve)
Linear response forces the player to choose between slow turning or twitchy micro-adjustments. A power response curve ($p \approx 2.2$) provides fine-grained precision around the center (essential for aligning with the manifestation pedestal or reading inscriptions) while preserving maximum turning velocity when the stick is pushed to its rim:

$$f(v) = \text{sign}(v) \cdot |v|^{2.2}$$

### 4.3 Pitch Clamping and Invert-Y Support
Vertical camera orientation (pitch) must be clamped to prevent the camera from flipping upside down at zenith or nadir. Standard limits are $\pm 85^\circ$ ($\pm 1.48$ radians):

```rust
let pitch_sign = if settings.invert_y { 1.0 } else { -1.0 };
controller.pitch += filtered_stick_y * controller.sensitivity * pitch_sign * dt;
controller.pitch = controller.pitch.clamp(-1.48, 1.48);

controller.yaw -= filtered_stick_x * controller.sensitivity * dt;
```

---

## 5. Extensible Action Binding Architecture

Hardcoding `if gamepad.just_pressed(GamepadButton::South)` inside gameplay logic creates an unmaintainable codebase where adding a new ability or remapping buttons requires editing multiple files.

Instead, Archetypes uses an **Action Binding System**. Gameplay code queries semantic actions (`GameAction`), while a centralized input manager resolves whether a keyboard key, mouse button, or controller button triggered that action.

### 5.1 The Semantic Action Enum
```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameAction {
    // Locomotion
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    Jump,
    Sprint,
    
    // Flight
    Ascend,
    Descend,
    ToggleFlight,
    
    // Camera
    LookHorizontal,
    LookVertical,
    ResetCamera,
    
    // Interactions
    Interact,           // Open manifestation altar, talk to council
    Cancel,             // Back / close
    OpenMenu,           // Pause / settings
    ToggleHud,          // Clean screenshot mode
}
```

### 5.2 The Unified Action Bindings Resource
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ActionBindings {
    pub keyboard: HashMap<GameAction, KeyCode>,
    pub gamepad_buttons: HashMap<GameAction, GamepadButton>,
    pub deadzone: f32,
    pub look_sensitivity: f32,
    pub invert_y: bool,
}

impl Default for ActionBindings {
    fn default() -> Self {
        let mut kb = HashMap::new();
        kb.insert(GameAction::MoveForward, KeyCode::KeyW);
        kb.insert(GameAction::MoveBackward, KeyCode::KeyS);
        kb.insert(GameAction::StrafeLeft, KeyCode::KeyA);
        kb.insert(GameAction::StrafeRight, KeyCode::KeyD);
        kb.insert(GameAction::Jump, KeyCode::Space);
        kb.insert(GameAction::Sprint, KeyCode::ShiftLeft);
        kb.insert(GameAction::Interact, KeyCode::KeyE);
        kb.insert(GameAction::OpenMenu, KeyCode::Escape);

        let mut gp = HashMap::new();
        gp.insert(GameAction::Jump, GamepadButton::South);          // A
        gp.insert(GameAction::Cancel, GamepadButton::East);         // B
        gp.insert(GameAction::Interact, GamepadButton::West);       // X
        gp.insert(GameAction::Sprint, GamepadButton::LeftThumb);    // L3
        gp.insert(GameAction::OpenMenu, GamepadButton::Start);      // Menu / Start
        gp.insert(GameAction::Ascend, GamepadButton::RightTrigger2); // RT
        gp.insert(GameAction::Descend, GamepadButton::LeftTrigger2); // LT

        Self {
            keyboard: kb,
            gamepad_buttons: gp,
            deadzone: 0.12,
            look_sensitivity: 1.25,
            invert_y: false,
        }
    }
}
```

### 5.3 How Future Agents Add New Capabilities
When an agent introduces a new mechanic (e.g., "Shield Block", "Summon Memory", or "Zoom Lens"):
1. Add the variant to `GameAction` (e.g. `GameAction::ShieldBlock`).
2. Add the default mapping in `ActionBindings::default()` (e.g. `gp.insert(GameAction::ShieldBlock, GamepadButton::LeftTrigger)`).
3. In the feature's system, query `action_state.is_pressed(GameAction::ShieldBlock)`.
4. The action is immediately playable on both keyboard and gamepad, and automatically exposed to the Settings remapping UI.

---

## 6. Audio Mixing Hierarchy & Independent Volume Buses

A critical requirement is the ability to **control background music volume over the system master volume**. If the player lowers the music to focus on council voices, sound effects (like thunderclaps and footsteps) must remain at full punch.

### 6.1 The Four-Tier Mixing Hierarchy
```text
               ┌──────────────────────────────────────────────┐
               │    MASTER VOLUME BUS (Global Multiplier)     │
               │             (0.0 to 1.0)                     │
               └──────────────────────┬───────────────────────┘
                                      │
         ┌────────────────────────────┼────────────────────────────┐
         ▼                            ▼                            ▼
┌──────────────────┐         ┌──────────────────┐         ┌──────────────────┐
│  MUSIC BUS (35%) │         │   SFX BUS (80%)  │         │  VOICE BUS (100%)│
│  - Council Amb   │         │  - Footsteps     │         │  - Kokoro TTS    │
│  - Abyss Drone   │         │  - Lightning Arc │         │  - Council Speech│
│  - Harmonic Stems│         │  - Altar Clang   │         │  - Verdicts      │
└────────┬─────────┘         └────────┬─────────┘         └────────┬─────────┘
         │                            │                            │
         ▼                            ▼                            ▼
  [ Audio Player ]             [ Audio Player ]             [ Audio Player ]
```

### 6.2 Mathematical Audio Attenuation Law
For any playing sound entity, its final playback volume is determined by the multiplicative product of three scalar factors:

$$V_{\text{final}} = V_{\text{master}} \times V_{\text{bus}} \times V_{\text{emitter\_gain}}$$

- If $V_{\text{master}} = 0.5$ and $V_{\text{music}} = 0.4$, the effective music volume is $0.5 \times 0.4 = 0.20$ ($20\%$).
- If the player slides $V_{\text{music}}$ to $0.0$, music is completely muted while SFX and Voice continue playing at $V_{\text{master}}$ level.

### 6.3 Bevy Audio Bus Implementation
In Bevy 0.18, audio channels are managed by assigning marker components to audio entities and synchronizing them through an audio manager resource:

```rust
use bevy::audio::{AudioPlayer, GlobalVolume, PlaybackSettings, Volume};
use bevy::prelude::*;

#[derive(Component)] pub struct MusicBus;
#[derive(Component)] pub struct SfxBus;
#[derive(Component)] pub struct VoiceBus;

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master: f32, // 0.0 .. 1.0
    pub music: f32,  // 0.0 .. 1.0
    pub sfx: f32,    // 0.0 .. 1.0
    pub voice: f32,  // 0.0 .. 1.0
}

pub fn sync_audio_buses(
    settings: Res<AudioSettings>,
    mut global_volume: ResMut<GlobalVolume>,
    mut music_query: Query<&mut PlaybackSettings, With<MusicBus>>,
    mut sfx_query: Query<&mut PlaybackSettings, (With<SfxBus>, Without<MusicBus>)>,
    mut voice_query: Query<&mut PlaybackSettings, (With<VoiceBus>, Without<MusicBus>, Without<SfxBus>)>,
) {
    // 1. Update Bevy's built-in global master volume
    global_volume.volume = Volume::new(settings.master);

    // 2. Dynamically adjust active music streams
    for mut playback in &mut music_query {
        playback.volume = Volume::new(settings.music);
    }

    // 3. Dynamically adjust active SFX streams
    for mut playback in &mut sfx_query {
        playback.volume = Volume::new(settings.sfx);
    }

    // 4. Dynamically adjust active voice streams
    for mut playback in &mut voice_query {
        playback.volume = Volume::new(settings.voice);
    }
}
```

---

## 7. Advanced In-Game Settings Menu

The Settings Menu must look and feel like an authentic ancient-technological terminal—a crystalline slate illuminated with the blue-white and gold hues of the Council Chamber.

### 7.1 Visual Layout & Component Structure
```text
┌────────────────────────────────────────────────────────────────────────┐
│                        ARCHETYPES SETTINGS                             │
│                  [Press B / Esc to Apply & Close]                      │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│   AUDIO CONTROLS                                                       │
│   ├── Master Volume           [ ──────────────●───── ]  75%            │
│   ├── Ambient Music Volume    [ ────────●─────────── ]  40%            │
│   ├── Sound Effects Volume    [ ──────────────────●─ ]  90%            │
│   └── Council Voice Volume    [ ────────────────────●] 100%            │
│                                                                        │
│   CONTROLS & MOTION                                                    │
│   ├── View Motion Sensitivity [ ──────────●───────── ]  1.25x          │
│   ├── Invert Y Axis           [ [X] ENABLED          ]                 │
│   ├── Stick Inner Deadzone    [ ───●──────────────── ]  12%            │
│   └── Controller Preset       [ < XBOX ONE S (USB) > ]                 │
│                                                                        │
│   BUTTON ASSIGNMENTS (XBOX ONE S)                                      │
│   ├── Movement                Left Thumbstick (Analog)                 │
│   ├── Camera Look             Right Thumbstick (Analog)                │
│   ├── Jump / Fly              (A) Button                               │
│   ├── Interact (Altar)        (X) Button                               │
│   └── Flight Climb / Sink     Right Trigger / Left Trigger             │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│  (A) Select    (B) Apply & Close    (X) Reset Defaults    (Y) Discard  │
└────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Gamepad Navigation Logic
To ensure full control without requiring a mouse:
1. **Vertical Focus Navigation:** Pressing `D-Pad Up` or `D-Pad Down` shifts active focus between rows (`Master Volume` $\to$ `Music Volume` $\to$ `Sensitivity`, etc.).
2. **Horizontal Slider Adjustment:** Pressing `D-Pad Left` / `D-Pad Right` (or pushing the Left Thumbstick) decrements/increments the slider by $5\%$. Holding `LB` / `RB` adjusts by $1\%$.
3. **Toggle Checkboxes:** Pressing `A` on a toggle row (like `Invert Y Axis`) flips its boolean state.
4. **Immediate Audio Preview:** Adjusting the music slider updates the background music volume in real-time, allowing the player to hear the change immediately before closing the menu.

---

## 8. "Apply on Close" Persistence Pipeline (Windows Metabolism)

In compliance with **[WINDOWS_METABOLISM.md](file:///C:/archetypes/docs/architecture/WINDOWS_METABOLISM.md)** and **[INSTALLER_WIZARD_SPEC.md](file:///C:/archetypes/docs/windows/INSTALLER_WIZARD_SPEC.md)**:
- Immutable binaries live in `%ProgramFiles%\Archetypes`.
- **All mutable user configuration MUST live in `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\`**.
- Settings must survive application updates, re-installs, and launcher syncs without being wiped.

### 8.1 The Two-Tier Staged Settings Lifecycle

```text
 [ Disk: settings.json ]
          │
          ▼ (At Engine Boot)
 [ Active PlayerSettings Resource ]
          │
          ▼ (When Player opens Settings Menu)
 [ StagedSettings Resource (Sandbox Copy) ] ◄── Player adjusts sliders / sticks
          │
          ├───────────────────────────────┐
          │ (Player presses B / "Apply")  │ (Player presses Y / "Discard")
          ▼                               ▼
 [ Active PlayerSettings ]           [ Discard Staged Copy ]
          │                          [ Revert Audio / Sensitivity ]
          ▼ (Atomic Write)
 [ Disk: settings.json ]
```

1. **Boot Initialization:** The engine attempts to load `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json`. If missing, it creates the file with canonical defaults.
2. **Staging:** When the settings menu is toggled open, a clone of `PlayerSettings` is stored in a `StagedSettings` resource. All slider movements mutate `StagedSettings`.
3. **Commit on Close:** When the player presses `B` (or `Escape` or selects "Apply on Close"):
   - The staged settings are copied into `PlayerSettings`.
   - An atomic asynchronous file write serializes `settings.json` to disk.
4. **Discard on Cancel:** If the player selects "Discard" or resets defaults, `StagedSettings` is rolled back and the audio buses are restored to their previous volume.

### 8.2 Production Persistence Implementation
```rust
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistentConfig {
    pub audio: AudioSettings,
    pub sensitivity: f32,
    pub invert_y: bool,
    pub deadzone: f32,
}

impl Default for PersistentConfig {
    fn default() -> Self {
        Self {
            audio: AudioSettings {
                master: 0.85,
                music: 0.40, // Elegant background level
                sfx: 0.90,
                voice: 1.00,
            },
            sensitivity: 1.20,
            invert_y: false,
            deadzone: 0.12,
        }
    }
}

pub fn settings_file_path() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| r"C:\Users\Default\AppData\Local".to_string());
    
    PathBuf::from(local_app_data)
        .join("NeuroCognica")
        .join("Archetypes")
        .join("config")
        .join("settings.json")
}

pub fn save_settings_to_disk(config: &PersistentConfig) -> Result<(), String> {
    let path = settings_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed creating config dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("failed serializing settings: {e}"))?;
    
    // Write via atomic temporary file to prevent corruption if process exits mid-write
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, json).map_err(|e| format!("failed writing temp config: {e}"))?;
    fs::rename(&temp_path, &path).map_err(|e| format!("failed persisting config: {e}"))?;
    
    println!("[Settings] Persisted settings successfully to {}", path.display());
    Ok(())
}

pub fn load_settings_from_disk() -> PersistentConfig {
    let path = settings_file_path();
    if !path.is_file() {
        println!("[Settings] No existing settings found at {}; applying defaults.", path.display());
        let default_config = PersistentConfig::default();
        let _ = save_settings_to_disk(&default_config);
        return default_config;
    }

    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("[Settings] Corrupt settings file ({e}); falling back to safe defaults.");
            PersistentConfig::default()
        }),
        Err(e) => {
            eprintln!("[Settings] Failed reading settings file ({e}); using defaults.");
            PersistentConfig::default()
        }
    }
}
```

---

## 9. Implementation Roadmap for Future Agents

When the operator authorizes the implementation of the controller and settings system, the following modular implementation sequence should be executed:

1. **Step 1 — Settings Substrate & Persistence Service:**
   - Create `crates/engine/src/services/settings.rs` containing `PersistentConfig`, file loading/saving, and the `PlayerSettings` resource.
   - Wire initialization in `main.rs` before plugin startup.
2. **Step 2 — Input Translation & Gamepad Service:**
   - Create `crates/engine/src/services/input.rs` implementing `ActionBindings` and the radial deadzone calculation.
   - Wire a system that reads `Query<&Gamepad>` and updates an `ActionState` resource.
3. **Step 3 — Locomotion & Camera Integration:**
   - Update `crates/engine/src/modes/inner_chambers/camera.rs` to read movement axes and look angles from `ActionState`.
   - Replace direct `keyboard.pressed(KeyCode::KeyW)` with `action_state.axis(GameAction::MoveForward)`.
4. **Step 4 — In-Game Settings UI:**
   - Create `crates/engine/src/modes/inner_chambers/settings_ui.rs` using Bevy UI.
   - Implement focus navigation with `DPadUp` / `DPadDown` and slider adjustments with `DPadLeft` / `DPadRight`.
   - Bind `GamepadButton::Start` (`Menu`) and `KeyCode::Escape` to toggle the menu.
   - Implement the "Apply on Close" pipeline when `GamepadButton::East` (`B`) is pressed.
5. **Step 5 — Verification & Shortcut Deployment:**
   - Run `cargo test --workspace`.
   - Run `pwsh -File scripts\install_shortcut.ps1` to sync the updated engine to the buyer-facing Taskbar icon.
   - Plug in the Xbox One S USB controller and verify walk, flight, sensitivity sliders, and persistent audio volume saves.
