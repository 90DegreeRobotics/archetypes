//! The Witness camera — the only moving element in the chamber.
//!
//! The star tetrahedron and the spheres are fixed; focus is expressed by flying the
//! camera. At rest the camera holds the Director's **authored** establishing frame
//! (the `Witness_Camera` node inside the temple); when a council member holds the
//! floor it glides to a pose that brings that archetype's sphere to the fore with
//! the star behind it.

use bevy::prelude::*;

use super::{spheres::ArchetypeSphere, ChamberState, CurrentFocus};
use crate::theme::Archetype;

/// Fallback establishing pose, used only until the authored camera node is loaded.
const ESTABLISHING_FALLBACK: Vec3 = Vec3::new(8.0, 6.2, 12.0);
const COUNCIL_CENTER: Vec3 = Vec3::new(0.0, 2.0, 0.0);
/// The table pose: a 3/4 view that shows the ornate council table as furniture,
/// standing on the chamber floor. The table (engine-scaled ×3) spans world y ≈ −1.8
/// (top) down to ≈ −5 (feet on the floor); the camera sits just above the top and
/// back, looking down at the table's mid-height so both the glowing top and the
/// arched legs read. On submit the camera sweeps up from here to the star.
const TABLE_CAMERA_POS: Vec3 = Vec3::new(0.0, -0.8, 8.0);
const TABLE_LOOK: Vec3 = Vec3::new(0.0, -3.4, 0.0);
/// When an archetype speaks, the camera swings to that sphere's compass bearing at a
/// fixed radius and height (well inside the temple walls at radius ~21, and always
/// above the floor), then looks at the sphere with the star beyond it. Positioning by
/// the sphere's *horizontal* bearing — not its full radial — keeps the camera from
/// diving under the floor for spheres in the lower hemisphere.
const FRAME_RADIUS: f32 = 11.0;
const FRAME_HEIGHT: f32 = 4.5;
/// Deliberation gets its own deliberately composed frame instead of falling back to
/// the wide Director establishing shot (authored for the main-menu backdrop, not
/// for "the council convenes"): pulled back and slightly elevated, centered
/// straight-on on the star so it reads as a clear, intentional composition while
/// the seven confer.
const DELIBERATION_CAMERA_POS: Vec3 = Vec3::new(0.0, 8.0, 21.0);
/// Per-second lerp/slerp response for the camera glide.
const CAMERA_RESPONSE: f32 = 1.6;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_witness_camera)
            .add_systems(
                Update,
                (disable_imported_cameras, gate_table_visibility, drive_camera).chain(),
            );
    }
}

fn gate_table_visibility(
    state: Res<State<ChamberState>>,
    mut named: Query<(&Name, &mut Visibility)>,
) {
    let visible = matches!(
        state.get(),
        ChamberState::Booting
            | ChamberState::MainMenu
            | ChamberState::Onboarding
            | ChamberState::IdleAtTable
            | ChamberState::ArtifactResult
    );
    for (name, mut visibility) in &mut named {
        if name.as_str() == "PortalTable" {
            *visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

#[derive(Component)]
pub struct WitnessCamera;

fn setup_witness_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(ESTABLISHING_FALLBACK).looking_at(COUNCIL_CENTER, Vec3::Y),
        WitnessCamera,
        Name::new("RuntimeWitnessCamera"),
    ));
}

/// The authored GLB cameras are part of the scene contract but not the live view;
/// the runtime Witness camera owns framing.
fn disable_imported_cameras(
    mut query: Query<&mut Camera, (With<Camera3d>, Without<WitnessCamera>)>,
) {
    for mut camera in &mut query {
        camera.is_active = false;
    }
}

fn drive_camera(
    state: Res<State<ChamberState>>,
    focus: Res<CurrentFocus>,
    spheres: Query<(&ArchetypeSphere, &GlobalTransform)>,
    authored: Query<(&Name, &GlobalTransform), (With<Camera3d>, Without<WitnessCamera>)>,
    mut camera: Query<&mut Transform, With<WitnessCamera>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    // Rest pose = the Director's authored establishing frame, read from the scene.
    let establishing = authored
        .iter()
        .find(|(name, _)| name.as_str() == "Witness_Camera")
        .map(|(_, global)| global.compute_transform())
        .unwrap_or_else(|| {
            Transform::from_translation(ESTABLISHING_FALLBACK).looking_at(COUNCIL_CENTER, Vec3::Y)
        });

    let target = match state.get() {
        // At the table: seated over the portal where intent is placed.
        ChamberState::Booting | ChamberState::MainMenu | ChamberState::Onboarding | ChamberState::IdleAtTable | ChamberState::ArtifactResult => {
            Transform::from_translation(TABLE_CAMERA_POS).looking_at(TABLE_LOOK, Vec3::Y)
        }
        // A council member holds the floor: frame that sphere.
        ChamberState::CouncilSpeaking => focus
            .0
            .and_then(|archetype| sphere_world_pos(&spheres, archetype))
            .map(frame_sphere)
            .unwrap_or(establishing),
        // The council convenes: a deliberate, centered frame on the star, not the
        // wide main-menu establishing shot.
        ChamberState::Deliberating => {
            Transform::from_translation(DELIBERATION_CAMERA_POS).looking_at(COUNCIL_CENTER, Vec3::Y)
        }
        // Verdict: reveal the council in the same establishing composition.
        _ => establishing,
    };

    // During boot the camera snaps to the establishing pose (t = 1.0) so that when the
    // title lifts it is already composed — no opening glide/flyby into position.
    let t = if *state.get() == ChamberState::Booting {
        1.0
    } else {
        (time.delta_secs() * CAMERA_RESPONSE).min(1.0)
    };
    transform.translation = transform.translation.lerp(target.translation, t);
    transform.rotation = transform.rotation.slerp(target.rotation, t);
}

/// A pose that frames the speaking sphere with the star beyond it. The camera sits at
/// the sphere's horizontal bearing, at a fixed radius and height, looking at the sphere.
fn frame_sphere(sphere_pos: Vec3) -> Transform {
    let bearing = Vec3::new(sphere_pos.x, 0.0, sphere_pos.z).normalize_or_zero();
    let camera_pos = bearing * FRAME_RADIUS + Vec3::Y * FRAME_HEIGHT;
    Transform::from_translation(camera_pos).looking_at(sphere_pos, Vec3::Y)
}

fn sphere_world_pos(
    spheres: &Query<(&ArchetypeSphere, &GlobalTransform)>,
    archetype: Archetype,
) -> Option<Vec3> {
    spheres.iter().find_map(|(sphere, transform)| {
        (sphere.archetype == archetype).then(|| transform.translation())
    })
}
