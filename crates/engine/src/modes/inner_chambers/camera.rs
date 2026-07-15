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
                camera_movement.run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(OnEnter(InnerChambersState::Exiting), teardown_camera);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
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
    // Disable WitnessCamera while we are navigating
    for mut cam in &mut existing_cameras {
        cam.is_active = false;
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 0.0).looking_at(Vec3::new(0.0, 2.0, -1.0), Vec3::Y),
        PlayerCamera,
        CameraController::default(),
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

fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let Ok((mut transform, mut controller)) = query.single_mut() else {
        return;
    };

    // Mouse look
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        mouse_delta += event.delta;
    }

    if mouse_delta != Vec2::ZERO {
        controller.yaw -= mouse_delta.x * controller.sensitivity;
        controller.pitch -= mouse_delta.y * controller.sensitivity;
        controller.pitch = controller.pitch.clamp(-1.54, 1.54);

        transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
            * Quat::from_axis_angle(Vec3::X, controller.pitch);
    }

    // Keyboard movement
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        direction.y -= 1.0;
    }

    if direction != Vec3::ZERO {
        let rotation = transform.rotation;
        let movement = rotation * direction.normalize() * controller.speed * time.delta_secs();
        transform.translation += movement;
    }
}
