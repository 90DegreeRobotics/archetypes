use bevy::prelude::*;

pub mod difficulty;
pub mod game_mode;

use game_mode::{GameMode, ModeRegistration};

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
        app.init_resource::<ModeRegistry>();
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
