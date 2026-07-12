use super::{ChamberState, CurrentFocus};
use crate::theme::Archetype;
use bevy::prelude::*;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_portal_input);
    }
}

fn handle_portal_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<ChamberState>>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut current_focus: ResMut<CurrentFocus>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match state.get() {
            ChamberState::IdleAtTable => {
                println!("Prompt submitted. Witness invokes the council.");
                next_state.set(ChamberState::Deliberating);
            }
            ChamberState::Deliberating => {
                current_focus.0 = Some(Archetype::Architect);
                println!("The Architect takes the focus lane.");
                next_state.set(ChamberState::FocusArchetype);
            }
            ChamberState::FocusArchetype => {
                current_focus.0 = None;
                println!("Returning to table.");
                next_state.set(ChamberState::IdleAtTable);
            }
        }
    }
}
