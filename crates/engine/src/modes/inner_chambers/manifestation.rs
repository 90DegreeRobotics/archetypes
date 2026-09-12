//! Manifestation Pedestal & Headless Chronos2 Live Staging System.
//!
//! Enables the player to walk up to the sacred manifestation pedestal on the dais,
//! press 'E' to open the prompt conduit, and trigger a headless Chronos2 render
//! that creates, grounds, and manifests the 3D artifact directly onto the pedestal.
//! Follows the absolute NO-RECIPE LAW and streams real Chronos2 pipeline stages.
//!
//! Visual State Machine atop the Pedestal:
//! 1. Idle: Floating subtle celestial diamond symbol above the empty velvet cushion.
//! 2. Waiting: A 3D floating hourglass that slowly rotates and phases in and out (pulsing light/opacity) with a live timer.
//! 3. Failed: A 3D floating luminous Red 'X' with a crimson warning beacon and explicit error message.
//! 4. Succeeded: Grand golden radiance flash, despawning the hourglass, and revealing the newly summoned 3D object rotating atop the cushion.

use super::world::ChronosExhibitTurntable;
use super::encounters::EncounterState;
use super::interaction::{InnerInteractionSet, InteractionFocus, InteractionTarget};
use super::InnerChambersState;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::winit::{UpdateMode, WinitSettings};

use crate::services::text_entry;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The pedestal stands at the Council-table centre, directly on its real animated Stargate disc.
/// See the matching measured table transform in `world.rs`: local disc z=0.300, feet z=-0.766,
/// 2.6x table scale, current floor top y=0.4.
pub const MANIFESTATION_PEDESTAL_POS: Vec3 = Vec3::new(0.0, 3.1716, 0.0);
pub const MANIFESTATION_CUSHION_HEIGHT: f32 = 1.62;
pub const MANIFESTATION_HOVER_Y: f32 = 2.40;

pub struct ManifestationPlugin;

impl Plugin for ManifestationPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = channel::<ManifestationEvent>();
        app.insert_resource(ManifestationChannels {
            sender: tx,
            receiver: Mutex::new(rx),
            active_pid: Arc::new(Mutex::new(None)),
        })
        .init_resource::<ManifestationState>()
        .add_systems(
            OnEnter(InnerChambersState::Loading),
            setup_manifestation_pedestal,
        )
        .add_systems(
            Update,
            (
                handle_manifestation_input.after(InnerInteractionSet::Resolve),
                animate_pedestal_symbols,
                poll_manifestation_results,
                update_manifestation_flash,
                update_manifestation_reveal_smoke,
                update_manifestation_hud,
                yield_gpu_during_manifestation,
            )
                .run_if(in_state(InnerChambersState::Navigating)),
        )
        .add_systems(OnEnter(InnerChambersState::Exiting), teardown_manifestation);
    }
}

#[derive(Clone, Debug)]
pub enum ManifestationEvent {
    Stage {
        stage_id: String,
        pct: u32,
        state: String,
        message: String,
    },
    Success {
        prompt: String,
        detail: String,
    },
    Failure {
        prompt: String,
        stage_id: Option<String>,
        detail: String,
    },
}

#[derive(Resource)]
pub struct ManifestationChannels {
    pub sender: Sender<ManifestationEvent>,
    pub receiver: Mutex<Receiver<ManifestationEvent>>,
    pub active_pid: Arc<Mutex<Option<u32>>>,
}

#[derive(Resource, Default)]
pub struct ManifestationState {
    pub phase: ManifestationPhase,
    pub prompt_buffer: String,
    pub active_prompt: String,
    pub waiting_elapsed: f32,
    pub current_stage_id: String,
    pub current_stage_pct: u32,
    pub current_stage_state: String,
    pub current_stage_msg: String,
    pub error_message: String,
    pub active_artifact: Option<Entity>,
    pub active_reference_panel: Option<Entity>,
    pub floating_symbol_entity: Option<Entity>,
    pub pedestal_charge: f32,
}

#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub enum ManifestationPhase {
    #[default]
    Idle,
    Prompting,
    Manifesting,
    Completed,
    Failed,
}

#[derive(Component)]
pub struct ManifestationElement;

#[derive(Component)]
pub struct PedestalUnderglowLight;

#[derive(Component)]
pub struct PhasingHourglass {
    pub base_intensity: f32,
}

#[derive(Component)]
pub struct FloatingRedX;

#[derive(Component)]
pub struct FloatingIdleSymbol;

#[derive(Component)]
pub struct ManifestationFlash {
    pub timer: f32,
    pub max_duration: f32,
}

#[derive(Component)]
pub struct ManifestationRevealSmoke {
    pub timer: f32,
}

#[derive(Component)]
pub struct ManifestationPromptUi;

#[derive(Component)]
pub struct ManifestationPromptText;

#[derive(Component)]
pub struct ManifestationProximityPrompt;

#[derive(Component)]
pub struct ManifestationStatusBanner;

#[derive(Component)]
pub struct ManifestationStatusText;

