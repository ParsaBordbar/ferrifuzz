//! Linguistic variables — a named universe `[min, max]` holding named fuzzy terms.

use std::collections::HashMap;
use std::fmt;

use crate::error::{invalid, Result};
use crate::membership::{mf, Shape, Type1MF, IT2MF};

/// A term's membership function: type-1 or interval type-2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Term {
    Type1(Type1MF),
    IT2(IT2MF),
}

impl Term {
    /// Membership interval at `x`; a type-1 term gives the degenerate `(μ, μ)`.
    pub fn interval(&self, x: f64) -> (f64, f64) {
        match self {
            Term::Type1(mf) => {
                let mu = mf.membership(x);
                (mu, mu)
            }
            Term::IT2(mf) => mf.membership(x),
        }
    }

    /// Analytic center (IT2: midpoint of the upper and lower centers).
    pub fn center(&self) -> f64 {
        match self {
            Term::Type1(mf) => mf.center(),
            Term::IT2(mf) => mf.center(),
        }
    }

    pub fn as_type1(&self) -> Option<&Type1MF> {
        match self {
            Term::Type1(mf) => Some(mf),
            Term::IT2(_) => None,
        }
    }
}

impl From<Type1MF> for Term {
    fn from(mf: Type1MF) -> Self {
        Term::Type1(mf)
    }
}

impl From<IT2MF> for Term {
    fn from(mf: IT2MF) -> Self {
        Term::IT2(mf)
    }
}

/// A linguistic variable over `[min_value, max_value]`; terms keep insertion order.
#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    name: String,
    min_value: f64,
    max_value: f64,
    terms: Vec<(String, Term)>,
}

impl Variable {
    pub fn new(name: impl Into<String>, min_value: f64, max_value: f64) -> Result<Self> {
        if min_value.is_nan() || max_value.is_nan() || min_value >= max_value {
            return invalid("min_value must be less than max_value");
        }
        Ok(Self {
            name: name.into(),
            min_value,
            max_value,
            terms: Vec::new(),
        })
    }

