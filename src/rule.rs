use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Rule {
    pub conditions: Vec<(String, String)>,
    pub conclusion: (String, String),
}

#[pymethods]
impl Rule {
    #[new]
    pub fn new(conditions: Vec<(String, String)>, conclusion: (String, String)) -> Self {
        Self {
            conditions,
            conclusion,
        }
    }

    fn __repr__(&self) -> String {
        let conds: Vec<String> = self
            .conditions
            .iter()
            .map(|(v, t)| format!("{} IS {}", v, t))
            .collect();
        format!(
            "IF {} THEN {} IS {}",
            conds.join(" AND "),
            self.conclusion.0,
            self.conclusion.1
        )
    }
}