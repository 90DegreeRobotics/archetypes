//! Manifestation Pedestal & Headless Chronos2 Live Staging System.
//!
//! Enables the player to walk up to the sacred manifestation pedestal on the dais,
//! press 'E' to open the prompt conduit, and trigger a headless Chronos2 render
//! that creates, grounds, and manifests the 3D artifact directly onto the pedestal
//! with a hovering wait indicator, grand entrance flash, and smooth turntable rotation.

use super::world::ChronosExhibitTurntable;
use super::InnerChambersState;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

pub const MANIFESTATION_PEDESTAL_POS: Vec3 = Vec3::new(0.0, 0.30, 3.4);
pub const MANIFESTATION_CUSHION_HEIGHT: f32 = 1.62;

pub struct ManifestationPlugin;

impl Plugin for ManifestationPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = channel::<ManifestationResult>();
        app.insert_resource(ManifestationChannels {
            sender: tx,
            receiver: Mutex::new(rx),
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
                animate_wait_indicator,
                poll_manifestation_results,
                update_manifestation_flash,
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
}

#[derive(Resource, Default)]
pub struct ManifestationState {
    pub phase: ManifestationPhase,
    pub prompt_buffer: String,
    pub status_message: String,
    pub active_artifact: Option<Entity>,
    pub wait_indicator_root: Option<Entity>,
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
pub struct WaitIndicatorRing {
    pub axis: Vec3,
    pub speed: f32,
}

#[derive(Component)]
pub struct WaitIndicatorBeacon {
    pub base_intensity: f32,
}

#[derive(Component)]
pub struct ManifestationFlash {
    pub timer: f32,
    pub max_duration: f32,
}

#[derive(Component)]
pub struct ManifestationPromptUi;

#[derive(Component)]
pub struct ManifestationPromptText;

#[derive(Component)]
pub struct ManifestationProximityPrompt;

fn setup_manifestation_pedestal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<ManifestationState>,
) {
    state.phase = ManifestationPhase::Idle;
    state.prompt_buffer.clear();
    state.status_message = "Walk up and press [E] to manifest an artifact.".to_string();
    state.active_artifact = None;
    state.wait_indicator_root = None;

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
    // Overhead spotlight
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

    // Underglow point light beneath the cushion
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

    // 7. Proximity prompt HUD (hidden by default)
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

    // 8. Text Input Prompt Modal (hidden by default)
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
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(player_transform) = camera_query.single() else {
        return;
    };

    let player_pos_2d = Vec2::new(player_transform.translation.x, player_transform.translation.z);
    let pedestal_pos_2d = Vec2::new(MANIFESTATION_PEDESTAL_POS.x, MANIFESTATION_PEDESTAL_POS.z);
    let dist = (player_pos_2d - pedestal_pos_2d).length();

    // 1. Proximity detection
    let is_near = dist <= 3.0;
    if let Ok(mut vis) = proximity_query.single_mut() {
        *vis = if is_near && state.phase != ManifestationPhase::Prompting && state.phase != ManifestationPhase::Manifesting {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // 2. Interaction state machine
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

            // Keyboard input handling
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

            // Capture alphanumeric typing
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
                (KeyCode::Digit9, '9'),
            ] {
                if keyboard.just_pressed(key) && state.prompt_buffer.len() < 64 {
                    state.prompt_buffer.push(ch);
                }
            }

            // Update modal text display with blinking cursor
            if let Ok(mut text) = text_query.single_mut() {
                let cursor = if (time.elapsed_secs() * 2.0).fract() < 0.5 { "_" } else { " " };
                text.0 = format!("> {}{}", state.prompt_buffer, cursor);
            }

            // Submit on Enter
            if keyboard.just_pressed(KeyCode::Enter) {
                let prompt = if state.prompt_buffer.trim().is_empty() {
                    "a glowing crystalline reliquary".to_string()
                } else {
                    state.prompt_buffer.trim().to_string()
                };

                state.phase = ManifestationPhase::Manifesting;
                state.status_message = format!("Manifesting '{prompt}'...");

                if let Ok(mut vis) = modal_query.single_mut() {
                    *vis = Visibility::Hidden;
                }
                if let Ok(mut cursor) = cursor_options.single_mut() {
                    cursor.visible = false;
                    cursor.grab_mode = CursorGrabMode::Locked;
                }

                // Spawn hovering wait indicator above the pedestal
                spawn_wait_indicator(&mut commands, meshes, materials, &mut state);

                // Dispatch background worker thread
                dispatch_manifestation_worker(prompt, channels.sender.clone());
            }
        }
        ManifestationPhase::Manifesting => {
            if let Ok(mut vis) = modal_query.single_mut() {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn spawn_wait_indicator(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: &mut ManifestationState,
) {
    let p = MANIFESTATION_PEDESTAL_POS;
    let center_y = p.y + 2.75; // Hovering 1.13m above the cushion

    let outer_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.80, 0.30),
        emissive: LinearRgba::new(0.95, 0.80, 0.30, 1.0),
        perceptual_roughness: 0.2,
        metallic: 0.9,
        ..default()
    });

    let inner_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.85, 1.0),
        emissive: LinearRgba::new(0.25, 0.85, 1.0, 1.0),
        perceptual_roughness: 0.1,
        metallic: 0.5,
        ..default()
    });

    let root = commands
        .spawn((
            Transform::from_xyz(p.x, center_y, p.z),
            Visibility::Visible,
            ManifestationElement,
            Name::new("ManifestationWaitIndicatorRoot"),
        ))
        .with_children(|parent| {
            // Outer rotating ring (rotates around Y)
            parent.spawn((
                Mesh3d(meshes.add(Torus::new(0.55, 0.025))),
                MeshMaterial3d(outer_mat),
                Transform::default(),
                WaitIndicatorRing {
                    axis: Vec3::Y,
                    speed: 2.2,
                },
                Name::new("WaitIndicator_OuterRing"),
            ));

            // Inner tilted rotating ring (rotates around X)
            parent.spawn((
                Mesh3d(meshes.add(Torus::new(0.40, 0.02))),
                MeshMaterial3d(inner_mat),
                Transform::from_rotation(Quat::from_rotation_z(0.65)),
                WaitIndicatorRing {
                    axis: Vec3::X,
                    speed: 3.4,
                },
                Name::new("WaitIndicator_InnerRing"),
            ));

            // Central pulsing beacon light
            parent.spawn((
                PointLight {
                    intensity: 45_000.0,
                    range: 8.0,
                    color: Color::srgb(0.35, 0.85, 1.0),
                    shadows_enabled: false,
                    ..default()
                },
                Transform::default(),
                WaitIndicatorBeacon {
                    base_intensity: 45_000.0,
                },
                Name::new("WaitIndicator_BeaconLight"),
            ));
        })
        .id();

    state.wait_indicator_root = Some(root);
}

