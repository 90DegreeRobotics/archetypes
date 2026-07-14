#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    /// Obvious subject/color/action clues.
    Literal,
    /// Concrete clues with a little composition noise.
    Metaphorical,
    /// Concrete clues that may need synonym or context credit.
    Obscured,
    /// Dramatic concrete clues, not abstract impossible ones.
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