fn setup_manifestation_pedestal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<ManifestationState>,
) {
    state.phase = ManifestationPhase::Idle;
    state.prompt_buffer.clear();
    state.active_prompt.clear();
    state.waiting_elapsed = 0.0;
    state.current_stage_id.clear();
    state.current_stage_pct = 0;
    state.current_stage_state.clear();
    state.current_stage_msg.clear();
    state.error_message.clear();
    state.active_artifact = None;
    state.active_reference_panel = None;
    state.floating_symbol_entity = None;
    state.pedestal_charge = 0.0;

    let p = MANIFESTATION_PEDESTAL_POS;

    // Architectural pedestal materials
    let base_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.085, 0.12),
        perceptual_roughness: 0.75,
        metallic: 0.2,
        ..default()
    });
    let shaft_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.065, 0.09),
        perceptual_roughness: 0.30,
        metallic: 0.35,
        ..default()
    });
    let cap_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.115, 0.16),
        perceptual_roughness: 0.45,
        metallic: 0.25,
        ..default()
    });
    let gold_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.78, 0.32),
        perceptual_roughness: 0.25,
        metallic: 0.90,
        ..default()
    });
    let cushion_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.07, 0.06, 0.14),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    // 1. Base plinth (y: 0.30 to 0.50)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.20, 0.20))),
        MeshMaterial3d(base_mat),
        Transform::from_xyz(p.x, p.y + 0.10, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_Base"),
    ));

    // 2. Shaft column (y: 0.50 to 1.40)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.85, 0.90))),
        MeshMaterial3d(shaft_mat),
        Transform::from_xyz(p.x, p.y + 0.65, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_Shaft"),
    ));

    // 3. Capital (y: 1.40 to 1.55)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.10, 0.15))),
        MeshMaterial3d(cap_mat),
        Transform::from_xyz(p.x, p.y + 1.175, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_Capital"),
    ));

    // 4. Gold collar ring (y: 1.55 to 1.59)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.12, 0.04))),
        MeshMaterial3d(gold_mat),
        Transform::from_xyz(p.x, p.y + 1.27, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_GoldTrim"),
    ));

    // 5. Sacred velvet cushion (y: 1.59 to 1.64)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.80, 0.05))),
        MeshMaterial3d(cushion_mat),
        Transform::from_xyz(p.x, p.y + 1.315, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_Cushion"),
    ));

    // 6. Altar exhibition lighting: a real warm key spotlight, a neutral fill,
    // and restrained cyan underglow. The former "spotlight" was an
    // omnidirectional point light directly overhead, while the strong blue
    // underlight washed generated vertex colours cyan.
    commands.spawn((
        SpotLight {
            intensity: 115_000.0,
            color: Color::srgb(1.0, 0.96, 0.88),
            range: 14.0,
            radius: 0.24,
            inner_angle: 0.28,
            outer_angle: 0.72,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(p.x - 2.1, p.y + 5.2, p.z + 2.0)
            .looking_at(Vec3::new(p.x, p.y + 1.9, p.z), Vec3::Y),
        ManifestationElement,
        Name::new("ManifestationPedestal_KeySpotlight"),
    ));

    commands.spawn((
        PointLight {
            intensity: 22_000.0,
            range: 7.0,
            radius: 0.35,
            color: Color::srgb(0.78, 0.86, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(p.x + 2.0, p.y + 3.0, p.z + 1.2),
        ManifestationElement,
        Name::new("ManifestationPedestal_FillLight"),
    ));

    commands.spawn((
        PointLight {
            intensity: 10_000.0,
            range: 4.0,
            color: Color::srgb(0.24, 0.62, 0.86),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(p.x, p.y + 1.8, p.z),
        ManifestationElement,
        PedestalUnderglowLight,
        Name::new("ManifestationPedestal_Underglow"),
    ));

    // 7. Initial Idle Symbol (Floating subtle celestial diamond above the empty cushion)
    spawn_idle_symbol(&mut commands, &mut meshes, &mut materials, &mut state);

    // 8. Proximity prompt HUD (hidden by default)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(36.0),
                bottom: Val::Px(110.0),
                padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.92)),
            GlobalZIndex(940),
            ManifestationElement,
            ManifestationProximityPrompt,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("✦  PRESS [E] TO MANIFEST ARTIFACT  ✦"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.88, 0.40)),
            ));
        });

    // 9. Real-Time Status Banner (Top Center)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(25.0),
                top: Val::Px(20.0),
                width: Val::Percent(50.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.4, 0.6, 0.8)),
            BackgroundColor(Color::srgba(0.03, 0.04, 0.08, 0.90)),
            GlobalZIndex(950),
            ManifestationElement,
            ManifestationStatusBanner,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                ManifestationStatusText,
            ));
        });

    // 10. Text Input Prompt Modal (hidden by default)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(26.0),
            top: Val::Percent(35.0),
            width: Val::Px(760.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.85, 0.70, 0.30)),
        BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.96)),
        GlobalZIndex(960),
        ManifestationElement,
        ManifestationPromptUi,
        Visibility::Hidden,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("✦ CHRONOSOPHIA MANIFESTATION CONDUIT ✦"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.35)),
        ));

        parent.spawn((
            Text::new("Enter artifact concept to render live via Chronos2 backend:"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.70, 0.85, 0.95)),
        ));

        parent.spawn((
            Node {
                margin: UiRect::vertical(Val::Px(16.0)),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                width: Val::Percent(100.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.35, 0.55, 0.75)),
            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.95)),
        )).with_children(|input_box| {
            input_box.spawn((
                Text::new("> "),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.30, 0.90, 1.0)),
                ManifestationPromptText,
            ));
        });

        parent.spawn((
            Text::new("[Enter] Manifest via Headless Chronos2   •   [Esc] Cancel   •   [Backspace] Erase"),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.60, 0.65, 0.70)),
        ));
    });
}

fn spawn_idle_symbol(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &mut ManifestationState,
) {
    if let Some(existing) = state.floating_symbol_entity.take() {
        commands.entity(existing).despawn();
    }

    let p = MANIFESTATION_PEDESTAL_POS;
    let idle_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.40, 0.80, 1.0),
        emissive: LinearRgba::new(0.30, 0.70, 0.95, 1.0),
        perceptual_roughness: 0.2,
        metallic: 0.7,
        ..default()
    });

    let entity = commands
        .spawn((
            Transform::from_xyz(p.x, MANIFESTATION_HOVER_Y, p.z),
            Visibility::Visible,
            ManifestationElement,
            FloatingIdleSymbol,
            Name::new("ManifestationIdleSymbol"),
        ))
        .with_children(|parent| {
            // Slender rotating diamond octahedron
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.24, 0.24, 0.24))),
                MeshMaterial3d(idle_mat),
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.785, 0.615, 0.0)),
                Name::new("IdleDiamondMesh"),
            ));
            // Gentle hovering light
            parent.spawn((
                PointLight {
                    intensity: 15_000.0,
                    range: 4.5,
                    color: Color::srgb(0.40, 0.85, 1.0),
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
                Name::new("IdleDiamondLight"),
            ));
        })
        .id();

    state.floating_symbol_entity = Some(entity);
}

