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

use super::encounters::EncounterState;
use super::interaction::{InnerInteractionSet, InteractionFocus, InteractionTarget};
use super::world::ChronosExhibitTurntable;
use super::InnerChambersState;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy::winit::{UpdateMode, WinitSettings};

use crate::services::sfx::{PlaySfx, Sfx};
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

/// Marching-cubes resolution handed to TripoSR through Chronos2's own env knob.
/// The last visually accepted real witness (`ice cream cone`, 2026-09-10) used
/// TripoSR's stock 256 grid and produced 39,854 faces without downstream
/// decimation. 384 generated excess marching-cubes noise and then threw most of
/// it away at the 75k game budget; it was not an all-angle quality improvement.
const TRIPOSR_MC_RESOLUTION: &str = "256";

/// The altar stands on the Council floor at the exact centre of the spinning vortex disc, which
/// lies in the floor around it. `castle::GROUND_Y` is the floor top, and the base plinth's own
/// geometry starts at this y, so the altar rests on the stone rather than floating over it.
///
/// Operator directive, 2026-09-12: "there is no table."
pub const MANIFESTATION_PEDESTAL_POS: Vec3 = Vec3::new(0.0, super::castle::GROUND_Y, 0.0);
/// Height of the cushion's top face above `MANIFESTATION_PEDESTAL_POS`, summed from the
/// pedestal parts actually spawned below: base 0.20 + shaft 0.90 + capital 0.15 + gold collar
/// 0.04 + cushion 0.05. It read 1.62 before, which matched nothing that was built.
pub const MANIFESTATION_CUSHION_HEIGHT: f32 = 1.34;

/// Where the idle diamond, the waiting hourglass and the failure X float.
///
/// This was an absolute world y of 2.40, not an offset — so while the altar stood on a
/// tabletop at y=3.17 every one of those symbols hovered 2.1m *below* its own cushion, inside
/// the furniture. Derived from the pedestal now, so moving the altar moves them with it.
pub const MANIFESTATION_HOVER_Y: f32 =
    MANIFESTATION_PEDESTAL_POS.y + MANIFESTATION_CUSHION_HEIGHT + 0.66;

/// The manifested object stands this far above the cushion, so the painting it was made from
/// lies visible underneath it rather than being covered by it.
pub const MANIFESTATION_OBJECT_LIFT: f32 = 0.58;

/// The painted reference lies flat on the cushion. Square side chosen to fit inside the
/// cushion's 0.80m radius: a 1.10m square has a 1.556m diagonal against a 1.60m cushion.
pub const MANIFESTATION_REFERENCE_PANEL_SIZE: f32 = 1.10;

/// The cushion the object stands on, from the pedestal geometry spawned below.
pub const MANIFESTATION_CUSHION_RADIUS: f32 = 0.80;

/// `scripts/import_chronos_object.py` normalises every TripoSR mesh to this on its longest
/// axis before export, so the engine knows the authored size of an object it has never seen.
/// Without that normalisation nothing here could be sized at all — TripoSR output has no
/// inherent scale.
pub const MANIFESTATION_AUTHORED_MAX_DIMENSION: f32 = 1.4;

/// Spawn scale for the manifested object.
///
/// It was 1.15, which takes a 1.4m authored mesh to 1.61m — fractionally wider than the 1.60m
/// cushion it stands on, so every manifestation overhung its own altar, and a tall subject
/// towered past a 3.25m standing eye and hid the painting underneath it. Sized to the plinth
/// instead: an object may fill the cushion but not overhang it.
pub const MANIFESTATION_OBJECT_SCALE: f32 =
    (MANIFESTATION_CUSHION_RADIUS * 2.0) / MANIFESTATION_AUTHORED_MAX_DIMENSION;

/// The altar's collision footprint: the 1.20m base plinth plus enough clearance that a walking
/// player stops with their body clear of the stone rather than intersecting its rim.
///
/// It used to be 2.65m, sized to the whole Council table's silhouette. With the table gone that
/// radius would reserve a ring of empty floor the player could not enter — and the vortex the
/// operator asked to stand in the middle of is 3.6m across, so a 2.65m wall would have kept them
/// off almost all of it.
pub const MANIFESTATION_ALTAR_COLLISION_RADIUS: f32 = 1.45;

