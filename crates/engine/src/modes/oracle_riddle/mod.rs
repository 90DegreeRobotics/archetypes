use bevy::prelude::*;

use crate::modes::difficulty::Difficulty;

pub mod scoring;
pub mod ui;

#[derive(Resource)]
pub struct TriggerOracleRiddle;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum OracleState {
    #[default]
    Inactive,
    Generating,
    Guessing,
    Scoring,
    Result,
}

#[derive(Resource, Default)]
pub struct OracleSession {
    pub difficulty: Difficulty,
    pub target_words: Vec<String>,
    pub guess_words: Vec<String>,
    pub draft: String,
    pub scores: Vec<f32>,
    pub image_path: Option<String>,
    pub error_msg: Option<String>,
    pub feedback_msg: Option<String>,
    pub outcome: Option<crate::services::chronos::ArtifactOutcome>,
}

pub struct OracleRiddlePlugin;

impl Plugin for OracleRiddlePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OracleState>()
            .add_systems(
                Update,
                check_trigger.run_if(in_state(OracleState::Inactive)),
            )
            .add_plugins((scoring::OracleScoringPlugin, ui::OracleUiPlugin));
    }
}

fn check_trigger(
    mut commands: Commands,
    trigger: Option<Res<TriggerOracleRiddle>>,
    mut next_state: ResMut<NextState<OracleState>>,
) {
    if trigger.is_some() {
        commands.remove_resource::<TriggerOracleRiddle>();
        next_state.set(OracleState::Generating);
    }
}
