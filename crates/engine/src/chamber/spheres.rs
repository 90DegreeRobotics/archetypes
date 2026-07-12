use super::{ChamberState, CurrentFocus};
use crate::theme::Archetype;
use bevy::prelude::*;
use std::f32::consts::PI;

pub struct SpheresPlugin;

impl Plugin for SpheresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_spheres)
            .add_systems(Update, animate_spheres);
    }
}

#[derive(Component)]
pub struct ArchetypeSphere {
    pub archetype: Archetype,
    pub base_angle: f32,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
}

fn setup_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere_mesh = meshes.add(Sphere::new(0.3));

    let council = [
        Archetype::Architect,
        Archetype::Sentinel,
        Archetype::Jester,
        Archetype::Mentor,
        Archetype::Explorer,
        Archetype::Oracle,
        Archetype::Empath,
    ];

    let num_spheres = council.len();

    for (i, &archetype) in council.iter().enumerate() {
        let theme = archetype.theme();
        let angle = (i as f32) * (2.0 * PI / num_spheres as f32);

        // Material inherits the archetype's exact glow / accent color
        let material = materials.add(StandardMaterial {
            base_color: theme.accent_primary.with_alpha(0.6), // Glassy but tinted
            emissive: theme.glow_primary.into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 0.0, 0.0), // Updated in system
            ArchetypeSphere {
                archetype,
                base_angle: angle,
                orbit_radius: 2.5,
                orbit_speed: 0.2,
            },
        ));
    }
}

fn animate_spheres(
    time: Res<Time>,
    state: Res<State<ChamberState>>,
    current_focus: Res<CurrentFocus>,
    mut query: Query<(&mut Transform, &ArchetypeSphere)>,
) {
    let center = Vec3::new(0.0, 5.0, 0.0); // Merkaba center
    for (mut transform, sphere) in &mut query {
        let current_angle = sphere.base_angle + time.elapsed_secs() * sphere.orbit_speed;
        let x = center.x + current_angle.cos() * sphere.orbit_radius;
        let z = center.z + current_angle.sin() * sphere.orbit_radius;
        // Add a slight bobbing
        let y = center.y + (time.elapsed_secs() + sphere.base_angle).sin() * 0.2;

        let orbit_position = Vec3::new(x, y, z);
        let is_focused = *state.get() == ChamberState::FocusArchetype
            && current_focus.0 == Some(sphere.archetype);
        let target = if is_focused {
            center + Vec3::Z
        } else {
            orbit_position
        };
        let response = if is_focused { 3.5 } else { 2.0 };

        transform.translation = transform
            .translation
            .lerp(target, (time.delta_secs() * response).min(1.0));

        let target_scale = if is_focused {
            Vec3::splat(1.5)
        } else {
            Vec3::ONE
        };
        transform.scale = transform
            .scale
            .lerp(target_scale, (time.delta_secs() * response).min(1.0));
    }
}