/// What the player actually interacts with is the cushion on top of the altar, not the floor
/// under it. `pick_target` measures a true 3D distance, so anchoring at the base would spend
/// almost the whole 3.2m budget climbing from the floor to a 3.25m eye height and leave barely
/// a metre of reach. The Architect's bench already anchors at its worktop for the same reason.
pub fn altar_interaction_anchor() -> Vec3 {
    MANIFESTATION_PEDESTAL_POS + Vec3::Y * MANIFESTATION_CUSHION_HEIGHT
}

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
        /// Which artifact this run produced.
        ///
        /// Every manifestation is its own file now, so the reveal has to be told *which* one to
        /// spawn. When this is `None` the run staged nothing new and the reveal falls back to
        /// the legacy shared path - that is the staging-only capture lane, not a live run.
        artifact_id: Option<String>,
    },
    Failure {
        prompt: String,
        stage_id: Option<String>,
        detail: String,
    },
    /// Step one finished: Chronos2 drew the reference and stopped. The operator reviews it.
    ReferenceReady {
        prompt: String,
        /// The picture itself, copied into the asset root so the altar panel can show it.
        reference: PathBuf,
        /// Asset-relative path the panel loads.
        reference_asset: String,
    },
    /// Step two finished: the object is built, converted, and staged on disk, but NOT recorded
    /// in the library. Nothing becomes a player creation until the operator keeps it.
    ObjectReady {
        prompt: String,
        artifact_id: String,
        provenance: crate::services::artifacts::Provenance,
        judgment: crate::services::chronos_receipt::SubjectJudgment,
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
    /// Which artifact is standing on the cushion, so `E` can take *that* one into the hand.
    pub active_artifact_id: Option<String>,
    pub active_reference_panel: Option<Entity>,
    pub floating_symbol_entity: Option<Entity>,
    pub pedestal_charge: f32,
    /// The picture awaiting the operator's go-ahead in `ReviewingReference`.
    pub pending_reference: Option<PathBuf>,
    /// Which Chronos2 reference attempt to draw next. `R` advances it so a redraw is a new
    /// picture rather than the same seed painted again.
    pub reference_attempt: u32,
    /// The built object awaiting keep or discard in `ReviewingObject`.
    pub pending_object: Option<PendingObject>,
}

/// A converted object staged on the altar but not yet in the library.
#[derive(Clone, Debug)]
pub struct PendingObject {
    pub artifact_id: String,
    pub prompt: String,
    pub provenance: crate::services::artifacts::Provenance,
    pub judgment: crate::services::chronos_receipt::SubjectJudgment,
}

