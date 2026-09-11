# Archetypes Gamepad & Settings Engineering Guide

> **Operator & Agent Reference.** This guide provides the complete engineering manual for wiring a USB gamepad (specifically the Microsoft Xbox One S Controller) into the Archetypes engine, configuring full analog locomotion and camera controls, utilizing the extensible action-binding framework, and implementing the advanced Settings Menu with independent music volume mixing and persistent "Apply-on-Close" storage.

*For the complete architectural design and mathematical formulas, see [docs/architecture/GAMEPAD_AND_SETTINGS_SPECIFICATION.md](file:///C:/archetypes/docs/architecture/GAMEPAD_AND_SETTINGS_SPECIFICATION.md).*

---

## 1. Executive Summary

This guide provides the complete, turn-key technical instructions for:
1. **USB Hardware Layer:** Connecting and recognizing an Xbox One S controller via Windows XInput through Bevy 0.18.
2. **Current Mechanics Mapping:** Full gamepad control over walking, jumping, double-tap flight, camera look, altar manifestation, and menu navigation.
3. **Extensible Action Bindings:** A data-driven system (`GameAction` & `ActionBindings`) allowing new controls, features, and buttons to be mapped without modifying gameplay code.
4. **Hierarchical Audio Mixing:** Independent volume sliders for Master Volume, Ambient Orchestral Music, Sound Effects (SFX), and Council Voices.
5. **Camera & Motion Tuning:** Analog deadzone filtration, power curve smoothing, view motion sensitivity sliders, and Invert-Y options.
6. **Apply on Close Persistence:** Automatic atomic saving to `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json` when the player closes the Settings menu.

---

## 2. Hardware Connection & Bevy 0.18 Driver Protocol

### 2.1 The Windows USB Substrate
The **Xbox One S Controller** connects to the Windows host via standard USB (Micro-USB or USB-C cable). Windows 10 and 11 provide native kernel-level driver support via `xinput1_4.dll` (Vendor ID `0x045E`, Product ID `0x02EA` or `0x0B12`).

### 2.2 How Bevy 0.18 Sees the Gamepad
In Bevy 0.15+, connected gamepads are represented as **Entities**:
- Connected controllers are queried via `Query<(Entity, &Gamepad)>`.
- Analog stick axes return `Option<f32>` values ranging from `-1.0` to `+1.0`.
- Analog triggers (`LT` / `RT`) return pressure values ranging from `0.0` (unpressed) to `1.0` (fully depressed).
- Digital buttons (`A, B, X, Y, LB, RB`, D-Pad, Menu, View) provide `gamepad.just_pressed(...)`, `gamepad.pressed(...)`, and `gamepad.just_released(...)`.

```rust
use bevy::prelude::*;

fn read_xbox_controller(gamepads: Query<(Entity, &Gamepad)>) {
    for (entity, gamepad) in &gamepads {
        // Left stick movement
        let stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

        // Buttons
        if gamepad.just_pressed(GamepadButton::South) {
            println!("[Gamepad] (A) Button Pressed -> Jump / Confirm");
        }
        if gamepad.just_pressed(GamepadButton::West) {
            println!("[Gamepad] (X) Button Pressed -> Interact / Manifest Altar");
        }
    }
}
```

---

## 3. Xbox One S Complete Controls Reference Table

| Controller Input | Bevy Gamepad Identifier | In-Game Walking Function | In-Game Flying Function | Menu / Settings Function |
| :--- | :--- | :--- | :--- | :--- |
| **Left Stick** | `LeftStickX / LeftStickY` | Analog Walk & Strafe | Horizontal Flight Direction | Navigate Sliders / Values |
| **Right Stick** | `RightStickX / RightStickY` | Camera Look (Pitch & Yaw) | Camera Look (Pitch & Yaw) | — |
| **(A) Button** | `GamepadButton::South` | **Jump** (Double-Tap: Fly) | Vertical Ascent Burst | **Confirm / Toggle Option** |
| **(B) Button** | `GamepadButton::East` | Crouch / Cancel | Cancel Flight (Triple-Tap) | **Apply on Close / Back** |
| **(X) Button** | `GamepadButton::West` | **Interact / Manifest Pedestal** | Interact with Pedestal | Reset Current Slider to Default |
| **(Y) Button** | `GamepadButton::North` | Open Lore Codex | Hover in Place | Discard Staged Changes |
| **Left Trigger (LT)** | `GamepadButton::LeftTrigger2` | Walk Slowly / Sneak | **Descend Vertically** | Jump to Previous Category |
| **Right Trigger (RT)** | `GamepadButton::RightTrigger2` | Analog Sprint Boost | **Ascend Vertically** | Jump to Next Category |
| **Left Bumper (LB)** | `GamepadButton::LeftTrigger` | Cycle Archetype Left | Level Flight Horizon | Step Slider Value -5% |
| **Right Bumper (RB)** | `GamepadButton::RightTrigger` | Cycle Archetype Right | Fast Flight Speed Boost | Step Slider Value +5% |
| **Left Stick Click (L3)**| `GamepadButton::LeftThumb` | Toggle Sprint | Cruise Control Toggle | — |
| **Right Stick Click (R3)**| `GamepadButton::RightThumb` | Center Camera Horizon | Level Roll and Pitch | Reset All Settings to Default |
| **D-Pad Up / Down** | `GamepadButton::DPadUp / Down` | Dialogue Cycle | Altitude Trim | **Navigate Menu Items Up/Down** |
| **D-Pad Left / Right**| `GamepadButton::DPadLeft / Right`| Cycle Altar Modes | Cycle Altar Modes | **Adjust Active Slider Value** |
| **Menu / Start** | `GamepadButton::Start` | **Open / Close Settings Menu** | **Open / Close Settings Menu** | **Apply on Close** |
| **View / Back** | `GamepadButton::Select` | Toggle HUD Clean View | Toggle Flight Speedometer | Discard Unsaved Changes |

---

## 4. Extensible Control Mapping Architecture

To allow future agents to bind new features, spells, or tools without hardcoding buttons into gameplay systems, Archetypes uses an **Action Binding Table**:

### 4.1 Define the Semantic Action
In `crates/engine/src/services/input.rs`:
```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameAction {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    Jump,
    Sprint,
    Ascend,
    Descend,
    Interact,
    OpenMenu,
    // Add future actions here:
    CastManifestationSpell,
    CycleCameraPerspective,
}
```

### 4.2 Map to Controller & Keyboard
```rust
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ActionBindings {
    pub keyboard: HashMap<GameAction, KeyCode>,
    pub gamepad_buttons: HashMap<GameAction, GamepadButton>,
    pub deadzone: f32,
    pub look_sensitivity: f32,
    pub invert_y: bool,
}
```

### 4.3 Querying Actions in Gameplay Systems
Inside [`camera.rs`](file:///C:/archetypes/crates/engine/src/modes/inner_chambers/camera.rs) or [`manifestation.rs`](file:///C:/archetypes/crates/engine/src/modes/inner_chambers/manifestation.rs):
```rust
if action_state.just_pressed(GameAction::Interact) {
    // Begins manifestation prompt or talks to council figure!
}
```

---

## 5. Audio Mixing Hierarchy: Independent Music & Master Volume

The player must be able to control **ambient music volume independently of system master volume**.

### 5.1 Volume Calculation Formula
The final volume heard by the player is the multiplicative product of the global master volume and the channel-specific volume:

$$V_{\text{final}} = V_{\text{master}} \times V_{\text{channel}}$$

- If **Master Volume** is at $80\%$ and **Music Volume** is at $50\%$, effective music volume is $0.80 \times 0.50 = 40\%$.
- If the player sets **Music Volume** to $0\%$, background music is completely silent, while footsteps, manifestation thunderclaps, and council voices remain at $80\%$!

### 5.2 Audio Channel Components
Every audio player spawned in the game receives an audio bus marker:
```rust
#[derive(Component)] pub struct MusicBus;
#[derive(Component)] pub struct SfxBus;
#[derive(Component)] pub struct VoiceBus;
```
A lightweight system updates all playing sounds when the player adjusts a slider in the Settings menu.

---

## 6. Advanced In-Game Settings Menu

### 6.1 Menu Interface Design
The Settings menu appears as an ancient-technological terminal overlaid on the paused Council Chamber:
- **Audio Controls:**
  - `Master Volume` slider (0% to 100%)
  - `Ambient Music Volume` slider (0% to 100%)
  - `Sound Effects Volume` slider (0% to 100%)
  - `Council Voice Volume` slider (0% to 100%)
- **Camera & Motion:**
  - `View Motion Sensitivity` slider (0.1x to 3.0x, default 1.25x)
  - `Invert Y Axis` checkbox (Off / On)
  - `Stick Deadzone` slider (5% to 25%, default 12%)
- **Gamepad Navigation:**
  - `D-Pad Up / Down`: Moves highlight between settings rows.
  - `D-Pad Left / Right`: Adjusts slider value by 5% increments.
  - `(A) Button`: Toggles checkboxes or confirms action.
  - `(X) Button`: Resets the active setting to default.
  - `(B) Button` or `Menu`: **Applies changes and closes menu.**

---

## 7. "Apply on Close" Persistence Architecture (Windows Metabolism)

In strict accordance with the **[Charter of Cognitive Sovereignty](file:///C:/archetypes/AGENTS.md)** and **[WINDOWS_METABOLISM.md](file:///C:/archetypes/docs/architecture/WINDOWS_METABOLISM.md)**:
- Immutable application binaries remain in `C:\Program Files\Archetypes\`.
- **All mutable user configuration is stored under:**
  `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json`

### 7.1 Staging and Committing Workflow
1. **Sandboxed Staging:** When the settings menu opens, a clone of active settings (`StagedSettings`) is created. As the player moves sliders, live audio previews update immediately so the player can hear the volume change.
2. **Apply on Close:** When the player presses `(B)` or `Menu` (or clicks "Apply & Close"):
   - `StagedSettings` is copied into the authoritative `PlayerSettings` resource.
   - An atomic write serializes the configuration to `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json`.
3. **Discard on Cancel:** If the player exits with `(Y)` ("Discard"), `StagedSettings` is dropped and audio volumes revert to their previous levels.

---

## 8. Implementation Checklist for Agents

When implementing this specification in the code:
- [ ] **1. Settings Service:** Create `crates/engine/src/services/settings.rs` for `PersistentConfig` JSON serialization and disk I/O.
- [ ] **2. Input Manager:** Create `crates/engine/src/services/input.rs` implementing `ActionBindings`, `GameAction`, and radial deadzone calculations for Xbox One S controllers.
- [ ] **3. Camera Integration:** Update `crates/engine/src/modes/inner_chambers/camera.rs` to read analog movement and right-stick look deltas from `ActionState` scaled by `look_sensitivity` and `invert_y`.
- [ ] **4. Settings UI:** Create `crates/engine/src/modes/inner_chambers/settings_ui.rs` rendering the Bevy UI sliders with D-Pad navigation and "Apply on Close" handling.
- [ ] **5. Audio Synchronization:** Wire the settings audio buses to Bevy's `GlobalVolume` and active `MusicBus` streams.
- [ ] **6. Validation:** Run `cargo test --workspace` and test directly from the pinned Windows Taskbar launcher with the USB Xbox One S controller connected.
