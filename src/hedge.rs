use pyo3::prelude::*;
use crate::operator::FuzzyDegree;

#[pyclass]
#[derive(Clone)]
pub struct Hedge {
    power: f64,
}

#[pymethods]
impl Hedge {
    #[new]
    pub fn new(power: f64) -> Self {
        Hedge { power }
    }

    /// Allows the hedge instance to be called like a function: `my_hedge(0.8)`
    pub fn __call__(&self, mu: FuzzyDegree, py: Python) -> PyObject {
        match mu {
            FuzzyDegree::Type1(v) => (v.powf(self.power)).into_py(py),
            FuzzyDegree::IT2(t) => (t.0.powf(self.power), t.1.powf(self.power)).into_py(py),
        }
    }

    //  Standard Hedges (returning pre-configured instances) 
    #[staticmethod]
    pub fn very() -> Self {
        Hedge { power: 2.0 }
    }

    #[staticmethod]
    pub fn somewhat() -> Self {
        Hedge { power: 0.5 }
    }

    #[staticmethod]
    pub fn plus() -> Self {
        Hedge { power: 1.25 }
    }

    #[staticmethod]
    pub fn minus() -> Self {
        Hedge { power: 0.75 }
    }
}
