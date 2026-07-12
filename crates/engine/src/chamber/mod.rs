use bevy::prelude::*;

pub mod camera;
pub mod merkaba;
pub mod portal;
pub mod spheres;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChamberState {
    #[default]
    IdleAtTable,
    Deliberating,
    FocusArchetype,
}

use crate::theme::Archetype;

#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentFocus(pub Option<Archetype>);

pub struct CouncilChamberPlugin;

impl Plugin for CouncilChamberPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ChamberState>()
            .init_resource::<CurrentFocus>()
            .add_plugins((
                camera::CameraPlugin,
                merkaba::MerkabaPlugin,
                spheres::SpheresPlugin,
                portal::PortalPlugin,
            ));
    }
}
