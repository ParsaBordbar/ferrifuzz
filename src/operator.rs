use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;

#[derive(FromPyObject)]
pub enum FuzzyDegree {
    Type1(f64),
    IT2((f64, f64)),
}

fn f_max(mu1: f64, mu2: f64) -> f64 { if mu1 >= mu2 { mu1 } else { mu2 } }
fn f_min(mu1: f64, mu2: f64) -> f64 { if mu1 >= mu2 { mu2 } else { mu1 } }
fn f_product(mu1: f64, mu2: f64) -> f64 { mu1 * mu2 }
fn f_prob_or(mu1: f64, mu2: f64) -> f64 { mu1 + mu2 - (mu1 * mu2) }

#[pyclass]
#[derive(Clone)]
pub struct Operator;

#[pymethods]
impl Operator {
    #[staticmethod]
    #[pyo3(signature = (mu1, mu2, kind="min"))]
    pub fn t_norm(mu1: FuzzyDegree, mu2: FuzzyDegree, kind: &str, py: Python) -> PyResult<PyObject> {
        let op = match kind.to_lowercase().as_str() {
            "product" | "algebraic" => f_product,
            "min" | "godel" | "zadeh" | "standard" => f_min,
            _ => f_min,
        };

        match (mu1, mu2) {
            (FuzzyDegree::Type1(v1), FuzzyDegree::Type1(v2)) => Ok(op(v1, v2).into_py(py)),
            (FuzzyDegree::IT2(t1), FuzzyDegree::IT2(t2)) => Ok((op(t1.0, t2.0), op(t1.1, t2.1)).into_py(py)),
            _ => Err(PyTypeError::new_err("Cannot mix Type-1 and IT2 degrees in 't_norm'")),
        }
    }

    /// Computes the T-conorm / S-norm (Union / OR)
    #[staticmethod]
    #[pyo3(signature = (mu1, mu2, kind="max"))]
    pub fn t_conorm(mu1: FuzzyDegree, mu2: FuzzyDegree, kind: &str, py: Python) -> PyResult<PyObject> {
        let op = match kind.to_lowercase().as_str() {
            "prob" | "algebraic" | "probabilistic" => f_prob_or,
            "max" | "godel" | "zadeh" | "standard" => f_max,
            _ => f_max,
        };

        match (mu1, mu2) {
            (FuzzyDegree::Type1(v1), FuzzyDegree::Type1(v2)) => Ok(op(v1, v2).into_py(py)),
            (FuzzyDegree::IT2(t1), FuzzyDegree::IT2(t2)) => Ok((op(t1.0, t2.0), op(t1.1, t2.1)).into_py(py)),
            _ => Err(PyTypeError::new_err("Cannot mix Type-1 and IT2 degrees in 't_conorm'")),
        }
    }

    #[staticmethod]
    pub fn complement(mu: FuzzyDegree, py: Python) -> PyObject {
        match mu {
            FuzzyDegree::Type1(v) => (1.0 - v).into_py(py),
            FuzzyDegree::IT2(t) => (1.0 - t.1, 1.0 - t.0).into_py(py),
        }
    }
}
