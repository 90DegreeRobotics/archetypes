#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Standard,
    OracleRiddle,
    InnerChambers,
    LivingEngine,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeRegistration {
    pub mode: GameMode,
    pub label: &'static str,
    pub available: bool,
}

impl GameMode {
    pub const REGISTRY: [ModeRegistration; 4] = [
        ModeRegistration {
            mode: GameMode::Standard,
            label: "STANDARD MODE",
            available: true,
        },
        ModeRegistration {
            mode: GameMode::OracleRiddle,
            label: "ORACLE RIDDLE",
            available: true,
        },
        ModeRegistration {
            mode: GameMode::InnerChambers,
            label: "INNER CHAMBERS - LOCKED",
            available: false,
        },
        ModeRegistration {
            mode: GameMode::LivingEngine,
            label: "LIVING ENGINE - LOCKED",
            available: false,
        },
    ];

    pub const fn id(self) -> &'static str {
        match self {
            GameMode::Standard => "standard",
            GameMode::OracleRiddle => "oracle_riddle",
            GameMode::InnerChambers => "inner_chambers",
            GameMode::LivingEngine => "living_engine",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            GameMode::Standard => "STANDARD MODE",
            GameMode::OracleRiddle => "ORACLE RIDDLE",
            GameMode::InnerChambers => "INNER CHAMBERS",
            GameMode::LivingEngine => "LIVING ENGINE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_all_lane_contracts_without_fake_playability() {
        assert_eq!(GameMode::REGISTRY.len(), 4);
        assert!(GameMode::REGISTRY
            .iter()
            .any(|entry| entry.mode == GameMode::Standard && entry.available));
        assert!(GameMode::REGISTRY
            .iter()
            .any(|entry| entry.mode == GameMode::OracleRiddle && entry.available));
        assert!(GameMode::REGISTRY
            .iter()
            .filter(|entry| !entry.available)
            .all(|entry| matches!(
                entry.mode,
                GameMode::InnerChambers | GameMode::LivingEngine
            )));
    }

    #[test]
    fn modes_have_stable_ledger_ids() {
        assert_eq!(GameMode::Standard.id(), "standard");
        assert_eq!(GameMode::OracleRiddle.id(), "oracle_riddle");
        assert_eq!(GameMode::InnerChambers.id(), "inner_chambers");
        assert_eq!(GameMode::LivingEngine.id(), "living_engine");
    }
}
