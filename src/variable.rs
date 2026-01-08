use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::membership::{MFType, Term, TriangularMF, TrapezoidalMF};

#[pyclass]
#[derive(Clone)]
pub struct Variable {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub min_value: f64,
    #[pyo3(get)]
    pub max_value: f64,
    pub terms: Vec<Term>,
}

#[pymethods]
impl Variable {
    #[new]
    pub fn new(name: String, min_value: f64, max_value: f64) -> PyResult<Self> {
        if min_value >= max_value {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "min_value must be less than max_value"
            ));
        }
        Ok(Self {
            name,
            min_value,
            max_value,
            terms: Vec::new(),
        })
    }

    pub fn add_triangular_term(&mut self, name: String, a: f64, b: f64, c: f64) -> PyResult<()> {
        let mf = TriangularMF::new(a, b, c)?;
        self.terms.push(Term {
            name,
            mf: MFType::Triangular(mf),
        });
        Ok(())
    }

    pub fn add_trapezoidal_term(&mut self, name: String, a: f64, b: f64, c: f64, d: f64) -> PyResult<()> {
        let mf = TrapezoidalMF::new(a, b, c, d)?;
        self.terms.push(Term {
            name,
            mf: MFType::Trapezoidal(mf),
        });
        Ok(())
    }

    pub fn fuzzify(&self, py: Python, value: f64) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for term in &self.terms {
            dict.set_item(&term.name, term.mf.membership(value))?;
        }
        Ok(dict.into())
    }

    pub fn get_terms(&self) -> Vec<String> {
        self.terms.iter().map(|t| t.name.clone()).collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "Variable('{}', {}, {}, {} terms)",
            self.name,
            self.min_value,
            self.max_value,
            self.terms.len()
        )
    }
}