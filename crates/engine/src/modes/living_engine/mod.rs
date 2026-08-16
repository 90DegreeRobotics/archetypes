//! Living Engine — orbital resonance as the playable instrument.
//!
//! Prototype scope from CODEX_LANE_C: three spheres (Architect, Sentinel, Oracle),
//! one Viren strain, two implant slots. The simulation is headless-testable;
//! the Bevy layer is a visualization of that sim, not a second source of truth.

use bevy::prelude::*;

pub mod sim;
pub mod world;

pub use sim::{Harmony, ImplantKind, ImplantSlot, LivingSim};

#[derive(Resource)]
pub struct TriggerLivingEngine;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum LivingEngineState {
    #[default]
    Inactive,
    Running,
    Exiting,
}

pub struct LivingEnginePlugin;

impl Plugin for LivingEnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LivingEngineState>()
            .add_systems(
                Update,
                check_trigger.run_if(in_state(LivingEngineState::Inactive)),
            )
            .add_plugins(world::WorldPlugin);
    }
}

fn check_trigger(
    mut commands: Commands,
    trigger: Option<Res<TriggerLivingEngine>>,
    mut next_state: ResMut<NextState<LivingEngineState>>,
) {
    if trigger.is_some() {
        commands.remove_resource::<TriggerLivingEngine>();
        commands.insert_resource(crate::chamber::ActiveGameMode(
            crate::modes::game_mode::GameMode::LivingEngine,
        ));
        next_state.set(LivingEngineState::Running);
    }
}
