//! Upright, camera-readable archetype panels.
//!
//! The authored panel spinners are fixed at each sphere. They do not spin; this system
//! only billboards their root toward the Witness camera with world-up locked to +Y so
//! portraits and icons can never appear inverted as the camera changes ritual poses.

use bevy::prelude::*;

use super::camera::WitnessCamera;

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (keep_panels_upright, show_portrait_face));
    }
}

/// The Icon and Portrait children (`author_panel` in
/// `scripts/author_temple_overhaul.py`) are two coplanar-ish, back-to-back planes
/// with backface culling disabled — so whichever one sits physically nearer the
/// camera wins the depth test and is what's actually seen, regardless of which way
/// its face normal points. Blender's Z-up authoring axis becomes Bevy's Y-up on
/// glTF export, so the offset that separates them at runtime is local `+/-Y`
/// (confirmed empirically: Icon at `y=+0.025`, Portrait at `y=-0.025`), not Z.
/// Icon was the near (visible) one on every sphere; swap them so the Portrait —
/// the actual archetype artwork — is what the Witness sees.
fn show_portrait_face(mut panels: Query<(&Name, &mut Transform)>) {
    for (name, mut local) in &mut panels {
        if name.as_str().ends_with("_Icon_Panel") {
            local.translation.y = -0.025;
        } else if name.as_str().ends_with("_Portrait_Panel") {
            local.translation.y = 0.025;
        }
    }
}

fn keep_panels_upright(
    camera: Query<&GlobalTransform, With<WitnessCamera>>,
    mut panels: Query<(&Name, &GlobalTransform, &mut Transform), Without<WitnessCamera>>,
) {
    let Ok(camera) = camera.single() else { return };
    let camera_pos = camera.translation();
    for (name, global, mut local) in &mut panels {
        if !name.as_str().ends_with("_PanelSpinner") { continue; }
        let direction = camera_pos - global.translation();
        if direction.length_squared() > 0.001 {
            local.look_at(camera_pos, Vec3::Y);
            // The authored panel faces lie in local XY (normal +Z). Bevy's look_at
            // aligns -Z to the target; this quarter turn restores the face plane
            // while retaining +Y as the visual up direction.
            local.rotate_local_x(-std::f32::consts::FRAC_PI_2);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn panel_roots_are_selected_without_selecting_other_nodes() {
        assert!("Oracle_PanelSpinner".ends_with("_PanelSpinner"));
        assert!(!"Oracle_Portrait_Art".ends_with("_PanelSpinner"));
    }
}