    /// Add or replace a term.
    pub fn add_term(&mut self, name: impl Into<String>, mf: impl Into<Term>) -> &mut Self {
        let (name, mf) = (name.into(), mf.into());
        match self.terms.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = mf,
            None => self.terms.push((name, mf)),
        }
        self
    }

    /// Builder form of [`Variable::add_term`].
    pub fn with_term(mut self, name: impl Into<String>, mf: impl Into<Term>) -> Self {
        self.add_term(name, mf);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn min_value(&self) -> f64 {
        self.min_value
    }

    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// Position of a term in insertion order.
    pub fn term_index(&self, name: &str) -> Option<usize> {
        self.terms.iter().position(|(n, _)| n == name)
    }

    /// Term at a position returned by [`Variable::term_index`].
    pub fn term_at(&self, index: usize) -> &Term {
        &self.terms[index].1
    }

    pub fn term(&self, name: &str) -> Option<&Term> {
        self.terms.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    pub fn terms(&self) -> impl Iterator<Item = (&str, &Term)> {
        self.terms.iter().map(|(n, t)| (n.as_str(), t))
    }

    pub fn term_names(&self) -> Vec<&str> {
        self.terms.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// True if any term is interval type-2.
    pub fn is_it2(&self) -> bool {
        self.terms.iter().any(|(_, t)| matches!(t, Term::IT2(_)))
    }

    /// Membership of `x` in every type-1 term.
    pub fn fuzzify(&self, x: f64) -> HashMap<String, f64> {
        self.terms
            .iter()
            .filter_map(|(n, t)| t.as_type1().map(|mf| (n.clone(), mf.membership(x))))
            .collect()
    }

    /// Membership interval of `x` in every term (type-1 terms are degenerate).
    pub fn fuzzify_interval(&self, x: f64) -> HashMap<String, (f64, f64)> {
        self.terms
            .iter()
            .map(|(n, t)| (n.clone(), t.interval(x)))
            .collect()
    }

    /// Type-1 degree of `x` in `term`; `None` for a missing or IT2 term.
    pub fn membership(&self, term: &str, x: f64) -> Option<f64> {
        self.term(term)?.as_type1().map(|mf| mf.membership(x))
    }

    /// Interval degree of `x` in `term`, IT2 or (degenerate) type-1.
    pub fn membership_interval(&self, term: &str, x: f64) -> Option<(f64, f64)> {
        self.term(term).map(|t| t.interval(x))
    }

    /// Clamp `x` into the universe.
    pub fn clamp(&self, x: f64) -> f64 {
        x.clamp(self.min_value, self.max_value)
    }

    /// Replace all terms with `names.len() ≥ 2` evenly spaced, overlapping terms over the
    /// universe (first and last peak at `min_value` / `max_value`).
    ///
    /// See [`default_term_names`] for conventional name sets.
    pub fn automf<S: AsRef<str>>(&mut self, names: &[S], shape: Shape) -> Result<&mut Self> {
        let terms = mf::evenly_spaced(self.min_value, self.max_value, names.len(), shape)?;
        self.terms = names
            .iter()
            .zip(terms)
            .map(|(n, (_, mf))| (n.as_ref().to_string(), Term::Type1(mf)))
            .collect();
        Ok(self)
    }
}

/// Conventional names for `n` evenly spaced terms: `low/medium/high` (3),
/// `very_low..very_high` (5), `NB NM NS ZE PS PM PB` (7, control convention),
/// otherwise `A1..An`.
pub fn default_term_names(n: usize) -> Vec<String> {
    let named: &[&str] = match n {
        2 => &["low", "high"],
        3 => &["low", "medium", "high"],
        5 => &["very_low", "low", "medium", "high", "very_high"],
        7 => &["NB", "NM", "NS", "ZE", "PS", "PM", "PB"],
        _ => &[],
    };
    if named.is_empty() {
        (1..=n).map(|i| format!("A{}", i)).collect()
    } else {
        named.iter().map(|s| s.to_string()).collect()
    }
}

/// `temp [0, 40]: cold = triangle(0, 0, 20), hot = it2(...)`.
impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}, {}]", self.name, self.min_value, self.max_value)?;
        for (i, (name, term)) in self.terms.iter().enumerate() {
            f.write_str(if i == 0 { ": " } else { ", " })?;
            match term {
                Term::Type1(mf) => write!(f, "{} = {}", name, mf)?,
                Term::IT2(mf) => write!(f, "{} = it2({}, {})", name, mf.upper(), mf.lower())?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mf;

    fn temperature() -> Variable {
        Variable::new("temp", 0.0, 40.0)
            .unwrap()
            .with_term("cold", mf::triangle(0.0, 0.0, 20.0).unwrap())
            .with_term("warm", mf::triangle(10.0, 20.0, 30.0).unwrap())
            .with_term("hot", mf::triangle(20.0, 40.0, 40.0).unwrap())
    }

    #[test]
    fn fuzzify_type1() {
        let v = temperature();
        let mu = v.fuzzify(25.0);
        assert_eq!(mu["cold"], 0.0);
        assert_eq!(mu["warm"], 0.5);
        assert_eq!(mu["hot"], 0.25);
        assert_eq!(v.term_names(), ["cold", "warm", "hot"]);
    }

    #[test]
    fn add_term_replaces_same_name() {
        let mut v = temperature();
        let it2 = mf::it2(
            mf::triangle(0.0, 5.0, 10.0).unwrap(),
            mf::triangle(2.0, 5.0, 8.0).unwrap(),
        )
        .unwrap();
        v.add_term("cold", it2);
        assert_eq!(v.term_names().len(), 3);
        assert!(v.is_it2());
        assert!(v.membership("cold", 5.0).is_none());
        assert_eq!(v.membership_interval("cold", 5.0), Some((1.0, 1.0)));
        assert_eq!(v.membership_interval("warm", 25.0), Some((0.5, 0.5)));
    }

    #[test]
    fn automf_and_display() {
        let mut v = Variable::new("e", -1.0, 1.0).unwrap();
        v.automf(&default_term_names(3), Shape::Triangle).unwrap();
        assert_eq!(v.term_names(), ["low", "medium", "high"]);
        assert_eq!(v.fuzzify(-1.0)["low"], 1.0);
        assert_eq!(v.fuzzify(0.5)["high"], 0.5);
        assert_eq!(
            v.to_string(),
            "e [-1, 1]: low = triangle(-2, -1, 0), medium = triangle(-1, 0, 1), high = triangle(0, 1, 2)"
        );
        assert_eq!(default_term_names(4), ["A1", "A2", "A3", "A4"]);
        assert!(v.automf(&["only"], Shape::Triangle).is_err());
    }

    #[test]
    fn rejects_empty_universe() {
        assert!(Variable::new("x", 1.0, 1.0).is_err());
        assert!(Variable::new("x", 2.0, 1.0).is_err());
    }
}
