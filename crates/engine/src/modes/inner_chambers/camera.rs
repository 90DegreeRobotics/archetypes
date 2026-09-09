use super::InnerChambersState;
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

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

    // Flight engage timing (double tap + hold 2.0s)
    pub last_space_press_time: f32,
    pub space_tap_count: u32,
    pub space_held_duration: f32,

    // Flight disengage timing (triple tap space)
    pub flight_space_press_history: [f32; 3],
    pub flight_space_press_index: usize,
}

impl CameraController {
    pub fn locomotion_hud_text(&self) -> String {
        match self.mode {
            LocomotionMode::Walking => {
                if self.space_held_duration > 0.05 {
                    let pct = ((self.space_held_duration / 2.0) * 100.0).clamp(0.0, 100.0) as u32;
                    let bars = pct / 10;
                    let bar_str = "=".repeat(bars as usize) + &" ".repeat((10 - bars) as usize);
                    format!(
                        "COUNCIL ROTUNDA  •  [STATUS: GROUND WALKING]\nWASD: Walk & Strafe  •  Space: Jump  •  Mouse: Look (Tilted Down)\n>>> HOLDING FOR FLIGHT: [{bar_str}] {pct}% <<<  •  Esc: Menu"
                    )
                } else {
                    "COUNCIL ROTUNDA  •  [STATUS: GROUND WALKING]\nWASD: Walk & Strafe  •  Space: Jump  •  Mouse: Look (Tilted Down)\nDouble-Jump + Hold Space (2s): Free Flight Mode  •  Esc: Menu".to_string()
                }
            }
            LocomotionMode::Flying => {
                "COUNCIL ROTUNDA  •  [STATUS: FREE FLIGHT]\nWASD: Fly & Strafe  •  Space: Ascend  •  Shift / C: Descend  •  Mouse: Look 360°\nTriple-Tap Space (3x): Turn Off Flight & Resume Gravity  •  Esc: Menu".to_string()
            }
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            mode: LocomotionMode::Walking,
            eye_height: 1.75,
            walk_speed: 6.2,
            flight_speed: 10.5,
            velocity_y: 0.0,
            gravity: 16.0,
            jump_impulse: 5.2,
            sensitivity: 0.00095, // Smooth, non-touchy FPS look sensitivity
            pitch: -0.15,         // Looking slightly downward at the objects
            yaw: 0.0,
            is_grounded: true,
            last_space_press_time: -100.0,
            space_tap_count: 0,
            space_held_duration: 0.0,
            flight_space_press_history: [0.0; 3],
            flight_space_press_index: 0,
        }
    }
}

fn setup_camera(
    mut commands: Commands,
    mut existing_cameras: Query<
        &mut Camera,
        (
            With<crate::chamber::camera::WitnessCamera>,
            Without<PlayerCamera>,
        ),
    >,
) {
    // Disable WitnessCamera while navigating
    for mut cam in &mut existing_cameras {
        cam.is_active = false;
    }

    // Spawn player standing on the floor (eye height 1.75m) in front of the dais
    let spawn_pos = Vec3::new(0.0, 1.75, 11.8);
    let initial_pitch = -0.15; // Slightly downward look (~8.6 degrees)
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
    query: Query<Entity, With<PlayerCamera>>,
    mut existing_cameras: Query<
        &mut Camera,
        (
            With<crate::chamber::camera::WitnessCamera>,
            Without<PlayerCamera>,
        ),
    >,
) {
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
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let now = time.elapsed_secs();

    // --- 1. MOUSE LOOK (Smooth, non-touchy FPS look) ---
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    if mouse_delta != Vec2::ZERO {
        let dx = mouse_delta.x.clamp(-120.0, 120.0);
        let dy = mouse_delta.y.clamp(-120.0, 120.0);

        controller.yaw -= dx * controller.sensitivity;
        controller.pitch -= dy * controller.sensitivity;
        controller.pitch = controller.pitch.clamp(-1.48, 1.48); // +/- 85 degrees

        transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
            * Quat::from_axis_angle(Vec3::X, controller.pitch);
    }

    // --- 2. FLIGHT TRANSITION LOGIC ---
    let space_just_pressed = keyboard.just_pressed(KeyCode::Space);
    let space_pressed = keyboard.pressed(KeyCode::Space);

    match controller.mode {
        LocomotionMode::Walking => {
            if space_just_pressed {
                let delta_since_last = now - controller.last_space_press_time;
                if delta_since_last < 0.45 {
                    controller.space_tap_count = controller.space_tap_count.saturating_add(1);
                } else {
                    controller.space_tap_count = 1;
                }
                controller.last_space_press_time = now;

                // Jump if on the ground
                if controller.is_grounded {
                    controller.velocity_y = controller.jump_impulse;
                    controller.is_grounded = false;
                }
            }

            // Holding space on second tap initiates flight charge
            if controller.space_tap_count >= 2 && space_pressed {
                controller.space_held_duration += dt;
                if controller.space_held_duration >= 2.0 {
                    // Activate flight mode!
                    controller.mode = LocomotionMode::Flying;
                    controller.velocity_y = 0.0;
                    controller.space_tap_count = 0;
                    controller.space_held_duration = 0.0;
                    controller.flight_space_press_index = 0;
                    controller.flight_space_press_history = [0.0; 3];
                }
            } else if !space_pressed {
                controller.space_held_duration = 0.0;
                if now - controller.last_space_press_time > 0.45 {
                    controller.space_tap_count = 0;
                }
            }
        }
        LocomotionMode::Flying => {
            controller.space_held_duration = 0.0;
            if space_just_pressed {
                let slot = controller.flight_space_press_index % 3;
                controller.flight_space_press_history[slot] = now;
                controller.flight_space_press_index += 1;

                if controller.flight_space_press_index >= 3 {
                    let h = controller.flight_space_press_history;
                    let min_t = h[0].min(h[1]).min(h[2]);
                    let max_t = h[0].max(h[1]).max(h[2]);
                    if max_t - min_t <= 0.80 {
                        // 3 Space taps within 0.80s! Turn off flying and resume gravity
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
            // Free flight 6DOF
            let mut fly_dir = Vec3::ZERO;
            if keyboard.pressed(KeyCode::KeyW) { fly_dir.z -= 1.0; }
            if keyboard.pressed(KeyCode::KeyS) { fly_dir.z += 1.0; }
            if keyboard.pressed(KeyCode::KeyA) { fly_dir.x -= 1.0; }
            if keyboard.pressed(KeyCode::KeyD) { fly_dir.x += 1.0; }
            if keyboard.pressed(KeyCode::Space) { fly_dir.y += 1.0; }
            if keyboard.pressed(KeyCode::ShiftLeft)
                || keyboard.pressed(KeyCode::ShiftRight)
                || keyboard.pressed(KeyCode::KeyC)
            {
                fly_dir.y -= 1.0;
            }

            if fly_dir != Vec3::ZERO {
                let rot = transform.rotation;
                let movement = rot * fly_dir.normalize() * controller.flight_speed * dt;
                transform.translation += movement;
            }

            // Clamp flight boundaries
            transform.translation.x = transform.translation.x.clamp(-36.0, 36.0);
            transform.translation.z = transform.translation.z.clamp(-36.0, 36.0);
            transform.translation.y = transform.translation.y.clamp(1.0, 21.0);
        }
    }
}
