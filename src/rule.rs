use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::PyType;

/// One conjunctive clause: a list of `(variable, term)` pairs combined with AND.
pub type AndGroup = Vec<(String, String)>;

#[pyclass(module = "ferrifuzz.rule")]
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// Antecedent in disjunctive normal form (DNF):
    /// an OR of AND-groups — `[[a AND b], [c]]` means `(a AND b) OR c`.
    /// Parsed once at construction so inference never touches strings.
    pub antecedent: Vec<AndGroup>,

    #[pyo3(get, set)]
    pub conclusion: (String, String),
}

#[pymethods]
impl Rule {
    /// Construct a rule.
    ///
    /// - `conditions` — a flat AND list `[(var, term), ...]` (the common case).
    /// - `antecedent` — an explicit DNF `[[...AND...], [...AND...]]` for OR rules.
    ///
    /// Exactly one of `conditions` / `antecedent` must be given.
    #[new]
    #[pyo3(signature = (conditions=None, conclusion=None, antecedent=None))]
    pub fn new(
        conditions: Option<AndGroup>,
        conclusion: Option<(String, String)>,
        antecedent: Option<Vec<AndGroup>>,
    ) -> PyResult<Self> {
        let conclusion = conclusion.ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("`conclusion` is required")
        })?;

        let antecedent = match (conditions, antecedent) {
            (Some(_), Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "pass either `conditions` or `antecedent`, not both",
                ))
            }
            (Some(conds), None) => vec![conds],
            (None, Some(ante)) => ante,
            (None, None) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "one of `conditions` or `antecedent` is required",
                ))
            }
        };

        Ok(Self {
            antecedent,
            conclusion,
        })
    }

    /// Parse a rule string. Supports `AND` and `OR`, with `AND` binding tighter
    /// than `OR` (standard precedence): `a AND b OR c` → `(a AND b) OR c`.
    #[classmethod]
    pub fn from_string(_cls: &Bound<'_, PyType>, rule_str: &str) -> PyResult<Self> {
        let s = rule_str.trim();

        if !s.starts_with("IF ") || !s.contains(" THEN ") {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Rule string must be in format: 'IF var IS term [AND/OR var IS term ...] THEN var IS term'",
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

        let mut antecedent: Vec<AndGroup> = Vec::new();
        for or_group in cond_str.split(" OR ") {
            let mut group: AndGroup = Vec::new();
            for cond in or_group.split(" AND ") {
                let tokens: Vec<&str> = cond.split(" IS ").collect();
                if tokens.len() != 2 {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Malformed condition: {}",
                        cond.trim()
                    )));
                }
                group.push((tokens[0].trim().to_string(), tokens[1].trim().to_string()));
            }
            antecedent.push(group);
        }

        let conc_tokens: Vec<&str> = conc_str.split(" IS ").collect();
        if conc_tokens.len() != 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Malformed conclusion: {}",
                conc_str.trim()
            )));
        }
        let conclusion = (
            conc_tokens[0].trim().to_string(),
            conc_tokens[1].trim().to_string(),
        );

        Ok(Self {
            antecedent,
            conclusion,
        })
    }

    /// The antecedent in DNF: a list of AND-groups combined with OR.
    #[getter]
    fn antecedent(&self) -> Vec<AndGroup> {
        self.antecedent.clone()
    }

    #[setter]
    fn set_antecedent(&mut self, antecedent: Vec<AndGroup>) {
        self.antecedent = antecedent;
    }

    /// Backward-compatible flat AND list. Only defined for pure-conjunction rules
    /// (a single OR-group). For rules that use OR, read `.antecedent` instead.
    #[getter]
    fn conditions(&self) -> PyResult<AndGroup> {
        match self.antecedent.as_slice() {
            [group] => Ok(group.clone()),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                "this rule uses OR; read `.antecedent` (DNF) instead of `.conditions`",
            )),
        }
    }

    #[setter]
    fn set_conditions(&mut self, conditions: AndGroup) {
        self.antecedent = vec![conditions];
    }

    /// Number of OR-groups in the antecedent (1 for a plain AND rule).
    fn group_count(&self) -> usize {
        self.antecedent.len()
    }

    pub fn __str__(&self) -> String {
        let groups: Vec<String> = self
            .antecedent
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|(v, t)| format!("{} IS {}", v, t))
                    .collect::<Vec<_>>()
                    .join(" AND ")
            })
            .collect();
        format!(
            "IF {} THEN {} IS {}",
            groups.join(" OR "),
            self.conclusion.0,
            self.conclusion.1
        )
    }

    /// Developer-friendly representation (REPL / list prints).
    fn __repr__(&self) -> String {
        format!("<Rule: '{}'>", self.__str__())
    }

    /// Enables `==` and `!=` on Rule objects.
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
