use super::{LivingEngineState, LivingSim};
use super::sim::{Harmony, ImplantKind, ImplantSlot, PROTOTYPE_SPHERES};
use crate::chamber::boot::spawn_main_menu;
use crate::modes::game_mode::GameMode;
use crate::modes::ModeRegistry;
use crate::services::ledger::append_to_ledger;
use crate::theme::Archetype;
use bevy::prelude::*;
use serde_json::json;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(LivingEngineState::Running), setup_living_world)
            .add_systems(
                Update,
                (
                    tick_living_engine,
                    handle_living_input,
                    sync_sphere_transforms,
                    update_living_hud,
                )
                    .chain()
                    .run_if(in_state(LivingEngineState::Running)),
            )
            .add_systems(OnEnter(LivingEngineState::Exiting), teardown_living_world);
    }
}

#[derive(Component)]
struct LivingWorldElement;

#[derive(Component)]
struct LivingSphere {
    index: usize,
}

#[derive(Component)]
struct LivingHud;

#[derive(Resource)]
struct LivingSession {
    sim: LivingSim,
    harmony: Harmony,
    status: String,
}

fn setup_living_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut clear: ResMut<ClearColor>,
) {
    clear.0 = Color::srgb(0.04, 0.04, 0.055);
    let sim = LivingSim::default();
    commands.insert_resource(LivingSession {
        sim: sim.clone(),
        harmony: Harmony::default(),
        status: "Tune the three. Tab selects. Q/E omega. R/F radius. 1/2 implants. V infects. C cures in the E-flat window. Esc returns.".to_owned(),
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.2, 14.0).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y),
        crate::chamber::camera::RuntimeGameplayCamera,
        LivingWorldElement,
        Name::new("LivingEngineCamera"),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 1_200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        LivingWorldElement,
    ));
    commands.spawn((
        PointLight {
            intensity: 40_000.0,
            range: 20.0,
            color: Color::srgb(0.55, 0.62, 1.0),
            ..default()
        },
        Transform::from_xyz(0.0, 3.0, 0.0),
        LivingWorldElement,
    ));

    let sphere_mesh = meshes.add(Sphere::new(0.55));
    for (index, archetype) in PROTOTYPE_SPHERES.iter().copied().enumerate() {
        let color = sphere_color(archetype);
        let [x, y, z] = sim.position(index);
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                emissive: LinearRgba::new(0.12, 0.22, 0.45, 1.0),
                metallic: 0.4,
                perceptual_roughness: 0.22,
                ..default()
            })),
            Transform::from_xyz(x, y, z),
            LivingSphere { index },
            LivingWorldElement,
            Name::new(format!("LivingSphere_{index}")),
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.22))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.04, 0.04, 0.05),
            emissive: LinearRgba::new(0.0, 0.0, 0.0, 1.0),
            metallic: 0.8,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.4, 0.0),
        LivingWorldElement,
        Name::new("VirenCore"),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                top: Val::Px(24.0),
                width: Val::Px(560.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.82)),
            GlobalZIndex(920),
            LivingWorldElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LIVING ENGINE"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.88, 0.92)),
                LivingHud,
            ));
        });
}

fn sphere_color(archetype: Archetype) -> Color {
    match archetype {
        Archetype::Architect => Color::srgb(0.23, 0.51, 0.96),
        Archetype::Sentinel => Color::srgb(0.48, 0.64, 0.97),
        Archetype::Oracle => Color::srgb(0.36, 0.30, 0.49),
        _ => Color::WHITE,
    }
}

fn tick_living_engine(time: Res<Time>, mut session: ResMut<LivingSession>) {
    session.harmony = session.sim.breath(time.delta_secs());
}

