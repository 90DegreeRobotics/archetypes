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
    asset_server: Res<AssetServer>,
) {
    clear.0 = Color::srgb(0.04, 0.045, 0.06);

    // --- 1. FLOOR & CENTRAL DAIS ---
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.07, 0.075, 0.09),
        perceptual_roughness: 0.85,
        metallic: 0.05,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(76.0, 76.0))),
        MeshMaterial3d(floor_mat.clone()),
        Transform::from_translation(Vec3::ZERO),
        InnerWorldElement,
        Name::new("CastleFloor"),
    ));

    // Raised circular stone dais beneath the Council Table
    let dais_step_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.12, 0.14),
        perceptual_roughness: 0.75,
        metallic: 0.1,
        ..default()
    });
    let dais_top_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.16, 0.19),
        perceptual_roughness: 0.65,
        metallic: 0.15,
        ..default()
    });
    // Outer step: radius 6.0m, height 0.15m
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(6.0, 0.15))),
        MeshMaterial3d(dais_step_mat),
        Transform::from_xyz(0.0, 0.075, 0.0),
        InnerWorldElement,
        Name::new("CouncilDaisLowerStep"),
    ));
    // Inner dais platform: radius 4.8m, height 0.30m (top surface at y = 0.30m)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(4.8, 0.30))),
        MeshMaterial3d(dais_top_mat),
        Transform::from_xyz(0.0, 0.15, 0.0),
        InnerWorldElement,
        Name::new("CouncilDaisUpperPlatform"),
    ));

    // --- 2. ANIMATED COUNCIL TABLE WITH STARGATE PORTAL ---
    // Feet authored at local z = -0.784. At scale 2.6, feet reach 2.0384m below origin.
    // Resting on dais top (y = 0.30m) requires origin y = 0.30 + 2.04 = 2.34m.
    commands.spawn((
        SceneRoot(asset_server.load("scenes/table.glb#Scene0")),
        Transform::from_xyz(0.0, 2.34, 0.0).with_scale(Vec3::splat(2.6)),
        InnerWorldElement,
        Name::new("PortalTable"),
    ));

    // --- 3. ENCLOSING CASTLE WALLS & BUTTRESS PILLARS ---
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.085, 0.10),
        perceptual_roughness: 0.9,
        ..default()
    });
    let pillar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.055, 0.07),
        perceptual_roughness: 0.92,
        ..default()
    });

    let wall_half = 38.0;
    let wall_height = 22.0;
    let wall_thick = 1.5;
    let wall_y = wall_height / 2.0; // 11.0m

    // North Wall (Z = -38.0)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_half * 2.0, wall_height, wall_thick))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, wall_y, -wall_half),
        InnerWorldElement,
        Name::new("CastleWallNorth"),
    ));
    // South Wall (Z = +38.0)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_half * 2.0, wall_height, wall_thick))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(0.0, wall_y, wall_half),
        InnerWorldElement,
        Name::new("CastleWallSouth"),
    ));
    // West Wall (X = -38.0)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thick, wall_height, wall_half * 2.0))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(-wall_half, wall_y, 0.0),
        InnerWorldElement,
        Name::new("CastleWallWest"),
    ));
    // East Wall (X = +38.0)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_thick, wall_height, wall_half * 2.0))),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(wall_half, wall_y, 0.0),
        InnerWorldElement,
        Name::new("CastleWallEast"),
    ));

    // Corner and wall buttress pillars
    let pillar_mesh = meshes.add(Cuboid::new(2.4, wall_height, 2.4));
    let pillar_positions = [
        Vec3::new(-wall_half + 1.0, wall_y, -wall_half + 1.0),
        Vec3::new(wall_half - 1.0, wall_y, -wall_half + 1.0),
        Vec3::new(-wall_half + 1.0, wall_y, wall_half - 1.0),
        Vec3::new(wall_half - 1.0, wall_y, wall_half - 1.0),
        Vec3::new(0.0, wall_y, -wall_half + 0.8),
        Vec3::new(0.0, wall_y, wall_half - 0.8),
        Vec3::new(-wall_half + 0.8, wall_y, 0.0),
        Vec3::new(wall_half - 0.8, wall_y, 0.0),
    ];
    for (i, pos) in pillar_positions.iter().enumerate() {
        commands.spawn((
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(pillar_mat.clone()),
            Transform::from_translation(*pos),
            InnerWorldElement,
            Name::new(format!("CastlePillar_{i}")),
        ));
    }

    // --- 4. CEILING SLAB & VAULTED CROSS-BEAMS ---
    let ceiling_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.045, 0.048, 0.06),
        perceptual_roughness: 0.95,
        ..default()
    });
    // Ceiling slab enclosing the roof at y = 22.0m
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_half * 2.0, 1.0, wall_half * 2.0))),
        MeshMaterial3d(ceiling_mat),
        Transform::from_xyz(0.0, wall_height + 0.5, 0.0),
        InnerWorldElement,
        Name::new("CastleCeilingSlab"),
    ));

    // Dark iron/timber structural cross-beams
    let beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.03, 0.03, 0.04),
        metallic: 0.3,
        perceptual_roughness: 0.7,
        ..default()
    });
    let beam_y = wall_height - 0.6; // 21.4m
    // Primary cross beams through center
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_half * 2.0, 1.2, 1.2))),
        MeshMaterial3d(beam_mat.clone()),
        Transform::from_xyz(0.0, beam_y, 0.0),
        InnerWorldElement,
        Name::new("CeilingCrossBeamX"),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 1.2, wall_half * 2.0))),
        MeshMaterial3d(beam_mat.clone()),
        Transform::from_xyz(0.0, beam_y, 0.0),
        InnerWorldElement,
        Name::new("CeilingCrossBeamZ"),
    ));
    // Secondary rafters
    for offset in [-19.0, 19.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(wall_half * 2.0, 0.8, 0.8))),
            MeshMaterial3d(beam_mat.clone()),
            Transform::from_xyz(0.0, beam_y, offset),
            InnerWorldElement,
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 0.8, wall_half * 2.0))),
            MeshMaterial3d(beam_mat.clone()),
            Transform::from_xyz(offset, beam_y, 0.0),
            InnerWorldElement,
        ));
    }

    // --- 5. ATMOSPHERIC CASTLE LIGHTING ---
    // Central overhead point light directly above the table
    commands.spawn((
        PointLight {
            intensity: 150_000.0,
            range: 35.0,
            color: Color::srgb(0.90, 0.94, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 8.5, 0.0),
        InnerWorldElement,
        Name::new("CentralTableLight"),
    ));
    // High ambient fill point light near the vaulted ceiling
    commands.spawn((
        PointLight {
            intensity: 80_000.0,
            range: 45.0,
            color: Color::srgb(0.75, 0.80, 0.95),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 18.0, 0.0),
        InnerWorldElement,
        Name::new("HighVaultFillLight"),
    ));
    // Directional rim/key light
    commands.spawn((
        DirectionalLight {
            illuminance: 1_800.0,
            shadows_enabled: true,
            color: Color::srgb(0.92, 0.94, 1.0),
            ..default()
        },
        Transform::from_xyz(10.0, 22.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        InnerWorldElement,
        Name::new("CastleSunShaft"),
    ));

    // Perimeter wall sconces / braziers
    let sconce_mesh = meshes.add(Cuboid::new(0.4, 0.6, 0.4));
    let sconce_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.6, 0.2),
        emissive: LinearRgba::new(2.0, 1.2, 0.4, 1.0),
        unlit: true,
        ..default()
    });
    let torch_positions = [
        Vec3::new(0.0, 5.0, -wall_half + 1.2),
        Vec3::new(0.0, 5.0, wall_half - 1.2),
        Vec3::new(-wall_half + 1.2, 5.0, 0.0),
        Vec3::new(wall_half - 1.2, 5.0, 0.0),
        Vec3::new(-20.0, 5.0, -wall_half + 1.2),
        Vec3::new(20.0, 5.0, -wall_half + 1.2),
        Vec3::new(-20.0, 5.0, wall_half - 1.2),
        Vec3::new(20.0, 5.0, wall_half - 1.2),
    ];
    for (i, pos) in torch_positions.iter().enumerate() {
        // Sconce bracket
        commands.spawn((
            Mesh3d(sconce_mesh.clone()),
            MeshMaterial3d(sconce_glow_mat.clone()),
            Transform::from_translation(*pos),
            InnerWorldElement,
            Name::new(format!("CastleBrazierMesh_{i}")),
        ));
        // Torch warm flame light
        commands.spawn((
            PointLight {
                intensity: 36_000.0,
                range: 22.0,
                color: Color::srgb(1.0, 0.68, 0.35),
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(*pos + Vec3::Y * 0.4),
            InnerWorldElement,
            Name::new(format!("CastleBrazierLight_{i}")),
        ));
    }

    // --- 6. ARCHETYPE SATELLITE CHAMBERS (R = 22m) ---
    for (chamber_index, spec) in council_chambers().iter().enumerate() {
        spawn_chamber(&mut commands, &mut meshes, &mut materials, chamber_index, spec);
    }

    // --- 7. HUD OVERLAY ---
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                bottom: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                max_width: Val::Px(780.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.88)),
            GlobalZIndex(920),
            InnerWorldElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(
                    "COUNCIL ROTUNDA\nWASD: Fly  •  Space: Up  •  Shift / C: Down  •  Mouse: Look 360°\nViewing Council Table & Stargate Portal  •  Esc: Return to Menu",
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
