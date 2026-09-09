//! The Witness camera — the only moving element in the chamber.
//!
//! The star tetrahedron and the spheres are fixed; focus is expressed by flying the
//! camera. At rest the camera holds the Director's **authored** establishing frame
//! (the `Witness_Camera` node inside the temple); when a council member holds the
//! floor it glides to a pose that brings that archetype's sphere to the fore with
//! the star behind it.

use bevy::prelude::*;

use super::{spheres::ArchetypeSphere, ChamberState, CurrentFocus, COUNCIL_CENTER};
use crate::modes::inner_chambers::InnerChambersState;
use crate::modes::living_engine::LivingEngineState;
use crate::theme::Archetype;

/// Fallback establishing pose, used only until the authored camera node is loaded.
const ESTABLISHING_FALLBACK: Vec3 = Vec3::new(8.0, 6.2, 12.0);
/// Main-menu pose for the generated lore chamber. The Blender authoring scene has
/// proof cameras, but the runtime GLB intentionally exports none so Bevy owns the
/// only active 3D camera from frame zero.
const MENU_CAMERA_POS: Vec3 = Vec3::new(0.0, 2.05, 7.4);
const MENU_LOOK: Vec3 = Vec3::new(0.0, 1.02, 0.0);
/// The table pose stays near the Witness seat so any future prompt surface lands
/// on the new meter-scale table instead of the parked legacy furniture.
const TABLE_CAMERA_POS: Vec3 = Vec3::new(0.0, 1.45, 6.2);
const TABLE_LOOK: Vec3 = Vec3::new(0.0, 0.98, 0.0);
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
        app.add_systems(Startup, setup_witness_camera).add_systems(
            Update,
            (
                disable_imported_cameras,
                gate_boot_and_table_visibility,
                drive_camera,
            )
                .chain(),
        );
    }
}

/// Own all boot/table Visibility + Witness camera activity in one system so it
/// cannot B0001-conflict with a parallel boot Update system.
fn occupying_exclusive_world(
    inner: &State<InnerChambersState>,
    living: &State<LivingEngineState>,
) -> bool {
    *inner.get() != InnerChambersState::Inactive
        || *living.get() != LivingEngineState::Inactive
}

