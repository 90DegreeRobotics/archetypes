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

    // Spawn player standing on the floor (eye height 2.85m) at the entrance of the exhibition court
    let spawn_pos = Vec3::new(0.0, 2.85, 17.5);
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

fn player_locomotion(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    manifestation_state: Option<Res<super::manifestation::ManifestationState>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    // If player is currently typing into the manifestation conduit, pause camera locomotion
    if manifestation_state
        .as_ref()
        .map(|s| s.phase == super::manifestation::ManifestationPhase::Prompting)
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

            // Central Table collision (radius 1.41m)
            let center_dist = (transform.translation.x * transform.translation.x
                + transform.translation.z * transform.translation.z)
                .sqrt();
            if center_dist < 1.41 {
                let push = if center_dist > 0.01 {
                    Vec2::new(transform.translation.x, transform.translation.z) / center_dist
                } else {
                    Vec2::Y
                };
                let corrected = push * 1.41;
                transform.translation.x = corrected.x;
                transform.translation.z = corrected.y;
            }

            // Archetype council character and manifestation altar obstacle collisions
            let character_obstacles = [
                (Vec2::new(-9.6, 13.5), 0.85), // Sentinel
                (Vec2::new(-5.0, 10.2), 0.85), // Aura
                (Vec2::new(0.0, 7.5), 0.85),   // Empath
                (Vec2::new(5.0, 10.2), 0.85),  // Oracle
                (Vec2::new(9.6, 13.5), 0.95),  // Nebula Jester (wide shoulders)
                (Vec2::new(0.0, 3.4), 1.25),   // Manifestation Altar Pedestal
            ];
            for (char_pos, radius) in character_obstacles {
                let to_char = Vec2::new(transform.translation.x, transform.translation.z) - char_pos;
                let dist_char = to_char.length();
                if dist_char < radius {
                    let push = if dist_char > 0.01 { to_char / dist_char } else { Vec2::Y };
                    let corrected = char_pos + push * radius;
                    transform.translation.x = corrected.x;
                    transform.translation.z = corrected.y;
                }
            }

            // Archetype niche wall collision: each figure's curved backdrop wall
            // blocks entry everywhere except through its own doorway gap, which
            // always opens toward the rotunda center. Mirrors the geometry built in
            // `world::build_wall_ring_mesh` / `world::NICHE_CENTERS` exactly so what
            // the player sees is what blocks them.
            for niche_center in super::world::NICHE_CENTERS {
                let center2 = Vec2::new(niche_center.x, niche_center.z);
                let to_player = Vec2::new(transform.translation.x, transform.translation.z) - center2;
                let dist = to_player.length();
                if dist >= super::world::NICHE_RING_RADIUS || dist < 0.01 {
                    continue;
                }
                let door_bearing = (-niche_center.z).atan2(-niche_center.x);
                let bearing = to_player.y.atan2(to_player.x); // Vec2(x,z) local axes match world's (cos,sin) parameterization
                let mut diff = bearing - door_bearing;
                diff = ((diff + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)) - std::f32::consts::PI;
                if diff.abs() < super::world::NICHE_DOOR_WIDTH * 0.5 {
                    continue; // Inside the doorway arc: free passage.
                }
                let push = to_player / dist;
                let corrected = center2 + push * super::world::NICHE_RING_RADIUS;
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

