use pyo3::prelude::*;

mod membership;
mod variable;
mod rule;
// mod system; // ⏳ kept for later

use membership::{triangle, trapezoid, gaussian, singleton};


use variable::Variable;
use rule::Rule;
// use system::FuzzySystem;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let mf_module = PyModule::new(py, "mf")?;

    mf_module.add_function(wrap_pyfunction!(triangle, mf_module)?)?;
    mf_module.add_function(wrap_pyfunction!(trapezoid, mf_module)?)?;
    mf_module.add_function(wrap_pyfunction!(gaussian, mf_module)?)?;
    mf_module.add_function(wrap_pyfunction!(singleton, mf_module)?)?;

    // 3. Attach the "mf" submodule to the main ferrifuzz module
    m.add_submodule(mf_module)?;

    m.add_class::<Type1MF>()?;
    m.add_class::<IT2MF>()?;
    m.add_class::<Variable>()?;
    m.add_class::<Rule>()?;

    Ok(())
}
