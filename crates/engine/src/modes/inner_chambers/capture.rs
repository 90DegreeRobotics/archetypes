//! Inner Chambers self-driving capture mode.
//!
//! Enabled by setting `ARCHETYPES_INNER_CAPTURE`. It auto-triggers the mode from a
//! cold boot, switches the player camera to free flight so obstacle/niche collision
//! never fights a teleport, and writes one screenshot per authored vantage point via
//! Bevy's own screenshot pipeline — the rotunda overview, the table/dais, and each of
//! the five archetype niches. This is the reproducible "rendered frame" proof the
//! repo's own doctrine requires; it is never on during normal play.

use super::camera::{CameraController, LocomotionMode, PlayerCamera};
use super::castle;
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
        let mut shots = vec![
            CaptureShot {
                name: "01_council_floor_inlay".to_string(),
                eye: Vec3::new(0.0, 4.4, 8.5),
                look_at: Vec3::new(0.0, 1.7, -1.3),
            },
            CaptureShot {
                name: "02_jester_council_host".to_string(),
                eye: Vec3::new(7.5, 2.8, 9.2),
                look_at: Vec3::new(10.2, 1.8, 6.2),
            },
            CaptureShot {
                name: "03_manifestation_altar".to_string(),
                eye: Vec3::new(0.0, 2.6, 7.8),
                look_at: Vec3::new(0.0, 1.6, 3.4),
            },
            CaptureShot {
                name: "04_council_circle_wide".to_string(),
                eye: Vec3::new(0.0, 6.5, 22.0),
                look_at: Vec3::new(0.0, 1.5, 0.0),
            },
            CaptureShot {
                name: "05_rotunda_looking_outward".to_string(),
                eye: Vec3::new(0.0, 3.25, 0.0),
                look_at: Vec3::new(0.0, 14.0, -100.0),
            },
            CaptureShot {
                name: "06_jester_in_great_hall".to_string(),
                eye: Vec3::new(14.5, 3.5, 2.5),
                look_at: Vec3::new(10.2, 1.8, 6.2),
            },
            CaptureShot {
                name: "07_abyss_threshold".to_string(),
                eye: Vec3::new(0.0, 2.5, -20.0),
                look_at: Vec3::new(0.0, -10.0, -45.0),
            },
        ];

        // Shots that exist to show the scale of the shell itself.
        let stair_foot_bearing = castle::stair_start_bearing(0) + 0.08;
        let stair_foot = Vec3::new(stair_foot_bearing.cos(), 0.0, stair_foot_bearing.sin())
            * castle::STAIR_CENTRE_RADIUS;
        shots.push(CaptureShot {
            name: "08_ascent_from_the_floor".to_string(),
            eye: stair_foot * 0.62 + Vec3::Y * 6.0,
            look_at: stair_foot + Vec3::Y * 40.0,
        });
        let top_bearing = castle::stair_start_bearing(castle::GALLERY_LEVELS - 1) + 0.30;
        let top = Vec3::new(top_bearing.cos(), 0.0, top_bearing.sin()) * castle::GALLERY_INNER_RADIUS;
        shots.push(CaptureShot {
            name: "09_top_gallery_looking_down".to_string(),
            eye: top + Vec3::Y * (castle::gallery_y(castle::GALLERY_LEVELS - 1) + 3.0),
            look_at: Vec3::new(0.0, 2.0, 0.0),
        });
        shots.push(CaptureShot {
            name: "10_hall_from_the_council_floor".to_string(),
            eye: Vec3::new(0.0, 4.0, 6.0),
            look_at: Vec3::new(0.0, 96.0, -110.0),
        });
        shots.push(CaptureShot {
            name: "00_seed_of_life_layout".to_string(),
            eye: Vec3::new(0.0, 360.0, 0.1),
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
