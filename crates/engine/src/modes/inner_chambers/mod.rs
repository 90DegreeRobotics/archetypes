//! Architect Inner Chambers — a navigable Luminous Blueprint interior.
//!
//! Reading the space is reading the Architect's mind. Extracted truth can seed
//! one future Oracle Riddle round.

use bevy::prelude::*;

pub mod camera;
pub mod catalog;
pub mod extraction;
pub mod seed;
pub mod world;

pub use seed::{persist_extracted_truth, take_seeded_truth};

#[derive(Resource)]
pub struct TriggerInnerChambers;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum InnerChambersState {
    #[default]
    Inactive,
    Loading,
    Navigating,
    Exiting,
}

pub struct InnerChambersPlugin;

impl Plugin for InnerChambersPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InnerChambersState>()
            .add_systems(
                Update,
                check_trigger.run_if(in_state(InnerChambersState::Inactive)),
            )
            .add_plugins((
                world::WorldPlugin,
                camera::CameraPlugin,
                extraction::ExtractionPlugin,
            ));
    }
}

fn check_trigger(
    mut commands: Commands,
    trigger: Option<Res<TriggerInnerChambers>>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    if trigger.is_some() {
        commands.remove_resource::<TriggerInnerChambers>();
        commands.insert_resource(crate::chamber::ActiveGameMode(
            crate::modes::game_mode::GameMode::InnerChambers,
        ));
        next_state.set(InnerChambersState::Loading);
    }
}
