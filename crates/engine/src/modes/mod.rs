use bevy::prelude::*;

pub mod difficulty;
pub mod game_mode;
pub mod inner_chambers;
pub mod oracle_riddle;
pub mod standard_mecha;

use game_mode::{GameMode, ModeRegistration};
use inner_chambers::InnerChambersPlugin;
use oracle_riddle::OracleRiddlePlugin;
use standard_mecha::StandardMechaPlugin;

#[derive(Resource, Debug, Clone)]
pub struct ModeRegistry {
    registrations: Vec<ModeRegistration>,
}

impl Default for ModeRegistry {
    fn default() -> Self {
        Self {
            registrations: GameMode::REGISTRY.to_vec(),
        }
    }
}

impl ModeRegistry {
    pub fn registrations(&self) -> &[ModeRegistration] {
        &self.registrations
    }
}

pub struct ModesPlugin;

impl Plugin for ModesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModeRegistry>()
            .add_plugins(OracleRiddlePlugin)
            .add_plugins(InnerChambersPlugin)
            .add_plugins(StandardMechaPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_registry_resource_matches_static_registry() {
        let registry = ModeRegistry::default();
        assert_eq!(registry.registrations(), GameMode::REGISTRY.as_slice());
    }
}