fn animate_wait_indicator(
    time: Res<Time>,
    mut ring_query: Query<(&mut Transform, &WaitIndicatorRing)>,
    mut beacon_query: Query<(&mut PointLight, &WaitIndicatorBeacon)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (mut transform, ring) in &mut ring_query {
        transform.rotate_axis(Dir3::new_unchecked(ring.axis), ring.speed * dt);
    }

    for (mut light, beacon) in &mut beacon_query {
        let pulse = (elapsed * 5.0).sin() * 0.5 + 0.5;
        light.intensity = beacon.base_intensity * (0.6 + 0.8 * pulse);
    }
}

fn dispatch_manifestation_worker(prompt: String, sender: Sender<ManifestationResult>) {
    std::thread::spawn(move || {
        let repo_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(r"C:\archetypes"));
        let script_path = repo_root.join("scripts").join("manifest_artifact.py");
        let output_glb = repo_root.join("assets").join("scenes").join("manifested_artifact.glb");

        let blender_exe = r"C:\Program Files\Blender Foundation\Blender 4.5\blender.exe";

        println!("[ManifestationWorker] Executing headless Chronos render for: '{prompt}'");

        let status = Command::new(blender_exe)
            .arg("-b")
            .arg("-P")
            .arg(&script_path)
            .arg("--")
            .arg("--prompt")
            .arg(&prompt)
            .arg("--output")
            .arg(&output_glb)
            .status();

        match status {
            Ok(s) if s.success() && output_glb.exists() => {
                println!("[ManifestationWorker] Successfully generated '{prompt}' at {output_glb:?}");
                let _ = sender.send(ManifestationResult {
                    prompt,
                    success: true,
                    detail: "Render complete".to_string(),
                });
            }
            Ok(s) => {
                let msg = format!("Blender exit code {s}");
                eprintln!("[ManifestationWorker] Render failed: {msg}");
                let _ = sender.send(ManifestationResult {
                    prompt,
                    success: false,
                    detail: msg,
                });
            }
            Err(e) => {
                let msg = format!("Failed to spawn Blender: {e}");
                eprintln!("[ManifestationWorker] Process spawn failed: {msg}");
                let _ = sender.send(ManifestationResult {
                    prompt,
                    success: false,
                    detail: msg,
                });
            }
        }
    });
}

fn poll_manifestation_results(
    mut commands: Commands,
    channels: Res<ManifestationChannels>,
    mut state: ResMut<ManifestationState>,
    asset_server: Res<AssetServer>,
) {
    let Ok(receiver) = channels.receiver.lock() else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        // Despawn wait indicator immediately before object manifests
        if let Some(indicator) = state.wait_indicator_root.take() {
            commands.entity(indicator).despawn();
        }

        if result.success {
            state.phase = ManifestationPhase::Completed;
            state.status_message = format!("✦ Manifested: '{}' ✦", result.prompt);

            // Despawn previous artifact if present
            if let Some(old_entity) = state.active_artifact.take() {
                commands.entity(old_entity).despawn();
            }

            let p = MANIFESTATION_PEDESTAL_POS;
            let cushion_y = p.y + 1.34; // Top of cushion

            // Spawn celestial manifestation entrance flash
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

            // Spawn manifested object rotating atop the pedestal cushion
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
            println!("[ManifestationSystem] Manifested '{}' atop the sacred pedestal!", result.prompt);
        } else {
            state.phase = ManifestationPhase::Failed;
            state.status_message = format!("Manifestation failed: {}", result.detail);
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

fn teardown_manifestation(
    mut commands: Commands,
    query: Query<Entity, With<ManifestationElement>>,
    mut state: ResMut<ManifestationState>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    state.active_artifact = None;
    state.wait_indicator_root = None;
    state.phase = ManifestationPhase::Idle;
    state.prompt_buffer.clear();
}
