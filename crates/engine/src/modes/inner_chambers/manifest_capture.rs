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
use super::castle::GROUND_Y;
use super::manifestation::{
    ManifestationChannels, ManifestationEvent, ManifestationPhase, ManifestationState,
    MANIFESTATION_CUSHION_HEIGHT, MANIFESTATION_OBJECT_LIFT, MANIFESTATION_PEDESTAL_POS,
};
use super::{InnerChambersState, TriggerInnerChambers};
use crate::chamber::boot::MainMenuUi;
use crate::chamber::ChamberState;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{save_to_disk, Screenshot};
use std::fs;
use std::path::PathBuf;

/// Where the harness stands: the real standing eye height on the Council floor, inside the
/// altar's interaction range. `2.85` is the eye height *above the floor*, not an absolute y —
/// passing it as an absolute put the camera 0.4m too low, which mattered once the altar came
/// down off its table and the shot became a close one.
fn stand_pose() -> Vec3 {
    Vec3::new(
        MANIFESTATION_PEDESTAL_POS.x,
        GROUND_Y + 2.85,
        MANIFESTATION_PEDESTAL_POS.z + 2.2,
    )
}

/// Points the player camera at the altar **through the controller**, not by overwriting the
/// transform's rotation.
///
/// `Transform::looking_at` alone does not survive a frame here: `player_locomotion` rebuilds
/// the rotation from `controller.yaw` / `controller.pitch` every tick, so a harness that only
/// sets the transform is aiming level at whatever the controller last held. The walking
/// harness sets both, which is why its frames point where they claim to.
fn aim_at(transform: &mut Transform, controller: &mut CameraController, eye: Vec3, target: Vec3) {
    let to_target = target - eye;
    let yaw = (-to_target.x).atan2(-to_target.z);
    let pitch = to_target.y.atan2(to_target.xz().length());
    controller.yaw = yaw;
    controller.pitch = pitch;
    transform.translation = eye;
    transform.rotation =
        Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}