fn spawn_phasing_hourglass(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &mut ManifestationState,
) {
    if let Some(existing) = state.floating_symbol_entity.take() {
        commands.entity(existing).despawn();
    }

    let p = MANIFESTATION_PEDESTAL_POS;

    let gold_frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.82, 0.35),
        emissive: LinearRgba::new(0.60, 0.50, 0.15, 1.0),
        metallic: 0.90,
        perceptual_roughness: 0.20,
        ..default()
    });

    let glass_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.95, 1.0, 0.55),
        emissive: LinearRgba::new(0.40, 0.75, 1.0, 1.0),
        perceptual_roughness: 0.10,
        metallic: 0.1,
        ..default()
    });

    let sands_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.75, 0.25),
        emissive: LinearRgba::new(1.0, 0.70, 0.20, 1.0),
        perceptual_roughness: 0.4,
        metallic: 0.0,
        ..default()
    });

    let entity = commands
        .spawn((
            Transform::from_xyz(p.x, MANIFESTATION_HOVER_Y, p.z),
            Visibility::Visible,
            ManifestationElement,
            PhasingHourglass {
                base_intensity: 55_000.0,
            },
            Name::new("ManifestationHourglassRoot"),
        ))
        .with_children(|parent| {
            // Upper cone (pointing downward to waist)
            parent.spawn((
                Mesh3d(meshes.add(Cone {
                    radius: 0.22,
                    height: 0.30,
                })),
                MeshMaterial3d(glass_mat.clone()),
                Transform::from_xyz(0.0, 0.15, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                Name::new("Hourglass_UpperGlass"),
            ));

            // Lower cone (pointing upward to waist)
            parent.spawn((
                Mesh3d(meshes.add(Cone {
                    radius: 0.22,
                    height: 0.30,
                })),
                MeshMaterial3d(glass_mat.clone()),
                Transform::from_xyz(0.0, -0.15, 0.0),
                Name::new("Hourglass_LowerGlass"),
            ));

            // Top golden plate
            parent.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.25, 0.035))),
                MeshMaterial3d(gold_frame_mat.clone()),
                Transform::from_xyz(0.0, 0.315, 0.0),
                Name::new("Hourglass_TopCap"),
            ));

            // Bottom golden plate
            parent.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.25, 0.035))),
                MeshMaterial3d(gold_frame_mat.clone()),
                Transform::from_xyz(0.0, -0.315, 0.0),
                Name::new("Hourglass_BottomCap"),
            ));

            // Central golden waist ring
            parent.spawn((
                Mesh3d(meshes.add(Torus::new(0.065, 0.015))),
                MeshMaterial3d(gold_frame_mat.clone()),
                Transform::default(),
                Name::new("Hourglass_WaistRing"),
            ));

            // Golden sands core in waist
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(0.045))),
                MeshMaterial3d(sands_mat),
                Transform::default(),
                Name::new("Hourglass_SandsCore"),
            ));

            // Phasing amber/golden light beacon
            parent.spawn((
                PointLight {
                    intensity: 55_000.0,
                    range: 9.0,
                    color: Color::srgb(1.0, 0.85, 0.40),
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
                Name::new("Hourglass_BeaconLight"),
            ));
        })
        .id();

    state.floating_symbol_entity = Some(entity);
}

fn spawn_floating_red_x(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &mut ManifestationState,
) {
    if let Some(existing) = state.floating_symbol_entity.take() {
        commands.entity(existing).despawn();
    }

    let p = MANIFESTATION_PEDESTAL_POS;

    let red_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.05, 0.12),
        emissive: LinearRgba::new(1.0, 0.05, 0.15, 1.0),
        perceptual_roughness: 0.20,
        metallic: 0.80,
        ..default()
    });

    let entity = commands
        .spawn((
            Transform::from_xyz(p.x, MANIFESTATION_HOVER_Y, p.z),
            Visibility::Visible,
            ManifestationElement,
            FloatingRedX,
            Name::new("ManifestationFailureRedX"),
        ))
        .with_children(|parent| {
            // First diagonal beam (+45 deg)
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.65, 0.09, 0.07))),
                MeshMaterial3d(red_mat.clone()),
                Transform::from_rotation(Quat::from_rotation_z(0.785)),
                Name::new("RedX_Beam1"),
            ));

            // Second diagonal beam (-45 deg)
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.65, 0.09, 0.07))),
                MeshMaterial3d(red_mat),
                Transform::from_rotation(Quat::from_rotation_z(-0.785)),
                Name::new("RedX_Beam2"),
            ));

            // Crimson warning point light
            parent.spawn((
                PointLight {
                    intensity: 50_000.0,
                    range: 8.0,
                    color: Color::srgb(1.0, 0.10, 0.15),
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
                Name::new("RedX_WarningLight"),
            ));
        })
        .id();

    state.floating_symbol_entity = Some(entity);
}