fn handle_living_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<LivingSession>,
    mut next_state: ResMut<NextState<LivingEngineState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let _ = append_to_ledger(
            GameMode::LivingEngine,
            "living_engine_session_closed",
            json!({
                "harmony": session.harmony.score(),
                "viren": session.sim.viren.is_active(),
            }),
        );
        next_state.set(LivingEngineState::Exiting);
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        session.sim.cycle_selected();
    }
    if keys.pressed(KeyCode::KeyQ) {
        session.sim.tune_selected_omega(-0.8 * 0.016);
    }
    if keys.pressed(KeyCode::KeyE) {
        session.sim.tune_selected_omega(0.8 * 0.016);
    }
    if keys.pressed(KeyCode::KeyR) {
        session.sim.tune_selected_radius(1.6 * 0.016);
    }
    if keys.pressed(KeyCode::KeyF) {
        session.sim.tune_selected_radius(-1.6 * 0.016);
    }
    if keys.just_pressed(KeyCode::Digit1) {
        session
            .sim
            .toggle_implant(ImplantSlot::Pathway, ImplantKind::Dampen);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        session
            .sim
            .toggle_implant(ImplantSlot::Reservoir, ImplantKind::Charge);
    }
    if keys.just_pressed(KeyCode::KeyV) {
        session.sim.infect_hottest();
        session.status = "Viren has entered. Select the infected sphere and press C in the E-flat window.".to_owned();
    }
    if keys.just_pressed(KeyCode::KeyC) {
        session.status = if session.sim.try_cure() {
            "Viren extinguished by counter-frequency.".to_owned()
        } else {
            "Cure refused. Select the infected sphere when its phase strikes the window.".to_owned()
        };
    }
}

fn sync_sphere_transforms(
    session: Res<LivingSession>,
    mut spheres: Query<(&LivingSphere, &mut Transform)>,
) {
    for (sphere, mut transform) in &mut spheres {
        let [x, y, z] = session.sim.position(sphere.index);
        transform.translation = Vec3::new(x, y, z);
        let selected = sphere.index == session.sim.selected;
        transform.scale = Vec3::splat(if selected { 1.18 } else { 1.0 });
    }
}

fn update_living_hud(session: Res<LivingSession>, mut hud: Query<&mut Text, With<LivingHud>>) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let selected = PROTOTYPE_SPHERES[session.sim.selected];
    let h = session.harmony;
    let fail = if h.overloaded() {
        "OVERLOAD"
    } else if h.starving() {
        "STARVATION"
    } else {
        "STABLE"
    };
    let pathway = session.sim.implants[0]
        .map(|_| "DAMPEN")
        .unwrap_or("EMPTY");
    let reservoir = session.sim.implants[1]
        .map(|_| "CHARGE")
        .unwrap_or("EMPTY");
    let viren = if session.sim.viren.is_active() {
        format!(
            "active on {:?}",
            PROTOTYPE_SPHERES[session.sim.viren.infected_index.unwrap_or(0)]
        )
    } else {
        "clear".to_owned()
    };
    text.0 = format!(
        "LIVING ENGINE  [{fail}]\nSelected: {selected:?}   score {:.2}\n\
         coherence {:.2}  strain {:.2}  asymmetry {:.2}  anomaly {:.2}\n\
         aura local {:.2}  reservoir {:.2}  ambient {:.2}\n\
         implants pathway {pathway} / reservoir {reservoir}\n\
         Viren {viren}\n{}",
        h.score(),
        h.coherence,
        h.metabolic_strain,
        h.asymmetry,
        h.anomaly_pressure,
        session.sim.aura.local,
        session.sim.aura.reservoir,
        session.sim.aura.ambient,
        session.status
    );
}

fn teardown_living_world(
    mut commands: Commands,
    query: Query<Entity, With<LivingWorldElement>>,
    registry: Res<ModeRegistry>,
    mut clear: ResMut<ClearColor>,
    mut next_state: ResMut<NextState<LivingEngineState>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<LivingSession>();
    commands.remove_resource::<crate::chamber::ActiveGameMode>();
    clear.0 = Color::BLACK;
    next_state.set(LivingEngineState::Inactive);
    spawn_main_menu(commands, registry);
}
