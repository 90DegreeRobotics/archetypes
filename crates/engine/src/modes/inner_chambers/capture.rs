//! Inner Chambers self-driving capture mode.
//!
//! Enabled by setting `ARCHETYPES_INNER_CAPTURE`. It auto-triggers the mode from a
//! cold boot, switches the player camera to free flight so obstacle/niche collision
//! never fights a teleport, and writes one screenshot per authored vantage point via
//! Bevy's own screenshot pipeline — the rotunda overview, the table/dais, and each of
//! the five archetype niches. This is the reproducible "rendered frame" proof the
//! repo's own doctrine requires; it is never on during normal play.

use super::camera::{CameraController, LocomotionMode, PlayerCamera};
use super::{InnerChambersState, TriggerInnerChambers};
use crate::chamber::boot::MainMenuUi;
use crate::chamber::ChamberState;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{save_to_disk, Screenshot};
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
struct CaptureShot {
    name: String,
    eye: Vec3,
    look_at: Vec3,
}

#[derive(Resource)]
pub struct InnerCaptureRun {
    dir: PathBuf,
    boot_skip_requested: bool,
    menu_skipped: bool,
    shots: Vec<CaptureShot>,
    next_shot: usize,
    shot_at: Option<f32>,
    exit_at: Option<f32>,
}

impl InnerCaptureRun {
    pub fn from_env() -> Option<Self> {
        if std::env::var_os("ARCHETYPES_INNER_CAPTURE").is_none() {
            return None;
        }
        let dir = std::env::var_os("ARCHETYPES_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("artifacts/visual-proof/inner-chambers-capture"));
        let _ = fs::create_dir_all(&dir);

        // Capture order deliberately is NOT gallery order: the table/niche shots go
        // first because they were already proven correct, and the wide rotunda
        // overview goes last so it gets the most real elapsed time to warm up. An
        // earlier attempt took the overview first and got an inexplicably close-in,
        // wrong-looking frame — before concluding that was a geometry bug, this
        // ordering rules out (or confirms) a first-teleport/asset-warmup artifact.
        // Filenames still reflect the intended gallery order, not capture order.
        let mut shots = vec![CaptureShot {
            name: "01_council_floor_inlay".to_string(),
            eye: Vec3::new(0.0, 4.4, 8.5),
            look_at: Vec3::new(0.0, 1.7, -1.3),
        }];
        let rooms = [
            ("architect", -std::f32::consts::FRAC_PI_2),
            ("sentinel", -std::f32::consts::FRAC_PI_6),
            ("explorer", std::f32::consts::FRAC_PI_6),
            ("empath", std::f32::consts::FRAC_PI_2),
            ("mentor", 5.0 * std::f32::consts::FRAC_PI_6),
            ("oracle", 7.0 * std::f32::consts::FRAC_PI_6),
        ];
        for (i, (name, angle)) in rooms.iter().enumerate() {
            let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
            let center = radial * 62.0;
            shots.push(CaptureShot {
                name: format!("{:02}_room_{name}", i + 2),
                eye: center - radial * 14.6 + Vec3::Y * 5.0,
                look_at: center + radial * 5.0 + Vec3::Y * 2.6,
            });
        }
        shots.push(CaptureShot {
            name: "00_seed_of_life_layout".to_string(),
            eye: Vec3::new(0.0, 215.0, 0.1),
            look_at: Vec3::ZERO,
        });

        Some(Self {
            dir,
            boot_skip_requested: false,
            menu_skipped: false,
            shots,
            next_shot: 0,
            shot_at: None,
            exit_at: None,
        })
    }
}

pub(crate) fn drive_inner_capture(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<InnerCaptureRun>,
    inner_state: Res<State<InnerChambersState>>,
    chamber_state: Res<State<ChamberState>>,
    mut next_chamber_state: ResMut<NextState<ChamberState>>,
    main_menu: Query<Entity, With<MainMenuUi>>,
    mut player: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let now = time.elapsed_secs();

    // Stage 1: skip the ~11s timed boot veil instead of waiting it out — a
    // capture-only shortcut, never taken during normal play.
    if !run.boot_skip_requested && now >= 0.3 {
        next_chamber_state.set(ChamberState::MainMenu);
        run.boot_skip_requested = true;
    }

    // Stage 2: once the main menu is actually up, skip it exactly the way the real
    // "Inner Chambers" button does — despawn the menu, then trigger the mode.
    if run.boot_skip_requested
        && !run.menu_skipped
        && *chamber_state.get() == ChamberState::MainMenu
    {
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

    // Free flight sidesteps every ground/obstacle/niche collision check so a
    // teleport to an authored vantage point is never fought back the next frame.
    if controller.mode != LocomotionMode::Flying {
        controller.mode = LocomotionMode::Flying;
    }

    if run.shot_at.is_none() && run.next_shot < run.shots.len() {
        // The very first shot needs real settle time: the scene just spawned
        // ~200 procedural meshes/materials this frame, and their GPU upload lags
        // a couple of frames behind `meshes.add()`/`images.add()`. A screenshot
        // taken too early captures nothing but the clear color. Later shots reuse
        // an already-warm scene, so they only need a short teleport settle.
        let settle = if run.next_shot == 0 { 4.0 } else { 0.8 };
        run.shot_at = Some(now + settle);
    }

    if let Some(t) = run.shot_at {
        if now >= t && run.next_shot < run.shots.len() {
            let shot = run.shots[run.next_shot].clone();
            *transform = Transform::from_translation(shot.eye).looking_at(shot.look_at, Vec3::Y);
            let path = run.dir.join(format!("{}.png", shot.name));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            run.next_shot += 1;
            if run.next_shot < run.shots.len() {
                run.shot_at = Some(now + 0.8);
            } else {
                // A full-resolution detailed frame's PNG encode + disk write has
                // been observed taking several real seconds in the background;
                // exiting too soon truncates the final screenshot mid-write.
                run.exit_at = Some(now + 12.0);
            }
        }
    }

    if let Some(exit_at) = run.exit_at {
        if now >= exit_at {
            std::process::exit(0);
        }
    }
}
