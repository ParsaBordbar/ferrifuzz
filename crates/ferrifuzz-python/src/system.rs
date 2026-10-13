use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyMapping};

use ferrifuzz::{Defuzz, FuzzySystem, Implication, InferenceType, TConorm, TNorm, TypeReduction};

use crate::rule::PyRule;
use crate::variable::PyVariable;
use crate::{parse_opt, to_array, to_py_err, InputError};

/// Fuzzy inference system — Mamdani, Sugeno (TSK) and interval type-2.
#[pyclass(name = "FuzzySystem", module = "ferrifuzz")]
pub struct PyFuzzySystem(FuzzySystem);

/// `{name: float}` from any mapping (dict, pandas row, ...).
fn crisp_inputs(inputs: &Bound<'_, PyAny>) -> PyResult<Vec<(String, f64)>> {
    let mapping = inputs
        .downcast::<PyMapping>()
        .map_err(|_| InputError::new_err("inputs must be a mapping such as {'temp': 25.0}"))?;
    let mut out = Vec::new();
    for item in mapping.items()?.iter()? {
        let (name, value): (String, Bound<'_, PyAny>) = item?.extract()?;
        let v: f64 = value.extract().map_err(|_| {
            InputError::new_err(format!(
                "input '{}' must be a number, got {}",
                name,
                value
                    .get_type()
                    .name()
                    .map(|n| n.to_string())
                    .unwrap_or_default()
            ))
        })?;
        out.push((name, v));
    }
    Ok(out)
}

#[pymethods]
impl PyFuzzySystem {
    #[new]
    #[pyo3(signature = (and_method = "min", or_method = "max", defuzz_method = "centroid", resolution = 200, r#type = "auto", implication = "mamdani", type_reduction = "km", clamp_inputs = true))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        and_method: &str,
        or_method: &str,
        defuzz_method: &str,
        resolution: usize,
        r#type: &str,
        implication: &str,
        type_reduction: &str,
        clamp_inputs: bool,
    ) -> PyResult<Self> {
        Ok(Self(
            FuzzySystem::new()
                .with_and(parse_opt::<TNorm>(and_method)?)
                .with_or(parse_opt::<TConorm>(or_method)?)
                .with_defuzz(parse_opt::<Defuzz>(defuzz_method)?)
                .with_resolution(resolution)
                .with_inference(parse_opt::<InferenceType>(r#type)?)
                .with_implication(parse_opt::<Implication>(implication)?)
                .with_type_reduction(parse_opt::<TypeReduction>(type_reduction)?)
                .with_input_clamping(clamp_inputs),
        ))
    }

    /// Register a copy of `variable` as an input (replaces one with the same name).
    fn add_input(&mut self, variable: PyVariable) {
        self.0.add_input(variable.0);
    }

    /// Register a copy of `variable` as an output (replaces one with the same name).
    fn add_output(&mut self, variable: PyVariable) {
        self.0.add_output(variable.0);
    }

    /// Register a first-order TSK consequent `z = bias + Σ coeffs[input]·input` for the
    /// Sugeno output `out_var IS term` (needs `defuzz_method="weighted_average"`). `term`
    /// may be a new name used only by rules, so each rule can have its own linear output.
    #[pyo3(signature = (out_var, term, coeffs, bias = 0.0))]
    fn add_consequent(
        &mut self,
        out_var: String,
        term: String,
        coeffs: HashMap<String, f64>,
        bias: f64,
    ) {
        let mut coeffs: Vec<(String, f64)> = coeffs.into_iter().collect();
        coeffs.sort_by(|a, b| a.0.cmp(&b.0));
        self.0.add_consequent(out_var, term, coeffs, bias);
    }

    /// Append a rule (a `Rule` or a rule string).
    fn add_rule(&mut self, rule: RuleArg) -> PyResult<()> {
        self.0.add_rule(rule.into_rule()?);
        Ok(())
    }

    /// Append several rules (`Rule` objects or rule strings); nothing is added on error.
    fn add_rules(&mut self, rules: Vec<RuleArg>) -> PyResult<()> {
        let rules = rules
            .into_iter()
            .map(RuleArg::into_rule)
            .collect::<PyResult<Vec<_>>>()?;
        for r in rules {
            self.0.add_rule(r);
        }
        Ok(())
    }

    /// `system += rule` (a `Rule` or a rule string).
    fn __iadd__(&mut self, rule: RuleArg) -> PyResult<()> {
        self.add_rule(rule)
    }

    /// Run inference — `{input: value}` in, `{output: value}` out (every output; NaN
    /// where no rule fired). Raises `ff.InputError` for missing / unknown / NaN inputs
    /// and `ff.ModelError` when a rule names an unknown variable or term.
    fn compute<'py>(
        &self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let crisp = crisp_inputs(inputs)?;
        let out = self.0.compute(crisp).map_err(to_py_err)?;
        self.ordered(py, out)
    }

    /// Callable shorthand — `system({"temp": 30})` is `system.compute({"temp": 30})`.
    fn __call__<'py>(
        &self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyDict>> {
        self.compute(py, inputs)
    }

    /// Type-reduced interval `{output: (y_l, y_r)}` (IT2); `(y, y)` for type-1 systems.
    fn compute_interval<'py>(
        &self,
        py: Python<'py>,
        inputs: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let crisp = crisp_inputs(inputs)?;
        let out = self.0.compute_interval(crisp).map_err(to_py_err)?;
        self.ordered(py, out)
    }

    /// Vectorized inference — `{input: column}` in (lists or NumPy arrays of equal length),
    /// `{output: column}` out as NumPy arrays (lists without NumPy). Runs on all cores.
    fn compute_batch<'py>(
        &self,
        py: Python<'py>,
        inputs: HashMap<String, Vec<f64>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let out = py
            .allow_threads(|| self.0.compute_batch(&inputs))
            .map_err(to_py_err)?;
        let result = PyDict::new_bound(py);
        let mut out = out;
        for name in self.0.output_names() {
            if let Some(col) = out.remove(name) {
                result.set_item(name, to_array(py, col)?)?;
            }
        }
        Ok(result)
    }

    /// Positional inference: input values in `input_names()` order → list of outputs in
    /// `output_names()` order. Fastest call for simulation loops.
    fn evaluate(&self, values: Vec<f64>) -> PyResult<Vec<f64>> {
        self.0.evaluate(&values).map_err(to_py_err)
    }

    /// Firing strength of every rule (in rule order) — e.g. for ANFIS-style least squares.
    fn rule_strengths(&self, inputs: &Bound<'_, PyAny>) -> PyResult<Vec<f64>> {
        self.0
            .rule_strengths(crisp_inputs(inputs)?)
            .map_err(to_py_err)
    }

    /// Firing interval `(lower, upper)` of every rule.
    fn rule_strengths_interval(&self, inputs: &Bound<'_, PyAny>) -> PyResult<Vec<(f64, f64)>> {
        self.0
            .rule_strengths_interval(crisp_inputs(inputs)?)
            .map_err(to_py_err)
    }

    /// Aggregated output fuzzy set of a type-1 Mamdani system: `(xs, mus)` arrays.
    fn output_set(
        &self,
        py: Python<'_>,
        inputs: &Bound<'_, PyAny>,
        output: &str,
    ) -> PyResult<(PyObject, PyObject)> {
        let (xs, mus) = self
            .0
            .output_set(crisp_inputs(inputs)?, output)
            .map_err(to_py_err)?;
        Ok((to_array(py, xs)?, to_array(py, mus)?))
    }

    /// Aggregated footprint of uncertainty of an output: `(xs, lower, upper)` arrays.
    fn output_fou(
        &self,
        py: Python<'_>,
        inputs: &Bound<'_, PyAny>,
        output: &str,
    ) -> PyResult<(PyObject, PyObject, PyObject)> {
        let (xs, lo, up) = self
            .0
            .output_fou(crisp_inputs(inputs)?, output)
            .map_err(to_py_err)?;
        Ok((to_array(py, xs)?, to_array(py, lo)?, to_array(py, up)?))
    }

    /// Raise `ff.ModelError` describing every problem that would make `compute` fail.
    fn check(&self) -> PyResult<()> {
        self.0.check().map_err(to_py_err)
    }

    /// Every problem found — `"error: ..."` lines first, then `"warning: ..."` lines
    /// (unused inputs, duplicate / conflicting rules, ...). Empty when the model is clean.
    fn validate(&self) -> Vec<String> {
        self.0.validate()
    }

    /// Copy of a registered input variable.
    fn input(&self, name: &str) -> Option<PyVariable> {
        self.0.input(name).cloned().map(PyVariable)
    }

    /// Copy of a registered output variable.
    fn output(&self, name: &str) -> Option<PyVariable> {
        self.0.output(name).cloned().map(PyVariable)
    }

    fn input_names(&self) -> Vec<String> {
        self.0.input_names().into_iter().map(String::from).collect()
    }

    fn output_names(&self) -> Vec<String> {
        self.0
            .output_names()
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn rule_count(&self) -> usize {
        self.0.rules().len()
    }

    /// All rules as strings (parseable by `Rule.from_string`).
    fn rules_str(&self) -> Vec<String> {
        self.0.rules().iter().map(ToString::to_string).collect()
    }

    /// Copies of the rules.
    #[getter]
    fn rules(&self) -> Vec<PyRule> {
        self.0.rules().iter().cloned().map(PyRule).collect()
    }

    #[getter]
    fn and_method(&self) -> &'static str {
        self.0.and_method().name()
    }

    #[getter]
    fn or_method(&self) -> &'static str {
        self.0.or_method().name()
    }

    #[getter]
    fn defuzz_method(&self) -> &'static str {
        self.0.defuzz().name()
    }

    #[getter]
    fn implication(&self) -> &'static str {
        self.0.implication().name()
    }

    /// Engine requested ("auto", "type1", "it2").
    #[getter]
    fn r#type(&self) -> &'static str {
        self.0.inference().name()
    }

    #[getter]
    fn type_reduction(&self) -> &'static str {
        self.0.type_reduction().name()
    }

    #[getter]
    fn resolution(&self) -> usize {
        self.0.resolution()
    }

    #[getter]
    fn clamp_inputs(&self) -> bool {
        self.0.input_clamping()
    }

    /// True when the interval type-2 engine runs.
    #[getter]
    fn is_it2(&self) -> bool {
        self.0.is_it2()
    }

    fn __repr__(&self) -> String {
        self.0.to_string()
    }
}

impl PyFuzzySystem {
    fn ordered<'py, T: ToPyObject>(
        &self,
        py: Python<'py>,
        mut out: HashMap<String, T>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let result = PyDict::new_bound(py);
        for name in self.0.output_names() {
            if let Some(v) = out.remove(name) {
                result.set_item(name, v)?;
            }
        }
        Ok(result)
    }
}

/// A `Rule` object or a rule string.
#[derive(FromPyObject)]
pub enum RuleArg {
    Rule(PyRule),
    Text(String),
}

impl RuleArg {
    fn into_rule(self) -> PyResult<ferrifuzz::Rule> {
        match self {
            RuleArg::Rule(r) => Ok(r.0),
            RuleArg::Text(s) => ferrifuzz::Rule::parse(&s).map_err(to_py_err),
        }
    }
}
