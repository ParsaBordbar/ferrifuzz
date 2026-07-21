use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use crate::variable::Variable;
use crate::rule::Rule;


/// A 1st-order (TSK) linear consequent: `z = bias + Σ coeffs[var] · input[var]`.
/// A 0th-order (constant) consequent is just this with empty `coeffs`.
#[derive(Clone)]
struct LinearConsequent {
    bias: f64,
    coeffs: HashMap<String, f64>,
}

impl LinearConsequent {
    #[inline]
    fn evaluate(&self, crisp: &HashMap<String, f64>) -> f64 {
        let mut z = self.bias;
        for (var, c) in &self.coeffs {
            if let Some(&x) = crisp.get(var) {
                z += c * x;
            }
        }
        z
    }
}


/// AND t-norm used to combine antecedent clauses within an OR-group.
#[derive(Clone, Copy)]
enum AndOp {
    Min,
    Product,
}

/// OR t-conorm used to combine the OR-groups of a rule's antecedent.
#[derive(Clone, Copy)]
enum OrOp {
    Max,
    ProbOr,
}

/// Defuzzification strategy for turning aggregated output MFs into a crisp value.
/// `Sugeno` short-circuits the Mamdani sampling path with a weighted average.
#[derive(Clone, Copy)]
enum Defuzz {
    Centroid,
    Bisector,
    Mom,
    Som,
    Lom,
    Sugeno,
}

impl AndOp {
    fn parse(s: &str) -> Self {
        match s {
            "product" | "algebraic" => AndOp::Product,
            _ => AndOp::Min,
        }
    }
}

impl OrOp {
    fn parse(s: &str) -> Self {
        match s {
            "prob" | "probor" | "probabilistic" | "algebraic" => OrOp::ProbOr,
            _ => OrOp::Max,
        }
    }
}

impl Defuzz {
    fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sugeno" | "weighted_average" | "wtaver" => Defuzz::Sugeno,
            "bisector" => Defuzz::Bisector,
            "mom" => Defuzz::Mom,
            "som" => Defuzz::Som,
            "lom" => Defuzz::Lom,
            _ => Defuzz::Centroid,
        }
    }
}


#[pyclass(module = "ferrifuzz")]
pub struct FuzzySystem {
    inputs: HashMap<String, Variable>,
    outputs: HashMap<String, Variable>,
    rules: Vec<Rule>,
    consequents: HashMap<String, HashMap<String, LinearConsequent>>,
    and_method: String,
    or_method: String,
    defuzz_method: String,
    resolution: usize,
    and_op: AndOp,
    or_op: OrOp,
    defuzz: Defuzz,
}