fn gate_boot_and_table_visibility(
    state: Res<State<ChamberState>>,
    inner: Res<State<InnerChambersState>>,
    living: Res<State<LivingEngineState>>,
    mut clear: ResMut<ClearColor>,
    mut named: Query<(&Name, &mut Visibility)>,
    mut cameras: Query<&mut Camera, With<WitnessCamera>>,
) {
    let booting = *state.get() == ChamberState::Booting;
    if booting {
        clear.0 = Color::BLACK;
    }

    let exclusive = occupying_exclusive_world(&inner, &living);
    let ritual = matches!(
        state.get(),
        ChamberState::Onboarding
            | ChamberState::IdleAtTable
            | ChamberState::Deliberating
            | ChamberState::CouncilSpeaking
            | ChamberState::WitnessVerdict
            | ChamberState::ArtifactPending
            | ChamberState::ArtifactResult
    );
    let lore_visible = !booting && !exclusive && !ritual;
    let council_visible = !booting && !exclusive && ritual;
    let table_visible = council_visible
        && matches!(
            state.get(),
            ChamberState::Onboarding | ChamberState::IdleAtTable | ChamberState::ArtifactResult
        );

    for (name, mut visibility) in &mut named {
        match name.as_str() {
            "LoreCouncilChamber" => {
                *visibility = if lore_visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            "AuthoritativeCouncilChamber" => {
                *visibility = if council_visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            "PortalTable" => {
                if !exclusive {
                    *visibility = if table_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            _ => {}
        }
    }

    if let Ok(mut camera) = cameras.single_mut() {
        camera.is_active = !booting && !exclusive;
    }
}

#[derive(Component)]
pub struct WitnessCamera;

/// Marks a mode-owned 3D camera that must remain active while WitnessCamera is gated off.
#[derive(Component)]
pub struct RuntimeGameplayCamera;

fn setup_witness_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(ESTABLISHING_FALLBACK).looking_at(COUNCIL_CENTER, Vec3::Y),
        WitnessCamera,
        Name::new("RuntimeWitnessCamera"),
    ));
}

/// Imported cameras, when present in legacy/reference assets, are never the live
/// view; the runtime Witness camera owns framing.
fn disable_imported_cameras(
    mut query: Query<
        &mut Camera,
        (
            Without<WitnessCamera>,
            Without<Camera2d>,
            Without<RuntimeGameplayCamera>,
        ),
    >,
) {
    for mut camera in &mut query {
        camera.is_active = false;
    }
}

fn drive_camera(
    state: Res<State<ChamberState>>,
    focus: Res<CurrentFocus>,
    spheres: Query<(&ArchetypeSphere, &GlobalTransform)>,
    authored: Query<
        (&Name, &GlobalTransform),
        (With<Camera>, Without<WitnessCamera>, Without<Camera2d>),
    >,
    mut camera: Query<&mut Transform, With<WitnessCamera>>,
    star: Query<&Transform, (With<crate::chamber::star::SolidStar>, Without<WitnessCamera>)>,
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
            Transform::from_translation(MENU_CAMERA_POS).looking_at(MENU_LOOK, Vec3::Y)
        });

    let target = match state.get() {
        // Opening/menu: use an authored Witness camera if a legacy asset provides
        // one, otherwise use the generated lore chamber's runtime pose.
        ChamberState::Booting | ChamberState::MainMenu => establishing,
        ChamberState::Onboarding | ChamberState::IdleAtTable | ChamberState::ArtifactResult => {
            Transform::from_translation(TABLE_CAMERA_POS).looking_at(TABLE_LOOK, Vec3::Y)
        }
        // A council member holds the floor: frame that sphere.
        ChamberState::CouncilSpeaking => focus
            .0
            .and_then(|archetype| sphere_world_pos(&spheres, archetype))
            .map(|sphere| {
                star.single()
                    .ok()
                    .map(|star_transform| hexagram_frame(sphere, star_transform))
                    .unwrap_or_else(|| frame_sphere(sphere))
            })
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

/// Cube vertices of a stellated octahedron — the eight 3-fold axes where two
/// tetrahedra project as a hexagram (Star of David).
const HEXAGRAM_LOCAL_AXES: [Vec3; 8] = [
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, -1.0),
];

pub(crate) fn hexagram_axis_for_sphere(sphere_pos: Vec3, star: &Transform) -> Vec3 {
    let to_sphere = (sphere_pos - star.translation).normalize_or_zero();
    HEXAGRAM_LOCAL_AXES
        .iter()
        .map(|local| (star.rotation * local.normalize()).normalize_or_zero())
        .max_by(|a, b| a.dot(to_sphere).total_cmp(&b.dot(to_sphere)))
        .unwrap_or(to_sphere)
}

fn hexagram_frame(sphere_pos: Vec3, star: &Transform) -> Transform {
    let axis = hexagram_axis_for_sphere(sphere_pos, star);
    let camera_pos = star.translation + axis * FRAME_RADIUS + Vec3::Y * 0.6;
    let look = sphere_pos.lerp(star.translation, 0.35);
    Transform::from_translation(camera_pos).looking_at(look, Vec3::Y)
}

fn sphere_world_pos(
    spheres: &Query<(&ArchetypeSphere, &GlobalTransform)>,
    archetype: Archetype,
) -> Option<Vec3> {
    spheres.iter().find_map(|(sphere, transform)| {
        (sphere.archetype == archetype).then(|| transform.translation())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexagram_axis_prefers_the_cube_vertex_facing_the_speaker() {
        let star = Transform::from_translation(Vec3::ZERO);
        let sphere = Vec3::new(4.0, 4.0, 4.0);
        let axis = hexagram_axis_for_sphere(sphere, &star);
        let expected = Vec3::ONE.normalize();
        assert!(axis.dot(expected) > 0.98, "axis={axis:?}");
    }

    #[test]
    fn hexagram_axes_are_eight_distinct_directions() {
        let mut unique = HEXAGRAM_LOCAL_AXES
            .map(|axis| {
                (
                    (axis.x.signum() as i32),
                    (axis.y.signum() as i32),
                    (axis.z.signum() as i32),
                )
            })
            .to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 8);
    }
}
