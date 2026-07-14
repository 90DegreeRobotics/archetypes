#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Standard,
    A,
    B,
    C,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Standard
    }
}