#[pymethods]
impl FuzzySystem {
    #[new]
    #[pyo3(signature = (and_method = "min", or_method = "max", defuzz_method = "centroid", resolution = 200))]
    pub fn new(and_method: &str, or_method: &str, defuzz_method: &str, resolution: usize) -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            rules: Vec::new(),
            consequents: HashMap::new(),
            and_op: AndOp::parse(and_method),
            or_op: OrOp::parse(or_method),
            defuzz: Defuzz::parse(defuzz_method),
            and_method: and_method.to_string(),
            or_method: or_method.to_string(),
            defuzz_method: defuzz_method.to_string(),
            resolution,
        }
    }

    /// Register a linguistic variable as a system input.
    ///
    /// Parameters
    /// ----------
    /// variable : Variable
    ///     The input variable (must have terms defined).
    pub fn add_input(&mut self, variable: Variable) {
        self.inputs.insert(variable.name.clone(), variable);
    }

    /// Register a linguistic variable as a system output.
    ///
    /// Parameters
    /// ----------
    /// variable : Variable
    ///     The output variable (must have terms defined).
    pub fn add_output(&mut self, variable: Variable) {
        self.outputs.insert(variable.name.clone(), variable);
    }

    /// Register a 1st-order (TSK) linear consequent for a Sugeno output term:
    /// `z = bias + Σ coeffs[input_var] · input_value`.
    ///
    /// Use with `defuzz_method="sugeno"`. A rule concluding `... THEN out IS term`
    /// then emits this linear function of the crisp inputs instead of an MF center.
    /// Pass empty `coeffs` for a plain constant (0th-order) consequent.
    ///
    /// Parameters
    /// ----------
    /// out_var : str    Output variable name.
    /// term : str       Consequent term name referenced by rules.
    /// coeffs : dict[str, float]   Input-variable name → coefficient.
    /// bias : float     Constant offset (default 0.0).
    #[pyo3(signature = (out_var, term, coeffs, bias = 0.0))]
    pub fn add_consequent(
        &mut self,
        out_var: String,
        term: String,
        coeffs: HashMap<String, f64>,
        bias: f64,
    ) {
        self.consequents
            .entry(out_var)
            .or_default()
            .insert(term, LinearConsequent { bias, coeffs });
    }

    /// Append a rule to the rule base.
    ///
    /// Parameters
    /// ----------
    /// rule : Rule
    ///     A ``Rule`` object to add.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Supports ``system += rule`` syntax for adding rules.
    fn __iadd__(&mut self, rule: Rule) -> PyResult<()> {
        self.rules.push(rule);
        Ok(())
    }

    /// Run inference given crisp input values.
    ///
    /// Parameters
    /// ----------
    /// inputs : dict[str, float]
    ///     Mapping of input variable name to crisp numeric value.
    ///
    /// Returns
    /// -------
    /// dict[str, float]
    ///     Defuzzified crisp output for each output variable.
    ///
    /// Examples
    /// --------
    /// >>> result = system.compute({"temperature": 28.0, "humidity": 70.0})
    /// >>> print(result["fan_speed"])
    pub fn compute<'py>(&self, py: Python<'py>, inputs: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyDict>> {
        let mut crisp: HashMap<String, f64> = HashMap::with_capacity(self.inputs.len());
        for var_name in self.inputs.keys() {
            if let Ok(Some(raw)) = inputs.get_item(var_name) {
                crisp.insert(var_name.clone(), raw.extract()?);
            }
        }

        let outputs = self.infer(&crisp);
        let result = PyDict::new_bound(py);
        for (var_name, value) in outputs {
            result.set_item(var_name, value)?;
        }
        Ok(result)
    }

    /// Vectorized inference: run a whole batch of input rows in one call.
    ///
    /// Columnar in, columnar out — `{var: [values]}` → `{out_var: [values]}`.
    /// Each input column is a 1-D sequence (Python list or NumPy float64 array);
    /// all columns must share the same length `N`. The result columns are NumPy
    /// arrays when NumPy is importable, otherwise plain lists. Rows where an
    /// output variable has no firing rule are filled with `NaN`.
    ///
    /// The entire `N`-row loop runs in Rust over the shared inference core — one
    /// FFI crossing and no per-row Python object churn, unlike calling
    /// `compute()` in a Python loop.
    ///
    /// Examples
    /// --------
    /// >>> import numpy as np
    /// >>> out = system.compute_batch({"temperature": np.array([20.0, 55.0, 90.0])})
    /// >>> out["fan"]            # np.ndarray of shape (3,)
    pub fn compute_batch<'py>(
        &self,
        py: Python<'py>,
        inputs: HashMap<String, Vec<f64>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut n: Option<usize> = None;
        for (name, col) in &inputs {
            match n {
                None => n = Some(col.len()),
                Some(len) if len != col.len() => {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "input column '{}' has length {}, expected {}",
                        name,
                        col.len(),
                        len
                    )));
                }
                _ => {}
            }
        }
        let n = n.unwrap_or(0);

        let mut out: HashMap<String, Vec<f64>> = HashMap::new();
        let mut crisp: HashMap<String, f64> = HashMap::with_capacity(inputs.len());
        for i in 0..n {
            crisp.clear();
            for (name, col) in &inputs {
                crisp.insert(name.clone(), col[i]);
            }
            for (var, val) in self.infer(&crisp) {
                out.entry(var).or_insert_with(|| vec![f64::NAN; n])[i] = val;
            }
        }

        let numpy = py.import_bound("numpy").ok();
        let result = PyDict::new_bound(py);
        for (var, col) in out {
            match &numpy {
                Some(np) => {
                    let arr = np.call_method1("asarray", (col,))?;
                    result.set_item(var, arr)?;
                }
                None => result.set_item(var, col)?,
            }
        }
        Ok(result)
    }

    /// Callable shorthand — equivalent to ``system.compute(inputs)``.
    ///
    /// Examples
    /// --------
    /// >>> output = system({"temperature": 35.0})
    fn __call__<'py>(&self, py: Python<'py>, inputs: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyDict>> {
        self.compute(py, inputs)
    }

    fn __repr__(&self) -> String {
        format!(
            "FuzzySystem(inputs={}, outputs={}, rules={}, and='{}', or='{}', defuzz='{}')",
            self.inputs.len(),
            self.outputs.len(),
            self.rules.len(),
            self.and_method,
            self.or_method,
            self.defuzz_method,
        )
    }

    /// Names of all registered input variables.
    pub fn input_names(&self) -> Vec<String> {
        self.inputs.keys().cloned().collect()
    }

    /// Names of all registered output variables.
    pub fn output_names(&self) -> Vec<String> {
        self.outputs.keys().cloned().collect()
    }

    /// Number of rules currently in the rule base.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// All rules as human-readable strings.
    pub fn rules_str(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.__str__()).collect()
    }
}