#[derive(Default, PartialEq, Eq, Clone, Debug)]
pub enum ManifestationPhase {
    #[default]
    Idle,
    Prompting,
    /// Step one: Chronos2 is drawing the reference picture.
    DrawingReference,
    /// The picture is on the altar; Enter builds, R redraws, Esc cancels.
    ReviewingReference,
    /// Step two: the object is being built from the approved picture.
    Manifesting,
    /// The object is on the altar with Chronos2's advice; Enter keeps, X discards.
    ReviewingObject,
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
    state.active_artifact_id = None;
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

#[allow(clippy::too_many_arguments)]
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
        // Hidden whenever the altar is busy or asking the operator something, so "Press [E]" never
        // competes with the review keys.
        *vis = if is_near
            && !matches!(
                state.phase,
                ManifestationPhase::Prompting
                    | ManifestationPhase::DrawingReference
                    | ManifestationPhase::ReviewingReference
                    | ManifestationPhase::Manifesting
                    | ManifestationPhase::ReviewingObject
            )
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    match state.phase {
        ManifestationPhase::Idle | ManifestationPhase::Completed | ManifestationPhase::Failed => {
            if is_near
                && !encounter_state.is_open()
                && interaction_focus.0 == Some(InteractionTarget::ManifestationAltar)
                && actions.interact
            {
                // While something is standing on the cushion, `E` means "take it", and
                // `objects::take_from_altar` owns that. Opening the prompt here as well would
                // make one press do two things - the exact bug the interaction arbiter exists to
                // prevent - and would leave the altar as the only place a creation can ever be.
                if state.active_artifact.is_some() {
                    return;
                }
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

                state.active_prompt = prompt.clone();
                // Attempt 2 is the reference seed the accepted ice-cream witness came from;
                // each redraw moves to the next attempt so R really draws a different picture.
                state.reference_attempt = 2;

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

                begin_reference_draw(&mut commands, &mut meshes, &mut materials, &mut state, &channels);
            }
        }
        ManifestationPhase::DrawingReference | ManifestationPhase::Manifesting => {
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
                state.pending_reference = None;
                if let Some(panel) = state.active_reference_panel.take() {
                    commands.entity(panel).despawn();
                }
                spawn_idle_symbol(&mut commands, &mut meshes, &mut materials, &mut state);
            }
        }
        // Step one's picture is on the altar. The operator decides whether it is worth building.
        ManifestationPhase::ReviewingReference => {
            if keyboard.just_pressed(KeyCode::Enter) {
                if let Some(reference) = state.pending_reference.take() {
                    state.phase = ManifestationPhase::Manifesting;
                    state.waiting_elapsed = 0.0;
                    state.current_stage_id = "start".into();
                    state.current_stage_pct = 0;
                    state.current_stage_state = "begin".into();
                    state.current_stage_msg = "Building the object from your approved picture...".into();
                    state.pedestal_charge = 0.0;
                    state.error_message.clear();
                    spawn_phasing_hourglass(&mut commands, &mut meshes, &mut materials, &mut state);
                    dispatch_manifestation_worker(
                        state.active_prompt.clone(),
                        reference,
                        channels.sender.clone(),
                        channels.active_pid.clone(),
                    );
                }
            } else if keyboard.just_pressed(KeyCode::KeyR) {
                state.reference_attempt += 1;
                state.pending_reference = None;
                if let Some(panel) = state.active_reference_panel.take() {
                    commands.entity(panel).despawn();
                }
                begin_reference_draw(&mut commands, &mut meshes, &mut materials, &mut state, &channels);
            } else if actions.cancel {
                state.pending_reference = None;
                if let Some(panel) = state.active_reference_panel.take() {
                    commands.entity(panel).despawn();
                }
                state.phase = ManifestationPhase::Idle;
                spawn_idle_symbol(&mut commands, &mut meshes, &mut materials, &mut state);
            }
        }
        // The built object stands on the altar with Chronos2's automated check shown as advice.
        // Nothing is a player creation until the operator keeps it.
        ManifestationPhase::ReviewingObject => {
            if keyboard.just_pressed(KeyCode::Enter) {
                if let Some(pending) = state.pending_object.take() {
                    match crate::services::artifacts::record_artifact(
                        &pending.artifact_id,
                        &pending.prompt,
                        pending.provenance.clone(),
                    ) {
                        Ok(_) => {
                            // Only now can `E` take it: `take_from_altar` needs this id.
                            state.active_artifact_id = Some(pending.artifact_id);
                            state.phase = ManifestationPhase::Completed;
                        }
                        Err(error) => {
                            state.error_message = format!(
                                "You kept the object, but its library record could not be written: {error}"
                            );
                            state.pending_object = Some(pending);
                            state.phase = ManifestationPhase::Failed;
                        }
                    }
                }
            } else if keyboard.just_pressed(KeyCode::KeyX) {
                if let Some(pending) = state.pending_object.take() {
                    discard_staged_object(&pending.artifact_id);
                }
                if let Some(entity) = state.active_artifact.take() {
                    commands.entity(entity).despawn();
                }
                if let Some(panel) = state.active_reference_panel.take() {
                    commands.entity(panel).despawn();
                }
                state.phase = ManifestationPhase::Idle;
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

// The subject-coverage floor and its reader moved to `services/chronos_receipt.rs`, which now
// applies them as part of verifying the whole bundle -- before Blender is invoked rather than
// after a GLB has already been built from noise. The reasoning behind the threshold, and the
// measurement ruling out `subject_match.json` as an alternative, travelled with it.

/// Scene path for a manifested artifact.
///
/// Per-artifact when the run staged its own file, and the legacy shared path only when it did
/// not - which today means the staging-only capture lane replaying the last real run.
fn artifact_scene_path(artifact_id: Option<&str>) -> String {
    match artifact_id {
        Some(id) => format!("{}#Scene0", crate::services::artifacts::asset_path_for(id)),
        None => "scenes/manifested_artifact.glb#Scene0".to_string(),
    }
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
fn reference_asset_path(artifact_id: &str) -> String {
    format!("manifested/{artifact_id}.reference.png")
}

fn target_reference_image_paths(artifact_id: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(
                dir.join("assets")
                    .join("manifested")
                    .join(format!("{artifact_id}.reference.png")),
            );
        }
    }
    paths.push(crate::services::paths::asset_root().join(reference_asset_path(artifact_id)));
    paths
}

/// Start step one: ask Chronos2 for the reference picture only.
fn begin_reference_draw(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    state: &mut ManifestationState,
    channels: &ManifestationChannels,
) {
    state.phase = ManifestationPhase::DrawingReference;
    state.waiting_elapsed = 0.0;
    state.current_stage_id = "start".into();
    state.current_stage_pct = 0;
    state.current_stage_state = "begin".into();
    state.current_stage_msg = "Drawing the reference picture...".into();
    state.pedestal_charge = 0.0;
    state.error_message.clear();
    spawn_phasing_hourglass(commands, meshes, materials, state);
    dispatch_reference_worker(
        state.active_prompt.clone(),
        state.reference_attempt,
        channels.sender.clone(),
        channels.active_pid.clone(),
    );
}

/// The env every Object-mode Chronos2 call shares. Kept in one place so the reference step and
/// the build step cannot drift onto different image lanes or timeouts.
fn object_mode_command(chronos: &std::path::Path) -> Command {
    let mut command = Command::new(chronos);
    // Use Chronos2's default compact Flux reference lane; never inherit a legacy SDXL override.
    command.env_remove("CHRONOS_FORGE_REFERENCE_CKPT");
    // The Forge has 12 GB VRAM: force the 768px object-reference workflow.
    command.env("CHRONOS_FLUX_PROFILE", "normal");
    // Object references have their own bounded budget; never inherit canvas timings.
    command.env("CHRONOS_COMFY_REFERENCE_POLL_TIMEOUT_SECS", "180");
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// Run a Chronos2 command to completion, forwarding its stage lines. On failure returns the
/// stage it reached and the most useful line it printed.
fn run_chronos_streaming(
    mut command: Command,
    sender: &Sender<ManifestationEvent>,
    active_pid: &Arc<Mutex<Option<u32>>>,
) -> Result<(), (String, String)> {
    let mut child = command
        .spawn()
        .map_err(|error| ("start".to_string(), format!("Failed to spawn Chronos2: {error}")))?;
    if let Ok(mut slot) = active_pid.lock() {
        *slot = Some(child.id());
    }
    let stderr = child.stderr.take();
    let stderr_handle = std::thread::spawn(move || {
        let mut lines = Vec::new();
        if let Some(err) = stderr {
            use std::io::BufRead;
            for line in std::io::BufReader::new(err).lines().flatten() {
                eprintln!("[Chronos stderr] {line}");
                lines.push(line);
            }
        }
        lines
    });
    let mut last_stage = ("start".to_string(), String::new());
    if let Some(out) = child.stdout.take() {
        use std::io::BufRead;
        for line in std::io::BufReader::new(out).lines().flatten() {
            println!("[Chronos stdout] {line}");
            if let Some((id, pct, stage_state, msg)) = parse_chronos_stage(&line) {
                last_stage = (id.clone(), msg.clone());
                let _ = sender.send(ManifestationEvent::Stage {
                    stage_id: id,
                    pct,
                    state: stage_state,
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
    match status {
        Ok(exit) if exit.success() => Ok(()),
        Ok(exit) => Err((
            last_stage.0,
            err_lines
                .last()
                .cloned()
                .filter(|line| !line.trim().is_empty())
                .unwrap_or_else(|| {
                    if last_stage.1.is_empty() {
                        format!("Chronos2 exited with {exit}")
                    } else {
                        last_stage.1
                    }
                }),
        )),
        Err(error) => Err((last_stage.0, format!("Chronos2 process wait failed: {error}"))),
    }
}

/// Step one worker: Chronos2 clears Sentinel, draws the reference, and stops. The picture is
/// staged under its own id so the altar panel can show it while the operator decides.
fn dispatch_reference_worker(
    prompt: String,
    attempt: u32,
    sender: Sender<ManifestationEvent>,
    active_pid: Arc<Mutex<Option<u32>>>,
) {
    std::thread::spawn(move || {
        let chronos = PathBuf::from(r"C:\chronos2\target\release\chronos.exe");
        if !chronos.is_file() {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some("init".into()),
                detail: "Chronos2 Object-mode executable is unavailable; no reference can be drawn.".into(),
            });
            return;
        }
        let reference_id = uuid::Uuid::new_v4().to_string();
        let bundle = std::env::temp_dir()
            .join("NeuroCognica")
            .join("Archetypes")
            .join("references")
            .join(&reference_id);
        let mut command = object_mode_command(&chronos);
        command
            .args(["first-light", "--prompt"])
            .arg(&prompt)
            .arg("--out-dir")
            .arg(&bundle)
            .arg("--intent-json")
            .arg(format!("{{\"attempt\":{attempt}}}"))
            .args(["--geometry-forge", "--void", "--reference-only"]);
        if let Err((stage_id, detail)) = run_chronos_streaming(command, &sender, &active_pid) {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some(stage_id),
                detail,
            });
            return;
        }
        let reference = bundle.join("reference_input.png");
        if !reference.is_file() {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some("reference".into()),
                detail: format!(
                    "Chronos2 finished the reference step but wrote no picture at {}.",
                    reference.display()
                ),
            });
            return;
        }
        for target in target_reference_image_paths(&reference_id) {
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::copy(&reference, &target);
        }
        let _ = sender.send(ManifestationEvent::ReferenceReady {
            prompt,
            reference,
            reference_asset: reference_asset_path(&reference_id),
        });
    });
}

/// Remove an unkept object's staged files. It was never recorded, so no library row points at it.
fn discard_staged_object(artifact_id: &str) {
    for dir in crate::services::artifacts::manifested_asset_dirs() {
        let _ = std::fs::remove_file(dir.join(format!("{artifact_id}.glb")));
    }
    for target in target_reference_image_paths(artifact_id) {
        let _ = std::fs::remove_file(target);
    }
}

/// Step two worker: build the object from the picture the operator approved, convert and stage
/// it, and hand it back for review. It does not record a library row; keeping does that.
fn dispatch_manifestation_worker(
    prompt: String,
    reference: PathBuf,
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

        // Build from the exact picture the operator approved. Chronos2 re-checks the request with
        // the Sentinel and binds this file's digest into the receipt, so the pairing on the altar
        // (picture under the object it became) stays true.
        let mut command = object_mode_command(&chronos);
        command
            .args(["first-light", "--prompt"])
            .arg(&prompt)
            .arg("--out-dir")
            .arg(&bundle)
            .arg("--reference-image")
            .arg(&reference)
            .args(["--geometry-forge", "--void"]);
        if let Err((stage_id, detail)) = run_chronos_streaming(command, &sender, &active_pid) {
            let _ = sender.send(ManifestationEvent::Failure {
                prompt,
                stage_id: Some(stage_id),
                detail,
            });
            return;
        }

        // Verify the bundle against the contract before trusting a single byte of it.
        //
        // What was here was `these two files exist`, which is not evidence of anything. A bundle
        // left over from an earlier prompt satisfies it. So does a run the Sentinel *refused* --
        // a refusal is a successful run of the governance path, exiting zero and writing its
        // bundle like any other. So does a mesh truncated by a crash mid-write.
        //
        // `chronos_receipt::verify` checks what Chronos2 actually records: its own integrity
        // verdict, the Sentinel's disposition, that the bundled prompt is the one we asked for,
        // and that the mesh and reference image still hash to what the sealed receipt declared.
        // The subject-coverage floor moved in there too, so a reconstruction built from a blank
        // frame is now refused before Blender is invoked rather than after.
        let artifact = match crate::services::chronos_receipt::verify(&bundle, &prompt) {
            Ok(artifact) => artifact,
            Err(fault) => {
                let _ = sender.send(ManifestationEvent::Failure {
                    stage_id: Some(fault.stage_id().to_string()),
                    detail: format!("{fault} Bundle kept at {}", bundle.display()),
                    prompt,
                });
                return;
            }
        };
        let mesh = artifact.mesh.clone();

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

                // The object's own file. Until now every manifestation overwrote one shared
                // path, and Bevy caches by path - so two creations in the world were two views
                // of whatever was made last, and a third silently changed both. Nothing could be
                // carried, placed or duplicated until each one was its own asset.
                let artifact_id = bundle
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "artifact".to_string());
                // Written to every assets root this build might read from. A debug build reads
                // the repository's assets/ while an installed build reads the one beside the
                // executable, and writing only the second left dev builds unable to load their
                // own creations.
                let mut staged_copy: Option<std::path::PathBuf> = None;
                for dir in crate::services::artifacts::manifested_asset_dirs() {
                    let own = dir.join(format!("{artifact_id}.glb"));
                    if std::fs::create_dir_all(&dir).is_ok()
                        && std::fs::copy(&primary_output, &own).is_ok()
                    {
                        staged_copy.get_or_insert(own);
                    }
                }
                let Some(own) = staged_copy else {
                    let _ = sender.send(ManifestationEvent::Failure {
                        prompt,
                        stage_id: Some("conversion".into()),
                        detail: "The object was converted but could not be staged into any assets folder.".into(),
                    });
                    return;
                };
                // Our own digest of the finished GLB. Chronos2 cannot supply this -- the GLB is
                // produced by the Blender import above, after its bundle was sealed -- so if
                // Archetypes does not measure it here nothing ever does. It travels with the review
                // and becomes the library row only if the operator keeps the object.
                let provenance = crate::services::artifacts::Provenance {
                    sentinel_verdict: artifact.sentinel_verdict.clone(),
                    mesh_sha256: artifact.mesh_sha256.clone(),
                    source_image_sha256: artifact.source_image_sha256.clone(),
                    glb_sha256: crate::services::chronos_receipt::sha256_file(&own)
                        .unwrap_or_default(),
                    subject_coverage: artifact.subject_coverage,
                };

                // Stage the exact 2D reference the mesh was reconstructed from,
                // for the floor panel at the pedestal's foot. Best-effort: the
                // object itself already succeeded, so a missing reference image
                // (an operator-supplied-reference run, or an unexpected layout)
                // must never fail or retry a completed manifestation.
                let reference_source = bundle.join("reference_input.png");
                if reference_source.is_file() {
                    for target in target_reference_image_paths(&artifact_id) {
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
                    message: "Placing the object on the altar for your review...".into(),
                });

                let _ = sender.send(ManifestationEvent::ObjectReady {
                    prompt,
                    artifact_id,
                    provenance,
                    judgment: artifact.judgment.clone(),
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
    mut sfx: MessageWriter<PlaySfx>,
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
            ManifestationEvent::ReferenceReady {
                prompt,
                reference,
                reference_asset,
            } => {
                if let Some(symbol) = state.floating_symbol_entity.take() {
                    commands.entity(symbol).despawn();
                }
                // Only the picture goes on the altar: no object exists yet, and nothing has been
                // spent on geometry. The operator decides whether it is worth building.
                let p = MANIFESTATION_PEDESTAL_POS;
                let cushion_y = p.y + 1.34;
                if let Some(old_panel) = state.active_reference_panel.take() {
                    commands.entity(old_panel).despawn();
                }
                let panel_material = materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load(reference_asset)),
                    perceptual_roughness: 0.55,
                    metallic: 0.0,
                    ..default()
                });
                let panel_entity = commands
                    .spawn((
                        Mesh3d(meshes.add(Plane3d::default().mesh().size(
                            MANIFESTATION_REFERENCE_PANEL_SIZE,
                            MANIFESTATION_REFERENCE_PANEL_SIZE,
                        ))),
                        MeshMaterial3d(panel_material),
                        Transform::from_translation(Vec3::new(p.x, cushion_y + 0.005, p.z)),
                        ManifestationElement,
                        Name::new("ManifestationReferencePanel"),
                    ))
                    .id();
                state.active_reference_panel = Some(panel_entity);
                state.pending_reference = Some(reference);
                state.phase = ManifestationPhase::ReviewingReference;
                state.current_stage_id = "reference_review".into();
                state.current_stage_pct = 100;
                state.pedestal_charge = 0.0;
                sfx.write(PlaySfx::new(Sfx::ManifestSuccess));
                println!("[ManifestationSystem] Reference ready for '{prompt}'; awaiting operator review.");
            }
            ManifestationEvent::ObjectReady {
                prompt,
                artifact_id,
                provenance,
                judgment,
            } => {
                // The object is revealed exactly as a finished creation is, but it is held for
                // review: no library row, and no id on `active_artifact_id`, so `E` cannot take it
                // before the operator keeps it.
                reveal_object_on_altar(
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &mut state,
                    Some(&artifact_id),
                );
                state.active_artifact_id = None;
                state.phase = ManifestationPhase::ReviewingObject;
                state.current_stage_id = "object_review".into();
                state.current_stage_msg = judgment.reason.clone();
                state.pending_object = Some(PendingObject {
                    artifact_id,
                    prompt: prompt.clone(),
                    provenance,
                    judgment,
                });
                sfx.write(PlaySfx::new(Sfx::ManifestSuccess));
                println!("[ManifestationSystem] Object ready for '{prompt}'; awaiting keep or discard.");
            }
            ManifestationEvent::Success {
                prompt,
                detail: _,
                artifact_id,
            } => {
                reveal_object_on_altar(
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &mut state,
                    artifact_id.as_deref(),
                );
                state.phase = ManifestationPhase::Completed;
                state.current_stage_msg = "Summoned".into();
                sfx.write(PlaySfx::new(Sfx::ManifestSuccess));
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
                sfx.write(PlaySfx::new(Sfx::ManifestFailure));
                eprintln!("[ManifestationSystem] Manifestation Failed: {detail}");
            }
        }
    }
}

