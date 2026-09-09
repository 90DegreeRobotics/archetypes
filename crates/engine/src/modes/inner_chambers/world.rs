use super::InnerChambersState;
use crate::chamber::boot::spawn_main_menu;
use crate::modes::ModeRegistry;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat};

/// The generated images are deliberately non-color/normal PBR inputs, not light-emitting
/// decals. Keeping them deterministic makes the chamber self-contained while the material
/// still travels through Bevy's normal-map lighting path.
const FLOOR_TEXTURE_SIZE: usize = 256;

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
    mut images: ResMut<Assets<Image>>,
    mut clear: ResMut<ClearColor>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
    asset_server: Res<AssetServer>,
) {
    clear.0 = Color::srgb(0.08, 0.09, 0.12);

    // --- 1. FLOOR & CENTRAL DAIS ---
    let (floor_albedo, floor_normal) = chamber_floor_textures();
    let floor_normal_handle = images.add(floor_normal);
    let floor_mat = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(floor_albedo)),
        normal_map_texture: Some(floor_normal_handle.clone()),
        base_color: Color::WHITE,
        perceptual_roughness: 0.72,
        metallic: 0.08,
        reflectance: 0.48,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(76.0, 76.0))),
        MeshMaterial3d(floor_mat.clone()),
        Transform::from_translation(Vec3::ZERO),
        InnerWorldElement,
        Name::new("CastleFloor"),
    ));

    // Physical stone courses with raised/recessed radial masonry relief
    let stone_course_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.22, 0.27),
        perceptual_roughness: 0.68,
        metallic: 0.10,
        reflectance: 0.45,
        normal_map_texture: Some(floor_normal_handle),
        ..default()
    });

    // Course 1: Dais border flagstones (r: 6.0 to 8.4m, 24 segmented blocks, height: 2.4cm)
    commands.spawn((
        Mesh3d(meshes.add(build_radial_flagstone_mesh(6.0, 8.4, 0.024, 24, 0.010))),
        MeshMaterial3d(stone_course_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        InnerWorldElement,
        Name::new("CouncilFloor_Course1_DaisBorder"),
    ));

    // Course 2: Middle rotunda flagstones (r: 8.54 to 12.0m, 36 segmented blocks, height: 0.020m)
    commands.spawn((
        Mesh3d(meshes.add(build_radial_flagstone_mesh(8.54, 12.0, 0.020, 36, 0.008))),
        MeshMaterial3d(stone_course_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        InnerWorldElement,
        Name::new("CouncilFloor_Course2_MiddleRotunda"),
    ));

    // Course 3: Outer rotunda flagstones (r: 12.16 to 17.0m, 48 segmented blocks, height: 0.016m)
    commands.spawn((
        Mesh3d(meshes.add(build_radial_flagstone_mesh(12.16, 17.0, 0.016, 48, 0.006))),
        MeshMaterial3d(stone_course_mat.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        InnerWorldElement,
        Name::new("CouncilFloor_Course3_OuterRotunda"),
    ));

    // Course 4: Perimeter ambulatory flagstones (r: 17.20 to 24.0m, 60 segmented blocks, height: 0.012m)
    commands.spawn((
        Mesh3d(meshes.add(build_radial_flagstone_mesh(17.20, 24.0, 0.012, 60, 0.005))),
        MeshMaterial3d(stone_course_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        InnerWorldElement,
        Name::new("CouncilFloor_Course4_Ambulatory"),
    ));

    // Raised circular stone dais beneath the Council Table
    let dais_step_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.24, 0.28),
        perceptual_roughness: 0.72,
        metallic: 0.1,
        ..default()
    });
    let dais_top_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.30, 0.35),
        perceptual_roughness: 0.60,
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
    // Scaled to 60% of original size (scale 1.56). Feet authored at local z = -0.784.
    // Resting on dais top (y = 0.30m) requires origin y = 0.30 + (0.784 * 1.56) = 1.523m.
    // Entity named "RotundaCouncilTable" so ritual visibility gates never hide it.
    // Child entity "Stargate_Portal" inside table.glb is auto-bound and animated by PortalPlugin.
    commands.spawn((
        SceneRoot(asset_server.load("scenes/table.glb#Scene0")),
        Transform::from_xyz(0.0, 1.523, 0.0).with_scale(Vec3::splat(1.56)),
        InnerWorldElement,
        Name::new("RotundaCouncilTable"),
    ));

    // Subtle glow light illuminating the stargate vortex disc
    commands.spawn((
        PointLight {
            intensity: 22_000.0,
            range: 9.0,
            color: Color::srgb(0.25, 0.75, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.74, 0.0),
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

    // --- 2c. EMBODIED ARCHETYPE COUNCIL FIGURES (FLOOR EXHIBITION AT PLAYER EYE LEVEL) ---
    // All characters stand directly on the polished stone floor (y = 0.0) at player eye height (2.85m).
    // Arranged in a grand exhibition crescent in the southern rotunda court, spaced ~5.7m apart,
    // allowing the player to freely walk up to, inspect, and circle around each figure individually.
    struct ArchetypeFigure {
        name: &'static str,
        asset_path: &'static str,
        pos: Vec3,
        scale: f32,
        rotation_y: f32,
        key_pos: Vec3,
        key_color: Color,
        key_intensity: f32,
        rim_pos: Vec3,
        rim_color: Color,
        rim_intensity: f32,
        core_pos: Vec3,
        core_color: Color,
        core_intensity: f32,
    }

    let figures = [
        // 1. Sentinel — Tactical obsidian steel & emerald/teal vigilance (left flank)
        ArchetypeFigure {
            name: "SentinelArchetypeFigure",
            asset_path: "scenes/sentinel.glb#Scene0",
            pos: Vec3::new(-9.6, 0.0, 13.5),
            scale: 1.614,
            rotation_y: 2.45, // Angled inward facing the central promenade
            key_pos: Vec3::new(-8.2, 4.2, 15.3),
            key_color: Color::srgb(0.90, 0.96, 1.0),
            key_intensity: 85_000.0,
            rim_pos: Vec3::new(-11.2, 3.8, 11.7),
            rim_color: Color::srgb(0.35, 0.85, 0.70),
            rim_intensity: 55_000.0,
            core_pos: Vec3::new(-9.4, 2.3, 13.8),
            core_color: Color::srgb(0.30, 0.90, 0.80),
            core_intensity: 16_000.0,
        },
        // 2. Aura — Radiant celestial gold & solar amber (mid left)
        ArchetypeFigure {
            name: "AuraArchetypeFigure",
            asset_path: "scenes/aura.glb#Scene0",
            pos: Vec3::new(-5.0, 0.0, 10.2),
            scale: 1.614,
            rotation_y: 2.75, // Welcoming gaze toward approach
            key_pos: Vec3::new(-3.8, 4.2, 12.0),
            key_color: Color::srgb(1.0, 0.88, 0.65),
            key_intensity: 90_000.0,
            rim_pos: Vec3::new(-6.5, 3.8, 8.5),
            rim_color: Color::srgb(0.80, 0.95, 1.0),
            rim_intensity: 55_000.0,
            core_pos: Vec3::new(-4.9, 2.2, 10.5),
            core_color: Color::srgb(1.0, 0.75, 0.35),
            core_intensity: 18_000.0,
        },
        // 3. Empath — Iridescent synth & psychic resonance (center)
        ArchetypeFigure {
            name: "EmpathArchetypeFigure",
            asset_path: "scenes/empath.glb#Scene0",
            pos: Vec3::new(0.0, 0.0, 7.5),
            scale: 1.60,
            rotation_y: std::f32::consts::PI, // Facing player entrance
            key_pos: Vec3::new(1.8, 4.2, 9.9),
            key_color: Color::srgb(1.0, 0.94, 0.96),
            key_intensity: 95_000.0,
            rim_pos: Vec3::new(-2.2, 3.8, 5.1),
            rim_color: Color::srgb(0.70, 0.45, 1.0),
            rim_intensity: 60_000.0,
            core_pos: Vec3::new(0.0, 2.2, 7.8),
            core_color: Color::srgb(0.50, 0.85, 1.0),
            core_intensity: 18_000.0,
        },
        // 4. Oracle — Astral twilight & cosmic sapphire (mid right)
        ArchetypeFigure {
            name: "OracleArchetypeFigure",
            asset_path: "scenes/oracle.glb#Scene0",
            pos: Vec3::new(5.0, 0.0, 10.2),
            scale: 1.602,
            rotation_y: -2.75, // Welcoming gaze toward approach
            key_pos: Vec3::new(3.8, 4.2, 12.0),
            key_color: Color::srgb(0.80, 0.88, 1.0),
            key_intensity: 90_000.0,
            rim_pos: Vec3::new(6.5, 3.8, 8.5),
            rim_color: Color::srgb(0.65, 0.40, 1.0),
            rim_intensity: 55_000.0,
            core_pos: Vec3::new(4.9, 2.3, 10.5),
            core_color: Color::srgb(0.55, 0.65, 1.0),
            core_intensity: 18_000.0,
        },
        // 5. Nebula Jester — Cosmic velvet & electric neon magenta (right flank)
        ArchetypeFigure {
            name: "NebulaJesterArchetypeFigure",
            asset_path: "scenes/nebula_jester.glb#Scene0",
            pos: Vec3::new(9.6, 0.0, 13.5),
            scale: 1.615,
            rotation_y: -2.45, // Angled inward facing the central promenade
            key_pos: Vec3::new(8.2, 4.2, 15.3),
            key_color: Color::srgb(1.0, 0.78, 0.92),
            key_intensity: 85_000.0,
            rim_pos: Vec3::new(11.2, 3.8, 11.7),
            rim_color: Color::srgb(0.95, 0.30, 0.85),
            rim_intensity: 55_000.0,
            core_pos: Vec3::new(9.4, 2.2, 13.8),
            core_color: Color::srgb(0.90, 0.40, 1.0),
            core_intensity: 18_000.0,
        },
    ];

    for fig in figures.iter() {
        // Character Figure
        commands.spawn((
            SceneRoot(asset_server.load(fig.asset_path)),
            Transform::from_translation(fig.pos)
                .with_rotation(Quat::from_rotation_y(fig.rotation_y))
                .with_scale(Vec3::splat(fig.scale)),
            InnerWorldElement,
            Name::new(fig.name),
        ));

        // Tailored Key Light
        commands.spawn((
            PointLight {
                intensity: fig.key_intensity,
                range: 14.0,
                color: fig.key_color,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(fig.key_pos),
            InnerWorldElement,
            Name::new(format!("{}_KeyLight", fig.name)),
        ));

        // Tailored Rim / Kicker Light
        commands.spawn((
            PointLight {
                intensity: fig.rim_intensity,
                range: 11.0,
                color: fig.rim_color,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(fig.rim_pos),
            InnerWorldElement,
            Name::new(format!("{}_RimLight", fig.name)),
        ));

        // Thematic Core / Visor Glow
        commands.spawn((
            PointLight {
                intensity: fig.core_intensity,
                range: 5.0,
                color: fig.core_color,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(fig.core_pos),
            InnerWorldElement,
            Name::new(format!("{}_CoreGlow", fig.name)),
        ));
    }

    // --- 3. ENCLOSING CASTLE WALLS & BUTTRESS PILLARS ---
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.25, 0.29),
        perceptual_roughness: 0.75,
        ..default()
    });
    let pillar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.19, 0.20, 0.24),
        perceptual_roughness: 0.80,
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
        base_color: Color::srgb(0.18, 0.19, 0.23),
        perceptual_roughness: 0.85,
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
        base_color: Color::srgb(0.13, 0.13, 0.16),
        metallic: 0.35,
        perceptual_roughness: 0.65,
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
            intensity: 240_000.0,
            range: 50.0,
            color: Color::srgb(0.92, 0.95, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 0.0),
        InnerWorldElement,
        Name::new("CentralTableLight"),
    ));
    // High ambient fill point light near the vaulted ceiling illuminating rafters and entire hall
    commands.spawn((
        PointLight {
            intensity: 450_000.0,
            range: 95.0,
            color: Color::srgb(0.85, 0.88, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0),
        InnerWorldElement,
        Name::new("HighVaultFillLight"),
    ));
    // Directional sunlight shaft cutting through the rotunda
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            color: Color::srgb(0.95, 0.96, 1.0),
            ..default()
        },
        Transform::from_xyz(14.0, 26.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        InnerWorldElement,
        Name::new("CastleSunShaft"),
    ));

    // Upper perimeter wall wash lights to reveal masonry height and room scale
    let upper_wall_positions = [
        Vec3::new(0.0, 16.0, -wall_half + 4.0),
        Vec3::new(-20.0, 16.0, -wall_half + 4.0),
        Vec3::new(20.0, 16.0, -wall_half + 4.0),
        Vec3::new(0.0, 16.0, wall_half - 4.0),
        Vec3::new(-20.0, 16.0, wall_half - 4.0),
        Vec3::new(20.0, 16.0, wall_half - 4.0),
        Vec3::new(-wall_half + 4.0, 16.0, 0.0),
        Vec3::new(-wall_half + 4.0, 16.0, -18.0),
        Vec3::new(-wall_half + 4.0, 16.0, 18.0),
        Vec3::new(wall_half - 4.0, 16.0, 0.0),
        Vec3::new(wall_half - 4.0, 16.0, -18.0),
        Vec3::new(wall_half - 4.0, 16.0, 18.0),
    ];
    for (i, pos) in upper_wall_positions.iter().enumerate() {
        commands.spawn((
            PointLight {
                intensity: 110_000.0,
                range: 45.0,
                color: Color::srgb(0.88, 0.90, 0.98),
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(*pos),
            InnerWorldElement,
            Name::new(format!("UpperWallWashLight_{i}")),
        ));
    }

    // Lower perimeter wall sconces / braziers
    let sconce_mesh = meshes.add(Cuboid::new(0.4, 0.6, 0.4));
    let sconce_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.6, 0.2),
        emissive: LinearRgba::new(2.5, 1.5, 0.5, 1.0),
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
                intensity: 75_000.0,
                range: 32.0,
                color: Color::srgb(1.0, 0.72, 0.40),
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
                    "COUNCIL ROTUNDA  •  [STATUS: GROUND WALKING]\nWASD: Move & Strafe  •  Space: Jump (Double-Tap: Fly)  •  Mouse: Look  •  Esc: Menu",
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