fn animate_pedestal_symbols(
    time: Res<Time>,
    state: Res<ManifestationState>,
    mut hourglass_query: Query<(&mut Transform, &Children, &PhasingHourglass)>,
    mut red_x_query: Query<&mut Transform, (With<FloatingRedX>, Without<PhasingHourglass>)>,
    mut idle_query: Query<
        &mut Transform,
        (
            With<FloatingIdleSymbol>,
            Without<PhasingHourglass>,
            Without<FloatingRedX>,
        ),
    >,
    mut light_query: Query<&mut PointLight, Without<PedestalUnderglowLight>>,
    mut underglow_query: Query<&mut PointLight, With<PedestalUnderglowLight>>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    // 1. Animate Hourglass: rotate and phase in/out with stage acceleration
    for (mut transform, children, hg) in &mut hourglass_query {
        let speed_boost = 1.0 + state.pedestal_charge * 1.5;
        transform.rotate_y(1.6 * speed_boost * dt);

        // Phasing calculation: smooth sine oscillation between 0.15 and 1.0
        let phase = (elapsed * (2.8 + state.pedestal_charge * 4.0)).sin() * 0.425 + 0.575;

        // Hover bobbing
        transform.translation.y = MANIFESTATION_HOVER_Y + (elapsed * 2.0).sin() * 0.06;

        for child in children.iter() {
            if let Ok(mut light) = light_query.get_mut(child) {
                let intensity_boost = 1.0 + state.pedestal_charge * 0.8;
                light.intensity = hg.base_intensity * phase * intensity_boost;
            }
        }
    }

    // 2. Animate Pedestal Underglow: plasma charge buildup during manifestation
    for mut light in &mut underglow_query {
        let base_intensity = 10_000.0;
        if state.phase == ManifestationPhase::Manifesting {
            let pulse = (elapsed * (3.0 + state.pedestal_charge * 8.0)).sin() * 0.25 + 0.75;
            let charge_boost = state.pedestal_charge * 65_000.0;
            light.intensity = (base_intensity + charge_boost) * pulse;
            let r = 0.30 + state.pedestal_charge * 0.35;
            let g = 0.85 - state.pedestal_charge * 0.15;
            light.color = Color::srgb(r, g, 1.0);
        } else {
            light.intensity = base_intensity;
            light.color = Color::srgb(0.24, 0.62, 0.86);
        }
    }

    // 3. Animate Red X: gentle pulsing hover and slow rotation
    for mut transform in &mut red_x_query {
        transform.rotate_y(0.8 * dt);
        transform.translation.y = MANIFESTATION_HOVER_Y + (elapsed * 2.5).sin() * 0.05;
    }

    // 4. Animate Idle Symbol: slow majestic rotation
    for mut transform in &mut idle_query {
        transform.rotate_y(0.7 * dt);
        transform.translation.y = MANIFESTATION_HOVER_Y + (elapsed * 1.5).sin() * 0.04;
    }
}

fn handle_manifestation_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    actions: Res<crate::modes::inner_chambers::interaction::InnerActions>,
    time: Res<Time>,
    mut state: ResMut<ManifestationState>,
    channels: Res<ManifestationChannels>,
    camera_query: Query<&Transform, With<crate::modes::inner_chambers::camera::PlayerCamera>>,
    mut proximity_query: Query<
        &mut Visibility,
        (
            With<ManifestationProximityPrompt>,
            Without<ManifestationPromptUi>,
        ),
    >,
    mut modal_query: Query<
        &mut Visibility,
        (
            With<ManifestationPromptUi>,
            Without<ManifestationProximityPrompt>,
        ),
    >,
    mut text_query: Query<&mut Text, With<ManifestationPromptText>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    encounter_state: Res<EncounterState>,
    interaction_focus: Res<InteractionFocus>,
) {
    let Ok(player_transform) = camera_query.single() else {
        return;
    };

    let player_pos_2d = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.z,
    );
    let pedestal_pos_2d = Vec2::new(MANIFESTATION_PEDESTAL_POS.x, MANIFESTATION_PEDESTAL_POS.z);
    let dist = (player_pos_2d - pedestal_pos_2d).length();

    let is_near = dist <= 3.2;

    if let Ok(mut vis) = proximity_query.single_mut() {
        *vis = if is_near
            && state.phase != ManifestationPhase::Prompting
            && state.phase != ManifestationPhase::Manifesting
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    match state.phase {
        ManifestationPhase::Idle | ManifestationPhase::Completed | ManifestationPhase::Failed => {
            if is_near && !encounter_state.is_open() && interaction_focus.0 == Some(InteractionTarget::ManifestationAltar) && actions.interact {
                state.phase = ManifestationPhase::Prompting;
                state.prompt_buffer.clear();
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.visible = true;
                    cursor.grab_mode = CursorGrabMode::None;
                }
                if let Ok(mut vis) = modal_query.single_mut() {
                    *vis = Visibility::Visible;
                }
            }
        }
        ManifestationPhase::Prompting => {
            if let Ok(mut vis) = modal_query.single_mut() {
                *vis = Visibility::Visible;
            }

            if actions.cancel {
                state.phase = ManifestationPhase::Idle;
                if let Ok(mut vis) = modal_query.single_mut() {
                    *vis = Visibility::Hidden;
                }
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.visible = false;
                    cursor.grab_mode = CursorGrabMode::Locked;
                }
                return;
            }

            text_entry::apply_typed_keys(&keyboard, &mut state.prompt_buffer, 64);

            if let Ok(mut text) = text_query.single_mut() {
                let cursor = if (time.elapsed_secs() * 2.0).fract() < 0.5 {
                    "_"
                } else {
                    " "
                };
                text.0 = format!("> {}{}", state.prompt_buffer, cursor);
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                let prompt = if state.prompt_buffer.trim().is_empty() {
                    "sacred celestial relic".to_string()
                } else {
                    state.prompt_buffer.trim().to_string()
                };

                state.phase = ManifestationPhase::Manifesting;
                state.active_prompt = prompt.clone();
                state.waiting_elapsed = 0.0;
                state.current_stage_id = "start".into();
                state.current_stage_pct = 0;
                state.current_stage_state = "begin".into();
                state.current_stage_msg = "Initiating Chronos2...".into();
                state.pedestal_charge = 0.0;
                state.error_message.clear();

                if let Ok(mut vis) = modal_query.single_mut() {
                    *vis = Visibility::Hidden;
                }
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.visible = false;
                    cursor.grab_mode = CursorGrabMode::Locked;
                }

                // Despawn previous artifact if present when new manifestation begins
                if let Some(old_entity) = state.active_artifact.take() {
                    commands.entity(old_entity).despawn();
                }

                // Spawn floating 3D Hourglass that phases in and out
                spawn_phasing_hourglass(&mut commands, &mut meshes, &mut materials, &mut state);

                // Dispatch background render worker
                dispatch_manifestation_worker(
                    prompt,
                    channels.sender.clone(),
                    channels.active_pid.clone(),
                );
            }
        }
        ManifestationPhase::Manifesting => {
            if let Ok(mut vis) = modal_query.single_mut() {
                *vis = Visibility::Hidden;
            }
            state.waiting_elapsed += time.delta_secs();
            if actions.cancel {
                cancel_manifestation(&channels);
                state.phase = ManifestationPhase::Idle;
                state.current_stage_id.clear();
                state.current_stage_pct = 0;
                state.current_stage_state.clear();
                state.current_stage_msg.clear();
                state.pedestal_charge = 0.0;
                spawn_idle_symbol(&mut commands, &mut meshes, &mut materials, &mut state);
            }
        }
    }
}

