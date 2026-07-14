#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    /// Interpretive distance: 0
    Literal,
    /// Interpretive distance: 1
    Metaphorical,
    /// Interpretive distance: 2
    Obscured,
    /// Interpretive distance: 3
    Abyssal,
}

impl Default for Difficulty {
    fn default() -> Self {
        Self::Literal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_ordering() {
        assert!(Difficulty::Literal < Difficulty::Metaphorical);
        assert!(Difficulty::Metaphorical < Difficulty::Obscured);
        assert!(Difficulty::Obscured < Difficulty::Abyssal);
    }
}
