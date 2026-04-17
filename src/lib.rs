use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub mod membership;
pub mod rule;
pub mod variable;

// Make sure spelling matches here!
use membership::{triangle, trapezoid, gaussian, singleton};

#[pymodule]
fn _core(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {

    m.add_class::<rule::Rule>()?;
    m.add_class::<membership::TriangularMF>()?;
    m.add_class::<membership::TrapezoidalMF>()?;
    m.add_class::<membership::GaussianMF>()?;
    m.add_class::<membership::SingletonMF>()?;
    m.add_class::<membership::Type1MF>()?;
    m.add_class::<membership::IT2MF>()?;
    m.add_class::<variable::Variable>()?;


    let mf_submodule = PyModule::new_bound(py, "mf")?;
    mf_submodule.add_function(wrap_pyfunction!(membership::triangle, &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::trapezoid, &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::gaussian, &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::singleton, &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::it2, &mf_submodule)?)?;

    m.add_submodule(&mf_submodule)?;

    Ok(())
}
