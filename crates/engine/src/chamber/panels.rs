//! The double-faced icon/portrait panels inside each glass sphere.
//!
//! Each archetype's vessel holds a `<Archetype>_PanelSpinner` node carrying an icon
//! face and a portrait face. The panel turns **only when that archetype holds the
//! floor** in `CouncilSpeaking`, rotating clockwise so the two identity layers face
//! the camera in turn; at rest it holds still. We drive the spinner transform
//! directly (the GLB also ships baked spin actions, but direct control lets focus
//! gate the motion exactly as the direction requires).

use bevy::prelude::*;

use super::{ChamberState, CurrentFocus};
use crate::theme::Archetype;

/// Radians per second while focused — roughly one revolution every ~6 seconds.
const SPIN_SPEED: f32 = 1.05;

pub struct PanelsPlugin;

impl Plugin for PanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (bind_panel_spinners, spin_focused_panels).chain());
    }
}

#[derive(Component)]
struct PanelSpinner {
    archetype: Archetype,
}

fn bind_panel_spinners(
    mut commands: Commands,
    query: Query<(Entity, &Name), Without<PanelSpinner>>,
) {
    for (entity, name) in &query {
        if let Some(archetype) = panel_archetype(name.as_str()) {
            commands.entity(entity).insert(PanelSpinner { archetype });
        }
    }
}

fn spin_focused_panels(
    time: Res<Time>,
    state: Res<State<ChamberState>>,
    focus: Res<CurrentFocus>,
    mut query: Query<(&PanelSpinner, &mut Transform)>,
) {
    if *state.get() != ChamberState::CouncilSpeaking {
        return;
    }
    let Some(active) = focus.0 else {
        return;
    };
    for (spinner, mut transform) in &mut query {
        if spinner.archetype == active {
            // Negative local-Y turn reads as clockwise from the Witness's vantage.
            transform.rotate_local_y(-SPIN_SPEED * time.delta_secs());
        }
    }
}

fn panel_archetype(name: &str) -> Option<Archetype> {
    match name.strip_suffix("_PanelSpinner")? {
        "Architect" => Some(Archetype::Architect),
        "Sentinel" => Some(Archetype::Sentinel),
        "Jester" => Some(Archetype::Jester),
        "Mentor" => Some(Archetype::Mentor),
        "Explorer" => Some(Archetype::Explorer),
        "Oracle" => Some(Archetype::Oracle),
        "Empath" => Some(Archetype::Empath),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_spinner_names_map_to_council_archetypes() {
        assert_eq!(
            panel_archetype("Oracle_PanelSpinner"),
            Some(Archetype::Oracle)
        );
        assert_eq!(
            panel_archetype("Architect_PanelSpinner"),
            Some(Archetype::Architect)
        );
        assert_eq!(panel_archetype("Witness_PanelSpinner"), None);
        assert_eq!(panel_archetype("Architect"), None);
    }
}
