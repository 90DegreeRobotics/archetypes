use super::InnerChambersState;
use crate::chamber::{boot::spawn_main_menu, spawn_authoritative_chamber};
use crate::modes::ModeRegistry;
use bevy::prelude::*;

pub(crate) const ARCHITECT_NODE_RADIUS: f32 = 3.25;
pub(crate) const ARCHITECT_NODE_POSITIONS: [Vec3; 6] = [
    Vec3::new(-10.0, 2.0, -10.0),
    Vec3::new(-10.0, 2.0, 10.0),
    Vec3::new(0.0, 2.0, -10.0),
    Vec3::new(0.0, 2.0, 10.0),
    Vec3::new(10.0, 2.0, -10.0),
    Vec3::new(10.0, 2.0, 10.0),
];

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Loading), setup_architect_world)
            .add_systems(
                OnEnter(InnerChambersState::Exiting),
                teardown_architect_world,
            );
    }
}

#[derive(Component)]
pub struct ArchitectWorldElement;

#[derive(Component)]
pub struct ArchitectTruthNode {
    pub index: usize,
}

#[derive(Component)]
pub struct InnerChambersHint;

fn setup_architect_world(
    mut commands: Commands,
    query_main_scene: Query<(Entity, &Name)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    for (entity, name) in &query_main_scene {
        if name.as_str() == "AuthoritativeCouncilChamber" || name.as_str() == "PortalTable" {
            commands.entity(entity).despawn();
        }
    }

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 0.0),
            emissive: LinearRgba::new(0.0, 0.1, 0.3, 1.0),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        ArchitectWorldElement,
    ));

    let cube_mesh = meshes.add(Cuboid::new(2.0, 2.0, 2.0));
    let cube_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        emissive: LinearRgba::new(0.0, 0.2, 0.5, 1.0),
        ..default()
    });

    for (index, position) in ARCHITECT_NODE_POSITIONS.iter().copied().enumerate() {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(cube_mat.clone()),
            Transform::from_translation(position),
            ArchitectTruthNode { index },
            ArchitectWorldElement,
            Name::new(format!("ArchitectTruthNode_{index}")),
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ArchitectWorldElement,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                bottom: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.03, 0.07, 0.72)),
            GlobalZIndex(920),
            ArchitectWorldElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "INNER CHAMBER LOCKED PROTOTYPE\nFind a blue node. E reads it. Esc returns.",
                ),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.78, 0.9, 1.0)),
                InnerChambersHint,
            ));
        });

    next_state.set(InnerChambersState::Navigating);
}

fn teardown_architect_world(
    mut commands: Commands,
    query: Query<Entity, With<ArchitectWorldElement>>,
    asset_server: Res<AssetServer>,
    registry: Res<ModeRegistry>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }

    spawn_authoritative_chamber(&mut commands, &asset_server);

    commands.remove_resource::<crate::chamber::ActiveGameMode>();
    next_state.set(InnerChambersState::Inactive);
    spawn_main_menu(commands, registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architect_world_has_six_truth_nodes() {
        assert_eq!(ARCHITECT_NODE_POSITIONS.len(), 6);
        assert!(ARCHITECT_NODE_RADIUS > 2.0);
    }
}