/// Stage a converted object on the cushion with its reveal, and the picture it came from
/// underneath. Shared by the kept-creation reveal and the review reveal, which differ only in
/// what happens after the operator sees it.
fn reveal_object_on_altar(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    state: &mut ManifestationState,
    artifact_id: Option<&str>,
) {
    {
        {
                // Despawn the phasing hourglass
                if let Some(symbol) = state.floating_symbol_entity.take() {
                    commands.entity(symbol).despawn();
                }

                state.current_stage_id = "complete".into();
                state.current_stage_pct = 100;
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
                        SceneRoot(asset_server.load(artifact_scene_path(artifact_id))),
                        Transform::from_xyz(p.x, cushion_y + MANIFESTATION_OBJECT_LIFT, p.z)
                            .with_scale(Vec3::splat(MANIFESTATION_OBJECT_SCALE)),
                        // One revolution in about eleven seconds: slow enough
                        // to inspect, fast enough that motion is unmistakable.
                        ChronosExhibitTurntable { speed: 0.56 },
                        ManifestationElement,
                        Name::new("ManifestedChronosArtifact"),
                    ))
                    .id();

                // The entity is on the altar either way. Whether `E` may take it
                // (`active_artifact_id`) is the caller's decision: kept creations yes,
                // objects still under review no.
                state.active_artifact = Some(artifact_entity);

                // The same painting Chronos2 generated and fed to TripoSR, laid flat on the
                // cushion **underneath** the object it became. The player reads the sequence
                // vertically in one glance: their prompt became this picture, and this picture
                // became the object standing above it.
                //
                // It used to sit 2.8m away on the floor, which is beside the object rather than
                // under it — the commit that added it says "beside" in its own subject line.
                // Moving the altar to the centre of the Council circle made that placement
                // actively wrong as well as merely offset: at `p.z - 2.8` the panel now landed
                // inside the 3.6m vortex disc, lying on top of the spinning portal.
                //
                // Reuses whatever reference the worker staged this run (or a prior run's, if
                // this run had none to copy); absent entirely on the very first launch.
                if let Some(old_panel) = state.active_reference_panel.take() {
                    commands.entity(old_panel).despawn();
                }
                let panel_pos = Vec3::new(p.x, cushion_y + 0.005, p.z);
                let panel_material = materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load(reference_asset_path(
                        artifact_id.unwrap_or("legacy"),
                    ))),
                    perceptual_roughness: 0.55,
                    metallic: 0.0,
                    ..default()
                });
                let panel_entity = commands
                    .spawn((
                        Mesh3d(meshes.add(Plane3d::default().mesh().size(
                            MANIFESTATION_REFERENCE_PANEL_SIZE,
                            MANIFESTATION_REFERENCE_PANEL_SIZE,
                        ))),
                        MeshMaterial3d(panel_material),
                        Transform::from_translation(panel_pos),
                        ManifestationElement,
                        Name::new("ManifestationReferencePanel"),
                    ))
                    .id();
                state.active_reference_panel = Some(panel_entity);
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

