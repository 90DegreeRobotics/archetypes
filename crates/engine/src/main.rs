use bevy::prelude::*;

pub mod chamber;
pub mod theme;

use chamber::CouncilChamberPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CouncilChamberPlugin)
        .run();
}
