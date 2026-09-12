//! Architect Inner Chambers — a navigable Luminous Blueprint interior.
//!
//! Reading the space is reading the Architect's mind. Extracted truth can seed
//! one future Oracle Riddle round.

use bevy::prelude::*;

pub mod camera;
pub mod castle;
pub mod capture;
pub mod catalog;
pub mod extraction;
pub mod encounters;
pub mod interaction;
pub mod library;
pub mod manifest_capture;
pub mod manifestation;
pub mod museum;
pub mod objects;
pub mod music;
pub mod seed;
pub mod settings_menu;
pub mod stone;
pub mod walk_capture;
pub mod workshop;
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
                encounters::EncounterPlugin,
                interaction::InteractionPlugin,
                manifestation::ManifestationPlugin,
                museum::MuseumPlugin,
                objects::ObjectsPlugin,
                library::LibraryPlugin,
                music::InnerCastleMusicPlugin,
                settings_menu::SettingsMenuPlugin,
                workshop::WorkshopPlugin,
            ));

        if let Some(run) = capture::InnerCaptureRun::from_env() {
            app.insert_resource(run)
                .add_systems(Update, capture::drive_inner_capture);
        }

        // Ordered before the action resolver for the same reason the walking harness is:
        // `just_pressed` lives for exactly one frame and Bevy clears it in `PreUpdate`, so a
        // synthesized press issued *after* `resolve_actions` has already run is wiped before
        // anything can observe it. Without this the harness pressed `E` once, lost it, and then
        // waited out its whole timeout in `Idle` while reporting that it was waiting on
        // `Manifesting` - which is how a working manifestation pipeline came to be reported as
        // broken.
        if let Some(run) = manifest_capture::ManifestCaptureRun::from_env() {
            app.insert_resource(run).add_systems(
                Update,
                manifest_capture::drive_manifest_capture
                    .before(interaction::InnerInteractionSet::Resolve),
            );
        }

        // Ordered before both the action resolver and locomotion: `just_pressed` lives for
        // exactly one frame, so a synthesized press issued after the resolver would never be
        // observed by anything.
        if let Some(run) = walk_capture::WalkCaptureRun::from_env() {
            app.insert_resource(run).add_systems(
                Update,
                walk_capture::drive_walk_capture
                    .before(interaction::InnerInteractionSet::Resolve)
                    .before(camera::player_locomotion),
            );
        }
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
