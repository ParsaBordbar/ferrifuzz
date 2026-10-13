//! Linguistic hedges — power modifiers such as *very* (`μ²`) and *somewhat* (`√μ`).

/// A power hedge: `μ ↦ μ^power`. `power > 1` concentrates, `< 1` dilates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hedge {
    power: f64,
}

impl Hedge {
    pub fn new(power: f64) -> Self {
        Self { power }
    }

    pub fn power(&self) -> f64 {
        self.power
    }

    /// *very* — power 2.0.
    pub fn very() -> Self {
        Self::new(2.0)
    }

    /// *extremely* — power 3.0.
    pub fn extremely() -> Self {
        Self::new(3.0)
    }

    /// *somewhat* — power 0.5.
    pub fn somewhat() -> Self {
        Self::new(0.5)
    }

    /// *plus* — power 1.25.
    pub fn plus() -> Self {
        Self::new(1.25)
    }

    /// *minus* — power 0.75.
    pub fn minus() -> Self {
        Self::new(0.75)
    }

    /// The hedge a rule word stands for (`very`, `extremely`, `somewhat`/`more_or_less`,
    /// `slightly`, `plus`, `minus`), case-insensitive; `None` for an unknown word.
    pub fn from_word(word: &str) -> Option<Self> {
        let power = match word.to_lowercase().as_str() {
            "very" => 2.0,
            "extremely" => 3.0,
            "somewhat" | "more_or_less" | "moreorless" | "slightly" => 0.5,
            "plus" => 1.25,
            "minus" => 0.75,
            _ => return None,
        };
        Some(Self::new(power))
    }

    pub fn apply(&self, mu: f64) -> f64 {
        mu.powf(self.power)
    }

    /// Bound-wise hedge of an interval (monotone, so the order is kept).
    pub fn apply_interval(&self, (lower, upper): (f64, f64)) -> (f64, f64) {
        (self.apply(lower), self.apply(upper))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets() {
        assert_eq!(Hedge::very().apply(0.5), 0.25);
        assert!((Hedge::somewhat().apply(0.25) - 0.5).abs() < 1e-12);
        assert_eq!(Hedge::from_word("VERY"), Some(Hedge::very()));
        assert_eq!(Hedge::from_word("more_or_less"), Some(Hedge::somewhat()));
        assert_eq!(Hedge::from_word("hot"), None);
        assert_eq!(Hedge::very().apply_interval((0.5, 0.8)), (0.25, 0.8 * 0.8));
    }
}
