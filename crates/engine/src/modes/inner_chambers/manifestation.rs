//! Manifestation Pedestal & Headless Chronos2 Live Staging System.
//!
//! Enables the player to walk up to the sacred manifestation pedestal on the dais,
//! press 'E' to open the prompt conduit, and trigger a headless Chronos2 render
//! that creates, grounds, and manifests the 3D artifact directly onto the pedestal.
//!
//! Visual State Machine atop the Pedestal:
//! 1. Idle: Floating subtle celestial diamond symbol above the empty velvet cushion.
//! 2. Waiting: A 3D floating hourglass that slowly rotates and phases in and out (pulsing light/opacity) with a live timer.
//! 3. Failed: A 3D floating luminous Red 'X' with a crimson warning beacon and explicit error message.
//! 4. Succeeded: Grand golden radiance flash, despawning the hourglass, and revealing the newly summoned 3D object rotating atop the cushion.

use super::world::ChronosExhibitTurntable;
use super::InnerChambersState;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub const MANIFESTATION_PEDESTAL_POS: Vec3 = Vec3::new(0.0, 0.30, 3.4);
pub const MANIFESTATION_CUSHION_HEIGHT: f32 = 1.62;
pub const MANIFESTATION_HOVER_Y: f32 = 2.40;

pub struct ManifestationPlugin;

impl Plugin for ManifestationPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = channel::<ManifestationResult>();
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
                handle_manifestation_input,
                animate_pedestal_symbols,
                poll_manifestation_results,
                update_manifestation_flash,
                update_manifestation_reveal_smoke,
                update_manifestation_hud,
            )
                .run_if(in_state(InnerChambersState::Navigating)),
        )
        .add_systems(
            OnEnter(InnerChambersState::Exiting),
            teardown_manifestation,
        );
    }
}

pub struct ManifestationResult {
    pub prompt: String,
    pub success: bool,
    pub detail: String,
}

#[derive(Resource)]
pub struct ManifestationChannels {
    pub sender: Sender<ManifestationResult>,
    pub receiver: Mutex<Receiver<ManifestationResult>>,
    pub active_pid: Arc<Mutex<Option<u32>>>,
}

#[derive(Resource, Default)]
pub struct ManifestationState {
    pub phase: ManifestationPhase,
    pub prompt_buffer: String,
    pub active_prompt: String,
    pub waiting_elapsed: f32,
    pub error_message: String,
    pub active_artifact: Option<Entity>,
    pub floating_symbol_entity: Option<Entity>,
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
pub struct ManifestationRevealSmoke { pub timer: f32 }

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
    state.error_message.clear();
    state.active_artifact = None;
    state.floating_symbol_entity = None;

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

