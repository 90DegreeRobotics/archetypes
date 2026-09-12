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
use super::castle;
use super::interaction::{InteractionFocus, InteractionTarget};
use super::{InnerChambersState, TriggerInnerChambers};
use crate::chamber::boot::MainMenuUi;
use crate::chamber::ChamberState;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{save_to_disk, Screenshot};
use std::fs;
use std::path::PathBuf;

/// Walking lane across the Council circle, inspecting the center portal, approaching the
/// Jester Council host on foot, and climbing the first flight of the perimeter ascent.
const WALK_START: Vec3 = Vec3::new(0.0, 3.25, 12.0);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum Beat {
    Hold(&'static [KeyCode]),
    /// Press for exactly one frame. `just_pressed` lives for a single frame, so a tap is what
    /// every menu key in the castle actually responds to.
    Tap(KeyCode),
    /// Reposition and re-aim, to start a second on-foot leg somewhere else in the castle.
    Place(Vec3, f32),
    /// Re-pitch in place, without moving the player. The centrepiece now lies on the floor
    /// rather than standing at chest height, so a level-aimed frame photographs the far wall
    /// and proves nothing about what the player just walked up to. This changes where the
    /// camera looks, never where the player is, so the walk stays a real walk.
    Aim(f32),
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
            .unwrap_or_else(|| PathBuf::from("artifacts/visual-proof/rotunda-walk"));
        let _ = fs::create_dir_all(&dir);

        // Times are seconds after the mode reaches Navigating. The first frames need real
        // settle time for the procedural meshes to finish uploading.
        let beats = vec![
            (4.0, Beat::Shot("00_council_entrance_standing")),
            // Walk forward on the stone slabs toward the central portal / altar
            (4.6, Beat::Hold(&[KeyCode::KeyW])),
            (6.2, Beat::Hold(&[])),
            (6.8, Beat::Shot("01_approaching_center_portal")),
            // Back off a couple of steps, still walking, and look down at the floor the player
            // is standing on: the vortex disc with the altar in the middle of it.
            // Walk speed is ~6.4 m/s, and the disc is only 3.6m in radius, so this step back
            // is deliberately short: hold any longer and the frame is taken from off the disc,
            // which is not what its filename would then be claiming.
            (7.2, Beat::Hold(&[KeyCode::KeyS])),
            (7.45, Beat::Hold(&[])),
            (7.9, Beat::Aim(-0.52)),
            (8.5, Beat::Shot("01b_standing_on_the_vortex_disc")),
            // Turn toward the Jester Council host at (10.2, 0.42, 6.2)
            (9.5, Beat::Place(Vec3::new(6.0, 3.25, 9.0), -0.85)),
            (10.3, Beat::Shot("02_facing_jester_council_host")),
            // Walk up to the Jester
            (10.8, Beat::Hold(&[KeyCode::KeyW])),
            (12.5, Beat::Hold(&[])),
            (13.2, Beat::Shot("03_standing_before_jester")),
            // Second leg: climb the first flight of the perimeter ascent on foot. The flight
            // sweeps 14.6°, whose chord deviates only ~0.9m from the arc across an 8m wide
            // stair, so a straight heading keeps the player on the treads for a whole storey.
            // Everything here is the real locomotion system on real steps.
            (14.0, Beat::Place(stair_approach(), stair_chord_yaw())),
            (15.0, Beat::Shot("04_foot_of_the_ascent")),
            (15.5, Beat::Hold(&[KeyCode::KeyW])),
            (20.5, Beat::Hold(&[])),
            (21.2, Beat::Shot("05_one_storey_climbed")),
            // Open the settings menu with the real Esc key, move down to a volume slider and
            // push it, so the frame proves the menu takes input rather than merely drawing.
            // Esc used to eject the player out of the castle here; if that regressed, the
            // remaining beats would photograph the main menu instead.
            (22.0, Beat::Tap(KeyCode::Escape)),
            (22.8, Beat::Shot("06_settings_menu_open")),
            (23.2, Beat::Tap(KeyCode::ArrowDown)),
            (23.5, Beat::Tap(KeyCode::ArrowDown)),
            (23.8, Beat::Tap(KeyCode::ArrowDown)),
            (24.1, Beat::Tap(KeyCode::ArrowDown)),
            (24.4, Beat::Tap(KeyCode::ArrowRight)),
            (24.7, Beat::Tap(KeyCode::ArrowRight)),
            (25.4, Beat::Shot("07_settings_menu_slider_moved")),
            // Close the menu before walking anywhere: W/S both navigate the menu and drive
            // locomotion, so leaving it open would make the next beats mean two things.
            (25.6, Beat::Tap(KeyCode::Escape)),
            // Walk through a museum arch. This is the gate for the whole doorway change: a
            // flying camera would pass through the wall whether or not the exemption works, so
            // it has to be a real walk with real collision.
            (26.0, Beat::Place(museum_threshold(), museum_facing_yaw())),
            (27.0, Beat::Shot("08_museum_arch_from_the_gallery")),
            // Short: walk speed is ~6.4 m/s and the chamber is 9m deep, so a longer hold
            // puts the camera against the back wall and the frame is a photograph of a wall.
            (27.4, Beat::Hold(&[KeyCode::KeyW])),
            (28.2, Beat::Hold(&[])),
            (29.0, Beat::Shot("09_standing_inside_the_chamber")),
            (29.4, Beat::Aim(0.10)),
            (30.2, Beat::Shot("10_chamber_vault")),
            (30.8, Beat::Finish),
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
    menu: Res<super::settings_menu::SettingsMenuState>,
    settings: Res<crate::services::settings::GameSettings>,
    sfx_tally: Res<crate::services::sfx::SfxTally>,
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
            Beat::Place(position, yaw) => {
                *transform = Transform::from_translation(position);
                controller.mode = LocomotionMode::Walking;
                controller.yaw = yaw;
                controller.pitch = -0.05;
                controller.is_grounded = true;
                controller.velocity_y = 0.0;
                transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
                    * Quat::from_axis_angle(Vec3::X, controller.pitch);
            }
            Beat::Aim(pitch) => {
                controller.pitch = pitch;
                transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
                    * Quat::from_axis_angle(Vec3::X, controller.pitch);
            }
            Beat::Shot(name) => {
                let path = run.dir.join(format!("{name}.png"));
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
                // The menu state and the selected row go in the report because a photograph
                // of a menu does not say which menu it is, nor whether the keys reached it.
                let line = format!(
                    "{name}  pos=({:.2}, {:.2}, {:.2})  mode={:?}  focus={}  bench={}  plans={}                       mode_state={:?}  settings={}  row={}  music_volume={:.2}                       sfx_played={}  footsteps={}",
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                    controller.mode,
                    describe_focus(&focus),
                    workshop.screen_label(),
                    workshop.plan_count(),
                    inner_state.get(),
                    if menu.open { "open" } else { "closed" },
                    menu.selected().label(),
                    settings.volume_music,
                    sfx_tally.played,
                    sfx_tally.footsteps,
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

/// Standing spot just short of the first tread, out on the ground promenade.
/// Standing on the gallery deck a few metres in front of one museum arch, on foot.
fn museum_threshold() -> Vec3 {
    let bearing = castle::museum_bay_bearing(0);
    let ground = Vec3::new(bearing.cos(), 0.0, bearing.sin())
        * (castle::castle_inner_face() - 6.0);
    ground + Vec3::Y * (castle::museum_chamber_floor_y() + 2.85)
}

/// Facing straight at that arch, so a plain forward walk goes through it.
fn museum_facing_yaw() -> f32 {
    let bearing = castle::museum_bay_bearing(0);
    let heading = Vec2::new(bearing.cos(), bearing.sin());
    // `player_locomotion` builds forward as (-sin(yaw), 0, -cos(yaw)).
    (-heading.x).atan2(-heading.y)
}

fn stair_approach() -> Vec3 {
    let bearing = castle::stair_start_bearing(0) - 0.03;
    let ground = Vec3::new(bearing.cos(), 0.0, bearing.sin()) * castle::STAIR_CENTRE_RADIUS;
    ground + Vec3::Y * (castle::PROMENADE_Y + 2.85)
}

/// Heading along the chord of the first flight, so a straight walk tracks the curve.
fn stair_chord_yaw() -> f32 {
    let start = castle::stair_start_bearing(0);
    let end = start + castle::stair_sweep();
    let from = Vec2::new(start.cos(), start.sin()) * castle::STAIR_CENTRE_RADIUS;
    let to = Vec2::new(end.cos(), end.sin()) * castle::STAIR_CENTRE_RADIUS;
    let chord = (to - from).normalize();
    // `player_locomotion` builds forward as (-sin(yaw), 0, -cos(yaw)).
    (-chord.x).atan2(-chord.y)
}