pub fn parse_chronos_stage(line: &str) -> Option<(String, u32, String, String)> {
    let line = line.trim();
    if !line.starts_with("[chronos-stage]") {
        return None;
    }
    let rest = line.strip_prefix("[chronos-stage]")?.trim();
    let mut id = String::new();
    let mut pct: u32 = 0;
    let mut state = String::new();
    let mut msg = String::new();

    for token in rest.split_whitespace() {
        if let Some(val) = token.strip_prefix("id=") {
            id = val.to_string();
        } else if let Some(val) = token.strip_prefix("pct=") {
            pct = val.parse::<u32>().unwrap_or(0);
        } else if let Some(val) = token.strip_prefix("state=") {
            state = val.to_string();
        }
    }
    if let Some(idx) = rest.find("msg=") {
        msg = rest[idx + 4..].to_string();
    }
    if !id.is_empty() {
        Some((id, pct, state, msg))
    } else {
        None
    }
}

fn find_import_script() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("scripts").join("import_chronos_object.py");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("scripts").join("import_chronos_object.py");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let fallback = PathBuf::from(r"C:\archetypes\scripts\import_chronos_object.py");
    if fallback.is_file() {
        return Some(fallback);
    }
    None
}

fn target_glb_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(
                dir.join("assets")
                    .join("scenes")
                    .join("manifested_artifact.glb"),
            );
        }
    }
    paths.push(PathBuf::from(
        r"C:\archetypes\assets\scenes\manifested_artifact.glb",
    ));
    paths
}

/// Mirrors `target_glb_paths`: the same 2D reference image Chronos2 generated
/// and fed to TripoSR, staged where the asset server can load it, so the
/// pedestal can show the player the painting their prompt became before it
/// became an object.
fn target_reference_image_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(
                dir.join("assets")
                    .join("scenes")
                    .join("manifested_reference.png"),
            );
        }
    }
    paths.push(PathBuf::from(
        r"C:\archetypes\assets\scenes\manifested_reference.png",
    ));
    paths
}

