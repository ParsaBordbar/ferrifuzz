use pyo3::prelude::*;
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::membership::{IT2MF, Type1MF};

#[pyclass(module = "ferrifuzz")]
#[derive(Clone)]
pub struct Variable {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub min_value: f64,
    #[pyo3(get)]
    pub max_value: f64,
    pub terms: HashMap<String, Type1MF>,
    pub it2_terms: HashMap<String, IT2MF>,
}

impl Variable {
    pub fn add_term(&mut self, name: &str, mf: Type1MF) {
        self.terms.insert(name.to_string(), mf);
    }

    pub fn evaluate(&self, value: f64) -> HashMap<String, f64> {
        self.terms
            .iter()
            .map(|(name, mf)| (name.clone(), mf.membership(value)))
            .collect()
    }

    /// True if this variable carries any interval type-2 term.
    pub fn is_it2(&self) -> bool {
        !self.it2_terms.is_empty()
    }

    /// Fuzzify to lower/upper membership intervals for the IT2 inference path.
    /// Type-1 terms collapse to a degenerate `(μ, μ)` interval so IT2 systems
    /// can mix crisp and interval variables.
    pub fn evaluate_it2(&self, value: f64) -> HashMap<String, (f64, f64)> {
        let mut out = HashMap::with_capacity(self.terms.len() + self.it2_terms.len());
        for (name, mf) in &self.terms {
            let mu = mf.membership(value);
            out.insert(name.clone(), (mu, mu));
        }
        for (name, mf) in &self.it2_terms {
            out.insert(name.clone(), mf.membership(value));
        }
        out
    }

    /// Lower/upper membership of a single term at `x`, IT2 or (degenerate) type-1.
    pub fn it2_term_membership(&self, term: &str, x: f64) -> Option<(f64, f64)> {
        if let Some(mf) = self.it2_terms.get(term) {
            return Some(mf.membership(x));
        }
        if let Some(mf) = self.terms.get(term) {
            let mu = mf.membership(x);
            return Some((mu, mu));
        }
        None
    }
}

#[pymethods]
impl Variable {
    #[new]
    pub fn new(name: String, min_value: f64, max_value: f64) -> PyResult<Self> {
        if min_value >= max_value {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "min_value must be less than max_value",
            ));
        }

        Ok(Self {
            name,
            min_value,
            max_value,
            terms: HashMap::new(),
            it2_terms: HashMap::new(),
        })
    }

    /// PYTHONIC: Allows `var["Hot"] = ff.mf.triangle(...)` for a Type-1 term or
    /// `var["Hot"] = ff.mf.it2(upper, lower)` for an interval type-2 term.
    fn __setitem__(&mut self, name: String, mf: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(t1) = mf.extract::<Type1MF>() {
            self.add_term(&name, t1);
            return Ok(());
        }
        if let Ok(it2) = mf.extract::<IT2MF>() {
            self.it2_terms.insert(name, it2);
            return Ok(());
        }
        Err(PyTypeError::new_err(
            "expected a Type1MF (ff.mf.triangle/gaussian/...) or IT2MF (ff.mf.it2)",
        ))
    }

    /// PYTHONIC: Allows `mf = var["Hot"]` — returns a Type1MF or IT2MF.
    fn __getitem__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        if let Some(mf) = self.terms.get(name) {
            return Ok(mf.clone().into_py(py));
        }
        if let Some(mf) = self.it2_terms.get(name) {
            return Ok(mf.clone().into_py(py));
        }
        Err(PyKeyError::new_err(format!("Term '{}' not found", name)))
    }

    pub fn fuzzify<'py>(&self, py: Python<'py>, value: f64) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);

        for (name, mu) in self.evaluate(value) {
            dict.set_item(name, mu)?;
        }

        Ok(dict)
    }

    pub fn get_terms(&self) -> Vec<String> {
        self.terms
            .keys()
            .chain(self.it2_terms.keys())
            .cloned()
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "Variable('{}', [{}, {}], {} terms)",
            self.name,
            self.min_value,
            self.max_value,
            self.terms.len() + self.it2_terms.len()
        )
    }

    fn __call__<'py>(&self, py: Python<'py>, value: f64) -> PyResult<Bound<'py, PyDict>> {
        self.fuzzify(py, value)
    }
}
