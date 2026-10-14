use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use ferrifuzz::{Complement, Hedge, TConorm, TNorm};

use crate::{parse_opt, to_py_err};

/// A type-1 degree `μ` or an interval type-2 degree `(lower, upper)`.
#[derive(FromPyObject)]
pub enum FuzzyDegree {
    Type1(f64),
    IT2((f64, f64)),
}

fn binary(
    a: FuzzyDegree,
    b: FuzzyDegree,
    op: impl Fn(f64, f64) -> f64,
    name: &str,
    py: Python<'_>,
) -> PyResult<PyObject> {
    match (a, b) {
        (FuzzyDegree::Type1(a), FuzzyDegree::Type1(b)) => Ok(op(a, b).into_py(py)),
        (FuzzyDegree::IT2(a), FuzzyDegree::IT2(b)) => Ok((op(a.0, b.0), op(a.1, b.1)).into_py(py)),
        _ => Err(PyTypeError::new_err(format!(
            "Cannot mix Type-1 and IT2 degrees in '{}'",
            name
        ))),
    }
}

/// Static fuzzy set operators — `t_norm` (AND), `t_conorm` (OR), `complement` (NOT).
///
/// Unknown `kind` names raise `ff.FuzzyError` (a `ValueError`) listing the valid names.
#[pyclass(name = "Operator", module = "ferrifuzz")]
pub struct PyOperator;

#[pymethods]
impl PyOperator {
    /// Fuzzy AND — `kind` ∈ min, product, lukasiewicz/bounded, drastic, einstein, hamacher.
    #[staticmethod]
    #[pyo3(signature = (mu1, mu2, kind="min"))]
    fn t_norm(
        mu1: FuzzyDegree,
        mu2: FuzzyDegree,
        kind: &str,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        let t: TNorm = parse_opt(kind)?;
        binary(mu1, mu2, |a, b| t.apply(a, b), "t_norm", py)
    }

    /// Fuzzy OR — `kind` ∈ max, prob, lukasiewicz/bounded, drastic, einstein, hamacher.
    #[staticmethod]
    #[pyo3(signature = (mu1, mu2, kind="max"))]
    fn t_conorm(
        mu1: FuzzyDegree,
        mu2: FuzzyDegree,
        kind: &str,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        let s: TConorm = parse_opt(kind)?;
        binary(mu1, mu2, |a, b| s.apply(a, b), "t_conorm", py)
    }

    /// Fuzzy NOT — `kind` ∈ standard (1-μ), sugeno (λ > -1), yager (w > 0); IT2 stays ordered.
    #[staticmethod]
    #[pyo3(signature = (mu, kind="standard", param=1.0))]
    fn complement(mu: FuzzyDegree, kind: &str, param: f64, py: Python<'_>) -> PyResult<PyObject> {
        let c = Complement::from_name(kind, param).map_err(to_py_err)?;
        Ok(match mu {
            FuzzyDegree::Type1(v) => c.apply(v).into_py(py),
            FuzzyDegree::IT2(iv) => c.apply_interval(iv).into_py(py),
        })
    }
}

/// Linguistic hedge — callable `hedge(mu)` returning `μ^power`; > 1 concentrates, < 1 dilates.
#[pyclass(name = "Hedge", module = "ferrifuzz")]
#[derive(Clone)]
pub struct PyHedge(Hedge);

#[pymethods]
impl PyHedge {
    #[new]
    fn new(power: f64) -> Self {
        Self(Hedge::new(power))
    }

    #[getter]
    fn power(&self) -> f64 {
        self.0.power()
    }

    /// Apply the hedge — `hedge(mu)` returns `mu^power` (works on IT2 too).
    fn __call__(&self, mu: FuzzyDegree, py: Python<'_>) -> PyObject {
        match mu {
            FuzzyDegree::Type1(v) => self.0.apply(v).into_py(py),
            FuzzyDegree::IT2(iv) => self.0.apply_interval(iv).into_py(py),
        }
    }

    /// Predefined *very* hedge — power = 2.0 (concentrates).
    #[staticmethod]
    fn very() -> Self {
        Self(Hedge::very())
    }

    /// Predefined *somewhat* hedge — power = 0.5 (dilates).
    #[staticmethod]
    fn somewhat() -> Self {
        Self(Hedge::somewhat())
    }

    /// Predefined *plus* hedge — power = 1.25 (mild concentrate).
    #[staticmethod]
    fn plus() -> Self {
        Self(Hedge::plus())
    }

    /// Predefined *extremely* hedge — power = 3.0 (strong concentrate).
    #[staticmethod]
    fn extremely() -> Self {
        Self(Hedge::extremely())
    }

    /// Predefined *slightly* hedge — power = 0.5 (same as `somewhat` in rule strings).
    #[staticmethod]
    fn slightly() -> Self {
        Self(Hedge::somewhat())
    }

    /// Predefined *minus* hedge — power = 0.75 (mild dilate).
    #[staticmethod]
    fn minus() -> Self {
        Self(Hedge::minus())
    }

    fn __repr__(&self) -> String {
        format!("Hedge(power={})", self.0.power())
    }
}