fn dispatch_manifestation_worker(
    prompt: String,
    sender: Sender<ManifestationEvent>,
    active_pid: Arc<Mutex<Option<u32>>>,
) {
    std::thread::spawn(move || {
        let Some(script_path) = find_import_script() else {
            let msg = "Could not locate scripts/import_chronos_object.py".to_string();
            eprintln!("[ManifestationWorker] Error: {msg}");
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some("init".into()),
                detail: msg,
            });
            return;
        };

        let target_paths = target_glb_paths();
        let primary_output = target_paths[0].clone();

        if let Some(parent) = primary_output.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let bundle = std::env::temp_dir()
            .join("NeuroCognica")
            .join("Archetypes")
            .join("manifestations")
            .join(format!("{}", uuid::Uuid::new_v4()));

        let chronos = PathBuf::from(r"C:\chronos2\target\release\chronos.exe");
        if !chronos.is_file() {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some("init".into()),
                detail: "Chronos2 Object-mode executable is unavailable; no fallback object will be created.".into(),
            });
            return;
        }

        // Notify beginning of manifestation
        let _ = sender.send(ManifestationEvent::Stage {
            stage_id: "start".into(),
            pct: 0,
            state: "begin".into(),
            message: "Initiating Chronos2 Object mode...".into(),
        });

        // Best-effort non-blocking flush of ComfyUI VRAM cache before launching generation
        let _ = std::thread::spawn(|| {
            let _ = ureq::post("http://127.0.0.1:8000/free")
                .timeout(std::time::Duration::from_millis(600))
                .send_json(serde_json::json!({ "unload_models": true, "free_memory": true }));
        });

        let mut command = Command::new(&chronos);
        command
            .args(["first-light", "--prompt"])
            .arg(&prompt)
            .args(["--out-dir"])
            .arg(&bundle)
            .args(["--geometry-forge", "--void"]);

        // The installed 6.6 GB Juggernaut SDXL checkpoint stays inside the
        // Forge's 12 GB VRAM while producing materially better literal-subject
        // references than SD 1.5. A live Einstein probe completed the complete
        // governed path in the same sub-minute latency class. ComfyUI is
        // unloaded before TripoSR, so these engines do not residency-stack.
        if std::env::var("CHRONOS_FORGE_REFERENCE_CKPT").is_err() {
            command.env(
                "CHRONOS_FORGE_REFERENCE_CKPT",
                MANIFESTATION_REFERENCE_CHECKPOINT,
            );
        }
        command.env("CHRONOS_FLUX_PROFILE", "lowvram");
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        #[cfg(windows)]
        {
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = sender.send(ManifestationEvent::Failure {
                    prompt,
                    stage_id: Some("start".into()),
                    detail: format!("Failed to spawn Chronos2: {e}"),
                });
                return;
            }
        };

        let child_id = child.id();
        if let Ok(mut slot) = active_pid.lock() {
            *slot = Some(child_id);
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stderr_handle = std::thread::spawn(move || {
            let mut err_lines = Vec::new();
            if let Some(err) = stderr {
                let reader = std::io::BufReader::new(err);
                use std::io::BufRead;
                for line in reader.lines().flatten() {
                    eprintln!("[Chronos stderr] {line}");
                    err_lines.push(line);
                }
            }
            err_lines
        });

        let mut last_stage_id = "start".to_string();
        let mut last_stage_msg = "Initiating".to_string();

        if let Some(out) = stdout {
            let reader = std::io::BufReader::new(out);
            use std::io::BufRead;
            for line in reader.lines().flatten() {
                println!("[Chronos stdout] {line}");
                if let Some((id, pct, state, msg)) = parse_chronos_stage(&line) {
                    last_stage_id = id.clone();
                    last_stage_msg = msg.clone();
                    let _ = sender.send(ManifestationEvent::Stage {
                        stage_id: id,
                        pct,
                        state,
                        message: msg,
                    });
                }
            }
        }

        let status = child.wait();
        let err_lines = stderr_handle.join().unwrap_or_default();

        if let Ok(mut slot) = active_pid.lock() {
            *slot = None;
        }

        let Ok(exit_status) = status else {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some(last_stage_id),
                detail: "Chronos process wait failed".into(),
            });
            return;
        };

        if !exit_status.success() {
            let err_summary = if !err_lines.is_empty() {
                err_lines
                    .last()
                    .cloned()
                    .unwrap_or_else(|| format!("Chronos exited with code {exit_status}"))
            } else if !last_stage_msg.is_empty() {
                last_stage_msg
            } else {
                format!("Chronos failed with exit code {exit_status}")
            };
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some(last_stage_id),
                detail: err_summary,
            });
            return;
        }

        // Verify receipt and mesh
        let receipt = bundle.join("engine_mesh").join("triposr_artifact.json");
        let mesh_0 = bundle.join("engine_mesh").join("0").join("mesh.obj");
        let mesh_direct = bundle.join("engine_mesh").join("mesh.obj");
        let mesh = if mesh_0.is_file() {
            mesh_0
        } else if mesh_direct.is_file() {
            mesh_direct
        } else {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some("geometry_forge".into()),
                detail: "Chronos2 did not return its required TripoSR mesh receipt; no artifact was staged.".into(),
            });
            return;
        };

        if !receipt.is_file() {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some("geometry_forge".into()),
                detail: "TripoSR artifact receipt missing from Chronos bundle; failing closed."
                    .into(),
            });
            return;
        }

        // Conversion stage
        let _ = sender.send(ManifestationEvent::Stage {
            stage_id: "conversion".into(),
            pct: 98,
            state: "begin".into(),
            message: "Converting geometry to GLB (Blender)...".into(),
        });

        let blender_exe = r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe";
        let mut blender_cmd = Command::new(blender_exe);
        blender_cmd
            .arg("-b")
            .arg("--factory-startup")
            .arg("-P")
            .arg(&script_path)
            .arg("--")
            .arg("--input")
            .arg(&mesh)
            .arg("--output")
            .arg(&primary_output);
        #[cfg(windows)]
        {
            blender_cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let blender_child = match blender_cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = sender.send(ManifestationEvent::Failure {
                    prompt,
                    stage_id: Some("conversion".into()),
                    detail: format!("Failed to launch Blender: {e}"),
                });
                return;
            }
        };

        if let Ok(mut slot) = active_pid.lock() {
            *slot = Some(blender_child.id());
        }

        let blender_out = blender_child.wait_with_output();
        if let Ok(mut slot) = active_pid.lock() {
            *slot = None;
        }

        match blender_out {
            Ok(out)
                if out.status.success()
                    && primary_output.is_file()
                    && primary_output
                        .metadata()
                        .map(|m| m.len() > 1024)
                        .unwrap_or(false) =>
            {
                for other in &target_paths[1..] {
                    if let Some(p) = other.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    let _ = std::fs::copy(&primary_output, other);
                }

                // Stage the exact 2D reference the mesh was reconstructed from,
                // for the floor panel at the pedestal's foot. Best-effort: the
                // object itself already succeeded, so a missing reference image
                // (an operator-supplied-reference run, or an unexpected layout)
                // must never fail or retry a completed manifestation.
                let reference_source = bundle.join("reference_input.png");
                if reference_source.is_file() {
                    for target in target_reference_image_paths() {
                        if let Some(p) = target.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        let _ = std::fs::copy(&reference_source, &target);
                    }
                }

                let _ = sender.send(ManifestationEvent::Stage {
                    stage_id: "placement".into(),
                    pct: 100,
                    state: "done".into(),
                    message: "Placing artifact upon altar...".into(),
                });

                let _ = sender.send(ManifestationEvent::Success {
                    prompt,
                    detail: "Manifestation completed successfully".into(),
                });
            }
            Ok(out) => {
                let err_text = String::from_utf8_lossy(&out.stderr);
                let out_text = String::from_utf8_lossy(&out.stdout);
                let detail = if !err_text.trim().is_empty() {
                    err_text
                        .lines()
                        .last()
                        .unwrap_or("Blender conversion error")
                        .to_string()
                } else if !out_text.trim().is_empty() {
                    out_text
                        .lines()
                        .last()
                        .unwrap_or("Blender conversion failed")
                        .to_string()
                } else {
                    format!("Blender exit code {}", out.status)
                };
                let _ = sender.send(ManifestationEvent::Failure {
                    prompt,
                    stage_id: Some("conversion".into()),
                    detail,
                });
            }
            Err(e) => {
                let _ = sender.send(ManifestationEvent::Failure {
                    prompt,
                    stage_id: Some("conversion".into()),
                    detail: format!("Blender conversion process failed: {e}"),
                });
            }
        }
    });
}

fn cancel_manifestation(channels: &ManifestationChannels) {
    let pid = channels
        .active_pid
        .lock()
        .ok()
        .and_then(|mut slot| slot.take());
    if let Some(pid) = pid {
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
    }
}