/// Aimed between the painting on the cushion and the object standing above it, so the proof
/// frame shows the pair rather than one of them.
fn altar_focus() -> Vec3 {
    Vec3::new(
        MANIFESTATION_PEDESTAL_POS.x,
        MANIFESTATION_PEDESTAL_POS.y + MANIFESTATION_CUSHION_HEIGHT + MANIFESTATION_OBJECT_LIFT * 0.5,
        MANIFESTATION_PEDESTAL_POS.z,
    )
}

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
    /// Skip the live Chronos2/ComfyUI subprocess and reveal the assets the **last real run**
    /// already staged, so the presentation layer can be photographed when the image backend
    /// is not up.
    ///
    /// This proves where the painting and the object are placed. It proves **nothing** about
    /// whether the pipeline works, and a frame taken this way must be labelled as staging
    /// geometry, never presented as a live manifestation.
    stage_only: bool,
    staged_at: Option<f32>,
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
            stage_only: std::env::var_os("ARCHETYPES_MANIFEST_STAGE_ONLY").is_some(),
            staged_at: None,
            // A cold run is SDXL through ComfyUI, then TripoSR, then Blender, in series. 240s
            // was not generous, it was optimistic: a slow-but-honest run was being reported as
            // a timeout. Overridable so a long run can be waited out without a rebuild.
            timeout_at: std::env::var("ARCHETYPES_MANIFEST_TIMEOUT_SECS")
                .ok()
                .and_then(|raw| raw.parse::<f32>().ok())
                .unwrap_or(240.0),
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
    channels: Res<ManifestationChannels>,
    scene: Query<(&Name, &GlobalTransform)>,
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
        aim_at(&mut transform, &mut controller, stand_pose(), altar_focus());
        run.positioned = true;
    }

    // Staging-only lane: push a Success straight into the same channel the real worker uses,
    // so the reveal runs through the exact `ManifestationEvent::Success` handler a live run
    // would take, using the GLB and PNG the last real run staged on disk. Nothing about the
    // pipeline is exercised or claimed.
    if run.stage_only {
        if run.staged_at.is_none() && now >= 3.5 {
            let _ = channels.sender.send(ManifestationEvent::Success {
                prompt: "staging geometry check (no live pipeline)".to_string(),
                detail: "reveal replayed from the assets the last real run staged".to_string(),
                // No id: this lane replays the legacy shared asset on purpose, because it is
                // photographing presentation rather than exercising the pipeline.
                artifact_id: None,
            });
            run.staged_at = Some(now);
            run.manifesting_seen = true;
        }
        if let Some(staged) = run.staged_at {
            // The reveal throws 22 expanding smoke spheres around the altar. They grow at
            // e^(0.7t) and despawn at 2.4s of their own life, so a shot taken three seconds
            // after the event still lands with the camera *inside* a 2m sphere 1.7m from the
            // eye — which is what two unreadable flat-blue frames turned out to be. Six
            // seconds clears them with margin.
            if run.settle_shot_at.is_none() && now >= staged + 6.0 {
                run.settle_shot_at = Some(now + 0.5);
            }
        }
    }

    if !run.stage_only && run.positioned && !run.pressed_e && now >= 3.5 {
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
    if !run.stage_only
        && run.manifesting_seen
        && !run.shot_taken
        && run.settle_shot_at.is_none()
        && manifest_state.phase != ManifestationPhase::Manifesting
    {
        // Same reason as above: the reveal smoke has to clear before the frame is of anything.
        run.settle_shot_at = Some(now + 6.0);
    }

    if let Some(t) = run.settle_shot_at {
        if now >= t && !run.shot_taken {
            let outcome = if run.stage_only {
                "staging_geometry_only"
            } else if manifest_state.phase == ManifestationPhase::Completed {
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
            aim_at(&mut transform, &mut controller, stand_pose(), altar_focus());
            let path = run.dir.join(format!("00_pedestal_and_reference_panel_{outcome}.png"));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));

            // Write what the frame is actually of. Two shots in a row came back as an
            // unreadable flat field and there was no way to tell from the image whether the
            // camera was in the wrong place, aimed the wrong way, or inside something.
            let eye = transform.translation;
            let focus = altar_focus();
            let report = format!(
                "outcome={outcome}
                 eye=({:.3}, {:.3}, {:.3})
                 yaw={:.3} pitch={:.3}
                 altar_focus=({:.3}, {:.3}, {:.3})
                 eye_to_focus={:.3}m
                 pedestal={:?}
                 cushion_y={:.3}
                 object_y={:.3}
                 panel_y={:.3}
                 nearby_meshes:
{}",
                eye.x, eye.y, eye.z,
                controller.yaw, controller.pitch,
                focus.x, focus.y, focus.z,
                eye.distance(focus),
                MANIFESTATION_PEDESTAL_POS,
                MANIFESTATION_PEDESTAL_POS.y + MANIFESTATION_CUSHION_HEIGHT,
                MANIFESTATION_PEDESTAL_POS.y + MANIFESTATION_CUSHION_HEIGHT
                    + MANIFESTATION_OBJECT_LIFT,
                MANIFESTATION_PEDESTAL_POS.y + MANIFESTATION_CUSHION_HEIGHT + 0.005,
                scene
                    .iter()
                    .map(|(name, global)| (name, global.compute_transform()))
                    .filter(|(_, t)| t.translation.distance(eye) < 12.0)
                    .map(|(name, t)| format!(
                        "  {name} at ({:.2},{:.2},{:.2}) scale ({:.2},{:.2},{:.2}) dist {:.2}
",
                        t.translation.x, t.translation.y, t.translation.z,
                        t.scale.x, t.scale.y, t.scale.z,
                        t.translation.distance(eye)
                    ))
                    .collect::<String>()
            );
            let _ = fs::write(run.dir.join("manifest_report.txt"), report);
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
