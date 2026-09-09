use super::InnerChambersState;
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Navigating), setup_camera)
            .add_systems(
                Update,
                player_locomotion.run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(OnEnter(InnerChambersState::Exiting), teardown_camera);
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
                "COUNCIL ROTUNDA  •  [STATUS: GROUND WALKING]\nWASD: Move & Strafe  •  Space: Jump (Double-Tap: Fly)  •  Mouse: Look  •  Esc: Menu".to_string()
            }
            LocomotionMode::Flying => {
                "COUNCIL ROTUNDA  •  [STATUS: FREE FLIGHT]\nWASD: Fly  •  Space: Ascend  •  Shift: Descend  •  3x Space: Land  •  Esc: Menu".to_string()
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
            sensitivity: 0.00045, // Calmed, smooth, non-twitchy head movement
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

    // Spawn player standing on the floor (eye height 2.85m) in front of the dais
    let spawn_pos = Vec3::new(0.0, 2.85, 11.8);
    let initial_pitch = -0.18; // Looking slightly downward (~10.3 degrees)
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

fn player_locomotion(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

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
        let dx = mouse_delta.x.clamp(-60.0, 60.0);
        let dy = mouse_delta.y.clamp(-60.0, 60.0);

        controller.yaw -= dx * controller.sensitivity;
        controller.pitch -= dy * controller.sensitivity;
        controller.pitch = controller.pitch.clamp(-1.48, 1.48); // +/- 85 degrees

        transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
            * Quat::from_axis_angle(Vec3::X, controller.pitch);
    }

    // --- 2. FLIGHT TRANSITION LOGIC ---
    let space_just_pressed = keyboard.just_pressed(KeyCode::Space);

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

            let forward = Vec3::new(-controller.yaw.sin(), 0.0, -controller.yaw.cos());
            let right = Vec3::new(controller.yaw.cos(), 0.0, -controller.yaw.sin());

            if move_dir != Vec2::ZERO {
                let norm = move_dir.normalize();
                let horizontal_vel = (forward * norm.y + right * norm.x) * controller.walk_speed;
                transform.translation += horizontal_vel * dt;
            }

            // Room boundary collision
            transform.translation.x = transform.translation.x.clamp(-36.0, 36.0);
            transform.translation.z = transform.translation.z.clamp(-36.0, 36.0);

            // Pedestal obstacle collision (radius ~1.45m)
            let pedestal_positions = [
                Vec2::new(7.42, 7.42),
                Vec2::new(-7.42, 7.42),
                Vec2::new(-7.42, -7.42),
                Vec2::new(7.42, -7.42),
            ];
            for ped in pedestal_positions {
                let to_ped = Vec2::new(transform.translation.x, transform.translation.z) - ped;
                let dist = to_ped.length();
                if dist < 1.45 {
                    let push = if dist > 0.01 { to_ped / dist } else { Vec2::Y };
                    let corrected = ped + push * 1.45;
                    transform.translation.x = corrected.x;
                    transform.translation.z = corrected.y;
                }
            }

            // Central Table collision (radius ~2.35m)
            let center_dist = (transform.translation.x * transform.translation.x
                + transform.translation.z * transform.translation.z)
                .sqrt();
            if center_dist < 2.35 {
                let push = if center_dist > 0.01 {
                    Vec2::new(transform.translation.x, transform.translation.z) / center_dist
                } else {
                    Vec2::Y
                };
                let corrected = push * 2.35;
                transform.translation.x = corrected.x;
                transform.translation.z = corrected.y;
            }

            // Empath statue obstacle collision (radius ~0.85m at (0.0, 7.2))
            let empath_pos = Vec2::new(0.0, 7.2);
            let to_empath = Vec2::new(transform.translation.x, transform.translation.z) - empath_pos;
            let dist_empath = to_empath.length();
            if dist_empath < 0.85 {
                let push = if dist_empath > 0.01 { to_empath / dist_empath } else { Vec2::Y };
                let corrected = empath_pos + push * 0.85;
                transform.translation.x = corrected.x;
                transform.translation.z = corrected.y;
            }

            // Ground height resolution
            let r = (transform.translation.x * transform.translation.x
                + transform.translation.z * transform.translation.z)
                .sqrt();
            let ground_y = if r <= 4.8 {
                0.30 // Dais upper platform
            } else if r <= 6.0 {
                0.15 // Dais lower step
            } else {
                0.0 // Base floor
            };

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

            let forward = Vec3::new(-controller.yaw.sin(), 0.0, -controller.yaw.cos());
            let right = Vec3::new(controller.yaw.cos(), 0.0, -controller.yaw.sin());

            if move_dir != Vec2::ZERO {
                let norm = move_dir.normalize();
                let horizontal_vel = (forward * norm.y + right * norm.x) * controller.flight_speed;
                transform.translation += horizontal_vel * dt;
            }

            // Vertical flight controls:
            // Spacebar hold moves upward
            if keyboard.pressed(KeyCode::Space) {
                transform.translation.y += controller.flight_speed * dt;
            }

            // Hold shift to descend to floor level
            let shift_pressed = keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight)
                || keyboard.pressed(KeyCode::KeyC);

            // Ground height resolution beneath current position
            let r = (transform.translation.x * transform.translation.x
                + transform.translation.z * transform.translation.z)
                .sqrt();
            let ground_y = if r <= 4.8 {
                0.30
            } else if r <= 6.0 {
                0.15
            } else {
                0.0
            };
            let floor_level = ground_y + controller.eye_height;

            if shift_pressed {
                transform.translation.y -= controller.flight_speed * dt;
                if transform.translation.y < floor_level {
                    transform.translation.y = floor_level;
                }
            }

            // Clamp flight boundaries (prevent falling through floor or clipping through roof)
            transform.translation.x = transform.translation.x.clamp(-36.0, 36.0);
            transform.translation.z = transform.translation.z.clamp(-36.0, 36.0);
            transform.translation.y = transform.translation.y.clamp(floor_level, 21.0);
        }
    }
}

