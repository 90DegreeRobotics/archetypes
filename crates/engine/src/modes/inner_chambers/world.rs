use super::catalog::{council_chambers, node_positions, ChamberSpec};
use super::InnerChambersState;
use crate::chamber::boot::spawn_main_menu;
use crate::modes::ModeRegistry;
use crate::theme::Archetype;
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Loading), setup_inner_world)
            .add_systems(OnEnter(InnerChambersState::Exiting), teardown_inner_world);
    }
}

#[derive(Component)]
pub struct InnerWorldElement;

#[derive(Component)]
pub struct InnerTruthNode {
    pub chamber_index: usize,
    pub node_index: usize,
}

#[derive(Component)]
pub struct InnerChambersHint;

fn setup_inner_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut clear: ResMut<ClearColor>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    clear.0 = Color::srgb(0.04, 0.045, 0.06);

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(90.0, 90.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.09, 0.11),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        InnerWorldElement,
        Name::new("InnerHubFloor"),
    ));
    commands.spawn((
        PointLight {
            intensity: 120_000.0,
            range: 40.0,
            color: Color::srgb(0.85, 0.88, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 0.0),
        InnerWorldElement,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 1_800.0,
            shadows_enabled: true,
            color: Color::srgb(0.92, 0.94, 1.0),
            ..default()
        },
        Transform::from_xyz(10.0, 22.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        InnerWorldElement,
    ));

    for (chamber_index, spec) in council_chambers().iter().enumerate() {
        spawn_chamber(&mut commands, &mut meshes, &mut materials, chamber_index, spec);
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                bottom: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                max_width: Val::Px(720.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.86)),
            GlobalZIndex(920),
            InnerWorldElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "INNER CHAMBERS\nSeven minds surround the hub. WASD moves. Mouse looks. Walk into a chamber, align with a node, then E reads it. Esc returns.",
                ),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.93, 0.88)),
                InnerChambersHint,
            ));
        });

    next_state.set(InnerChambersState::Navigating);
}

fn spawn_chamber(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    chamber_index: usize,
    spec: &ChamberSpec,
) {
    let theme = spec.archetype.theme();
    let floor = materials.add(StandardMaterial {
        base_color: theme.bg_void,
        perceptual_roughness: 0.55,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(16.0, 16.0))),
        MeshMaterial3d(floor),
        Transform::from_translation(spec.origin),
        InnerWorldElement,
        Name::new(format!("{}Floor", spec.title)),
    ));

    spawn_law_geometry(commands, meshes, materials, spec);

    let node_mesh = meshes.add(Cuboid::new(1.4, 1.4, 1.4));
    let node_mat = materials.add(StandardMaterial {
        base_color: theme.accent_primary,
        emissive: LinearRgba::from(theme.accent_primary) * theme.glow_intensity,
        metallic: 0.4,
        perceptual_roughness: 0.2,
        ..default()
    });
    for (node_index, position) in node_positions(spec).into_iter().enumerate() {
        commands.spawn((
            Mesh3d(node_mesh.clone()),
            MeshMaterial3d(node_mat.clone()),
            Transform::from_translation(position),
            InnerTruthNode {
                chamber_index,
                node_index,
            },
            InnerWorldElement,
            Name::new(format!("{}Node_{node_index}", spec.title)),
        ));
        commands.spawn((
            PointLight {
                intensity: 55_000.0,
                range: 12.0,
                color: theme.accent_primary,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(position + Vec3::Y * 2.4),
            InnerWorldElement,
        ));
    }
}

fn spawn_law_geometry(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spec: &ChamberSpec,
) {
    let theme = spec.archetype.theme();
    let accent = materials.add(StandardMaterial {
        base_color: theme.accent_primary,
        emissive: LinearRgba::from(theme.accent_primary) * 0.35,
        unlit: true,
        ..default()
    });
    match spec.archetype {
        Archetype::Architect => {
            let line = meshes.add(Cuboid::new(16.0, 0.05, 0.06));
            for i in -2..=2 {
                let offset = i as f32 * 3.0;
                commands.spawn((
                    Mesh3d(line.clone()),
                    MeshMaterial3d(accent.clone()),
                    Transform::from_translation(spec.origin + Vec3::new(0.0, 0.04, offset)),
                    InnerWorldElement,
                ));
            }
        }
        Archetype::Sentinel => {
            let wall = meshes.add(Cuboid::new(14.0, 6.0, 0.2));
            for yaw in [0.0, std::f32::consts::FRAC_PI_2] {
                commands.spawn((
                    Mesh3d(wall.clone()),
                    MeshMaterial3d(accent.clone()),
                    Transform::from_translation(spec.origin + Vec3::Y * 3.0)
                        .with_rotation(Quat::from_rotation_y(yaw)),
                    InnerWorldElement,
                ));
            }
        }
        Archetype::Mentor => {
            for radius in [2.0, 3.6, 5.2] {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(radius * 2.0, 0.08, 0.08))),
                    MeshMaterial3d(accent.clone()),
                    Transform::from_translation(spec.origin + Vec3::Y * 0.2),
                    InnerWorldElement,
                ));
            }
        }
        Archetype::Explorer => {
            for i in 0..5 {
                let t = i as f32 / 4.0;
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 2.4))),
                    MeshMaterial3d(accent.clone()),
                    Transform::from_translation(
                        spec.origin + Vec3::new((t - 0.5) * 8.0, 0.4, (t - 0.5) * 6.0),
                    ),
                    InnerWorldElement,
                ));
            }
        }
        Archetype::Oracle => {
            let veil = meshes.add(Cuboid::new(8.0, 7.0, 0.08));
            commands.spawn((
                Mesh3d(veil),
                MeshMaterial3d(accent),
                Transform::from_translation(spec.origin + Vec3::new(0.0, 3.4, -4.0)),
                InnerWorldElement,
            ));
        }
        Archetype::Empath => {
            let orb = meshes.add(Sphere::new(0.55));
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                commands.spawn((
                    Mesh3d(orb.clone()),
                    MeshMaterial3d(accent.clone()),
                    Transform::from_translation(
                        spec.origin + Vec3::new(a.sin() * 3.2, 1.6, a.cos() * 3.2),
                    ),
                    InnerWorldElement,
                ));
            }
        }
        Archetype::Jester => {
            let cube = meshes.add(Cuboid::new(1.1, 1.1, 1.1));
            for i in 0..4 {
                commands.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(accent.clone()),
                    Transform::from_translation(
                        spec.origin + Vec3::new((i as f32 - 1.5) * 2.2, 1.0 + (i % 2) as f32, 1.5),
                    )
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.4, 0.7 * i as f32, 0.2)),
                    InnerWorldElement,
                ));
            }
        }
        Archetype::Codex | Archetype::Viren => {}
    }
}

fn teardown_inner_world(
    mut commands: Commands,
    query: Query<Entity, With<InnerWorldElement>>,
    registry: Res<ModeRegistry>,
    mut clear: ResMut<ClearColor>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    clear.0 = Color::BLACK;
    commands.remove_resource::<crate::chamber::ActiveGameMode>();
    next_state.set(InnerChambersState::Inactive);
    spawn_main_menu(commands, registry);
}
