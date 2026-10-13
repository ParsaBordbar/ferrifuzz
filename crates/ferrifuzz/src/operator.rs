//! Fuzzy set operators — t-norms (AND), t-conorms (OR), and complements (NOT).
//!
//! Every operator works on a type-1 degree `f64` via `apply` and on an interval
//! type-2 degree `(lower, upper)` via `apply_interval`.

use crate::error::Error;

/// Fuzzy AND.
///
/// Names: `min` (`godel`, `zadeh`, `standard`), `product` (`prod`, `algebraic`),
/// `lukasiewicz` (`luk`, `bounded`), `drastic`, `einstein`, `hamacher`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TNorm {
    /// Gödel / Zadeh minimum.
    #[default]
    Min,
    /// Algebraic product `ab`.
    Product,
    /// Łukasiewicz / bounded difference `max(0, a+b-1)`.
    Lukasiewicz,
    /// Drastic product: `b` if `a==1`, `a` if `b==1`, else 0.
    Drastic,
    /// Einstein product `ab / (2 - (a + b - ab))`.
    Einstein,
    /// Hamacher product `ab / (a + b - ab)`.
    Hamacher,
}

impl TNorm {
    pub fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            TNorm::Min => a.min(b),
            TNorm::Product => a * b,
            TNorm::Lukasiewicz => (a + b - 1.0).max(0.0),
            TNorm::Drastic => {
                if a >= 1.0 {
                    b
                } else if b >= 1.0 {
                    a
                } else {
                    0.0
                }
            }
            TNorm::Einstein => {
                let d = 2.0 - (a + b - a * b);
                if d == 0.0 {
                    0.0
                } else {
                    (a * b) / d
                }
            }
            TNorm::Hamacher => {
                let d = a + b - a * b;
                if d == 0.0 {
                    0.0
                } else {
                    (a * b) / d
                }
            }
        }
    }

    /// Bound-wise AND of two membership intervals.
    pub fn apply_interval(self, a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        (self.apply(a.0, b.0), self.apply(a.1, b.1))
    }
}

named_enum!(TNorm, "t-norm", {
    Min => "min" | "godel" | "zadeh" | "standard",
    Product => "product" | "prod" | "algebraic",
    Lukasiewicz => "lukasiewicz" | "luk" | "bounded",
    Drastic => "drastic",
    Einstein => "einstein",
    Hamacher => "hamacher",
});

/// Fuzzy OR.
///
/// Names: `max` (`godel`, `zadeh`, `standard`), `probor` (`prob`, `probabilistic`, `algebraic`),
/// `lukasiewicz` (`luk`, `bounded`), `drastic`, `einstein`, `hamacher`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TConorm {
    /// Gödel / Zadeh maximum.
    #[default]
    Max,
    /// Probabilistic sum `a + b - ab`.
    Probabilistic,
    /// Łukasiewicz / bounded sum `min(1, a+b)`.
    Lukasiewicz,
    /// Drastic sum: `b` if `a==0`, `a` if `b==0`, else 1.
    Drastic,
    /// Einstein sum `(a + b) / (1 + ab)`.
    Einstein,
    /// Hamacher sum `(a + b - 2ab) / (1 - ab)`.
    Hamacher,
}

impl TConorm {
    pub fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            TConorm::Max => a.max(b),
            TConorm::Probabilistic => a + b - a * b,
            TConorm::Lukasiewicz => (a + b).min(1.0),
            TConorm::Drastic => {
                if a <= 0.0 {
                    b
                } else if b <= 0.0 {
                    a
                } else {
                    1.0
                }
            }
            TConorm::Einstein => (a + b) / (1.0 + a * b),
            TConorm::Hamacher => {
                let d = 1.0 - a * b;
                if d == 0.0 {
                    1.0
                } else {
                    (a + b - 2.0 * a * b) / d
                }
            }
        }
    }

    /// Bound-wise OR of two membership intervals.
    pub fn apply_interval(self, a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        (self.apply(a.0, b.0), self.apply(a.1, b.1))
    }
}

named_enum!(TConorm, "t-conorm", {
    Max => "max" | "godel" | "zadeh" | "standard",
    Probabilistic => "probor" | "prob" | "probabilistic" | "algebraic",
    Lukasiewicz => "lukasiewicz" | "luk" | "bounded",
    Drastic => "drastic",
    Einstein => "einstein",
    Hamacher => "hamacher",
});

/// Fuzzy NOT.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Complement {
    /// `1 - μ`.
    #[default]
    Standard,
    /// Sugeno λ-complement `(1 - μ) / (1 + λμ)`, `λ > -1`.
    Sugeno(f64),
    /// Yager w-complement `(1 - μ^w)^(1/w)`, `w > 0`.
    Yager(f64),
}

