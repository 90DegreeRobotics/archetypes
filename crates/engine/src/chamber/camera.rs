use super::ChamberState;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_witness_camera).add_systems(
            Update,
            (disable_imported_cameras, handle_camera_transitions).chain(),
        );
    }
}

#[derive(Component)]
pub struct WitnessCamera {
    authored: Transform,
}

fn setup_witness_camera(mut commands: Commands) {
    let authored = Transform::from_xyz(12.0, 10.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3d::default(),
        authored,
        WitnessCamera { authored },
        Name::new("RuntimeWitnessCamera"),
    ));
}

fn disable_imported_cameras(
    mut query: Query<&mut Camera, (With<Camera3d>, Without<WitnessCamera>)>,
) {
    for mut camera in &mut query {
        camera.is_active = false;
    }
}

fn handle_camera_transitions(
    state: Res<State<ChamberState>>,
    mut query: Query<(&mut Transform, &WitnessCamera)>,
    time: Res<Time>,
) {
    let Ok((mut transform, witness_camera)) = query.single_mut() else {
        return;
    };

    let approach = match state.get() {
        ChamberState::IdleAtTable => 1.0,
        ChamberState::Onboarding => 1.0,
        ChamberState::Deliberating => 0.82,
        ChamberState::FocusArchetype
        | ChamberState::ArchitectInterior
        | ChamberState::WitnessVerdict
        | ChamberState::ArtifactPending
        | ChamberState::ArtifactResult => 0.68,
    };
    let target = witness_camera.authored.translation * approach;
    transform.translation = transform
        .translation
        .lerp(target, (time.delta_secs() * 2.0).min(1.0));
}
