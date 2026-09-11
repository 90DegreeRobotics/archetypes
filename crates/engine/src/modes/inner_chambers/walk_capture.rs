//! Walking capture for the Architect workshop route.
//!
//! The existing `ARCHETYPES_INNER_CAPTURE` harness forces free flight and teleports the camera
//! to each vantage point, which is exactly why a movement bug survived in the room-wall
//! collision for so long: a flying, teleporting camera never touches the walking code path.
//!
//! This harness proves the opposite. It stands the player at the Architect doorway in
//! `Walking` mode and then *holds movement keys*, so every frame runs the real
//! `player_locomotion` system with real ground, obstacle and wall collision. It then presses
//! the real interact key and the real cancel key. If the wall still ejected a walking player,
//! or the bench prompt were still being clobbered, or cancelling at the bench still exited the
//! castle, the resulting frames and the written report would show it.
//!
//! Enabled by `ARCHETYPES_WALK_CAPTURE`; never on during normal play.

use super::camera::{CameraController, LocomotionMode, PlayerCamera};
use super::interaction::{InteractionFocus, InteractionTarget};
use super::{InnerChambersState, TriggerInnerChambers};
use crate::chamber::boot::MainMenuUi;
use crate::chamber::ChamberState;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{save_to_disk, Screenshot};
use std::fs;
use std::path::PathBuf;

/// Walking lane through the Architect room. The lane is offset from the room's centre line
/// because the archetype figure is now a real obstacle standing on it — walking straight at
/// the figure is correctly blocked, so the player goes around it exactly as a person would.
const WALK_START: Vec3 = Vec3::new(1.8, 3.25, -46.0);