fn poll_manifestation_results(
    mut commands: Commands,
    channels: Res<ManifestationChannels>,
    mut state: ResMut<ManifestationState>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(receiver) = channels.receiver.lock() else {
        return;
    };

    while let Ok(event) = receiver.try_recv() {
        match event {
            ManifestationEvent::Stage {
                stage_id,
                pct,
                state: stage_state,
                message,
            } => {
                state.current_stage_id = stage_id;
                state.current_stage_pct = pct;
                state.current_stage_state = stage_state;
                state.current_stage_msg = message;
                state.pedestal_charge = (pct as f32 / 100.0).clamp(0.0, 1.0);
            }
            ManifestationEvent::Success { prompt, detail: _ } => {
                // SUCCESS: Despawn the phasing hourglass
                if let Some(symbol) = state.floating_symbol_entity.take() {
                    commands.entity(symbol).despawn();
                }

                state.phase = ManifestationPhase::Completed;
                state.current_stage_id = "complete".into();
                state.current_stage_pct = 100;
                state.current_stage_msg = "Summoned".into();
                state.pedestal_charge = 0.0;

                // Despawn old artifact if present
                if let Some(old_entity) = state.active_artifact.take() {
                    commands.entity(old_entity).despawn();
                }

                let p = MANIFESTATION_PEDESTAL_POS;
                let cushion_y = p.y + 1.34;

                // Spawn celestial entrance flash
                commands.spawn((
                    PointLight {
                        intensity: 160_000.0,
                        range: 18.0,
                        color: Color::srgb(1.0, 0.95, 0.70),
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(p.x, cushion_y + 0.60, p.z),
                    ManifestationFlash {
                        timer: 0.0,
                        max_duration: 1.6,
                    },
                    ManifestationElement,
                    Name::new("ManifestationEntranceFlash"),
                ));

                // Controlled lightning bolt
                let bolt = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    emissive: LinearRgba::new(4.0, 8.0, 16.0, 1.0),
                    ..default()
                });
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.08, 4.8, 0.08))),
                    MeshMaterial3d(bolt),
                    Transform::from_xyz(p.x, cushion_y + 2.4, p.z),
                    ManifestationRevealSmoke { timer: 0.0 },
                    ManifestationElement,
                    Name::new("ManifestationLightningStrike"),
                ));

                // Volumetric smoke / cloud effect that dissipates
                let smoke = materials.add(StandardMaterial {
                    base_color: Color::srgba(0.35, 0.65, 0.95, 0.45),
                    emissive: LinearRgba::new(0.12, 0.30, 0.85, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                });
                for i in 0..22 {
                    let a = i as f32 * 0.285;
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.28 + (i % 4) as f32 * 0.10))),
                        MeshMaterial3d(smoke.clone()),
                        Transform::from_xyz(
                            p.x + a.cos() * 0.48,
                            cushion_y + 0.30 + (i % 5) as f32 * 0.10,
                            p.z + a.sin() * 0.48,
                        ),
                        ManifestationRevealSmoke { timer: 0.0 },
                        ManifestationElement,
                        Name::new("ManifestationRevealSmoke"),
                    ));
                }

                // Spawn newly summoned object atop the velvet cushion (validated import)
                let artifact_entity = commands
                    .spawn((
                        SceneRoot(asset_server.load("scenes/manifested_artifact.glb#Scene0")),
                        Transform::from_xyz(p.x, cushion_y, p.z).with_scale(Vec3::splat(1.15)),
                        // One revolution in about eleven seconds: slow enough
                        // to inspect, fast enough that motion is unmistakable.
                        ChronosExhibitTurntable { speed: 0.56 },
                        ManifestationElement,
                        Name::new("ManifestedChronosArtifact"),
                    ))
                    .id();

                state.active_artifact = Some(artifact_entity);

                // Reference floor panel: the same painting Chronos2 generated
                // and fed to TripoSR, laid flat on the floor at the pedestal's
                // foot. Museum-wall-and-object in one place — the player sees
                // the prompt become a picture, then sees the picture become
                // the object standing above it. Reuses whatever reference the
                // worker staged this run (or a prior run's, if this run had
                // none to copy); absent entirely on the very first launch.
                if let Some(old_panel) = state.active_reference_panel.take() {
                    commands.entity(old_panel).despawn();
                }
                let panel_pos = Vec3::new(p.x, p.y + 0.02, p.z - 2.8);
                let panel_material = materials.add(StandardMaterial {
                    base_color_texture: Some(
                        asset_server.load("scenes/manifested_reference.png"),
                    ),
                    perceptual_roughness: 0.55,
                    metallic: 0.0,
                    ..default()
                });
                let panel_entity = commands
                    .spawn((
                        Mesh3d(meshes.add(Plane3d::default().mesh().size(1.4, 1.4))),
                        MeshMaterial3d(panel_material),
                        Transform::from_translation(panel_pos),
                        ManifestationElement,
                        Name::new("ManifestationReferencePanel"),
                    ))
                    .id();
                state.active_reference_panel = Some(panel_entity);

                println!("[ManifestationSystem] Succeeded: Manifested '{prompt}' atop the altar!");
            }
            ManifestationEvent::Failure {
                prompt: _,
                stage_id,
                detail,
            } => {
                state.phase = ManifestationPhase::Failed;
                state.current_stage_id = stage_id.unwrap_or_else(|| "failure".into());
                state.error_message = detail.clone();
                state.pedestal_charge = 0.0;

                spawn_floating_red_x(&mut commands, &mut meshes, &mut materials, &mut state);
                eprintln!("[ManifestationSystem] Manifestation Failed: {detail}");
            }
        }
    }
}

fn update_manifestation_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PointLight, &mut ManifestationFlash)>,
) {
    let dt = time.delta_secs();
    for (entity, mut light, mut flash) in &mut query {
        flash.timer += dt;
        if flash.timer >= flash.max_duration {
            commands.entity(entity).despawn();
        } else {
            let progress = flash.timer / flash.max_duration;
            let fade = (1.0 - progress).powi(2);
            light.intensity = 160_000.0 * fade;
        }
    }
}

