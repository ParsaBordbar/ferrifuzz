use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub mod membership;
pub mod rule;
pub mod variable;
pub mod operator;
pub mod hedge;
pub mod system;

#[pymodule]
fn _core(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<rule::Rule>()?;
    m.add_class::<variable::Variable>()?;
    m.add_class::<operator::Operator>()?;
    m.add_class::<hedge::Hedge>()?;
    m.add_class::<system::FuzzySystem>()?;

    m.add_class::<membership::TriangularMF>()?;
    m.add_class::<membership::TrapezoidalMF>()?;
    m.add_class::<membership::GaussianMF>()?;
    m.add_class::<membership::SingletonMF>()?;
    m.add_class::<membership::BellMF>()?;
    m.add_class::<membership::SigmoidMF>()?;
    m.add_class::<membership::Type1MF>()?;
    m.add_class::<membership::IT2MF>()?;

    let mf_submodule = PyModule::new_bound(py, "mf")?;
    mf_submodule.add_function(wrap_pyfunction!(membership::triangle,  &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::trapezoid, &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::gaussian,  &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::singleton, &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::bell,      &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::sigmoid,   &mf_submodule)?)?;
    mf_submodule.add_function(wrap_pyfunction!(membership::it2,       &mf_submodule)?)?;
    m.add_submodule(&mf_submodule)?;

    Ok(())
}
