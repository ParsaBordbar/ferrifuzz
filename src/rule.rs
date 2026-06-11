use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass(module = "ferrifuzz.rule")]
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    #[pyo3(get, set)]
    pub conditions: Vec<(String, String)>,

    #[pyo3(get, set)]
    pub conclusion: (String, String),
}

#[pymethods]
impl Rule {
    #[new]
    #[pyo3(signature = (conditions, conclusion))]
    pub fn new(conditions: Vec<(String, String)>, conclusion: (String, String)) -> Self {
        Self {
            conditions,
            conclusion,
        }
    }

    #[classmethod]
    pub fn from_string(_cls: &Bound<'_, PyType>, rule_str: &str) -> PyResult<Self> {
        let s = rule_str.trim();

        if !s.starts_with("IF ") || !s.contains(" THEN ") {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Rule string must be in format: 'IF var IS term AND ... THEN var IS term'",
            ));
        }

        let parts: Vec<&str> = s[3..].split(" THEN ").collect();
        if parts.len() != 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Malformed rule string",
            ));
        }

        let cond_str = parts[0];
        let conc_str = parts[1];

        // Parseing conditions (AND logic here!)
        let mut conditions = Vec::new();

        for cond in cond_str.split(" AND ") {
            let tokens: Vec<&str> = cond.split(" IS ").collect();
            if tokens.len() != 2 {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Malformed condition: {}",
                    cond
                )));
            }
            conditions.push((tokens[0].trim().to_string(), tokens[1].trim().to_string()));
        }

        let conc_tokens: Vec<&str> = conc_str.split(" IS ").collect();

        if conc_tokens.len() != 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Malformed conclusion: {}",
                conc_str
            )));
        }
        let conclusion = (
            conc_tokens[0].trim().to_string(),
            conc_tokens[1].trim().to_string(),
        );

        Ok(Self {
            conditions,
            conclusion,
        })
    }

    // let's users to print the rule! (print(rule))
    pub fn __str__(&self) -> String {
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

    /// Developer-friendly representation (called in REPL or list prints)
    fn __repr__(&self) -> String {
        format!("<Rule: '{}'>", self.__str__())
    }

    /// Allows Python users to use `==` and `!=` operators on Rule objects
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Only == and != comparisons are supported for Rules",
            )),
        }
    }
}