#[derive(Clone, Copy, Debug)]
enum Beat {
    Hold(&'static [KeyCode]),
    /// Press for exactly one frame. `just_pressed` lives for a single frame, so a tap is what
    /// every menu key in the castle actually responds to.
    Tap(KeyCode),
    Shot(&'static str),
    Finish,
}

#[derive(Resource)]
pub struct WalkCaptureRun {
    dir: PathBuf,
    boot_skip_requested: bool,
    menu_skipped: bool,
    placed: bool,
    entered_at: Option<f32>,
    beats: Vec<(f32, Beat)>,
    next_beat: usize,
    held: Vec<KeyCode>,
    tapped: Vec<KeyCode>,
    report: Vec<String>,
    exit_at: Option<f32>,
}

impl WalkCaptureRun {
    pub fn from_env() -> Option<Self> {
        if std::env::var_os("ARCHETYPES_WALK_CAPTURE").is_none() {
            return None;
        }
        let dir = std::env::var_os("ARCHETYPES_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("artifacts/visual-proof/architect-workshop-walk"));
        let _ = fs::create_dir_all(&dir);

        // Times are seconds after the mode reaches Navigating. The first frames need real
        // settle time for the ~200 procedural meshes to finish uploading.
        // Writes one real plan titled "proof" with the intent "walk", through the same confirm
        // step a player uses. It lands in the player's own local plan journal, which is the
        // point: this proves the write path in the installed build, not in a test fixture.
        let beats = vec![
            (4.0, Beat::Shot("00_doorway_standing")),
            (4.6, Beat::Hold(&[KeyCode::KeyW])),
            // Stopping short of the bench keeps the whole bench in frame, and keeps the
            // sidestep clear of the archetype figure, which is now a real obstacle.
            (8.05, Beat::Hold(&[KeyCode::KeyA])),
            (8.19, Beat::Hold(&[])),
            (9.0, Beat::Shot("01_at_the_bench_prompt")),
            (9.5, Beat::Tap(KeyCode::KeyE)),
            (10.3, Beat::Shot("02_bench_open")),
            // Taps are spaced 0.25s apart: an unfocused window runs the reactive-low-power
            // schedule at ~15fps, and beats closer than one frame apart bunch into a single
            // frame, which silently desynchronises the typed sequence.
            (10.8, Beat::Tap(KeyCode::KeyN)),
            (11.1, Beat::Tap(KeyCode::KeyP)),
            (11.35, Beat::Tap(KeyCode::KeyR)),
            (11.6, Beat::Tap(KeyCode::KeyO)),
            (11.85, Beat::Tap(KeyCode::KeyO)),
            (12.1, Beat::Tap(KeyCode::KeyF)),
            (12.45, Beat::Tap(KeyCode::Enter)),
            (12.8, Beat::Tap(KeyCode::KeyW)),
            (13.05, Beat::Tap(KeyCode::KeyA)),
            (13.3, Beat::Tap(KeyCode::KeyL)),
            (13.55, Beat::Tap(KeyCode::KeyK)),
            (13.9, Beat::Tap(KeyCode::Enter)),
            (14.4, Beat::Shot("03_confirm_before_writing")),
            (14.9, Beat::Tap(KeyCode::Enter)),
            (15.6, Beat::Shot("04_plan_written")),
            (16.1, Beat::Tap(KeyCode::Escape)),
            (16.8, Beat::Shot("05_back_in_the_room")),
            (17.2, Beat::Finish),
        ];

        Some(Self {
            dir,
            boot_skip_requested: false,
            menu_skipped: false,
            placed: false,
            entered_at: None,
            beats,
            next_beat: 0,
            held: Vec::new(),
            tapped: Vec::new(),
            report: Vec::new(),
            exit_at: None,
        })
    }
}

pub(crate) fn drive_walk_capture(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<WalkCaptureRun>,
    inner_state: Res<State<InnerChambersState>>,
    chamber_state: Res<State<ChamberState>>,
    mut next_chamber_state: ResMut<NextState<ChamberState>>,
    main_menu: Query<Entity, With<MainMenuUi>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    focus: Res<InteractionFocus>,
    workshop: Res<super::workshop::WorkshopState>,
    mut player: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let now = time.elapsed_secs();

    if !run.boot_skip_requested && now >= 0.3 {
        next_chamber_state.set(ChamberState::MainMenu);
        run.boot_skip_requested = true;
    }
    if run.boot_skip_requested && !run.menu_skipped && *chamber_state.get() == ChamberState::MainMenu {
        for entity in &main_menu {
            commands.entity(entity).despawn();
        }
        commands.insert_resource(TriggerInnerChambers);
        run.menu_skipped = true;
    }
    if *inner_state.get() != InnerChambersState::Navigating {
        return;
    }

    let Ok((mut transform, mut controller)) = player.single_mut() else {
        return;
    };

    if !run.placed {
        // One teleport, to the doorway only. Everything after this is real walking.
        *transform = Transform::from_translation(WALK_START);
        controller.mode = LocomotionMode::Walking;
        controller.yaw = 0.0; // forward is -Z: straight into the Architect room
        controller.pitch = -0.10;
        controller.is_grounded = true;
        controller.velocity_y = 0.0;
        transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
            * Quat::from_axis_angle(Vec3::X, controller.pitch);
        run.placed = true;
        run.entered_at = Some(now);
        run.report.push(format!("start  pos={:?} mode=Walking", transform.translation));
        return;
    }

    let Some(entered_at) = run.entered_at else {
        return;
    };
    let elapsed = now - entered_at;

    // Release last frame's taps first, so a repeated key (the two O's in "proof") registers as
    // two separate presses rather than one held key.
    for key in run.tapped.clone() {
        keyboard.release(key);
    }
    run.tapped.clear();

    while run.next_beat < run.beats.len() && elapsed >= run.beats[run.next_beat].0 {
        let (_, beat) = run.beats[run.next_beat];
        run.next_beat += 1;
        match beat {
            Beat::Hold(keys) => {
                for key in run.held.clone() {
                    keyboard.release(key);
                }
                run.held.clear();
                for key in keys {
                    keyboard.press(*key);
                    run.held.push(*key);
                }
            }
            Beat::Tap(key) => {
                keyboard.press(key);
                run.tapped.push(key);
            }
            Beat::Shot(name) => {
                let path = run.dir.join(format!("{name}.png"));
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
                let line = format!(
                    "{name}  pos=({:.2}, {:.2}, {:.2})  mode={:?}  focus={}  bench={}  plans={}  mode_state={:?}",
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                    controller.mode,
                    describe_focus(&focus),
                    workshop.screen_label(),
                    workshop.plan_count(),
                    inner_state.get(),
                );
                run.report.push(line);
            }
            Beat::Finish => {
                let report = run.report.join("\n");
                let path = run.dir.join("walk_report.txt");
                let _ = fs::write(path, format!("{report}\n"));
                run.exit_at = Some(now + 10.0);
            }
        }
    }

    if let Some(exit_at) = run.exit_at {
        if now >= exit_at {
            std::process::exit(0);
        }
    }
}

fn describe_focus(focus: &InteractionFocus) -> String {
    match focus.0 {
        Some(InteractionTarget::ArchitectWorkshop) => "ArchitectWorkshop".to_owned(),
        Some(InteractionTarget::ManifestationAltar) => "ManifestationAltar".to_owned(),
        Some(InteractionTarget::Archetype(embodiment)) => {
            format!("Archetype({})", embodiment.archetype.theme().name)
        }
        None => "none".to_owned(),
    }
}
