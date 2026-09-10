//! Inner Chambers self-driving capture mode.
//!
//! Enabled by setting `ARCHETYPES_INNER_CAPTURE`. It auto-triggers the mode from a
//! cold boot, switches the player camera to free flight so obstacle/niche collision
//! never fights a teleport, and writes one screenshot per authored vantage point via
//! Bevy's own screenshot pipeline — the rotunda overview, the table/dais, and each of
//! the five archetype niches. This is the reproducible "rendered frame" proof the
//! repo's own doctrine requires; it is never on during normal play.

use super::camera::{CameraController, LocomotionMode, PlayerCamera};
use super::world::{NICHE_CENTERS, NICHE_RING_RADIUS};
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
            name: "01_table_and_dais".to_string(),
            eye: Vec3::new(0.0, 4.5, 8.0),
            look_at: Vec3::new(0.0, 1.5, 0.0),
        }];
        let niche_names = ["sentinel", "aura", "empath", "oracle", "nebula_jester"];
        for (i, center) in NICHE_CENTERS.iter().enumerate() {
            // Stand off from the niche center along the hall-center approach line,
            // just outside its own wall ring. The five niches sit only ~5.67m apart
            // center-to-center, so this must stay well short of a neighbor's ring
            // too (checked against NICHE_RING_RADIUS below) — an earlier 6m pull-back
            // landed inside the *next* niche's wall.
            let center_xz = Vec2::new(center.x, center.z);
            let toward_hall = -center_xz.normalize();
            let stand_off = NICHE_RING_RADIUS + 1.1;
            let eye_xz = center_xz + toward_hall * stand_off;
            shots.push(CaptureShot {
                name: format!("{:02}_niche_{}", i + 2, niche_names[i]),
                eye: Vec3::new(eye_xz.x, 3.0, eye_xz.y),
                look_at: Vec3::new(center.x, 2.1, center.z),
            });
        }
        shots.push(CaptureShot {
            name: "00_rotunda_overview".to_string(),
            // Two ground-level/oblique attempts both put the 23m-wide, 5.6m-tall
            // niche row directly in frame at a scale matching its real proportion to
            // the room — correct perspective, not a bug, but it always crowded out
            // the table/dais beyond. A near-top-down plan view is more useful for
            // verification anyway: it proves the whole layout — rotunda, dais,
            // table, and all five niches — actually exists as authored, in one shot.
            eye: Vec3::new(0.0, 34.0, 7.0),
            look_at: Vec3::new(0.0, 0.0, 6.9),
        });
        shots.push(CaptureShot {
            name: "07_rotunda_drum_and_portal".to_string(),
            // A low interior long view proves the new circular shell as actual
            // rendered architecture: curved 64-bay masonry, radial roof ribs,
            // continuous cornice, and the north processional portal frame.
            eye: Vec3::new(0.0, 3.5, 22.0),
            look_at: Vec3::new(0.0, 8.0, -36.0),
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
