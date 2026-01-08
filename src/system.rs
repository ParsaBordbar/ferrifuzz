use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use crate::variable::Variable;
use crate::rule::Rule;

#[pyclass]
pub struct FuzzySystem {
    pub inputs: HashMap<String, Variable>,
    pub outputs: HashMap<String, Variable>,
    pub rules: Vec<Rule>,
}

#[pymethods]
impl FuzzySystem {
    #[new]
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            rules: Vec::new(),
        }
    }

    pub fn add_input(&mut self, variable: Bound<Variable>) -> PyResult<()> {
        let var = variable.borrow();
        let name = var.name.clone();
        let var_clone = var.clone();
        drop(var);
        self.inputs.insert(name, var_clone);
        Ok(())
    }

    pub fn add_output(&mut self, variable: Bound<Variable>) -> PyResult<()> {
        let var = variable.borrow();
        let name = var.name.clone();
        let var_clone = var.clone();
        drop(var);
        self.outputs.insert(name, var_clone);
        Ok(())
    }

    pub fn add_rule(&mut self, rule: Rule) -> PyResult<()> {
        self.rules.push(rule);
        Ok(())
    }

    pub fn infer<'py>(&self, py: Python<'py>, inputs: Bound<'py, PyDict>) -> PyResult<PyObject> {
        // Fuzzification
        let mut fuzzy_inputs: HashMap<String, HashMap<String, f64>> = HashMap::new();
        
        for (var_name, var) in &self.inputs {
            if let Ok(Some(value)) = inputs.get_item(var_name) {
                let val: f64 = value.extract()?;
                let fuzz_dict = var.fuzzify(py, val)?;
                let fuzz_dict_bound = fuzz_dict.bind(py);
                let fuzz_dict_ref: &Bound<PyDict> = fuzz_dict_bound.downcast()?;
                
                let mut term_memberships = HashMap::new();
                for (key, value) in fuzz_dict_ref.iter() {
                    let k: String = key.extract()?;
                    let v: f64 = value.extract()?;
                    term_memberships.insert(k, v);
                }
                fuzzy_inputs.insert(var_name.clone(), term_memberships);
            }
        }

        // Rule evaluation
        let mut activations: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for rule in &self.rules {
            let mut strength: f64 = 1.0;
            for (var_name, term_name) in &rule.conditions {
                if let Some(terms) = fuzzy_inputs.get(var_name) {
                    if let Some(&membership) = terms.get(term_name) {
                        strength = strength.min(membership);
                    }
                }
            }

            let (out_var, out_term) = &rule.conclusion;
            activations
                .entry(out_var.clone())
                .or_insert_with(HashMap::new)
                .entry(out_term.clone())
                .and_modify(|v| *v = v.max(strength))
                .or_insert(strength);
        }

        // Defuzzification
        let result_dict = PyDict::new_bound(py);
        
        for (var_name, term_activations) in activations {
            if let Some(var) = self.outputs.get(&var_name) {
                let crisp = self.defuzzify(var, &term_activations);
                result_dict.set_item(var_name, crisp)?;
            }
        }

        Ok(result_dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "FuzzySystem({} inputs, {} outputs, {} rules)",
            self.inputs.len(),
            self.outputs.len(),
            self.rules.len()
        )
    }
}

impl FuzzySystem {
    fn defuzzify(&self, var: &Variable, activations: &HashMap<String, f64>) -> f64 {
        const STEPS: usize = 100;
        let range = var.max_value - var.min_value;
        let step = range / STEPS as f64;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..=STEPS {
            let x = var.min_value + i as f64 * step;
            let mut aggregated_membership: f64 = 0.0;

            for term in &var.terms {
                if let Some(&activation) = activations.get(&term.name) {
                    let membership = term.mf.membership(x);
                    aggregated_membership = aggregated_membership.max(activation.min(membership));
                }
            }

            numerator += x * aggregated_membership;
            denominator += aggregated_membership;
        }

        if denominator > 0.0 {
            numerator / denominator
        } else {
            (var.min_value + var.max_value) / 2.0
        }
    }
}