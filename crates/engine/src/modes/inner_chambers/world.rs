use super::InnerChambersState;
use crate::chamber::boot::spawn_main_menu;
use crate::modes::ModeRegistry;
use bevy::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Loading), setup_inner_world)
            .add_systems(
                Update,
                rotate_chronos_exhibits.run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(OnEnter(InnerChambersState::Exiting), teardown_inner_world);
    }
}

#[derive(Component)]
pub struct InnerWorldElement;

#[derive(Component)]
pub struct InnerChambersHint;

#[derive(Component)]
pub struct ChronosExhibitTurntable {
    pub speed: f32,
}

fn rotate_chronos_exhibits(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ChronosExhibitTurntable)>,
) {
    let delta = time.delta_secs();
    for (mut transform, turntable) in &mut query {
        transform.rotate_y(turntable.speed * delta);
    }
}

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
    // Entity named "RotundaCouncilTable" so ritual visibility gates never hide it.
    // Child entity "Stargate_Portal" inside table.glb is auto-bound and animated by PortalPlugin.
    commands.spawn((
        SceneRoot(asset_server.load("scenes/table.glb#Scene0")),
        Transform::from_xyz(0.0, 2.34, 0.0).with_scale(Vec3::splat(2.6)),
        InnerWorldElement,
        Name::new("RotundaCouncilTable"),
    ));

    // Subtle glow light illuminating the stargate vortex disc
    commands.spawn((
        PointLight {
            intensity: 30_000.0,
            range: 12.0,
            color: Color::srgb(0.25, 0.75, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 2.7, 0.0),
        InnerWorldElement,
        Name::new("PortalDiscGlowLight"),
    ));

    // --- 2b. CHRONOSOPHIA ARTIFACT EXHIBITION PEDESTALS ---
    let pedestal_base_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.09, 0.095, 0.11),
        perceptual_roughness: 0.8,
        metallic: 0.1,
        ..default()
    });
    let pedestal_shaft_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.055, 0.07),
        perceptual_roughness: 0.35,
        metallic: 0.25,
        ..default()
    });
    let pedestal_cap_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.125, 0.15),
        perceptual_roughness: 0.5,
        metallic: 0.2,
        ..default()
    });
    let pedestal_gold_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.68, 0.28),
        perceptual_roughness: 0.3,
        metallic: 0.85,
        ..default()
    });
    let pedestal_cushion_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.07, 0.13),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    let pedestal_base_mesh = meshes.add(Cylinder::new(1.35, 0.25));
    let pedestal_shaft_mesh = meshes.add(Cylinder::new(0.95, 1.0));
    let pedestal_cap_mesh = meshes.add(Cylinder::new(1.20, 0.20));
    let pedestal_gold_mesh = meshes.add(Cylinder::new(1.22, 0.04));
    let pedestal_cushion_mesh = meshes.add(Cylinder::new(0.85, 0.05));

    struct ChronosExhibit {
        name: &'static str,
        asset_path: &'static str,
        pos: Vec3,
        scale: f32,
        light_color: Color,
        turntable_speed: f32,
    }

    let exhibits = [
        ChronosExhibit {
            name: "ChronosArtifact_CeramicTeapot",
            asset_path: "scenes/chronos_teapot.glb#Scene0",
            pos: Vec3::new(7.42, 0.0, 7.42),
            scale: 1.0,
            light_color: Color::srgb(1.0, 0.88, 0.65), // Warm porcelain amber
            turntable_speed: 0.35,
        },
        ChronosExhibit {
            name: "ChronosArtifact_AltarWeddingCake",
            asset_path: "scenes/chronos_cake.glb#Scene0",
            pos: Vec3::new(-7.42, 0.0, 7.42),
            scale: 1.0,
            light_color: Color::srgb(1.0, 0.75, 0.85), // Soft rose ivory
            turntable_speed: 0.30,
        },
        ChronosExhibit {
            name: "ChronosArtifact_GoldenPinecone",
            asset_path: "scenes/chronos_pinecone.glb#Scene0",
            pos: Vec3::new(-7.42, 0.0, -7.42),
            scale: 1.0,
            light_color: Color::srgb(1.0, 0.82, 0.35), // Deep gold
            turntable_speed: 0.35,
        },
        ChronosExhibit {
            name: "ChronosArtifact_EvergreenPine",
            asset_path: "scenes/chronos_pinetree.glb#Scene0",
            pos: Vec3::new(7.42, 0.0, -7.42),
            scale: 1.0,
            light_color: Color::srgb(0.55, 0.95, 0.65), // Verdant forest glow
            turntable_speed: 0.25,
        },
    ];

    for exhibit in exhibits.iter() {
        let p = exhibit.pos;

        // 1. Pedestal base stepped tier (y: 0.0 to 0.25)
        commands.spawn((
            Mesh3d(pedestal_base_mesh.clone()),
            MeshMaterial3d(pedestal_base_mat.clone()),
            Transform::from_xyz(p.x, 0.125, p.z),
            InnerWorldElement,
            Name::new(format!("{}_BasePlinth", exhibit.name)),
        ));

        // 2. Pedestal main column shaft (y: 0.25 to 1.25)
        commands.spawn((
            Mesh3d(pedestal_shaft_mesh.clone()),
            MeshMaterial3d(pedestal_shaft_mat.clone()),
            Transform::from_xyz(p.x, 0.75, p.z),
            InnerWorldElement,
            Name::new(format!("{}_Shaft", exhibit.name)),
        ));

        // 3. Pedestal upper capital (y: 1.25 to 1.45)
        commands.spawn((
            Mesh3d(pedestal_cap_mesh.clone()),
            MeshMaterial3d(pedestal_cap_mat.clone()),
            Transform::from_xyz(p.x, 1.35, p.z),
            InnerWorldElement,
            Name::new(format!("{}_Capital", exhibit.name)),
        ));

        // 4. Gold trim rim ring (y: 1.45 to 1.49)
        commands.spawn((
            Mesh3d(pedestal_gold_mesh.clone()),
            MeshMaterial3d(pedestal_gold_mat.clone()),
            Transform::from_xyz(p.x, 1.47, p.z),
            InnerWorldElement,
            Name::new(format!("{}_GoldTrim", exhibit.name)),
        ));

        // 5. Display velvet cushion (y: 1.49 to 1.54)
        commands.spawn((
            Mesh3d(pedestal_cushion_mesh.clone()),
            MeshMaterial3d(pedestal_cushion_mat.clone()),
            Transform::from_xyz(p.x, 1.515, p.z),
            InnerWorldElement,
            Name::new(format!("{}_Cushion", exhibit.name)),
        ));

        // 6. ChronoSophia Object model placed on cushion (y = 1.54)
        commands.spawn((
            SceneRoot(asset_server.load(exhibit.asset_path)),
            Transform::from_xyz(p.x, 1.54, p.z).with_scale(Vec3::splat(exhibit.scale)),
            ChronosExhibitTurntable {
                speed: exhibit.turntable_speed,
            },
            InnerWorldElement,
            Name::new(exhibit.name),
        ));

        // 7. Showcase underglow point light
        commands.spawn((
            PointLight {
                intensity: 22_000.0,
                range: 4.5,
                color: exhibit.light_color,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(p.x, 1.75, p.z),
            InnerWorldElement,
            Name::new(format!("{}_UnderglowLight", exhibit.name)),
        ));

        // 8. Overhead spotlight illuminating the artifact from above
        commands.spawn((
            PointLight {
                intensity: 65_000.0,
                range: 12.0,
                color: Color::srgb(1.0, 0.98, 0.94),
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(p.x, 5.2, p.z),
            InnerWorldElement,
            Name::new(format!("{}_Spotlight", exhibit.name)),
        ));
    }

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
            intensity: 160_000.0,
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

    // --- 6. HUD OVERLAY ---
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
                    "COUNCIL ROTUNDA  •  [STATUS: GROUND WALKING]\nWASD: Walk & Strafe  •  Space: Jump  •  Mouse: Look (Tilted Down)\nDouble-Jump + Hold Space (2s): Free Flight Mode  •  Esc: Menu",
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
