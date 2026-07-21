use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::membership::Type1MF;

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
        })
    }

    /// PYTHONIC: Allows `var["Hot"] = ff.mf.triangle(...)`
    fn __setitem__(&mut self, name: String, mf: Type1MF) {
        self.add_term(&name, mf);
    }

    /// PYTHONIC: Allows `mf = var["Hot"]`
    fn __getitem__(&self, name: &str) -> PyResult<Type1MF> {
        self.terms.get(name).cloned().ok_or_else(|| {
            pyo3::exceptions::PyKeyError::new_err(format!("Term '{}' not found", name))
        })
    }

    pub fn fuzzify<'py>(&self, py: Python<'py>, value: f64) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);

        for (name, mu) in self.evaluate(value) {
            dict.set_item(name, mu)?;
        }

        Ok(dict)
    }

    pub fn get_terms(&self) -> Vec<String> {
        self.terms.keys().cloned().collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "Variable('{}', [{}, {}], {} terms)",
            self.name,
            self.min_value,
            self.max_value,
            self.terms.len()
        )
    }

    fn __call__<'py>(&self, py: Python<'py>, value: f64) -> PyResult<Bound<'py, PyDict>> {
        self.fuzzify(py, value)
    }
}