impl FuzzySystem {
    /// Pure-Rust inference core: crisp inputs → crisp outputs.
    ///
    /// This holds no Python objects, so it is the reusable engine behind both the
    /// scalar `compute()` and the vectorized batch path — the hot loop stays in Rust.
    fn infer(&self, crisp: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut fuzzy_inputs: HashMap<String, HashMap<String, f64>> =
            HashMap::with_capacity(self.inputs.len());
        for (name, var) in &self.inputs {
            if let Some(&val) = crisp.get(name) {
                fuzzy_inputs.insert(name.clone(), var.evaluate(val));
            }
        }

        let use_product = matches!(self.and_op, AndOp::Product);
        let use_probor = matches!(self.or_op, OrOp::ProbOr);
        let is_sugeno = matches!(self.defuzz, Defuzz::Sugeno);

        let mut activations: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let mut sug_num: HashMap<String, f64> = HashMap::new();
        let mut sug_den: HashMap<String, f64> = HashMap::new();

        for rule in &self.rules {
            let mut rule_strength: f64 = 0.0;
            let mut any_valid = false;

            for group in &rule.antecedent {
                let mut group_strength: f64 = 1.0;
                let mut valid = true;

                for (var_name, term_name) in group {
                    match fuzzy_inputs.get(var_name) {
                        Some(terms) => {
                            let mu = terms.get(term_name).copied().unwrap_or(0.0);
                            group_strength = if use_product {
                                group_strength * mu
                            } else {
                                group_strength.min(mu)
                            };
                        }
                        None => { valid = false; break; }
                    }
                }

                if !valid { continue; }
                any_valid = true;
                rule_strength = if use_probor {
                    rule_strength + group_strength - rule_strength * group_strength
                } else {
                    rule_strength.max(group_strength)
                };
            }

            if !any_valid || rule_strength <= 0.0 { continue; }

            let (out_var, out_term) = &rule.conclusion;
            if is_sugeno {
                let z = self.consequent_value(out_var, out_term, crisp);
                *sug_num.entry(out_var.clone()).or_insert(0.0) += rule_strength * z;
                *sug_den.entry(out_var.clone()).or_insert(0.0) += rule_strength;
            } else {
                activations
                    .entry(out_var.clone())
                    .or_default()
                    .entry(out_term.clone())
                    .and_modify(|v| *v = v.max(rule_strength))
                    .or_insert(rule_strength);
            }
        }

        let mut result = HashMap::new();
        if is_sugeno {
            for (var, num) in &sug_num {
                let den = sug_den[var];
                if den > 0.0 {
                    result.insert(var.clone(), num / den);
                }
            }
        } else {
            for (var_name, term_activations) in &activations {
                if let Some(var) = self.outputs.get(var_name) {
                    result.insert(var_name.clone(), self.defuzzify(var, term_activations));
                }
            }
        }
        result
    }

    /// Value emitted by a fired Sugeno rule for `(out_var, out_term)`:
    /// a registered 1st-order linear consequent if present, else the output
    /// term's MF analytic center (0th-order / constant behaviour).
    fn consequent_value(&self, out_var: &str, out_term: &str, crisp: &HashMap<String, f64>) -> f64 {
        if let Some(lc) = self.consequents.get(out_var).and_then(|m| m.get(out_term)) {
            return lc.evaluate(crisp);
        }
        if let Some(mf) = self.outputs.get(out_var).and_then(|v| v.terms.get(out_term)) {
            return mf.center();
        }
        0.0
    }