    // 6. Altar Dual Lighting
    commands.spawn((
        PointLight {
            intensity: 75_000.0,
            range: 12.0,
            color: Color::srgb(1.0, 0.96, 0.88),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(p.x, p.y + 4.5, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_Spotlight"),
    ));

    commands.spawn((
        PointLight {
            intensity: 28_000.0,
            range: 5.0,
            color: Color::srgb(0.30, 0.85, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(p.x, p.y + 1.8, p.z),
        ManifestationElement,
        Name::new("ManifestationPedestal_Underglow"),
    ));

    // 7. Initial Idle Symbol (Floating subtle celestial diamond above the empty cushion)
    spawn_idle_symbol(&mut commands, &mut meshes, &mut materials, &mut state);

    // 8. Proximity prompt HUD (hidden by default)
    commands.spawn((
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
    )).with_children(|parent| {
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
    commands.spawn((
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
    )).with_children(|parent| {
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
                Mesh3d(meshes.add(Cone { radius: 0.22, height: 0.30 })),
                MeshMaterial3d(glass_mat.clone()),
                Transform::from_xyz(0.0, 0.15, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                Name::new("Hourglass_UpperGlass"),
            ));

            // Lower cone (pointing upward to waist)
            parent.spawn((
                Mesh3d(meshes.add(Cone { radius: 0.22, height: 0.30 })),
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
    mut hourglass_query: Query<(&mut Transform, &Children, &PhasingHourglass)>,
    mut red_x_query: Query<&mut Transform, (With<FloatingRedX>, Without<PhasingHourglass>)>,
    mut idle_query: Query<&mut Transform, (With<FloatingIdleSymbol>, Without<PhasingHourglass>, Without<FloatingRedX>)>,
    mut light_query: Query<&mut PointLight>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    // 1. Animate Hourglass: rotate and phase in/out
    for (mut transform, children, hg) in &mut hourglass_query {
        transform.rotate_y(1.6 * dt);

        // Phasing calculation: smooth sine oscillation between 0.15 and 1.0
        let phase = (elapsed * 2.8).sin() * 0.425 + 0.575;

        // Hover bobbing
        transform.translation.y = MANIFESTATION_HOVER_Y + (elapsed * 2.0).sin() * 0.06;

        for child in children.iter() {
            if let Ok(mut light) = light_query.get_mut(child) {
                light.intensity = hg.base_intensity * phase;
            }
        }
    }

    // 2. Animate Red X: gentle pulsing hover and slow rotation
    for mut transform in &mut red_x_query {
        transform.rotate_y(0.8 * dt);
        transform.translation.y = MANIFESTATION_HOVER_Y + (elapsed * 2.5).sin() * 0.05;
    }

    // 3. Animate Idle Symbol: slow majestic rotation
    for mut transform in &mut idle_query {
        transform.rotate_y(0.7 * dt);
        transform.translation.y = MANIFESTATION_HOVER_Y + (elapsed * 1.5).sin() * 0.04;
    }
}

fn handle_manifestation_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut state: ResMut<ManifestationState>,
    channels: Res<ManifestationChannels>,
    camera_query: Query<&Transform, With<crate::modes::inner_chambers::camera::PlayerCamera>>,
    mut proximity_query: Query<&mut Visibility, (With<ManifestationProximityPrompt>, Without<ManifestationPromptUi>)>,
    mut modal_query: Query<&mut Visibility, (With<ManifestationPromptUi>, Without<ManifestationProximityPrompt>)>,
    mut text_query: Query<&mut Text, With<ManifestationPromptText>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(player_transform) = camera_query.single() else {
        return;
    };

    let player_pos_2d = Vec2::new(player_transform.translation.x, player_transform.translation.z);
    let pedestal_pos_2d = Vec2::new(MANIFESTATION_PEDESTAL_POS.x, MANIFESTATION_PEDESTAL_POS.z);
    let dist = (player_pos_2d - pedestal_pos_2d).length();

    let is_near = dist <= 3.2;

    if let Ok(mut vis) = proximity_query.single_mut() {
        *vis = if is_near && state.phase != ManifestationPhase::Prompting && state.phase != ManifestationPhase::Manifesting {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    match state.phase {
        ManifestationPhase::Idle | ManifestationPhase::Completed | ManifestationPhase::Failed => {
            if is_near && keyboard.just_pressed(KeyCode::KeyE) {
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

            if keyboard.just_pressed(KeyCode::Escape) {
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

            if keyboard.just_pressed(KeyCode::Backspace) {
                state.prompt_buffer.pop();
            }

            for (key, ch) in [
                (KeyCode::KeyA, 'a'), (KeyCode::KeyB, 'b'), (KeyCode::KeyC, 'c'),
                (KeyCode::KeyD, 'd'), (KeyCode::KeyE, 'e'), (KeyCode::KeyF, 'f'),
                (KeyCode::KeyG, 'g'), (KeyCode::KeyH, 'h'), (KeyCode::KeyI, 'i'),
                (KeyCode::KeyJ, 'j'), (KeyCode::KeyK, 'k'), (KeyCode::KeyL, 'l'),
                (KeyCode::KeyM, 'm'), (KeyCode::KeyN, 'n'), (KeyCode::KeyO, 'o'),
                (KeyCode::KeyP, 'p'), (KeyCode::KeyQ, 'q'), (KeyCode::KeyR, 'r'),
                (KeyCode::KeyS, 's'), (KeyCode::KeyT, 't'), (KeyCode::KeyU, 'u'),
                (KeyCode::KeyV, 'v'), (KeyCode::KeyW, 'w'), (KeyCode::KeyX, 'x'),
                (KeyCode::KeyY, 'y'), (KeyCode::KeyZ, 'z'), (KeyCode::Space, ' '),
                (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'),
                (KeyCode::Digit3, '3'), (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
                (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'),
                (KeyCode::Digit9, '9'), (KeyCode::Minus, '-'),
            ] {
                if keyboard.just_pressed(key) && state.prompt_buffer.len() < 64 {
                    state.prompt_buffer.push(ch);
                }
            }

            if let Ok(mut text) = text_query.single_mut() {
                let cursor = if (time.elapsed_secs() * 2.0).fract() < 0.5 { "_" } else { " " };
                text.0 = format!("> {}{}", state.prompt_buffer, cursor);
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                let prompt = if state.prompt_buffer.trim().is_empty() {
                    "buddha".to_string()
                } else {
                    state.prompt_buffer.trim().to_string()
                };

                state.phase = ManifestationPhase::Manifesting;
                state.active_prompt = prompt.clone();
                state.waiting_elapsed = 0.0;
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
                dispatch_manifestation_worker(prompt, channels.sender.clone(), channels.active_pid.clone());
            }
        }
        ManifestationPhase::Manifesting => {
            if let Ok(mut vis) = modal_query.single_mut() {
                *vis = Visibility::Hidden;
            }
            state.waiting_elapsed += time.delta_secs();
            if keyboard.just_pressed(KeyCode::Escape) {
                cancel_manifestation(&channels);
                state.phase = ManifestationPhase::Idle;
                spawn_idle_symbol(&mut commands, &mut meshes, &mut materials, &mut state);
            }
        }
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
            paths.push(dir.join("assets").join("scenes").join("manifested_artifact.glb"));
        }
    }
    paths.push(PathBuf::from(r"C:\archetypes\assets\scenes\manifested_artifact.glb"));
    paths
}

fn dispatch_manifestation_worker(prompt: String, sender: Sender<ManifestationResult>, active_pid: Arc<Mutex<Option<u32>>>) {
    std::thread::spawn(move || {
        let Some(script_path) = find_import_script() else {
            let msg = "Could not locate scripts/import_chronos_object.py".to_string();
            eprintln!("[ManifestationWorker] Error: {msg}");
            let _ = sender.send(ManifestationResult {
                prompt,
                success: false,
                detail: msg,
            });
            return;
        };

        let target_paths = target_glb_paths();
        let primary_output = target_paths[0].clone();

        if let Some(parent) = primary_output.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let bundle = std::env::temp_dir().join("NeuroCognica").join("Archetypes").join("manifestations").join(format!("{}", uuid::Uuid::new_v4()));
        let chronos = PathBuf::from(r"C:\chronos2\target\release\chronos.exe");
        if !chronos.is_file() {
            let _ = sender.send(ManifestationResult { prompt, success: false, detail: "Chronos2 Object-mode executable is unavailable; no fallback object will be created.".into() });
            return;
        }
        let mut command = Command::new(&chronos);
        command.args(["first-light", "--prompt"]).arg(&prompt).args(["--out-dir"]).arg(&bundle).args(["--geometry-forge", "--void"]);
        #[cfg(windows)] { command.creation_flags(CREATE_NO_WINDOW); }
        let Ok(child) = command.spawn() else { let _ = sender.send(ManifestationResult { prompt, success: false, detail: "Could not start Chronos2 Object mode.".into() }); return; };
        if let Ok(mut slot) = active_pid.lock() { *slot = Some(child.id()); }
        let run = child.wait_with_output();
        if let Ok(mut slot) = active_pid.lock() { *slot = None; }
        let Ok(run) = run else { let _ = sender.send(ManifestationResult { prompt, success: false, detail: "Could not start Chronos2 Object mode.".into() }); return; };
        if !run.status.success() {
            let detail = String::from_utf8_lossy(&run.stderr).lines().last().unwrap_or("Chronos2 Object mode failed").to_string();
            let _ = sender.send(ManifestationResult { prompt, success: false, detail }); return;
        }
        let receipt = bundle.join("engine_mesh").join("triposr_artifact.json");
        let mesh = bundle.join("engine_mesh").join("0").join("mesh.obj");
        if !receipt.is_file() || !mesh.is_file() {
            let _ = sender.send(ManifestationResult { prompt, success: false, detail: "Chronos2 did not return its required TripoSR mesh receipt; no artifact was staged.".into() }); return;
        }
        let blender_exe = r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe";
        let output = Command::new(blender_exe).arg("-b").arg("-P").arg(&script_path)
            .arg("--")
            .arg("--input").arg(&mesh)
            .arg("--output")
            .arg(&primary_output)
            .output();

        match output {
            Ok(out) if out.status.success() && primary_output.exists() => {
                // Copy to secondary target path if different (e.g. repo assets vs installed assets)
                for other in &target_paths[1..] {
                    if let Some(p) = other.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    let _ = std::fs::copy(&primary_output, other);
                }

                println!("[ManifestationWorker] Successfully manifested '{prompt}' at {primary_output:?}");
                let _ = sender.send(ManifestationResult {
                    prompt,
                    success: true,
                    detail: "Render complete".to_string(),
                });
            }
            Ok(out) => {
                let err_text = String::from_utf8_lossy(&out.stderr);
                let out_text = String::from_utf8_lossy(&out.stdout);
                let detail = if !err_text.trim().is_empty() {
                    err_text.lines().last().unwrap_or("Blender script error").to_string()
                } else if !out_text.trim().is_empty() {
                    out_text.lines().last().unwrap_or("Render failed").to_string()
                } else {
                    format!("Exit code {}", out.status)
                };
                eprintln!("[ManifestationWorker] Render failed: {detail}");
                let _ = sender.send(ManifestationResult {
                    prompt,
                    success: false,
                    detail,
                });
            }
            Err(e) => {
                let msg = format!("Failed to launch Blender: {e}");
                eprintln!("[ManifestationWorker] Execution error: {msg}");
                let _ = sender.send(ManifestationResult {
                    prompt,
                    success: false,
                    detail: msg,
                });
            }
        }
    });
}

fn cancel_manifestation(channels: &ManifestationChannels) {
    let pid = channels.active_pid.lock().ok().and_then(|mut slot| slot.take());
    if let Some(pid) = pid {
        #[cfg(windows)] { let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(CREATE_NO_WINDOW).output(); }
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

    if let Ok(result) = receiver.try_recv() {
        if result.success {
            // SUCCESS: Despawn the phasing hourglass
            if let Some(symbol) = state.floating_symbol_entity.take() {
                commands.entity(symbol).despawn();
            }

            state.phase = ManifestationPhase::Completed;

            // Despawn old artifact if present
            if let Some(old_entity) = state.active_artifact.take() {
                commands.entity(old_entity).despawn();
            }

            let p = MANIFESTATION_PEDESTAL_POS;
            let cushion_y = p.y + 1.34;

            // Spawn celestial entrance flash
            commands.spawn((
                PointLight {
                    intensity: 140_000.0,
                    range: 16.0,
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
            // A real return gets a theatrical reveal; it is never played while Chronos is merely waiting.
            let smoke = materials.add(StandardMaterial { base_color: Color::srgba(0.28, 0.55, 0.95, 0.42), emissive: LinearRgba::new(0.08, 0.22, 0.65, 1.0), alpha_mode: AlphaMode::Blend, ..default() });
            let bolt = materials.add(StandardMaterial { base_color: Color::WHITE, emissive: LinearRgba::new(3.0, 6.0, 12.0, 1.0), ..default() });
            for i in 0..18 {
                let a = i as f32 * 0.349;
                commands.spawn((Mesh3d(meshes.add(Sphere::new(0.28 + (i % 3) as f32 * 0.12))), MeshMaterial3d(smoke.clone()), Transform::from_xyz(p.x + a.cos() * 0.45, cushion_y + 0.35 + (i % 4) as f32 * 0.12, p.z + a.sin() * 0.45), ManifestationRevealSmoke { timer: 0.0 }, ManifestationElement, Name::new("ManifestationRevealSmoke")));
            }
            commands.spawn((Mesh3d(meshes.add(Cuboid::new(0.08, 4.8, 0.08))), MeshMaterial3d(bolt), Transform::from_xyz(p.x, cushion_y + 2.4, p.z), ManifestationRevealSmoke { timer: 0.0 }, ManifestationElement, Name::new("ManifestationLightningStrike")));

            // Spawn the newly summoned object atop the velvet cushion
            let artifact_entity = commands
                .spawn((
                    SceneRoot(asset_server.load("scenes/manifested_artifact.glb#Scene0")),
                    Transform::from_xyz(p.x, cushion_y, p.z).with_scale(Vec3::splat(1.15)),
                    ChronosExhibitTurntable { speed: 0.38 },
                    ManifestationElement,
                    Name::new("ManifestedChronosArtifact"),
                ))
                .id();

            state.active_artifact = Some(artifact_entity);
            println!("[ManifestationSystem] Succeeded: Manifested '{}' atop the sacred pedestal!", result.prompt);
        } else {
            // FAILURE: Despawn the hourglass and spawn the Floating Red X
            state.phase = ManifestationPhase::Failed;
            state.error_message = result.detail.clone();

            spawn_floating_red_x(&mut commands, &mut meshes, &mut materials, &mut state);
            eprintln!("[ManifestationSystem] Failed: {}", result.detail);
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
            light.intensity = 140_000.0 * fade;
        }
    }
}

fn update_manifestation_reveal_smoke(mut commands: Commands, time: Res<Time>, mut smoke: Query<(Entity, &mut Transform, &mut ManifestationRevealSmoke)>) {
    for (entity, mut transform, mut reveal) in &mut smoke {
        reveal.timer += time.delta_secs();
        transform.translation.y += time.delta_secs() * 0.45;
        transform.scale *= 1.0 + time.delta_secs() * 0.7;
        if reveal.timer > 2.4 { commands.entity(entity).despawn(); }
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
            text.0 = format!(
                "⏳ MANIFESTING: \"{}\" (Chronos2 rendering...) [{:.1}s]",
                state.active_prompt, state.waiting_elapsed
            );
            color.0 = Color::srgb(1.0, 0.85, 0.30); // Warm amber
        }
        ManifestationPhase::Failed => {
            *banner_vis = Visibility::Visible;
            text.0 = format!(
                "❌ MANIFESTATION FAILED: {} — Press [E] to retry.",
                state.error_message
            );
            color.0 = Color::srgb(1.0, 0.25, 0.30); // Warning red
        }
        ManifestationPhase::Completed => {
            *banner_vis = Visibility::Visible;
            text.0 = format!(
                "✨ SUMMONED: \"{}\" — Successfully manifested upon the altar!",
                state.active_prompt
            );
            color.0 = Color::srgb(0.40, 1.0, 0.70); // Radiant celestial green
        }
        ManifestationPhase::Idle | ManifestationPhase::Prompting => {
            *banner_vis = Visibility::Hidden;
        }
    }
}

fn teardown_manifestation(
    mut commands: Commands,
    query: Query<Entity, With<ManifestationElement>>,
    mut state: ResMut<ManifestationState>,
    channels: Res<ManifestationChannels>,
) {
    cancel_manifestation(&channels);
    for entity in &query {
        commands.entity(entity).despawn();
    }
    state.active_artifact = None;
    state.floating_symbol_entity = None;
    state.phase = ManifestationPhase::Idle;
    state.prompt_buffer.clear();
}