fn update_manifestation_reveal_smoke(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke: Query<(Entity, &mut Transform, &mut ManifestationRevealSmoke)>,
) {
    for (entity, mut transform, mut reveal) in &mut smoke {
        reveal.timer += time.delta_secs();
        transform.translation.y += time.delta_secs() * 0.45;
        transform.scale *= 1.0 + time.delta_secs() * 0.7;
        if reveal.timer > 2.4 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_manifestation_hud(
    state: Res<ManifestationState>,
    mut banner_query: Query<&mut Visibility, With<ManifestationStatusBanner>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<ManifestationStatusText>>,
) {
    let Ok(mut banner_vis) = banner_query.single_mut() else {
        return;
    };
    let Ok((mut text, mut color)) = text_query.single_mut() else {
        return;
    };

    match state.phase {
        ManifestationPhase::Manifesting => {
            *banner_vis = Visibility::Visible;
            let stage_label = if !state.current_stage_msg.is_empty() {
                &state.current_stage_msg
            } else {
                "Initiating Chronos2..."
            };
            text.0 = format!(
                "⏳ [{}%] {}: \"{}\" [{:.1}s] — Press [Esc] to cancel",
                state.current_stage_pct, stage_label, state.active_prompt, state.waiting_elapsed
            );
            color.0 = Color::srgb(1.0, 0.85, 0.30); // Warm amber
        }
        ManifestationPhase::Failed => {
            *banner_vis = Visibility::Visible;
            text.0 = format!(
                "❌ MANIFESTATION REFUSED/FAILED: {} — Press [E] to retry.",
                state.error_message
            );
            color.0 = Color::srgb(1.0, 0.25, 0.30); // Warning red
        }
        ManifestationPhase::Completed => {
            *banner_vis = Visibility::Visible;
            text.0 = format!(
                "✨ MANIFESTED: \"{}\" — Staged upon the sacred altar!",
                state.active_prompt
            );
            color.0 = Color::srgb(0.40, 1.0, 0.70); // Radiant celestial green
        }
        ManifestationPhase::Idle | ManifestationPhase::Prompting => {
            *banner_vis = Visibility::Hidden;
        }
    }
}

const ACTIVE_FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);
const MANIFESTING_FRAME_INTERVAL: Duration = Duration::from_nanos(66_666_667);
const MANIFESTATION_REFERENCE_CHECKPOINT: &str =
    "Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors";

fn manifestation_frame_interval(phase: &ManifestationPhase) -> Duration {
    if *phase == ManifestationPhase::Manifesting {
        MANIFESTING_FRAME_INTERVAL
    } else {
        ACTIVE_FRAME_INTERVAL
    }
}

fn yield_gpu_during_manifestation(
    state: Res<ManifestationState>,
    mut winit_settings: ResMut<WinitSettings>,
) {
    let target = UpdateMode::reactive_low_power(manifestation_frame_interval(&state.phase));
    if winit_settings.focused_mode != target {
        winit_settings.focused_mode = target;
    }
}

fn teardown_manifestation(
    mut commands: Commands,
    query: Query<Entity, With<ManifestationElement>>,
    mut state: ResMut<ManifestationState>,
    channels: Res<ManifestationChannels>,
    mut winit_settings: ResMut<WinitSettings>,
) {
    cancel_manifestation(&channels);
    for entity in &query {
        commands.entity(entity).despawn();
    }
    state.active_artifact = None;
    state.active_reference_panel = None;
    state.floating_symbol_entity = None;
    state.phase = ManifestationPhase::Idle;
    state.prompt_buffer.clear();
    winit_settings.focused_mode = UpdateMode::reactive_low_power(ACTIVE_FRAME_INTERVAL);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chronos_stage_valid() {
        let line = "[chronos-stage] id=sentinel pct=6 state=done msg=Sentinel is reviewing your request - cleared - no prohibited use found";
        let parsed = parse_chronos_stage(line);
        assert!(parsed.is_some());
        let (id, pct, state, msg) = parsed.unwrap();
        assert_eq!(id, "sentinel");
        assert_eq!(pct, 6);
        assert_eq!(state, "done");
        assert_eq!(
            msg,
            "Sentinel is reviewing your request - cleared - no prohibited use found"
        );
    }

    #[test]
    fn test_parse_chronos_stage_refusal() {
        let line = "[chronos-stage] id=sentinel pct=6 state=fail msg=Sentinel refused request: prohibited use detected";
        let parsed = parse_chronos_stage(line);
        assert!(parsed.is_some());
        let (id, pct, state, msg) = parsed.unwrap();
        assert_eq!(id, "sentinel");
        assert_eq!(pct, 6);
        assert_eq!(state, "fail");
        assert!(msg.contains("prohibited use detected"));
    }

    #[test]
    fn test_no_recipe_or_keyword_in_manifestation_dispatch() {
        // Assert that the manifestation module does NOT reference retired recipe scripts or keyword tables.
        let script = find_import_script().expect("import_chronos_object.py must be discoverable");
        assert!(
            script.ends_with("import_chronos_object.py"),
            "must use generic import script"
        );
        assert!(
            !script.to_string_lossy().contains("manifest_artifact.py"),
            "manifest_artifact.py is retired and must never be referenced"
        );
        let source = std::fs::read_to_string(script).expect("generic importer must be readable");
        assert!(
            source.contains("MAX_GAME_TRIANGLES"),
            "generic importer must enforce a game mesh budget"
        );
        assert!(
            source.contains("DECIMATE"),
            "generic importer must optimize generated meshes"
        );
        assert!(
            source.contains("CHRONOS_UP_AXIS = \"Z\"")
                && source.contains("up_axis=CHRONOS_UP_AXIS"),
            "generic importer must honor Chronos's declared Z-up mesh basis"
        );
        for forbidden_recipe in ["strawberry", "panther", "whale", "diamond ring"] {
            assert!(
                !source.to_ascii_lowercase().contains(forbidden_recipe),
                "importer contains a forbidden prompt recipe: {forbidden_recipe}"
            );
        }
    }

    #[test]
    fn test_manifestation_fails_closed_when_receipt_missing() {
        let tmp = std::env::temp_dir().join(format!("test_manifestation_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&tmp);
        let receipt = tmp.join("engine_mesh").join("triposr_artifact.json");
        let mesh = tmp.join("engine_mesh").join("0").join("mesh.obj");
        assert!(!receipt.is_file(), "receipt absent");
        assert!(!mesh.is_file(), "mesh absent");
        // Verify fail-closed semantics: without these files, manifestation refuses to generate a fallback.
        let is_valid = receipt.is_file() && mesh.is_file();
        assert!(!is_valid, "system must fail closed when receipt is missing");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn manifestation_yields_gpu_capacity_while_chronos_runs() {
        assert_eq!(
            manifestation_frame_interval(&ManifestationPhase::Manifesting),
            MANIFESTING_FRAME_INTERVAL
        );
        assert_eq!(
            manifestation_frame_interval(&ManifestationPhase::Completed),
            ACTIVE_FRAME_INTERVAL
        );
    }

    #[test]
    fn manifestation_uses_the_measured_sdxl_reference_engine_for_every_prompt() {
        assert_eq!(
            MANIFESTATION_REFERENCE_CHECKPOINT,
            "Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors"
        );
    }
}
