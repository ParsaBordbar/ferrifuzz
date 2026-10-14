//! PyO3 bindings — every class wraps a type from the pure-Rust `ferrifuzz` crate.

// PyO3 0.22's #[pyfunction]/#[pymethods] expansion trips this lint on every PyResult return.
#![allow(clippy::useless_conversion)]

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod membership;
mod operator;
mod rule;
mod system;
mod variable;

create_exception!(
    ferrifuzz,
    FuzzyError,
    PyValueError,
    "Base class for ferrifuzz errors (a ValueError)."
);
create_exception!(
    ferrifuzz,
    InputError,
    FuzzyError,
    "Missing, unknown or NaN input passed to compute()."
);
create_exception!(
    ferrifuzz,
    ModelError,
    FuzzyError,
    "The fuzzy system is inconsistent: a rule names an unknown variable or term, an option conflicts, ..."
);
create_exception!(
    ferrifuzz,
    RuleSyntaxError,
    FuzzyError,
    "A rule string could not be parsed."
);

pub(crate) fn to_py_err(e: ferrifuzz::Error) -> PyErr {
    use ferrifuzz::Error as E;
    let msg = e.to_string();
    match e {
        E::NotFitted => PyRuntimeError::new_err(msg),
        E::MissingInput(_) | E::UnknownInput(_) | E::InvalidInput(_) => InputError::new_err(msg),
        E::InvalidModel(_) => ModelError::new_err(msg),
        E::Parse(_) => RuleSyntaxError::new_err(msg),
        E::InvalidParameter(_) => FuzzyError::new_err(msg),
    }
}

/// Parse an option name strictly (`"centriod"` → error listing the valid names).
pub(crate) fn parse_opt<T: std::str::FromStr<Err = ferrifuzz::Error>>(s: &str) -> PyResult<T> {
    s.parse().map_err(to_py_err)
}

/// Values as a NumPy array when NumPy is importable, else a list.
pub(crate) fn to_array(py: Python<'_>, values: Vec<f64>) -> PyResult<PyObject> {
    match py.import_bound("numpy") {
        Ok(np) => Ok(np.call_method1("asarray", (values,))?.unbind()),
        Err(_) => Ok(values.into_py(py)),
    }
}

/// Apply `f` to a float, or element-wise to any sequence / NumPy array of floats.
pub(crate) fn map_values(
    py: Python<'_>,
    x: &Bound<'_, PyAny>,
    f: impl Fn(f64) -> f64,
) -> PyResult<PyObject> {
    if let Ok(v) = x.extract::<f64>() {
        return Ok(f(v).into_py(py));
    }
    let xs: Vec<f64> = x.extract().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err("expected a float or a sequence / array of floats")
    })?;
    to_array(py, xs.into_iter().map(f).collect())
}

#[pymodule]
fn _core(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("FuzzyError", py.get_type_bound::<FuzzyError>())?;
    m.add("InputError", py.get_type_bound::<InputError>())?;
    m.add("ModelError", py.get_type_bound::<ModelError>())?;
    m.add("RuleSyntaxError", py.get_type_bound::<RuleSyntaxError>())?;

    m.add_class::<rule::PyRule>()?;
    m.add_class::<rule::PyClause>()?;
    m.add_class::<variable::PyVariable>()?;
    m.add_class::<operator::PyOperator>()?;
    m.add_class::<operator::PyHedge>()?;
    m.add_class::<system::PyFuzzySystem>()?;

    m.add_class::<membership::PyTriangularMF>()?;
    m.add_class::<membership::PyTrapezoidalMF>()?;
    m.add_class::<membership::PyGaussianMF>()?;
    m.add_class::<membership::PySingletonMF>()?;
    m.add_class::<membership::PyBellMF>()?;
    m.add_class::<membership::PySigmoidMF>()?;
    m.add_class::<membership::PyType1MF>()?;
    m.add_class::<membership::PyIT2MF>()?;

    let mf = PyModule::new_bound(py, "mf")?;
    mf.add_function(wrap_pyfunction!(membership::triangle, &mf)?)?;
    mf.add_function(wrap_pyfunction!(membership::trapezoid, &mf)?)?;
    mf.add_function(wrap_pyfunction!(membership::gaussian, &mf)?)?;
    mf.add_function(wrap_pyfunction!(membership::singleton, &mf)?)?;
    mf.add_function(wrap_pyfunction!(membership::bell, &mf)?)?;
    mf.add_function(wrap_pyfunction!(membership::sigmoid, &mf)?)?;
    mf.add_function(wrap_pyfunction!(membership::it2, &mf)?)?;
    m.add_submodule(&mf)?;

    Ok(())
}
