use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use ferrifuzz::{default_term_names, Shape, Term, Variable};

use crate::membership::{PyIT2MF, PyType1MF, Type1Arg};
use crate::{parse_opt, to_py_err};

/// Linguistic variable over `[min_value, max_value]`; add terms with `var["Hot"] = mf`.
///
/// `FuzzySystem.add_input` / `add_output` store a copy: change terms before adding the
/// variable, or add it again after changing it.
#[pyclass(name = "Variable", module = "ferrifuzz")]
#[derive(Clone)]
pub struct PyVariable(pub Variable);

/// Names given as an int (`3` → low/medium/high) or an explicit list.
#[derive(FromPyObject)]
pub enum TermNames {
    Count(usize),
    Names(Vec<String>),
}

#[pymethods]
impl PyVariable {
    #[new]
    fn new(name: String, min_value: f64, max_value: f64) -> PyResult<Self> {
        Variable::new(name, min_value, max_value)
            .map(Self)
            .map_err(to_py_err)
    }

    /// Variable name, as used in rule strings and `compute` dicts.
    #[getter]
    fn name(&self) -> &str {
        self.0.name()
    }

    /// Lower bound of the universe.
    #[getter]
    fn min_value(&self) -> f64 {
        self.0.min_value()
    }

    /// Upper bound of the universe.
    #[getter]
    fn max_value(&self) -> f64 {
        self.0.max_value()
    }

    /// Set a term — `var["Hot"] = ff.mf.triangle(...)` or `ff.mf.it2(upper, lower)`.
    fn __setitem__(&mut self, name: String, mf: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(it2) = mf.extract::<PyIT2MF>() {
            self.0.add_term(name, it2.0);
        } else if let Ok(t1) = mf.extract::<Type1Arg>() {
            self.0.add_term(name, t1.0);
        } else {
            return Err(PyTypeError::new_err(format!(
                "var[{:?}] = ... expects a membership function (ff.mf.triangle/gaussian/... or ff.mf.it2), got {}",
                name,
                mf.get_type().name()?
            )));
        }
        Ok(())
    }

    /// Get a term — `mf = var["Hot"]` returns a Type1MF or IT2MF.
    fn __getitem__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        match self.0.term(name) {
            Some(Term::Type1(mf)) => Ok(PyType1MF(*mf).into_py(py)),
            Some(Term::IT2(mf)) => Ok(PyIT2MF(*mf).into_py(py)),
            None => Err(PyKeyError::new_err(format!(
                "variable '{}' has no term '{}'; its terms are: {:?}",
                self.0.name(),
                name,
                self.0.term_names()
            ))),
        }
    }

    fn __contains__(&self, name: &str) -> bool {
        self.0.term(name).is_some()
    }

    fn __len__(&self) -> usize {
        self.0.term_names().len()
    }

    fn __iter__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<PyObject> {
        let names = slf.get_terms().into_py(py);
        Ok(names.bind(py).iter()?.into_py(py))
    }

    /// Membership of `value` in every term — `{term: μ}`; IT2 terms give `(lower, upper)`.
    ///
    /// >>> temp = ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"])
    /// >>> temp.fuzzify(30)
    /// {'cold': 0.0, 'warm': 0.5, 'hot': 0.5}
    fn fuzzify<'py>(&self, py: Python<'py>, value: f64) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);
        for (name, term) in self.0.terms() {
            match term {
                Term::Type1(mf) => dict.set_item(name, mf.membership(value))?,
                Term::IT2(mf) => dict.set_item(name, mf.membership(value))?,
            }
        }
        Ok(dict)
    }

    /// Replace all terms with evenly spaced, overlapping terms over the universe.
    ///
    /// `names` is an int (3 → low/medium/high, 5 → very_low..very_high,
    /// 7 → NB NM NS ZE PS PM PB, other n → A1..An) or a list of names.
    /// `shape` is "triangle" or "gaussian". Returns the variable for chaining.
    ///
    /// >>> ff.Variable("speed", 0, 100).automf(3).get_terms()
    /// ['low', 'medium', 'high']
    /// >>> ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"], shape="gaussian")
    #[pyo3(
        signature = (names=TermNames::Count(3), shape="triangle"),
        text_signature = "($self, names=3, shape=\"triangle\")"
    )]
    fn automf<'py>(
        mut slf: PyRefMut<'py, Self>,
        names: TermNames,
        shape: &str,
    ) -> PyResult<PyRefMut<'py, Self>> {
        let shape: Shape = parse_opt(shape)?;
        let names = match names {
            TermNames::Count(n) => default_term_names(n),
            TermNames::Names(v) => v,
        };
        slf.0.automf(&names, shape).map_err(to_py_err)?;
        Ok(slf)
    }

    /// Term names in insertion order.
    fn get_terms(&self) -> Vec<String> {
        self.0.term_names().into_iter().map(String::from).collect()
    }

    /// Clamp `value` into `[min_value, max_value]`.
    fn clamp(&self, value: f64) -> f64 {
        self.0.clamp(value)
    }

    fn __repr__(&self) -> String {
        format!(
            "Variable('{}', [{}, {}], {} terms)",
            self.0.name(),
            self.0.min_value(),
            self.0.max_value(),
            self.0.term_names().len()
        )
    }

    /// Every term with its parameters, e.g. `temp [0, 40]: cold = triangle(0, 0, 20), ...`.
    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __call__<'py>(&self, py: Python<'py>, value: f64) -> PyResult<Bound<'py, PyDict>> {
        self.fuzzify(py, value)
    }
}
