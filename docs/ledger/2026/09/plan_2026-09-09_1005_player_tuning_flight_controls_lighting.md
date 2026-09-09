# Plan: Player Controls Tuning, Instant Flight, Cursor Hiding & Room Lighting — 2026-09-09 10:05

## Status
COMPLETED

## Goal
Tweak Inner Chambers player experience and physics according to operator feedback:
1. Hide the system mouse cursor during 3D navigation and lock it into the window.
2. Reduce mouse-look sensitivity and head movement to eliminate twitchiness.
3. Increase player eye height (from 1.75m to ~2.75m-2.85m) so the camera stands above the table surface and looks slightly downward at the artifacts in natural proportion.
4. Remove the 2.0-second delay in flight activation: double-tap Space instantly enables flight; triple-tap Space immediately restores gravity and lands.
5. In flight mode: holding Space moves upward, holding Shift descends towards floor level.
6. Clean up in-game HUD/console text: eliminate verbose charging/timer displays.
7. Substantially elevate overall ceiling and wall illumination so the grand scale and architecture of the 76m rotunda is clearly visible.
Note: Per operator directive, DO NOT build new release packages yet.

## Steps

### Step 1 — Window Cursor Hiding & Locking
- [x] Action: In `crates/engine/src/modes/inner_chambers/camera.rs`:
  - Query `CursorOptions` on `PrimaryWindow` on entering `InnerChambersState::Navigating`.
  - Set `cursor.visible = false` and `cursor.grab_mode = CursorGrabMode::Locked`.
  - On teardown/exit, restore `cursor.visible = true` and `cursor.grab_mode = CursorGrabMode::None`.
  - Re-lock and hide cursor if clicked inside the window.
- Files touched: `crates/engine/src/modes/inner_chambers/camera.rs`
- Expected outcome: No mouse arrow/icon visible in screen during gameplay.

### Step 2 — Player Height, Smooth Head Movement & Instant Flight Controls
- [x] Action: In `crates/engine/src/modes/inner_chambers/camera.rs`:
  - Elevate default `eye_height` to $2.85\text{m}$ (with initial pitch $\approx -0.18\text{ rad} \approx -10.3^\circ$).
  - Lower sensitivity to $0.00045$ and clamp mouse deltas to provide calm, smooth head rotation.
  - Simplify flight state machine:
    - Double-tap Space (2 presses within $0.45\text{s}$) instantly engages `LocomotionMode::Flying` (no hold delay).
    - In flight: holding Space ascends (+Y), holding Shift descends (-Y) down to floor level (`ground_y + eye_height`).
    - Triple-tap Space (3 presses within $0.85\text{s}$) instantly returns to `LocomotionMode::Walking` and gravity.
  - Streamline HUD text to remove timers and progress bars.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/camera.rs`
  - `crates/engine/src/modes/inner_chambers/world.rs`
- Expected outcome: Natural height looking down at table and pedestals; instant, intuitive double-tap flight and triple-tap landing; smooth mouse look.

### Step 3 — Architectural Room Scale & Wall/Ceiling Illumination
- [x] Action: In `crates/engine/src/modes/inner_chambers/world.rs` and `crates/engine/src/chamber/interior.rs`:
  - Elevate wall, floor, dais, and ceiling material tones so stone surfaces catch illumination rather than absorbing all light.
  - Add ambient lighting to the scene so corners and rafters are not pitch black (elevate `ambient.brightness` to 480 during Inner Chambers).
  - Increase the range and intensity of high vault fill lights (450,000, 95m range), central table light (240,000, 50m range), sun shaft (15,000 lx), and add 12 upper wall-wash lights to showcase the 22m-high ceiling, rafters, and 76m wide perimeter walls.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/chamber/interior.rs`
- Expected outcome: Full room scale, massive rafters, and outer walls are clearly visible and aesthetically stunning.

### Step 4 — Verification Gate
- [x] Action: Run `cargo test --workspace` to ensure all tests compile and pass. Do NOT run installer/shortcut builds per operator request.
- Expected outcome: Clean test pass with zero errors (all 100 tests passed).