/// Produce a dark basalt tile albedo and its matching tangent-space normal map.
///
/// The mortar grooves are recessed and the stones have restrained grain, so point and
/// directional lights move across visible relief instead of merely brightening a flat color.
fn chamber_floor_textures() -> (Image, Image) {
    let side = FLOOR_TEXTURE_SIZE;
    let mut albedo = vec![0_u8; side * side * 4];
    let mut heights = vec![0.0_f32; side * side];

    for y in 0..side {
        for x in 0..side {
            let fx = x as f32 / side as f32;
            let fy = y as f32 / side as f32;
            // Eight by eight laid basalt tiles, with offset courses and narrow mortar.
            let course = (fy * 8.0).floor() as i32;
            let shifted_x = (fx * 8.0 + if course % 2 == 0 { 0.0 } else { 0.5 }).fract();
            let tile_y = (fy * 8.0).fract();
            let edge = shifted_x.min(1.0 - shifted_x).min(tile_y.min(1.0 - tile_y));
            let mortar = edge < 0.028;
            let grain = ((fx * 101.0).sin() * (fy * 79.0).cos()) * 0.045
                + ((fx * 313.0 + fy * 197.0).sin()) * 0.018;
            let height = if mortar { -0.42 } else { 0.26 + grain };
            heights[y * side + x] = height;

            let (r, g, b) = if mortar {
                (20_u8, 23_u8, 29_u8)
            } else {
                let shade = (42.0 + grain * 46.0).clamp(0.0, 255.0) as u8;
                (shade, shade.saturating_add(4), shade.saturating_add(12))
            };
            let index = (y * side + x) * 4;
            albedo[index..index + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }

    let mut normal = vec![0_u8; side * side * 4];
    for y in 0..side {
        for x in 0..side {
            let sample = |sx: usize, sy: usize| heights[sy * side + sx];
            let left = sample((x + side - 1) % side, y);
            let right = sample((x + 1) % side, y);
            let up = sample(x, (y + side - 1) % side);
            let down = sample(x, (y + 1) % side);
            let n = Vec3::new((left - right) * 1.65, (up - down) * 1.65, 1.0).normalize();
            let index = (y * side + x) * 4;
            normal[index..index + 4].copy_from_slice(&[
                ((n.x * 0.5 + 0.5) * 255.0) as u8,
                ((n.y * 0.5 + 0.5) * 255.0) as u8,
                ((n.z * 0.5 + 0.5) * 255.0) as u8,
                255,
            ]);
        }
    }

    let size = Extent3d {
        width: side as u32,
        height: side as u32,
        depth_or_array_layers: 1,
    };
    (
        Image::new(
            size,
            TextureDimension::D2,
            albedo,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ),
        Image::new(
            size,
            TextureDimension::D2,
            normal,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        ),
    )
}

/// Constructs an annular segmented flagstone paving course with real physical 3D mesh relief.
/// Generates raised stone tops, vertical joint side faces, and radial mortar channels.
fn build_radial_flagstone_mesh(
    inner_r: f32,
    outer_r: f32,
    height: f32,
    segments: usize,
    gap_rad: f32,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    let push_quad = |p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, n: Vec3, pos: &mut Vec<[f32; 3]>, norm: &mut Vec<[f32; 3]>, uv: &mut Vec<[f32; 2]>| {
        // Triangle 1: p0, p1, p2
        pos.push([p0.x, p0.y, p0.z]);
        pos.push([p1.x, p1.y, p1.z]);
        pos.push([p2.x, p2.y, p2.z]);
        norm.push([n.x, n.y, n.z]);
        norm.push([n.x, n.y, n.z]);
        norm.push([n.x, n.y, n.z]);
        uv.push([p0.x * 0.1, p0.z * 0.1]);
        uv.push([p1.x * 0.1, p1.z * 0.1]);
        uv.push([p2.x * 0.1, p2.z * 0.1]);

        // Triangle 2: p0, p2, p3
        pos.push([p0.x, p0.y, p0.z]);
        pos.push([p2.x, p2.y, p2.z]);
        pos.push([p3.x, p3.y, p3.z]);
        norm.push([n.x, n.y, n.z]);
        norm.push([n.x, n.y, n.z]);
        norm.push([n.x, n.y, n.z]);
        uv.push([p0.x * 0.1, p0.z * 0.1]);
        uv.push([p2.x * 0.1, p2.z * 0.1]);
        uv.push([p3.x * 0.1, p3.z * 0.1]);
    };

    let two_pi = std::f32::consts::PI * 2.0;
    let step = two_pi / segments as f32;

    for i in 0..segments {
        let t0 = i as f32 * step + gap_rad * 0.5;
        let t1 = (i + 1) as f32 * step - gap_rad * 0.5;

        let cos0 = t0.cos();
        let sin0 = t0.sin();
        let cos1 = t1.cos();
        let sin1 = t1.sin();

        // 4 top vertices (raised)
        let p0 = Vec3::new(inner_r * cos0, height, inner_r * sin0);
        let p1 = Vec3::new(outer_r * cos0, height, outer_r * sin0);
        let p2 = Vec3::new(outer_r * cos1, height, outer_r * sin1);
        let p3 = Vec3::new(inner_r * cos1, height, inner_r * sin1);

        // 4 base vertices (floor plane)
        let b0 = Vec3::new(inner_r * cos0, 0.0, inner_r * sin0);
        let b1 = Vec3::new(outer_r * cos0, 0.0, outer_r * sin0);
        let b2 = Vec3::new(outer_r * cos1, 0.0, outer_r * sin1);
        let b3 = Vec3::new(inner_r * cos1, 0.0, inner_r * sin1);

        // Top stone surface
        push_quad(p0, p1, p2, p3, Vec3::Y, &mut positions, &mut normals, &mut uvs);

        // Outer rim face (radial outward)
        let mid_t = (t0 + t1) * 0.5;
        let outer_norm = Vec3::new(mid_t.cos(), 0.0, mid_t.sin());
        push_quad(p1, b1, b2, p2, outer_norm, &mut positions, &mut normals, &mut uvs);

        // Inner rim face (radial inward)
        let inner_norm = -outer_norm;
        push_quad(p3, b3, b0, p0, inner_norm, &mut positions, &mut normals, &mut uvs);

        // Start joint face (facing -theta)
        let start_norm = Vec3::new(sin0, 0.0, -cos0);
        push_quad(p1, b1, b0, p0, start_norm, &mut positions, &mut normals, &mut uvs);

        // End joint face (facing +theta)
        let end_norm = Vec3::new(-sin1, 0.0, cos1);
        push_quad(p3, b3, b2, p2, end_norm, &mut positions, &mut normals, &mut uvs);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chamber_floor_has_a_linear_normal_map_for_lit_relief() {
        let (albedo, normal) = chamber_floor_textures();
        assert_eq!(albedo.texture_descriptor.size.width, FLOOR_TEXTURE_SIZE as u32);
        assert_eq!(normal.texture_descriptor.size.height, FLOOR_TEXTURE_SIZE as u32);
        assert_eq!(albedo.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb);
        // Normal maps are data, never gamma-corrected color images.
        assert_eq!(normal.texture_descriptor.format, TextureFormat::Rgba8Unorm);
    }

    #[test]
    fn chamber_floor_radial_flagstones_have_physical_3d_relief() {
        let mesh = build_radial_flagstone_mesh(6.0, 8.4, 0.024, 24, 0.010);
        let pos_attr = mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("positions present");
        let positions = pos_attr.as_float3().expect("float3 positions");
        assert!(!positions.is_empty(), "mesh must have vertices");
        // 24 segments * 5 quads * 6 vertices per quad = 720 vertices
        assert_eq!(positions.len(), 24 * 5 * 6);

        // Confirm physical height relief: contains both raised stone heights and base plane vertices
        let has_raised = positions.iter().any(|p| (p[1] - 0.024).abs() < 1e-4);
        let has_base = positions.iter().any(|p| p[1].abs() < 1e-4);
        assert!(has_raised, "flagstone mesh must have raised top vertices");
        assert!(has_base, "flagstone mesh must have base groove vertices");
    }
}
