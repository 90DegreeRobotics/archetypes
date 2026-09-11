use super::InnerChambersState;
use crate::services::gamepad_input;
use crate::services::settings::GameSettings;
use bevy::ecs::message::MessageReader;
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct CameraPlugin;

use super::castle;

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

#[derive(Component)]
pub struct CameraController {
    pub mode: LocomotionMode,
    pub eye_height: f32,
    pub walk_speed: f32,
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
}

impl CameraController {
    pub fn locomotion_hud_text(&self) -> String {
        match self.mode {
            LocomotionMode::Walking => {
                "INNER CASTLE  •  [STATUS: GROUND WALKING]\nWASD/L-Stick: Move & Strafe  •  Space/A: Jump (Double-Tap: Fly)  •  Mouse/R-Stick: Look  •  Esc/B: Menu".to_string()
            }
            LocomotionMode::Flying => {
                "INNER CASTLE  •  [STATUS: FREE FLIGHT]\nWASD/L-Stick: Fly  •  Space/RT: Ascend  •  Shift/LT: Descend  •  3x Space/A: Land  •  Esc/B: Menu".to_string()
            }
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            mode: LocomotionMode::Walking,
            eye_height: 2.85,     // Natural vantage standing above table and artifact pedestals
            walk_speed: 6.5,
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

/// Standing obstacles the player cannot walk through: the two Council-circle figures, the
/// manifestation pedestal, and each outer room's archetype figure.
///
/// The six room figures are derived from the same offset `world.rs` spawns them at, because
/// the previous hand-transcribed table had the Architect and Empath entries mirrored onto the
/// wrong side of their rooms — a 10m error that put an invisible pillar in each of those
/// doorways and left both figures uncollidable.
fn character_obstacles() -> [(Vec2, f32); 9] {
    let rooms = castle::room_centres()
        .map(|center| (center + center.normalize() * castle::EMBODIMENT_RADIAL_OFFSET, 1.5));
    [
        (Vec2::new(0.0, -8.5), 1.5), // AURA central embodiment
        (Vec2::new(10.2, 6.2), 1.5), // Jester Council host
        (Vec2::new(0.0, 3.4), 1.25), // Active manifestation pedestal
        rooms[0], rooms[1], rooms[2], rooms[3], rooms[4], rooms[5],
    ]
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
            // Standard FPS walking and strafing on horizontal plane
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
                let horizontal_vel = (forward * norm.y + right * norm.x) * controller.walk_speed;
                transform.translation += horizontal_vel * dt;
            }


            // Center/room embodiment and active manifestation obstacles, then each outer
            // room's cobblestone wall.
            let resolved = castle::clamp_inside_wall(castle::resolve_room_walls(
                resolve_character_obstacles(Vec2::new(transform.translation.x, transform.translation.z)),
            ));
            transform.translation.x = resolved.x;
            transform.translation.z = resolved.y;

            let current_feet_y = transform.translation.y - controller.eye_height;
            let ground_y = castle::castle_surface_y(Vec2::new(transform.translation.x, transform.translation.z), current_feet_y);

            // Gravity & Vertical Position
            if !controller.is_grounded {
                controller.velocity_y -= controller.gravity * dt;
                transform.translation.y += controller.velocity_y * dt;
            }

            let current_feet_y = transform.translation.y - controller.eye_height;
            if current_feet_y <= ground_y {
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
        let obstacles = character_obstacles();
        for (index, centre) in castle::room_centres().iter().enumerate() {
            let (obstacle, radius) = obstacles[3 + index];
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
        let architect = character_obstacles()[3].0;
        let expected = -(castle::OUTER_ROOM_DISTANCE + castle::EMBODIMENT_RADIAL_OFFSET);
        assert!((architect - Vec2::new(0.0, expected)).length() < 0.01);
    }

    #[test]
    fn walking_into_a_figure_is_pushed_back_out_to_its_radius() {
        let figure = character_obstacles()[3].0;
        let inside = figure + Vec2::new(0.0, 0.4);
        let resolved = resolve_character_obstacles(inside);
        assert!((resolved - figure).length() >= 1.49);
    }
}