    fn defuzzify(&self, var: &Variable, activations: &HashMap<String, f64>) -> f64 {
        let sampled = self.sample(var, activations);
        match self.defuzz {
            Defuzz::Bisector => sampled.bisector(var),
            Defuzz::Mom => sampled.mom(var),
            Defuzz::Som => sampled.som(var),
            Defuzz::Lom => sampled.lom(var),
            Defuzz::Centroid | Defuzz::Sugeno => sampled.centroid(var),
        }
    }

    /// Max-aggregate all active terms at x (clipped to their firing strength).
    fn aggregate(&self, var: &Variable, x: f64, activations: &HashMap<String, f64>) -> f64 {
        let mut agg: f64 = 0.0;
        for (term_name, mf) in &var.terms {
            if let Some(&activation) = activations.get(term_name) {
                agg = agg.max(mf.membership(x).min(activation));
            }
        }
        agg
    }

    /// Sample the aggregated output MF once over the resolution grid.
    /// Every Mamdani defuzzifier consumes this shared `(xs, mus)` pair, so the
    /// per-point aggregation runs a single time regardless of method.
    fn sample(&self, var: &Variable, activations: &HashMap<String, f64>) -> Sampled {
        let step = (var.max_value - var.min_value) / self.resolution as f64;
        let xs: Vec<f64> = (0..=self.resolution)
            .map(|i| var.min_value + i as f64 * step)
            .collect();
        let mus: Vec<f64> = xs.iter().map(|&x| self.aggregate(var, x, activations)).collect();
        Sampled { xs, mus, step }
    }
}


/// The aggregated output MF evaluated on the resolution grid: parallel `xs`/`mus`
/// vectors plus the grid `step`. Each defuzzifier is a pure reduction over it.
struct Sampled {
    xs: Vec<f64>,
    mus: Vec<f64>,
    step: f64,
}

impl Sampled {
    fn midpoint(var: &Variable) -> f64 {
        (var.min_value + var.max_value) / 2.0
    }

    fn peak(&self) -> f64 {
        self.mus.iter().cloned().fold(0.0_f64, f64::max)
    }

    fn centroid(&self, var: &Variable) -> f64 {
        let (mut num, mut den) = (0.0_f64, 0.0_f64);
        for (&x, &mu) in self.xs.iter().zip(&self.mus) {
            num += x * mu;
            den += mu;
        }
        if den > 0.0 { num / den } else { Self::midpoint(var) }
    }

    fn bisector(&self, var: &Variable) -> f64 {
        let half = self.mus.iter().sum::<f64>() * self.step / 2.0;
        let mut cum = 0.0;
        for (&x, &mu) in self.xs.iter().zip(&self.mus) {
            cum += mu * self.step;
            if cum >= half { return x; }
        }
        Self::midpoint(var)
    }

    fn mom(&self, var: &Variable) -> f64 {
        let peak = self.peak();
        if peak == 0.0 { return Self::midpoint(var); }
        let (sum, count) = self.xs.iter().zip(&self.mus)
            .filter(|(_, &mu)| (mu - peak).abs() < 1e-9)
            .fold((0.0_f64, 0usize), |(s, c), (&x, _)| (s + x, c + 1));
        if count > 0 { sum / count as f64 } else { Self::midpoint(var) }
    }

    fn som(&self, var: &Variable) -> f64 {
        let peak = self.peak();
        if peak == 0.0 { return var.min_value; }
        self.xs.iter().zip(&self.mus)
            .find(|(_, &mu)| (mu - peak).abs() < 1e-9)
            .map(|(&x, _)| x)
            .unwrap_or(var.min_value)
    }

    fn lom(&self, var: &Variable) -> f64 {
        let peak = self.peak();
        if peak == 0.0 { return var.max_value; }
        self.xs.iter().zip(&self.mus).rev()
            .find(|(_, &mu)| (mu - peak).abs() < 1e-9)
            .map(|(&x, _)| x)
            .unwrap_or(var.max_value)
    }
}
