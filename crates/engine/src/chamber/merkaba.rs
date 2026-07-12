use bevy::prelude::*;

pub struct MerkabaPlugin;

impl Plugin for MerkabaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_merkaba)
            .add_systems(Update, animate_merkaba);
    }
}

#[derive(Component)]
pub struct MerkabaRing {
    pub axis: Dir3,
    pub speed: f32,
}

fn setup_merkaba(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let center = Vec3::new(0.0, 5.0, 0.0);

    // Core light
    commands.spawn((
        PointLight {
            color: Color::srgb(0.8, 0.9, 1.0),
            intensity: 1000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(center),
    ));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.8, 0.9, 0.5),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Ring 1
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(1.5, 0.05))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(center),
        MerkabaRing {
            axis: Dir3::X,
            speed: 0.5,
        },
    ));

    // Ring 2
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(2.0, 0.05))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(center),
        MerkabaRing {
            axis: Dir3::Y,
            speed: -0.3,
        },
    ));
}

fn animate_merkaba(time: Res<Time>, mut query: Query<(&mut Transform, &MerkabaRing)>) {
    for (mut transform, ring) in &mut query {
        transform.rotate_axis(ring.axis, ring.speed * time.delta_secs());
    }
}
