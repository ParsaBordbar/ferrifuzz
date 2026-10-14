use pyo3::class::basic::CompareOp;
use pyo3::exceptions::{PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;

use ferrifuzz::{Clause, Rule};

use crate::to_py_err;

/// One clause: `var IS [NOT] [hedge] term`.
#[pyclass(name = "Clause", module = "ferrifuzz.rule")]
#[derive(Clone)]
pub struct PyClause(pub Clause);

#[pymethods]
impl PyClause {
    #[new]
    #[pyo3(signature = (var, term, negated=false, hedge=None))]
    fn new(var: String, term: String, negated: bool, hedge: Option<String>) -> PyResult<Self> {
        let mut clause = Clause::new(var, term);
        clause.negated = negated;
        if let Some(word) = hedge {
            clause = clause.with_hedge(&word).map_err(to_py_err)?;
        }
        Ok(Self(clause))
    }

    #[getter]
    fn var(&self) -> &str {
        &self.0.var
    }

    #[getter]
    fn term(&self) -> &str {
        &self.0.term
    }

    /// True if the clause is negated (`... IS NOT term`).
    #[getter]
    fn negated(&self) -> bool {
        self.0.negated
    }

    /// Hedge word applied to the term, e.g. `"very"` (`None` if unhedged).
    #[getter]
    fn hedge(&self) -> Option<&str> {
        self.0.hedge.as_deref()
    }

    /// The hedge exponent (`μ^power`) or `None` when the clause is unhedged.
    #[getter]
    fn hedge_power(&self) -> Option<f64> {
        self.0.hedge().map(|h| h.power())
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<Clause: '{}'>", self.0)
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        compare(&self.0, &other.0, op, "Clauses")
    }
}

type AndGroup = Vec<(String, String)>;

fn group_from_tuples(group: AndGroup) -> Vec<Clause> {
    group.into_iter().map(Clause::from).collect()
}

fn group_to_tuples(group: &[Clause]) -> AndGroup {
    group
        .iter()
        .map(|c| (c.var.clone(), c.term.clone()))
        .collect()
}

/// A rule's consequent as given from Python — one or many, tuples or `Clause`s.
#[derive(FromPyObject)]
pub enum ConclusionArg {
    Tuples(Vec<(String, String)>),
    Clauses(Vec<PyClause>),
    Tuple((String, String)),
    Single(PyClause),
}

impl ConclusionArg {
    fn into_clauses(self) -> PyResult<Vec<Clause>> {
        let clauses = match self {
            ConclusionArg::Tuples(ts) => group_from_tuples(ts),
            ConclusionArg::Clauses(cs) => cs.into_iter().map(|c| c.0).collect(),
            ConclusionArg::Tuple(t) => vec![t.into()],
            ConclusionArg::Single(c) => vec![c.0],
        };
        if clauses.is_empty() {
            return Err(PyValueError::new_err(
                "a rule needs at least one consequent",
            ));
        }
        Ok(clauses)
    }
}

/// A fuzzy IF-THEN rule; antecedent is stored in DNF (an OR of AND-groups).
#[pyclass(name = "Rule", module = "ferrifuzz.rule")]
#[derive(Clone)]
pub struct PyRule(pub Rule);

#[pymethods]
impl PyRule {
    /// Build from a flat AND `conditions` list or an explicit DNF `antecedent` — exactly one.
    ///
    /// `conclusion` takes a single `(var, term)` tuple or a list of them for a rule
    /// that drives several outputs; `Clause` objects are accepted in either position.
    #[new]
    #[pyo3(signature = (conditions=None, conclusion=None, antecedent=None, weight=1.0))]
    fn new(
        conditions: Option<AndGroup>,
        conclusion: Option<ConclusionArg>,
        antecedent: Option<Vec<AndGroup>>,
        weight: f64,
    ) -> PyResult<Self> {
        let consequent = conclusion
            .ok_or_else(|| PyValueError::new_err("`conclusion` is required"))?
            .into_clauses()?;
        let antecedent = match (conditions, antecedent) {
            (Some(_), Some(_)) => {
                return Err(PyValueError::new_err(
                    "pass either `conditions` or `antecedent`, not both",
                ))
            }
            (Some(conds), None) => vec![group_from_tuples(conds)],
            (None, Some(ante)) => ante.into_iter().map(group_from_tuples).collect(),
            (None, None) => {
                return Err(PyValueError::new_err(
                    "one of `conditions` or `antecedent` is required",
                ))
            }
        };
        Ok(Self(Rule {
            antecedent,
            consequent,
            weight,
        }))
    }

    /// Parse a rule string with AND/OR (AND binds tighter), `IS NOT`, and inline hedges.
    ///
    /// The consequent may itself be an AND-list — `THEN fan IS fast AND alarm IS on`.
    #[classmethod]
    fn from_string(_cls: &Bound<'_, PyType>, rule_str: &str) -> PyResult<Self> {
        Rule::parse(rule_str).map(Self).map_err(to_py_err)
    }

    #[getter]
    fn weight(&self) -> f64 {
        self.0.weight
    }

    #[setter]
    fn set_weight(&mut self, weight: f64) {
        self.0.weight = weight;
    }

    /// The antecedent as full `Clause` objects (DNF: OR of AND-groups).
    #[getter]
    fn clauses(&self) -> Vec<Vec<PyClause>> {
        self.0
            .antecedent
            .iter()
            .map(|g| g.iter().cloned().map(PyClause).collect())
            .collect()
    }

    /// The antecedent as `(var, term)` tuples — negation and hedges are dropped.
    #[getter]
    fn antecedent(&self) -> Vec<AndGroup> {
        self.0
            .antecedent
            .iter()
            .map(|g| group_to_tuples(g))
            .collect()
    }

    #[setter]
    fn set_antecedent(&mut self, antecedent: Vec<AndGroup>) {
        self.0.antecedent = antecedent.into_iter().map(group_from_tuples).collect();
    }

    /// Flat `(var, term)` AND list — errors on OR rules; read `.antecedent` instead.
    #[getter]
    fn conditions(&self) -> PyResult<AndGroup> {
        match self.0.antecedent.as_slice() {
            [group] => Ok(group_to_tuples(group)),
            _ => Err(PyValueError::new_err(
                "this rule uses OR; read `.antecedent` (DNF) instead of `.conditions`",
            )),
        }
    }

    #[setter]
    fn set_conditions(&mut self, conditions: AndGroup) {
        self.0.antecedent = vec![group_from_tuples(conditions)];
    }

    /// The single `(var, term)` consequent — errors on multi-output rules.
    #[getter]
    fn conclusion(&self) -> PyResult<(String, String)> {
        match self.0.consequent.as_slice() {
            [c] => Ok((c.var.clone(), c.term.clone())),
            other => Err(PyValueError::new_err(format!(
                "this rule has {} consequents; read `.conclusions` instead",
                other.len()
            ))),
        }
    }

    #[setter]
    fn set_conclusion(&mut self, conclusion: ConclusionArg) -> PyResult<()> {
        self.0.consequent = conclusion.into_clauses()?;
        Ok(())
    }

    /// Every consequent as `(var, term)` — negation and hedges are dropped.
    #[getter]
    fn conclusions(&self) -> AndGroup {
        group_to_tuples(&self.0.consequent)
    }

    #[setter]
    fn set_conclusions(&mut self, conclusions: ConclusionArg) -> PyResult<()> {
        self.set_conclusion(conclusions)
    }

    /// The consequent as full `Clause` objects, keeping negation and hedges.
    #[getter]
    fn consequent_clauses(&self) -> Vec<PyClause> {
        self.0.consequent.iter().cloned().map(PyClause).collect()
    }

    /// Number of consequents (1 for a plain single-output rule).
    fn consequent_count(&self) -> usize {
        self.0.consequent.len()
    }

    /// Number of OR-groups in the antecedent (1 for a plain AND rule).
    fn group_count(&self) -> usize {
        self.0.antecedent.len()
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __repr__(&self) -> String {
        format!("<Rule: '{}'>", self.0)
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        compare(&self.0, &other.0, op, "Rules")
    }
}

fn compare<T: PartialEq>(a: &T, b: &T, op: CompareOp, what: &str) -> PyResult<bool> {
    match op {
        CompareOp::Eq => Ok(a == b),
        CompareOp::Ne => Ok(a != b),
        _ => Err(PyNotImplementedError::new_err(format!(
            "Only == and != comparisons are supported for {}",
            what
        ))),
    }
}