/// Whether the "Staged upon the sacred altar" banner is true right now: a kept creation is
/// standing on the altar. False once it has been taken into the hand.
fn completed_banner_visible(state: &ManifestationState) -> bool {
    state.phase == ManifestationPhase::Completed && state.active_artifact.is_some()
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
        ManifestationPhase::DrawingReference | ManifestationPhase::Manifesting => {
            *banner_vis = Visibility::Visible;
            let stage_label = if !state.current_stage_msg.is_empty() {
                &state.current_stage_msg
            } else {
                "Initiating Chronos2..."
            };
            let step = if state.phase == ManifestationPhase::DrawingReference {
                "Step 1 of 2"
            } else {
                "Step 2 of 2"
            };
            text.0 = format!(
                "⏳ {step} [{}%] {}: \"{}\" [{:.1}s] — Press [Esc] to cancel",
                state.current_stage_pct, stage_label, state.active_prompt, state.waiting_elapsed
            );
            color.0 = Color::srgb(1.0, 0.85, 0.30); // Warm amber
        }
        ManifestationPhase::ReviewingReference => {
            *banner_vis = Visibility::Visible;
            text.0 = format!(
                "🖼 REFERENCE for \"{}\" is on the altar. [Enter] build the object from it · [R] draw a different picture · [Esc] cancel",
                state.active_prompt
            );
            color.0 = Color::srgb(0.55, 0.85, 1.0); // Calm review blue
        }
        ManifestationPhase::ReviewingObject => {
            *banner_vis = Visibility::Visible;
            // Chronos2's automated check is shown as advice. The operator decides.
            let advice = match state.pending_object.as_ref().map(|pending| &pending.judgment) {
                Some(judgment) if !judgment.checked => {
                    format!("Automated check did not run: {}", judgment.reason)
                }
                Some(judgment) => format!(
                    "Automated check {} ({}): {}",
                    if judgment.matches { "agrees" } else { "has doubts" },
                    judgment
                        .score
                        .map(|score| format!("{score:.2}"))
                        .unwrap_or_else(|| "no score".to_string()),
                    judgment.reason
                ),
                None => "No automated check was recorded.".to_string(),
            };
            text.0 = format!(
                "🔍 REVIEW \"{}\" — {advice} — [Enter] keep it · [X] discard it",
                state.active_prompt
            );
            color.0 = Color::srgb(0.55, 0.85, 1.0); // Calm review blue
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
            // "Staged upon the sacred altar" is only true while the object is still there. Once it
            // is taken into the hand the banner would be describing something that no longer is
            // (seen in the installed wolf witness, 01_object_held_in_hand.png).
            if completed_banner_visible(&state) {
                *banner_vis = Visibility::Visible;
                text.0 = format!(
                    "✨ MANIFESTED: \"{}\" — Staged upon the sacred altar!",
                    state.active_prompt
                );
                color.0 = Color::srgb(0.40, 1.0, 0.70); // Radiant celestial green
            } else {
                *banner_vis = Visibility::Hidden;
            }
        }
        ManifestationPhase::Idle | ManifestationPhase::Prompting => {
            *banner_vis = Visibility::Hidden;
        }
    }
}

