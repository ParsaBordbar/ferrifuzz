//! # ferrifuzz
//!
//! Fuzzy logic in pure Rust: membership functions, linguistic variables and hedges,
//! fuzzy operators, Mamdani / Sugeno (TSK) / interval type-2 inference, and fuzzy
//! time series forecasting.
//!
//! ```
//! use ferrifuzz::prelude::*;
//!
//! let service = Variable::new("service", 0.0, 10.0)?
//!     .with_term("poor", mf::trapezoid(0.0, 0.0, 2.0, 5.0)?)
//!     .with_term("good", mf::triangle(2.0, 5.0, 8.0)?)
//!     .with_term("excellent", mf::trapezoid(5.0, 8.0, 10.0, 10.0)?);
//! let tip = Variable::new("tip", 0.0, 30.0)?
//!     .with_term("low", mf::triangle(0.0, 5.0, 10.0)?)
//!     .with_term("medium", mf::triangle(10.0, 15.0, 20.0)?)
//!     .with_term("high", mf::triangle(20.0, 25.0, 30.0)?);
//!
//! let mut tipper = FuzzySystem::new();
//! tipper.add_input(service).add_output(tip);
//! tipper.add_rules_str([
//!     "IF service IS poor THEN tip IS low",
//!     "IF service IS good THEN tip IS medium",
//!     "IF service IS excellent THEN tip IS high",
//! ])?;
//!
//! let out = tipper.compute([("service", 9.0)])?;
//! assert!(out["tip"] > 20.0);
//! # Ok::<(), ferrifuzz::Error>(())
//! ```
//!
//! The same engine powers the `ferrifuzz` Python package.

#[macro_use]
mod names;

pub mod error;
pub mod fts;
pub mod hedge;
pub mod it2;
pub mod membership;
pub mod metrics;
pub mod operator;
pub mod rule;
pub mod system;
pub mod variable;

pub use error::{Error, Result};
pub use hedge::Hedge;
pub use membership::{
    mf, BellMF, GaussianMF, MembershipFunction, Shape, SigmoidMF, SingletonMF, TrapezoidalMF,
    TriangularMF, Type1MF, IT2MF,
};
pub use operator::{Complement, TConorm, TNorm};
pub use rule::{Clause, Rule};
pub use system::{Defuzz, FuzzySystem, Implication, InferenceType, TypeReduction};
pub use variable::{default_term_names, Term, Variable};

/// Everything needed to build and run a fuzzy system: `use ferrifuzz::prelude::*;`.
pub mod prelude {
    pub use crate::fts::{FuzzyTimeSeries, PartitionShape};
    pub use crate::metrics::{self, Scores};
    pub use crate::{
        default_term_names, mf, Clause, Complement, Defuzz, Error, FuzzySystem, Hedge, Implication,
        InferenceType, MembershipFunction, Rule, Shape, TConorm, TNorm, Term, Type1MF,
        TypeReduction, Variable, IT2MF,
    };
}
