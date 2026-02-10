use pyo3::prelude::*;

mod membership;
mod variable;
mod rule;
// mod system; // ⏳ kept for later

use membership::{
    TriangularMF,
    TrapezoidalMF,
    GaussianMF,
    Type1MF,
    IT2MF,
};

use variable::Variable;
use rule::Rule;
// use system::FuzzySystem;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TriangularMF>()?;
    m.add_class::<TrapezoidalMF>()?;
    m.add_class::<GaussianMF>()?;
    m.add_class::<Type1MF>()?;
    m.add_class::<IT2MF>()?;
    m.add_class::<Variable>()?;
    m.add_class::<Rule>()?;
    // m.add_class::<FuzzySystem>()?;
    Ok(())
}