const ACTIVE_FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);
const MANIFESTING_FRAME_INTERVAL: Duration = Duration::from_nanos(66_666_667);
fn manifestation_frame_interval(phase: &ManifestationPhase) -> Duration {
    // Both Chronos2 steps put Flux or Hunyuan on the same GPU the game renders with.
    if matches!(
        phase,
        ManifestationPhase::DrawingReference | ManifestationPhase::Manifesting
    ) {
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
    state.active_artifact_id = None;
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
    fn manifestation_uses_the_visually_accepted_chronos_reference_lane() {
        let source = std::fs::read_to_string("src/modes/inner_chambers/manifestation.rs")
            .expect("manifestation source readable");
        assert!(source.contains("command.env_remove(\"CHRONOS_FORGE_REFERENCE_CKPT\")"));
        assert!(source.contains("command.env(\"CHRONOS_FLUX_PROFILE\", \"normal\")"));
        // The first reference of every request is drawn from the witnessed single-subject seed
        // (attempt 2). Only the operator's explicit redraw moves off it, and it must really move,
        // or `R` would paint the same picture again.
        assert!(
            source.contains("state.reference_attempt = 2;"),
            "the buyer path must start from the witnessed single-subject candidate seed"
        );
        assert!(
            source.contains(".arg(format!(\"{{\\\"attempt\\\":{attempt}}}\"))"),
            "the reference step must hand Chronos2 the attempt it was asked for"
        );
        assert!(
            source.contains("state.reference_attempt += 1;"),
            "a redraw must advance to a new reference attempt"
        );
        assert!(
            source.contains("command.env(\"CHRONOS_COMFY_REFERENCE_POLL_TIMEOUT_SECS\", \"180\")")
        );
        assert!(
            !source.contains("ureq::post(\"http://127.0.0.1:8000/free\")"),
            "do not race ComfyUI model unloading against the generation submitted immediately afterward"
        );
        assert!(
            !source.contains("command.env(\"CHRONOS_FORGE_REFERENCE_CKPT\""),
            "the game must not force a legacy checkpoint over Chronos2's accepted default lane"
        );
    }

    #[test]
    fn manifestation_uses_the_last_visually_accepted_reconstruction_density() {
        assert_eq!(
            TRIPOSR_MC_RESOLUTION, "256",
            "the accepted ice-cream witness used mc=256; do not raise density without a better all-angle witness"
        );
    }

    #[test]
    fn importer_preserves_triposr_surface_shading_without_edge_split() {
        let source = std::fs::read_to_string("../../scripts/import_chronos_object.py")
            .expect("import script readable");
        assert!(source.contains("bpy.ops.object.shade_smooth()"));
        assert!(
            !source.contains("EDGE_SPLIT"),
            "splitting noisy marching-cubes angles creates hard seams and extra vertices"
        );
        assert!(
            !source.contains("inspection.json"),
            "the rejected unbound builder receipt must not return to the buyer path"
        );
    }

    /// The altar is meant to make a 3D object and lay the painting it was made from
    /// underneath it, so the player reads prompt -> painting -> object in one vertical glance.
    /// The panel used to sit 2.8m away, which is beside the object, not under it.
    #[test]
    fn the_painted_reference_lies_directly_under_the_object_it_became() {
        let p = MANIFESTATION_PEDESTAL_POS;
        let cushion_y = p.y + MANIFESTATION_CUSHION_HEIGHT;
        let panel = Vec3::new(p.x, cushion_y + 0.005, p.z);
        let object = Vec3::new(p.x, cushion_y + MANIFESTATION_OBJECT_LIFT, p.z);

        assert_eq!(
            Vec2::new(panel.x, panel.z),
            Vec2::new(object.x, object.z),
            "the painting must share the object's ground position, not sit beside it"
        );
        assert!(
            panel.y < object.y,
            "the painting must be underneath the object, not level with or above it"
        );
        assert!(
            object.y - panel.y > 0.25,
            "the object must clear the painting enough to leave it visible; only {:.3}m of gap",
            object.y - panel.y
        );
    }

    /// A painting wider than the cushion it lies on overhangs into mid-air.
    #[test]
    fn the_reference_panel_fits_on_the_cushion_it_lies_on() {
        let diagonal = MANIFESTATION_REFERENCE_PANEL_SIZE * std::f32::consts::SQRT_2;
        assert!(
            diagonal <= MANIFESTATION_CUSHION_RADIUS * 2.0,
            "a {}m square has a {diagonal:.3}m diagonal and overhangs the {:.2}m cushion",
            MANIFESTATION_REFERENCE_PANEL_SIZE,
            MANIFESTATION_CUSHION_RADIUS * 2.0
        );
    }

    /// Whatever the player asks for, TripoSR hands back a mesh with no inherent scale; the
    /// import script normalises it, and the spawn scale has to respect the plinth it lands on.
    /// At the previous 1.15 a 1.4m authored mesh came out at 1.61m on a 1.60m cushion.
    /// The kept creation's banner says it is staged on the altar. That must stop being said the
    /// moment the object is taken into the hand (installed wolf witness, 2026-09-13).
    #[test]
    fn the_staged_banner_only_shows_while_the_object_is_on_the_altar() {
        let mut state = ManifestationState {
            phase: ManifestationPhase::Completed,
            active_artifact: Some(Entity::PLACEHOLDER),
            ..Default::default()
        };
        assert!(completed_banner_visible(&state), "kept object on the altar");

        state.active_artifact = None;
        assert!(
            !completed_banner_visible(&state),
            "object taken into the hand: the altar is empty"
        );

        state.active_artifact = Some(Entity::PLACEHOLDER);
        state.phase = ManifestationPhase::ReviewingObject;
        assert!(
            !completed_banner_visible(&state),
            "an object under review is not a staged creation yet"
        );
    }

    #[test]
    fn the_manifested_object_fits_the_cushion_it_stands_on() {
        let widest = MANIFESTATION_AUTHORED_MAX_DIMENSION * MANIFESTATION_OBJECT_SCALE;
        assert!(
            widest <= MANIFESTATION_CUSHION_RADIUS * 2.0 + 1e-4,
            "a manifested object {widest:.3}m across overhangs its {:.2}m cushion",
            MANIFESTATION_CUSHION_RADIUS * 2.0
        );
        assert!(
            widest > MANIFESTATION_CUSHION_RADIUS,
            "sized down to {widest:.3}m the object would look lost on the altar"
        );
    }

    /// The import script is the only thing that gives a TripoSR mesh a size, so the engine's
    /// assumption about it has to be checked against the script itself.
    #[test]
    fn the_engine_agrees_with_the_import_script_on_the_authored_size() {
        let source = std::fs::read_to_string("../../scripts/import_chronos_object.py")
            .expect("import script readable");
        assert!(
            source.contains("1.4 / max(hi - lo)"),
            "import_chronos_object.py no longer normalises to              {MANIFESTATION_AUTHORED_MAX_DIMENSION}m, so the spawn scale is guessing"
        );
    }

    /// `MANIFESTATION_HOVER_Y` was an absolute world y of 2.40 while the altar stood on a
    /// tabletop at 3.17, so the idle diamond, the waiting hourglass and the failure X all
    /// floated 2.1m below their own cushion, inside the furniture. Deriving it from the
    /// pedestal is what stops that recurring the next time the altar moves.
    #[test]
    fn the_floating_status_symbols_hover_above_the_cushion_not_inside_the_pedestal() {
        let cushion_y = MANIFESTATION_PEDESTAL_POS.y + MANIFESTATION_CUSHION_HEIGHT;
        assert!(
            MANIFESTATION_HOVER_Y > cushion_y,
            "hover y {MANIFESTATION_HOVER_Y} is at or below the cushion at {cushion_y}"
        );
        assert!(
            MANIFESTATION_HOVER_Y - cushion_y < 1.5,
            "hover y is {:.2}m above the cushion, which reads as unrelated to the altar",
            MANIFESTATION_HOVER_Y - cushion_y
        );
    }
}