impl Complement {
    /// Parse a complement name (`standard`/`zadeh`, `sugeno`, `yager`) with its parameter
    /// (λ > -1 for Sugeno, w > 0 for Yager; ignored for standard).
    pub fn from_name(kind: &str, param: f64) -> Result<Self, Error> {
        let c = match kind.trim().to_lowercase().as_str() {
            "standard" | "zadeh" => Complement::Standard,
            "sugeno" => Complement::Sugeno(param),
            "yager" => Complement::Yager(param),
            _ => {
                return Err(crate::error::unknown_name(
                    "complement",
                    kind,
                    &["standard", "sugeno", "yager"],
                ))
            }
        };
        match c {
            Complement::Sugeno(l) if l.is_nan() || l <= -1.0 => {
                crate::error::invalid("Sugeno complement needs λ > -1")
            }
            Complement::Yager(w) if w.is_nan() || w <= 0.0 => {
                crate::error::invalid("Yager complement needs w > 0")
            }
            c => Ok(c),
        }
    }

    pub fn apply(self, mu: f64) -> f64 {
        match self {
            Complement::Standard => 1.0 - mu,
            Complement::Sugeno(lambda) => ((1.0 - mu) / (1.0 + lambda * mu)).clamp(0.0, 1.0),
            Complement::Yager(w) => (1.0 - mu.powf(w)).max(0.0).powf(1.0 / w),
        }
    }

    /// Complement of an interval; the bounds swap so `lower ≤ upper` still holds.
    pub fn apply_interval(self, (lower, upper): (f64, f64)) -> (f64, f64) {
        (self.apply(upper), self.apply(lower))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_T: [TNorm; 6] = [
        TNorm::Min,
        TNorm::Product,
        TNorm::Lukasiewicz,
        TNorm::Drastic,
        TNorm::Einstein,
        TNorm::Hamacher,
    ];
    const ALL_S: [TConorm; 6] = [
        TConorm::Max,
        TConorm::Probabilistic,
        TConorm::Lukasiewicz,
        TConorm::Drastic,
        TConorm::Einstein,
        TConorm::Hamacher,
    ];

    fn grid() -> impl Iterator<Item = f64> + Clone {
        (0..=10).map(|i| i as f64 / 10.0)
    }

    #[test]
    fn t_norm_axioms() {
        for t in ALL_T {
            for a in grid() {
                assert!((t.apply(a, 1.0) - a).abs() < 1e-12, "{:?}: identity", t);
                for b in grid() {
                    assert!(
                        (t.apply(a, b) - t.apply(b, a)).abs() < 1e-12,
                        "{:?}: commutative",
                        t
                    );
                    assert!(t.apply(a, b) <= a.min(b) + 1e-12, "{:?}: bounded by min", t);
                }
            }
        }
    }

    #[test]
    fn t_conorm_axioms() {
        for s in ALL_S {
            for a in grid() {
                assert!((s.apply(a, 0.0) - a).abs() < 1e-12, "{:?}: identity", s);
                for b in grid() {
                    assert!(
                        (s.apply(a, b) - s.apply(b, a)).abs() < 1e-12,
                        "{:?}: commutative",
                        s
                    );
                    assert!(s.apply(a, b) >= a.max(b) - 1e-12, "{:?}: bounded by max", s);
                }
            }
        }
    }

    #[test]
    fn known_values() {
        assert_eq!(TNorm::Product.apply(0.5, 0.4), 0.2);
        assert!((TNorm::Lukasiewicz.apply(0.7, 0.6) - 0.3).abs() < 1e-12);
        assert_eq!(TNorm::Drastic.apply(0.7, 0.6), 0.0);
        assert!((TConorm::Probabilistic.apply(0.5, 0.4) - 0.7).abs() < 1e-12);
        assert_eq!(TConorm::Lukasiewicz.apply(0.7, 0.6), 1.0);
    }

    #[test]
    fn complements() {
        assert!((Complement::Standard.apply(0.3) - 0.7).abs() < 1e-12);
        // λ = 0 and w = 1 both reduce to the standard complement.
        assert!((Complement::Sugeno(0.0).apply(0.3) - 0.7).abs() < 1e-12);
        assert!((Complement::Yager(1.0).apply(0.3) - 0.7).abs() < 1e-12);
        assert_eq!(Complement::Standard.apply_interval((0.2, 0.6)), (0.4, 0.8));
    }

    #[test]
    fn parse_names() {
        assert_eq!("PROD".parse::<TNorm>().unwrap(), TNorm::Product);
        assert_eq!("probor".parse::<TConorm>().unwrap(), TConorm::Probabilistic);
        let err = "prdouct".parse::<TNorm>().unwrap_err().to_string();
        assert!(err.contains("did you mean 'product'"), "{}", err);
        assert_eq!(TNorm::ALL.len(), 6);
        assert_eq!(TConorm::Probabilistic.to_string(), "probor");
        for t in TNorm::ALL {
            assert_eq!(t.name().parse::<TNorm>().unwrap(), *t);
        }
        assert!(Complement::from_name("yager", 0.0).is_err());
        assert_eq!(
            Complement::from_name("yager", 2.0).unwrap(),
            Complement::Yager(2.0)
        );
    }
}
