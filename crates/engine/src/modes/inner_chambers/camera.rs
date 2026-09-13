use super::InnerChambersState;
use crate::services::gamepad_input;
use crate::services::settings::GameSettings;
use crate::services::sfx::{PlaySfx, Sfx};
use bevy::ecs::message::MessageReader;
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct CameraPlugin;

use super::castle;
use super::manifestation::{MANIFESTATION_ALTAR_COLLISION_RADIUS, MANIFESTATION_PEDESTAL_POS};

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Navigating), setup_camera)
            .add_systems(
                Update,
                player_locomotion.run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(
                Update,
                request_locomotion_hint
                    .in_set(super::world::InnerHintSet::Request)
                    .run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(OnEnter(InnerChambersState::Exiting), teardown_camera);
    }
}

/// The lowest-priority hint: whatever the player is standing next to may override it.
fn request_locomotion_hint(
    mut hint: ResMut<super::world::HintRequest>,
    controller: Query<&CameraController, With<PlayerCamera>>,
) {
    if let Ok(controller) = controller.single() {
        hint.request(
            super::world::HintPriority::Locomotion,
            controller.locomotion_hud_text(),
        );
    }
}

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocomotionMode {
    Walking,
    Flying,
}

/// Whether the player is holding sprint (Shift / left-stick click).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SprintState {
    #[default]
    Normal,
    Sprinting,
}

#[derive(Component)]
pub struct CameraController {
    pub mode: LocomotionMode,
    pub sprint: SprintState,
    pub eye_height: f32,
    pub walk_speed: f32,
    /// Multiplier applied to `walk_speed` while sprinting.
    pub sprint_multiplier: f32,
    pub flight_speed: f32,
    pub velocity_y: f32,
    pub gravity: f32,
    pub jump_impulse: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub is_grounded: bool,

    // Flight engage timing (instant double-tap space)
    pub last_space_press_time: f32,
    pub space_tap_count: u32,

    // Flight disengage timing (triple tap space)
    pub flight_space_press_history: [f32; 3],
    pub flight_space_press_index: usize,

    /// Ground covered since the last footstep, in metres.
    ///
    /// Footsteps are spaced by **distance walked**, not by a timer. A timer plays the same
    /// rhythm whether the player is sprinting or edging forward, which is what makes footfalls
    /// in a lot of games sound detached from the legs underneath them.
    pub stride_accumulated: f32,
    /// Which footstep variant comes next, so a run cycles rather than repeating one sample.
    pub stride_index: usize,
}

impl CameraController {
    /// Short, context-sensitive status line for the bottom-left HUD.
    ///
    /// The full keybinding reference belongs in the settings/help overlay. This line tells
    /// the player only what mode they are in — walking, sprinting, or flying.
    pub fn locomotion_hud_text(&self) -> String {
        match self.mode {
            LocomotionMode::Walking => match self.sprint {
                SprintState::Normal => "Walking  ·  Shift: Sprint  ·  2×Space: Fly  ·  Esc: Menu".to_string(),
                SprintState::Sprinting => "Sprinting  ·  Release Shift: Walk  ·  2×Space: Fly  ·  Esc: Menu".to_string(),
            },
            LocomotionMode::Flying => {
                "Flying  ·  Space: Ascend  ·  Shift: Descend  ·  3×Space: Land  ·  Esc: Menu".to_string()
            }
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            mode: LocomotionMode::Walking,
            sprint: SprintState::Normal,
            eye_height: 2.85,     // Natural vantage standing above table and artifact pedestals
            walk_speed: 6.5,
            sprint_multiplier: 1.8,  // 6.5 × 1.8 = 11.7 m/s — brisk run, not a blur
            flight_speed: 10.5,
            velocity_y: 0.0,
            gravity: 16.0,
            jump_impulse: 5.6,
            sensitivity: 0.0013,  // Responsive, standard FPS mouse look
            pitch: -0.18,         // Looking slightly downward at the objects (~10.3 degrees)
            yaw: 0.0,
            is_grounded: true,
            last_space_press_time: -100.0,
            space_tap_count: 0,
            flight_space_press_history: [0.0; 3],
            flight_space_press_index: 0,
            stride_accumulated: 0.0,
            stride_index: 0,
        }
    }
}

