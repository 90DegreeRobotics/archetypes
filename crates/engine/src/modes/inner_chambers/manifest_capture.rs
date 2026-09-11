//! Manifestation-panel self-driving capture mode.
//!
//! Enabled by setting `ARCHETYPES_MANIFEST_CAPTURE`. A separate tool from
//! `capture::InnerCaptureRun` (which is actively driven by the current
//! Seed-of-Life world-layout work) so the two never fight over the player
//! camera or the boot-skip sequence. This one exercises the real "walk up,
//! press E, submit" manifestation flow end to end — including the live
//! Chronos2 subprocess — and screenshots the pedestal once the reference
//! floor panel and the summoned object are both real, staged assets. It is
//! never on during normal play.

use super::camera::{CameraController, LocomotionMode, PlayerCamera};
use super::manifestation::{ManifestationPhase, ManifestationState, MANIFESTATION_PEDESTAL_POS};
use super::{InnerChambersState, TriggerInnerChambers};
use crate::chamber::boot::MainMenuUi;
use crate::chamber::ChamberState;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{save_to_disk, Screenshot};
use std::fs;
use std::path::PathBuf;

#[derive(Resource)]
pub struct ManifestCaptureRun {
    dir: PathBuf,
    boot_skip_requested: bool,
    menu_skipped: bool,
    positioned: bool,
    pressed_e: bool,
    pressed_enter_at: Option<f32>,
    manifesting_seen: bool,
    settle_shot_at: Option<f32>,
    shot_taken: bool,
    exit_at: Option<f32>,
    timeout_at: f32,
}

impl ManifestCaptureRun {
    pub fn from_env() -> Option<Self> {
        if std::env::var_os("ARCHETYPES_MANIFEST_CAPTURE").is_none() {
            return None;
        }
        let dir = std::env::var_os("ARCHETYPES_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("artifacts/visual-proof/manifest-capture"));
        let _ = fs::create_dir_all(&dir);
        Some(Self {
            dir,
            boot_skip_requested: false,
            menu_skipped: false,
            positioned: false,
            pressed_e: false,
            pressed_enter_at: None,
            manifesting_seen: false,
            settle_shot_at: None,
            shot_taken: false,
            // A real manifestation is a live Chronos2/ComfyUI/TripoSR/Blender
            // pipeline run, not a mock. Give it real time rather than fail
            // a slow-but-honest run.
            exit_at: None,
            timeout_at: 240.0,
        })
    }
}

pub(crate) fn drive_manifest_capture(
    time: Res<Time>,
    mut commands: Commands,
    mut run: ResMut<ManifestCaptureRun>,
    inner_state: Res<State<InnerChambersState>>,
    chamber_state: Res<State<ChamberState>>,
    mut next_chamber_state: ResMut<NextState<ChamberState>>,
    main_menu: Query<Entity, With<MainMenuUi>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    manifest_state: Option<Res<ManifestationState>>,
    mut player: Query<(&mut Transform, &mut CameraController), With<PlayerCamera>>,
) {
    let now = time.elapsed_secs();

    if !run.boot_skip_requested && now >= 0.3 {
        next_chamber_state.set(ChamberState::MainMenu);
        run.boot_skip_requested = true;
    }

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

    let Some(manifest_state) = manifest_state else {
        return;
    };

    let Ok((mut transform, mut controller)) = player.single_mut() else {
        return;
    };

    // Stand within the real proximity radius (3.2, see handle_manifestation_input)
    // on real footing (Walking, not Flying) so this is the same input path a
    // player actually uses, not a privileged shortcut.
    if !run.positioned && now >= 3.0 {
        controller.mode = LocomotionMode::Walking;
        *transform = Transform::from_translation(Vec3::new(
            MANIFESTATION_PEDESTAL_POS.x,
            2.85,
            MANIFESTATION_PEDESTAL_POS.z + 2.0,
        ))
        .looking_at(MANIFESTATION_PEDESTAL_POS, Vec3::Y);
        run.positioned = true;
    }

    if run.positioned && !run.pressed_e && now >= 3.5 {
        keyboard.press(KeyCode::KeyE);
        run.pressed_e = true;
    }

    if run.pressed_e && run.pressed_enter_at.is_none() && manifest_state.phase == ManifestationPhase::Prompting {
        run.pressed_enter_at = Some(now + 0.5);
    }
    if let Some(t) = run.pressed_enter_at {
        if now >= t && manifest_state.phase == ManifestationPhase::Prompting {
            keyboard.press(KeyCode::Enter);
        }
    }

    if manifest_state.phase == ManifestationPhase::Manifesting {
        run.manifesting_seen = true;
    }

    // Once real Manifesting was observed and the phase later leaves it
    // (Completed or Failed), settle a moment for the reveal VFX/GPU upload,
    // then take the proof shots.
    if run.manifesting_seen
        && !run.shot_taken
        && run.settle_shot_at.is_none()
        && manifest_state.phase != ManifestationPhase::Manifesting
    {
        run.settle_shot_at = Some(now + 2.0);
    }

    if let Some(t) = run.settle_shot_at {
        if now >= t && !run.shot_taken {
            let outcome = if manifest_state.phase == ManifestationPhase::Completed {
                "completed"
            } else {
                "failed"
            };
            // Two guessed offsets both landed wrong once the room was
            // rescaled for the in-progress Seed-of-Life layout (one inside
            // nearby floor geometry, one above the ceiling looking through a
            // gap). Reuse the exact stand-and-interact pose from `positioned`
            // above instead of guessing a third fly-camera offset: that spot
            // is proven reachable and unobstructed, because it is the same
            // pose that just passed the game's own <=3.2 proximity check.
            *transform = Transform::from_translation(Vec3::new(
                MANIFESTATION_PEDESTAL_POS.x,
                2.85,
                MANIFESTATION_PEDESTAL_POS.z + 2.0,
            ))
            .looking_at(
                Vec3::new(
                    MANIFESTATION_PEDESTAL_POS.x,
                    MANIFESTATION_PEDESTAL_POS.y + 1.2,
                    MANIFESTATION_PEDESTAL_POS.z - 1.0,
                ),
                Vec3::Y,
            );
            let path = run.dir.join(format!("00_pedestal_and_reference_panel_{outcome}.png"));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            run.shot_taken = true;
            run.exit_at = Some(now + 12.0);
        }
    }

    if let Some(exit_at) = run.exit_at {
        if now >= exit_at {
            std::process::exit(0);
        }
    } else if now >= run.timeout_at {
        eprintln!("[manifest-capture] TIMEOUT waiting for manifestation to leave Manifesting phase");
        std::process::exit(1);
    }
}
