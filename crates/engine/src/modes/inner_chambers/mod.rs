use bevy::prelude::*;

pub mod camera;
pub mod extraction;
pub mod world;

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
        next_state.set(InnerChambersState::Loading);
    }
}
