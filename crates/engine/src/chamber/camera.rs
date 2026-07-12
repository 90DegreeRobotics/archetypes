use super::ChamberState;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, handle_camera_transitions);
    }
}

#[derive(Component)]
pub struct WitnessCamera;

fn setup_camera(mut commands: Commands) {
    // Initial Witness position at the table, looking forward/up
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 3.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Vec3::Y),
        WitnessCamera,
    ));
}

fn handle_camera_transitions(
    state: Res<State<ChamberState>>,
    mut query: Query<&mut Transform, With<WitnessCamera>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    // Simple placeholder lerp towards merkaba (y=5.0) during deliberation
    let target_y = match state.get() {
        ChamberState::IdleAtTable => 1.5,
        ChamberState::Deliberating | ChamberState::FocusArchetype => 3.5,
    };

    let target_z = match state.get() {
        ChamberState::IdleAtTable => 3.0,
        ChamberState::Deliberating | ChamberState::FocusArchetype => 5.0,
    };

    transform.translation.y = transform
        .translation
        .y
        .lerp(target_y, time.delta_secs() * 2.0);
    transform.translation.z = transform
        .translation
        .z
        .lerp(target_z, time.delta_secs() * 2.0);
}