fn setup_camera(
    mut commands: Commands,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut existing_cameras: Query<
        &mut Camera,
        (
            With<crate::chamber::camera::WitnessCamera>,
            Without<PlayerCamera>,
        ),
    >,
) {
    // Hide mouse cursor and lock it into the window during 3D gameplay
    if let Ok(mut cursor) = cursor_options.single_mut() {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    }

    // Disable WitnessCamera while navigating
    for mut cam in &mut existing_cameras {
        cam.is_active = false;
    }

    // Start on the Council circle facing the flush portal inlay.
    let spawn_pos = Vec3::new(0.0, 3.25, 12.0);
    let initial_pitch = -0.14; // Looking slightly downward (~8.0 degrees)
    let initial_yaw = 0.0;

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(spawn_pos).with_rotation(
            Quat::from_axis_angle(Vec3::Y, initial_yaw)
                * Quat::from_axis_angle(Vec3::X, initial_pitch),
        ),
        PlayerCamera,
        CameraController {
            pitch: initial_pitch,
            yaw: initial_yaw,
            ..default()
        },
        crate::chamber::camera::RuntimeGameplayCamera,
    ));
}

fn teardown_camera(
    mut commands: Commands,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    query: Query<Entity, With<PlayerCamera>>,
    mut existing_cameras: Query<
        &mut Camera,
        (
            With<crate::chamber::camera::WitnessCamera>,
            Without<PlayerCamera>,
        ),
    >,
) {
    // Restore system mouse cursor when leaving navigation
    if let Ok(mut cursor) = cursor_options.single_mut() {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
    // Re-enable WitnessCamera
    for mut cam in &mut existing_cameras {
        cam.is_active = true;
    }
}

/// Standing obstacles currently present in the world: the Jester Council host and the combined
/// Council table / manifestation altar. When AURA and the satellite rooms are hidden, only the
/// entities actually on the Council floor push the player, preventing invisible phantom collision.
fn character_obstacles() -> [(Vec2, f32); 2] {
    [
        (Vec2::new(10.2, 6.2), 1.5), // Jester Council host
        (
            Vec2::new(MANIFESTATION_PEDESTAL_POS.x, MANIFESTATION_PEDESTAL_POS.z),
            // The altar's own footprint. There is no table around it any more, so this is the
            // only solid thing at the centre; the vortex disc is floor inlay and is walked on.
            MANIFESTATION_ALTAR_COLLISION_RADIUS,
        ), // Altar centre; both position and radius derived from the shared constants.
    ]
}

/// Ground covered between footfalls, in metres.
///
/// Walk speed is about 6.4 m/s, so this is a footfall a little over three times a second - a
/// brisk walk rather than a jog. It is a distance and not an interval on purpose: a timer plays
/// the same rhythm whether the player is moving or pressed against a wall.
const STRIDE_LENGTH: f32 = 1.9;
/// Shorter stride when sprinting — the cadence quickens to match the faster pace.
const STRIDE_LENGTH_SPRINT: f32 = 1.25;

/// Canonical room-figure obstacles, derived from each room's radial embodiment offset.
/// Pinned by test so that when satellite rooms are re-enabled or relocated, their positions
/// match the world-builder contract.
pub fn canonical_room_figure_obstacles() -> [(Vec2, f32); 6] {
    castle::room_centres()
        .map(|center| (center + center.normalize() * castle::EMBODIMENT_RADIAL_OFFSET, 1.5))
}

/// Pushes the player out of any standing obstacle they have walked into.
fn resolve_character_obstacles(position: Vec2) -> Vec2 {
    let mut position = position;
    for (obstacle, radius) in character_obstacles() {
        let to_player = position - obstacle;
        let distance = to_player.length();
        if distance < radius {
            let push = if distance > 0.01 { to_player / distance } else { Vec2::Y };
            position = obstacle + push * radius;
        }
    }
    position
}

pub(super) fn player_locomotion(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    gamepads: Query<&Gamepad>,
    settings: Res<GameSettings>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    manifestation_state: Option<Res<super::manifestation::ManifestationState>>,
    encounter_state: Option<Res<super::encounters::EncounterState>>,
    workshop_state: Option<Res<super::workshop::WorkshopState>>,
    settings_menu: Option<Res<super::settings_menu::SettingsMenuState>>,
    mut sfx: MessageWriter<PlaySfx>,
    mut query: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    // Any open typing surface pauses locomotion — otherwise the player walks or flies away
    // from the thing they are typing into.
    if manifestation_state
        .as_ref()
        .map(|s| s.phase == super::manifestation::ManifestationPhase::Prompting)
        .unwrap_or(false)
        || encounter_state
            .as_ref()
            .map(|state| state.is_open())
            .unwrap_or(false)
        || workshop_state
            .as_ref()
            .map(|state| state.is_open())
            .unwrap_or(false)
        // The settings menu navigates on W/S and adjusts on A/D, which are also the movement
        // keys. Without this the player walks off across the castle while reading the menu,
        // and every keystroke does two things at once.
        || settings_menu
            .as_ref()
            .map(|menu| menu.open)
            .unwrap_or(false)
    {
        return;
    }

    // Re-lock and hide cursor if clicked inside window
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Ok(mut cursor) = cursor_options.single_mut() {
            if cursor.visible || cursor.grab_mode != CursorGrabMode::Locked {
                cursor.visible = false;
                cursor.grab_mode = CursorGrabMode::Locked;
            }
        }
    }



    let dt = time.delta_secs();
    let now = time.elapsed_secs();

    // --- 1. MOUSE LOOK (Calmed, smooth, non-twitchy head movement) ---
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    if mouse_delta != Vec2::ZERO {
        let dx = mouse_delta.x.clamp(-240.0, 240.0);
        let dy = mouse_delta.y.clamp(-240.0, 240.0);

        controller.yaw -= dx * controller.sensitivity;
        controller.pitch -= dy * controller.sensitivity;
        controller.pitch = controller.pitch.clamp(-1.48, 1.48); // +/- 85 degrees

        transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
            * Quat::from_axis_angle(Vec3::X, controller.pitch);
    }

    // --- 1b. GAMEPAD LOOK (right stick, rate-based so it works alongside mouse look) ---
    let look_stick = gamepad_input::combined_right_stick(&gamepads, settings.gamepad_deadzone);
    if look_stick != Vec2::ZERO {
        controller.yaw -= look_stick.x * settings.gamepad_look_sensitivity * dt;
        controller.pitch += look_stick.y * settings.gamepad_look_sensitivity * dt;
        controller.pitch = controller.pitch.clamp(-1.48, 1.48);

        transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
            * Quat::from_axis_angle(Vec3::X, controller.pitch);
    }

    // --- 2. FLIGHT TRANSITION LOGIC ---
    let space_just_pressed = keyboard.just_pressed(KeyCode::Space)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::South);

    match controller.mode {
        LocomotionMode::Walking => {
            if space_just_pressed {
                let delta_since_last = now - controller.last_space_press_time;
                if delta_since_last < 0.45 && controller.space_tap_count >= 1 {
                    // Double space press: INSTANTLY engage flight mode! (No delay)
                    controller.mode = LocomotionMode::Flying;
                    controller.velocity_y = 0.0;
                    controller.space_tap_count = 0;
                    controller.last_space_press_time = -100.0;
                    controller.flight_space_press_index = 0;
                    controller.flight_space_press_history = [0.0; 3];
                } else {
                    // First space press: jump if grounded
                    controller.space_tap_count = 1;
                    controller.last_space_press_time = now;
                    if controller.is_grounded {
                        controller.velocity_y = controller.jump_impulse;
                        controller.is_grounded = false;
                        sfx.write(PlaySfx::new(Sfx::Jump));
                    }
                }
            } else if now - controller.last_space_press_time > 0.45 {
                controller.space_tap_count = 0;
            }
        }
        LocomotionMode::Flying => {
            if space_just_pressed {
                let slot = controller.flight_space_press_index % 3;
                controller.flight_space_press_history[slot] = now;
                controller.flight_space_press_index += 1;

                if controller.flight_space_press_index >= 3 {
                    let h = controller.flight_space_press_history;
                    let min_t = h[0].min(h[1]).min(h[2]);
                    let max_t = h[0].max(h[1]).max(h[2]);
                    if max_t - min_t <= 0.85 {
                        // 3 Space taps in a row: Turn off flying and immediately resume gravity!
                        controller.mode = LocomotionMode::Walking;
                        controller.velocity_y = 0.0;
                        controller.space_tap_count = 0;
                        controller.flight_space_press_index = 0;
                        controller.flight_space_press_history = [0.0; 3];
                    }
                }
            }
        }
    }

    // --- 3. MOVEMENT BY MODE ---
    match controller.mode {
        LocomotionMode::Walking => {
            // Sprint: hold Shift or click the left stick on gamepad.
            let shift_held = keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight)
                || gamepad_input::any_pressed(&gamepads, GamepadButton::LeftThumb);
            controller.sprint = if shift_held {
                SprintState::Sprinting
            } else {
                SprintState::Normal
            };
            let effective_speed = match controller.sprint {
                SprintState::Normal => controller.walk_speed,
                SprintState::Sprinting => controller.walk_speed * controller.sprint_multiplier,
            };

            // Standard FPS walking and strafing on horizontal plane
            let mut move_dir = Vec2::ZERO;
            if keyboard.pressed(KeyCode::KeyW) { move_dir.y += 1.0; }
            if keyboard.pressed(KeyCode::KeyS) { move_dir.y -= 1.0; }
            if keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
            if keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }
            move_dir += gamepad_input::combined_left_stick(&gamepads, settings.gamepad_deadzone);

            let forward = Vec3::new(-controller.yaw.sin(), 0.0, -controller.yaw.cos());
            let right = Vec3::new(controller.yaw.cos(), 0.0, -controller.yaw.sin());

            let walk_origin = Vec2::new(transform.translation.x, transform.translation.z);
            if move_dir != Vec2::ZERO {
                let norm = move_dir.normalize();
                let horizontal_vel = (forward * norm.y + right * norm.x) * effective_speed;
                transform.translation += horizontal_vel * dt;
            }


            // Center/room embodiment and active manifestation obstacles, then each outer
            // room's cobblestone wall.
            let before = Vec2::new(transform.translation.x, transform.translation.z);
            let resolved = castle::clamp_inside_wall(castle::resolve_room_walls(
                resolve_character_obstacles(before),
            ));
            transform.translation.x = resolved.x;
            transform.translation.z = resolved.y;

            // Footsteps are spaced by ground actually covered, measured *after* collision has
            // had its say. Measuring intent instead would keep the boots marching while the
            // player is pressed against a wall going nowhere.
            //
            // The stride shortens when sprinting so the rhythm quickens with the pace.
            let stride = match controller.sprint {
                SprintState::Normal => STRIDE_LENGTH,
                SprintState::Sprinting => STRIDE_LENGTH_SPRINT,
            };
            if controller.is_grounded {
                controller.stride_accumulated += resolved.distance(walk_origin);
                if controller.stride_accumulated >= stride {
                    controller.stride_accumulated -= stride;
                    controller.stride_index = controller.stride_index.wrapping_add(1);
                    sfx.write(PlaySfx::variant(Sfx::Footstep, controller.stride_index));
                }
            }

            let current_feet_y = transform.translation.y - controller.eye_height;
            let ground_y = castle::castle_surface_y(Vec2::new(transform.translation.x, transform.translation.z), current_feet_y);

            // Gravity & Vertical Position
            if !controller.is_grounded {
                controller.velocity_y -= controller.gravity * dt;
                transform.translation.y += controller.velocity_y * dt;
            }

            let current_feet_y = transform.translation.y - controller.eye_height;
            if current_feet_y <= ground_y {
                // Only a real arrival makes a noise. This branch also runs every frame the
                // player merely stands still, so landing has to be the *transition* into
                // grounded, and a gentle settle onto a step is not a landing.
                if !controller.is_grounded && controller.velocity_y < -2.0 {
                    sfx.write(PlaySfx::new(Sfx::Land));
                    // A landing restarts the stride, so the first step after it is a full one.
                    controller.stride_accumulated = 0.0;
                }
                transform.translation.y = ground_y + controller.eye_height;
                controller.velocity_y = 0.0;
                controller.is_grounded = true;
            } else if controller.is_grounded && (current_feet_y - ground_y) > 0.08 {
                // Stepped off a ledge
                controller.is_grounded = false;
            }
        }
        LocomotionMode::Flying => {
            // Free flight movement
            let mut move_dir = Vec2::ZERO;
            if keyboard.pressed(KeyCode::KeyW) { move_dir.y += 1.0; }
            if keyboard.pressed(KeyCode::KeyS) { move_dir.y -= 1.0; }
            if keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
            if keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }
            move_dir += gamepad_input::combined_left_stick(&gamepads, settings.gamepad_deadzone);

            let forward = Vec3::new(-controller.yaw.sin(), 0.0, -controller.yaw.cos());
            let right = Vec3::new(controller.yaw.cos(), 0.0, -controller.yaw.sin());

            if move_dir != Vec2::ZERO {
                let norm = move_dir.normalize();
                let horizontal_vel = (forward * norm.y + right * norm.x) * controller.flight_speed;
                transform.translation += horizontal_vel * dt;
            }

            // Vertical flight controls:
            // Spacebar or the right trigger (RT) holds ascend
            if keyboard.pressed(KeyCode::Space)
                || gamepad_input::any_pressed(&gamepads, GamepadButton::RightTrigger2)
            {
                transform.translation.y += controller.flight_speed * dt;
            }

            // Hold shift, or the left trigger (LT), to descend to floor level
            let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight)
                || keyboard.pressed(KeyCode::KeyC)
                || gamepad_input::any_pressed(&gamepads, GamepadButton::LeftTrigger2);

            let current_feet_y = transform.translation.y - controller.eye_height;
            let ground_y = castle::castle_surface_y(Vec2::new(transform.translation.x, transform.translation.z), current_feet_y);
            let floor_level = ground_y + controller.eye_height;

            if shift_pressed {
                transform.translation.y -= controller.flight_speed * dt;
                if transform.translation.y < floor_level {
                    transform.translation.y = floor_level;
                }
            }

            // Flight stays inside the shell: the wall is a circle, and the ceiling is just
            // under the wall head so the top gallery is reachable but the vault is not a door.
            let inside = castle::clamp_inside_wall(Vec2::new(transform.translation.x, transform.translation.z));
            transform.translation.x = inside.x;
            transform.translation.z = inside.y;
            transform.translation.y = transform.translation.y.clamp(floor_level, castle::flight_ceiling());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_room_figure_is_collidable_where_it_actually_stands() {
        // Two of the six used to be transcribed with a flipped sign, putting an invisible
        // pillar in the Architect and Empath doorways and leaving both figures walk-through.
        let obstacles = canonical_room_figure_obstacles();
        for (index, centre) in castle::room_centres().iter().enumerate() {
            let (obstacle, radius) = obstacles[index];
            let expected = *centre + centre.normalize() * castle::EMBODIMENT_RADIAL_OFFSET;
            assert!((obstacle - expected).length() < 0.001, "room {index} obstacle is misplaced");
            assert!(
                obstacle.length() > centre.length(),
                "room {index} figure must sit further from the hub than its room centre"
            );
            assert!(radius > 0.0);
        }
    }

    #[test]
    fn the_architect_figure_is_collidable_behind_its_room_centre() {
        let architect = canonical_room_figure_obstacles()[0].0;
        let expected = -(castle::OUTER_ROOM_DISTANCE + castle::EMBODIMENT_RADIAL_OFFSET);
        assert!((architect - Vec2::new(0.0, expected)).length() < 0.01);
    }

    #[test]
    fn walking_into_a_figure_is_pushed_back_out_to_its_radius() {
        let figure = character_obstacles()[0].0;
        let inside = figure + Vec2::new(0.0, 0.4);
        let resolved = resolve_character_obstacles(inside);
        assert!((resolved - figure).length() >= 1.49);
    }

    #[test]
    fn manifestation_obstacle_is_at_the_shared_pedestal_position() {
        let obstacle = character_obstacles()[1].0;
        assert_eq!(
            obstacle,
            Vec2::new(MANIFESTATION_PEDESTAL_POS.x, MANIFESTATION_PEDESTAL_POS.z)
        );
    }

    /// The operator asked to stand in the middle of the spinning disc. A collision radius
    /// sized to the old table would have fenced the player off almost all of it, so this pins
    /// the altar's footprint between the stone it has to stop them hitting and the disc it
    /// must not swallow.
    #[test]
    fn altar_collision_clears_its_base_without_fencing_off_the_vortex_disc() {
        let radius = character_obstacles()[1].1;
        assert_eq!(radius, MANIFESTATION_ALTAR_COLLISION_RADIUS);
        assert!(
            radius > 1.20,
            "collision must clear the 1.20m base plinth or the player walks into the stone"
        );
        assert!(
            radius < super::super::world::COUNCIL_PORTAL_DISC_RADIUS,
            "a radius at or beyond the {}m disc would stop the player standing on the vortex at all",
            super::super::world::COUNCIL_PORTAL_DISC_RADIUS
        );
    }

    /// Walking up to the altar has to actually reach it. `pick_target` measures a real 3D
    /// distance from the eye, so the vertical climb from the cushion to a 3.25m standing eye
    /// height eats into the same budget the horizontal approach needs.
    #[test]
    fn a_player_stopped_by_altar_collision_is_still_inside_interaction_range() {
        let radius = character_obstacles()[1].1;
        let eye = Vec3::new(
            MANIFESTATION_PEDESTAL_POS.x,
            castle::GROUND_Y + 2.85,
            MANIFESTATION_PEDESTAL_POS.z + radius,
        );
        let anchor = super::super::manifestation::altar_interaction_anchor();
        assert!(
            eye.distance(anchor) <= super::super::interaction::ALTAR_INTERACTION_RANGE,
            "stopped at {radius}m the eye is {:.3}m from the cushion, beyond the {}m reach",
            eye.distance(anchor),
            super::super::interaction::ALTAR_INTERACTION_RANGE
        );
    }
}
