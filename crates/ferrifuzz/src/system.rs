//! The inference engine — type-1 Mamdani, Sugeno/TSK, and interval type-2 Mamdani.
//!
//! A [`FuzzySystem`] is checked and compiled the first time it runs (and again after
//! any change). Every name a rule or consequent uses is resolved then, so a typo is a
//! clear [`Error::InvalidModel`] instead of a silently wrong number; inference itself
//! works on indices only.

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

use crate::error::{did_you_mean, quoted_list, Error, Result};
use crate::hedge::Hedge;
use crate::it2;
use crate::operator::{TConorm, TNorm};
use crate::rule::{Clause, Rule};
use crate::variable::{Term, Variable};

/// How an output fuzzy set is turned into a crisp number (type-1 systems).
///
/// Names: `centroid` (`cog`, `coa`), `bisector`, `mom`, `som`, `lom`,
/// `weighted_average` (`sugeno`, `wtaver`).
///
/// The sampling-based methods evaluate the aggregated set on `resolution + 1` evenly
/// spaced points, so their result lies on that grid (e.g. a peak at 5 on `[0, 30]` with
/// 200 steps is reported as 4.95). Raise the resolution if that matters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Defuzz {
    /// Center of gravity of the aggregated set.
    #[default]
    Centroid,
    /// Point splitting the aggregated area in half.
    Bisector,
    /// Mean of the points at maximum membership.
    MeanOfMaximum,
    /// Smallest point at maximum membership.
    SmallestOfMaximum,
    /// Largest point at maximum membership.
    LargestOfMaximum,
    /// Sugeno/TSK: firing-weighted average of each rule's output `z` (no sampling).
    /// `z` is the consequent term's [`center`](crate::Type1MF::center) or a linear
    /// consequent registered with [`FuzzySystem::add_consequent`].
    WeightedAverage,
}

named_enum!(Defuzz, "defuzzification method", {
    Centroid => "centroid" | "cog" | "coa",
    Bisector => "bisector",
    MeanOfMaximum => "mom" | "mean_of_maximum",
    SmallestOfMaximum => "som" | "smallest_of_maximum",
    LargestOfMaximum => "lom" | "largest_of_maximum",
    WeightedAverage => "weighted_average" | "sugeno" | "wtaver",
});

/// How a firing strength shapes a consequent set (Mamdani systems).
///
/// Names: `mamdani` (`min`), `larsen` (`product`, `prod`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Implication {
    /// Mamdani: clip the consequent at the firing strength (`min`).
    #[default]
    Min,
    /// Larsen: scale the consequent by the firing strength (`product`).
    Product,
}

named_enum!(Implication, "implication", {
    Min => "mamdani" | "min",
    Product => "larsen" | "product" | "prod",
});

impl Implication {
    pub fn apply(self, mu: f64, strength: f64) -> f64 {
        match self {
            Implication::Min => mu.min(strength),
            Implication::Product => mu * strength,
        }
    }
}

/// Which engine runs.
///
/// Names: `auto`, `type1` (`t1`, `type-1`, `mamdani`, `sugeno`), `it2` (`type2`, `type-2`, `interval`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InferenceType {
    /// Interval type-2 when any input or output variable has an IT2 term, else type-1.
    #[default]
    Auto,
    /// Classic type-1 inference (Mamdani or Sugeno, chosen by [`Defuzz`]). Every term
    /// must be type-1.
    Type1,
    /// Interval type-2 Mamdani: interval firing strengths, an aggregated footprint of
    /// uncertainty, type reduction by [`TypeReduction`], midpoint as the crisp output.
    /// Type-1 terms are treated as IT2 terms with zero-width FOU.
    IntervalType2,
}

named_enum!(InferenceType, "inference type", {
    Auto => "auto",
    Type1 => "type1" | "t1" | "type-1" | "mamdani" | "sugeno",
    IntervalType2 => "it2" | "type2" | "type-2" | "interval",
});

/// IT2 type-reduction method (see [`crate::it2`]). Only used by interval type-2 systems.
///
/// Names: `km` (`karnik_mendel`), `cos` (`center_of_sets`), `height`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TypeReduction {
    /// Karnik–Mendel centroid of the sampled output footprint of uncertainty.
    #[default]
    KarnikMendel,
    /// Center-of-sets: KM over the consequent sets' centers weighted by firing intervals.
    /// Centers are the terms' [`center`](crate::Type1MF::center) (numerically integrated
    /// for hedged / negated consequents).
    CenterOfSets,
    /// Firing-weighted average of consequent centers, per interval bound.
    Height,
}

named_enum!(TypeReduction, "type reduction", {
    KarnikMendel => "km" | "karnik_mendel" | "karnik-mendel",
    CenterOfSets => "cos" | "center_of_sets" | "centerofsets",
    Height => "height",
});

/// First-order TSK consequent: `z = bias + Σ coeffs[var] · input[var]`.
#[derive(Clone, Debug, PartialEq)]
struct LinearConsequent {
    out_var: String,
    term: String,
    bias: f64,
    coeffs: Vec<(String, f64)>,
}

/// A fuzzy inference system.
///
/// ```
/// use ferrifuzz::prelude::*;
///
/// let temp = Variable::new("temp", 0.0, 40.0)?
///     .with_term("cold", mf::triangle(0.0, 0.0, 20.0)?)
///     .with_term("hot", mf::triangle(20.0, 40.0, 40.0)?);
/// let fan = Variable::new("fan", 0.0, 100.0)?
///     .with_term("slow", mf::triangle(0.0, 0.0, 50.0)?)
///     .with_term("fast", mf::triangle(50.0, 100.0, 100.0)?);
///
/// let mut sys = FuzzySystem::new();
/// sys.add_input(temp).add_output(fan);
/// sys.add_rules_str(["IF temp IS cold THEN fan IS slow", "IF temp IS hot THEN fan IS fast"])?;
///
/// let out = sys.compute([("temp", 35.0)])?;
/// assert!(out["fan"] > 50.0);
///
/// // Typos are errors, not silent zeros:
/// sys.add_rule_str("IF temp IS hott THEN fan IS fast")?;
/// let err = sys.compute([("temp", 35.0)]).unwrap_err().to_string();
/// assert!(err.contains("did you mean 'hot'"));
/// # Ok::<(), ferrifuzz::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct FuzzySystem {
    inputs: Vec<Variable>,
    outputs: Vec<Variable>,
    rules: Vec<Rule>,
    consequents: Vec<LinearConsequent>,
    and_method: TNorm,
    or_method: TConorm,
    defuzz: Defuzz,
    implication: Implication,
    inference: InferenceType,
    type_reduction: TypeReduction,
    resolution: usize,
    clamp_inputs: bool,
    /// Compiled model; reset by every mutation.
    compiled: OnceLock<Result<Compiled>>,
}

impl Default for FuzzySystem {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            rules: Vec::new(),
            consequents: Vec::new(),
            and_method: TNorm::Min,
            or_method: TConorm::Max,
            defuzz: Defuzz::Centroid,
            implication: Implication::Min,
            inference: InferenceType::Auto,
            type_reduction: TypeReduction::KarnikMendel,
            resolution: 200,
            clamp_inputs: true,
            compiled: OnceLock::new(),
        }
    }
}

macro_rules! setting {
    ($with:ident, $set:ident, $get:ident, $field:ident: $ty:ty, $doc:literal) => {
        #[doc = $doc]
        pub fn $with(mut self, value: $ty) -> Self {
            self.$set(value);
            self
        }

        #[doc = $doc]
        pub fn $set(&mut self, value: $ty) -> &mut Self {
            self.$field = value;
            self.invalidate();
            self
        }

        pub fn $get(&self) -> $ty {
            self.$field
        }
    };
}

/// Settings — builder (`with_*`), setter (`set_*`) and getter for each.
impl FuzzySystem {
    /// Mamdani defaults: min AND, max OR, min implication, centroid over 200 steps,
    /// inputs clamped to their variable's range.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shorthand for `FuzzySystem::new().with_defuzz(Defuzz::WeightedAverage)` — a
    /// Sugeno/TSK system. Nothing else differs.
    pub fn sugeno() -> Self {
        Self::default().with_defuzz(Defuzz::WeightedAverage)
    }

    setting!(with_and, set_and, and_method, and_method: TNorm, "T-norm combining the clauses of an AND-group.");
    setting!(with_or, set_or, or_method, or_method: TConorm, "T-conorm combining a rule's OR-groups.");
    setting!(with_defuzz, set_defuzz, defuzz, defuzz: Defuzz, "Defuzzification method (type-1 systems).");
    setting!(with_implication, set_implication, implication, implication: Implication, "Implication (Mamdani systems).");
    setting!(with_inference, set_inference, inference, inference: InferenceType, "Engine selection.");
    setting!(with_type_reduction, set_type_reduction, type_reduction, type_reduction: TypeReduction, "IT2 type reduction.");
    setting!(with_input_clamping, set_input_clamping, input_clamping, clamp_inputs: bool,
        "Clamp inputs into their variable's `[min, max]` before fuzzifying (default `true`), so values just outside the range still fire the edge terms.");

    /// Number of sampling steps over each output universe (at least 1).
    pub fn with_resolution(mut self, steps: usize) -> Self {
        self.set_resolution(steps);
        self
    }

    /// Number of sampling steps over each output universe (at least 1).
    pub fn set_resolution(&mut self, steps: usize) -> &mut Self {
        self.resolution = steps.max(1);
        self.invalidate();
        self
    }

    pub fn resolution(&self) -> usize {
        self.resolution
    }

    fn invalidate(&mut self) {
        self.compiled = OnceLock::new();
    }
}

/// Model building and inspection.
impl FuzzySystem {
    /// Register an input variable (replaces one with the same name). The variable is
    /// moved in; edit it later through [`FuzzySystem::input_mut`].
    pub fn add_input(&mut self, variable: Variable) -> &mut Self {
        upsert(&mut self.inputs, variable);
        self.invalidate();
        self
    }

    /// Register an output variable (replaces one with the same name).
    pub fn add_output(&mut self, variable: Variable) -> &mut Self {
        upsert(&mut self.outputs, variable);
        self.invalidate();
        self
    }

    pub fn add_rule(&mut self, rule: Rule) -> &mut Self {
        self.rules.push(rule);
        self.invalidate();
        self
    }

    /// Parse and add a rule string (see [`Rule::parse`] for the syntax).
    pub fn add_rule_str(&mut self, rule: &str) -> Result<&mut Self> {
        Ok(self.add_rule(Rule::parse(rule)?))
    }

    /// Parse and add several rule strings; nothing is added if any fails to parse.
    pub fn add_rules_str<S: AsRef<str>>(
        &mut self,
        rules: impl IntoIterator<Item = S>,
    ) -> Result<&mut Self> {
        let parsed = rules
            .into_iter()
            .map(|r| Rule::parse(r.as_ref()))
            .collect::<Result<Vec<_>>>()?;
        self.rules.extend(parsed);
        self.invalidate();
        Ok(self)
    }

    /// Register a first-order TSK consequent `z = bias + Σ coeffs·input` for the Sugeno
    /// output `out_var IS term`.
    ///
    /// Only valid with [`Defuzz::WeightedAverage`]. `term` does not need to exist in the
    /// output variable — rules can conclude any term that has a consequent, so each
    /// rule can carry its own linear function. Without one, a Sugeno rule outputs the
    /// term's MF center as a constant. Coefficient names must be input variables.
    pub fn add_consequent<K: Into<String>>(
        &mut self,
        out_var: impl Into<String>,
        term: impl Into<String>,
        coeffs: impl IntoIterator<Item = (K, f64)>,
        bias: f64,
    ) -> &mut Self {
        let (out_var, term) = (out_var.into(), term.into());
        let coeffs = coeffs.into_iter().map(|(k, c)| (k.into(), c)).collect();
        self.consequents
            .retain(|c| !(c.out_var == out_var && c.term == term));
        self.consequents.push(LinearConsequent {
            out_var,
            term,
            bias,
            coeffs,
        });
        self.invalidate();
        self
    }

    pub fn inputs(&self) -> &[Variable] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[Variable] {
        &self.outputs
    }

    pub fn input(&self, name: &str) -> Option<&Variable> {
        self.inputs.iter().find(|v| v.name() == name)
    }

    pub fn output(&self, name: &str) -> Option<&Variable> {
        self.outputs.iter().find(|v| v.name() == name)
    }

    /// Mutable access to a registered input (e.g. to change a term).
    pub fn input_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.invalidate();
        self.inputs.iter_mut().find(|v| v.name() == name)
    }

    /// Mutable access to a registered output.
    pub fn output_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.invalidate();
        self.outputs.iter_mut().find(|v| v.name() == name)
    }

    pub fn input_names(&self) -> Vec<&str> {
        self.inputs.iter().map(Variable::name).collect()
    }

    pub fn output_names(&self) -> Vec<&str> {
        self.outputs.iter().map(Variable::name).collect()
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Mutable access to the rule base (edit weights, remove rules, ...).
    pub fn rules_mut(&mut self) -> &mut Vec<Rule> {
        self.invalidate();
        &mut self.rules
    }

    /// True for interval type-2 inference (explicitly, or `Auto` with IT2 terms).
    pub fn is_it2(&self) -> bool {
        match self.inference {
            InferenceType::IntervalType2 => true,
            InferenceType::Type1 => false,
            InferenceType::Auto => self
                .inputs
                .iter()
                .chain(&self.outputs)
                .any(Variable::is_it2),
        }
    }

    /// Check the model without running it. `Err` describes every problem that would
    /// make [`FuzzySystem::compute`] fail.
    pub fn check(&self) -> Result<()> {
        self.compiled().map(|_| ())
    }

    /// Every problem found, errors first, then warnings (unused inputs, outputs no rule
    /// concludes, duplicate or conflicting rules, terms outside their range, ...).
    /// Empty when the model is clean. Rules are numbered from 1.
    pub fn validate(&self) -> Vec<String> {
        let mut issues: Vec<String> = match self.compiled() {
            Err(Error::InvalidModel(msg)) => msg
                .lines()
                .filter_map(|l| l.strip_prefix("  - "))
                .map(|l| format!("error: {}", l))
                .collect(),
            Err(e) => vec![format!("error: {}", e)],
            Ok(_) => Vec::new(),
        };
        issues.extend(
            self.warnings()
                .into_iter()
                .map(|w| format!("warning: {}", w)),
        );
        issues
    }

    fn warnings(&self) -> Vec<String> {
        let mut out = Vec::new();
        let is_sugeno = self.defuzz == Defuzz::WeightedAverage && !self.is_it2();
        let mut used_inputs = HashSet::new();
        let mut concluded = HashSet::new();
        for (i, rule) in self.rules.iter().enumerate() {
            for c in rule.antecedent.iter().flatten() {
                used_inputs.insert(c.var.as_str());
            }
            for c in &rule.consequent {
                concluded.insert(c.var.as_str());
                if is_sugeno && c.is_modified() {
                    out.push(format!(
                        "{}: Sugeno ignores NOT/hedges in the conclusion '{}'",
                        rule_label(i, rule),
                        c
                    ));
                }
            }
        }
        for v in &self.outputs {
            if !concluded.contains(v.name()) {
                out.push(format!(
                    "output variable '{}' is not concluded by any rule",
                    v.name()
                ));
            }
        }
        for v in &self.inputs {
            if !used_inputs.contains(v.name()) {
                out.push(format!(
                    "input variable '{}' is never used by any rule",
                    v.name()
                ));
            }
        }
        for v in self.inputs.iter().chain(&self.outputs) {
            for (name, term) in v.terms() {
                let n = 200;
                let upper = match term {
                    Term::Type1(mf) => *mf,
                    Term::IT2(mf) => mf.upper(),
                };
                let span = v.max_value() - v.min_value();
                let grid = (0..=n).map(|k| v.min_value() + span * k as f64 / n as f64);
                let keys = upper.key_points().into_iter().map(|x| v.clamp(x));
                let peak = grid
                    .chain(keys)
                    .map(|x| term.interval(x).1)
                    .fold(0.0, f64::max);
                if peak < 1e-6 {
                    out.push(format!(
                        "term '{}' of '{}' is zero everywhere in [{}, {}]",
                        name,
                        v.name(),
                        v.min_value(),
                        v.max_value()
                    ));
                }
            }
        }
        for lc in &self.consequents {
            let used = self.rules.iter().any(|r| {
                r.consequent
                    .iter()
                    .any(|c| c.var == lc.out_var && c.term == lc.term)
            });
            if !used {
                out.push(format!(
                    "consequent for '{} IS {}' is not used by any rule",
                    lc.out_var, lc.term
                ));
            }
        }
        for i in 0..self.rules.len() {
            for j in (i + 1)..self.rules.len() {
                let (a, b) = (&self.rules[i], &self.rules[j]);
                if a == b {
                    out.push(format!("rules #{} and #{} are duplicates", i + 1, j + 1));
                } else if a.antecedent == b.antecedent {
                    for ca in &a.consequent {
                        if let Some(cb) = b
                            .consequent
                            .iter()
                            .find(|cb| cb.var == ca.var && cb.term != ca.term)
                        {
                            out.push(format!(
                                "rules #{} and #{} have the same condition but conclude '{}' and '{}'",
                                i + 1,
                                j + 1,
                                ca,
                                cb
                            ));
                        }
                    }
                }
            }
        }
        out
    }
}

/// Inference.
impl FuzzySystem {
    /// Run inference; returns `{output: value}` for every output variable.
    ///
    /// Every input variable must be supplied exactly by name; unknown names, missing
    /// inputs and NaN are errors. Inputs are clamped to their range unless
    /// [`with_input_clamping(false)`](FuzzySystem::with_input_clamping). An output for
    /// which no rule fired is `NaN`.
    pub fn compute<I, K, V>(&self, inputs: I) -> Result<HashMap<String, f64>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        let values = self.bind_inputs(inputs)?;
        let c = self.compiled()?;
        Ok(self.name_outputs(self.infer(c, &values).into_iter().map(midpoint)))
    }

    /// Positional variant of [`FuzzySystem::compute`]: `inputs` in registration order,
    /// outputs in registration order. Handy in simulation loops.
    pub fn evaluate(&self, inputs: &[f64]) -> Result<Vec<f64>> {
        if inputs.len() != self.inputs.len() {
            return Err(Error::InvalidInput(format!(
                "expected {} inputs ({}), got {}",
                self.inputs.len(),
                quoted_list(self.input_names()),
                inputs.len()
            )));
        }
        let mut values = inputs.to_vec();
        for (i, v) in values.iter_mut().enumerate() {
            *v = self.prepare_value(i, *v)?;
        }
        let c = self.compiled()?;
        Ok(self.infer(c, &values).into_iter().map(midpoint).collect())
    }

    /// Like [`FuzzySystem::compute`] but returns the type-reduced interval `(y_l, y_r)`
    /// per output for IT2 systems (`(y, y)` for type-1). The crisp IT2 output is its midpoint.
    pub fn compute_interval<I, K, V>(&self, inputs: I) -> Result<HashMap<String, (f64, f64)>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        let values = self.bind_inputs(inputs)?;
        let c = self.compiled()?;
        Ok(self.name_outputs(self.infer(c, &values)))
    }

    /// Row-wise inference over equal-length columns, one per input variable; returns one
    /// column per output (`NaN` where no rule fired). Rows run in parallel with the
    /// `parallel` cargo feature.
    pub fn compute_batch<I, K, C>(&self, columns: I) -> Result<HashMap<String, Vec<f64>>>
    where
        I: IntoIterator<Item = (K, C)>,
        K: AsRef<str>,
        C: AsRef<[f64]>,
    {
        let columns: Vec<(K, C)> = columns.into_iter().collect();
        let mut bound: Vec<Option<&[f64]>> = vec![None; self.inputs.len()];
        let mut n: Option<usize> = None;
        for (name, col) in &columns {
            let i = self.input_index_checked(name.as_ref())?;
            let col = col.as_ref();
            match n {
                None => n = Some(col.len()),
                Some(expected) if expected != col.len() => {
                    return Err(Error::InvalidInput(format!(
                        "input column '{}' has length {}, expected {}",
                        name.as_ref(),
                        col.len(),
                        expected
                    )));
                }
                _ => {}
            }
            bound[i] = Some(col);
        }
        let cols: Vec<&[f64]> = bound
            .iter()
            .enumerate()
            .map(|(i, c)| c.ok_or_else(|| self.missing(i)))
            .collect::<Result<_>>()?;
        let n = n.unwrap_or(0);
        let c = self.compiled()?;
        let n_out = self.outputs.len();

        let row = |r: usize, out: &mut [f64]| -> Result<()> {
            let mut values = Vec::with_capacity(cols.len());
            for (i, col) in cols.iter().enumerate() {
                values.push(self.prepare_value(i, col[r])?);
            }
            for (o, y) in out.iter_mut().zip(self.infer(c, &values)) {
                *o = midpoint(y);
            }
            Ok(())
        };
        let mut flat = vec![f64::NAN; n * n_out];
        if n_out > 0 {
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                flat.par_chunks_mut(n_out)
                    .enumerate()
                    .try_for_each(|(r, out)| row(r, out))?;
            }
            #[cfg(not(feature = "parallel"))]
            for (r, out) in flat.chunks_mut(n_out).enumerate() {
                row(r, out)?;
            }
        }
        Ok(self
            .outputs
            .iter()
            .enumerate()
            .map(|(o, v)| {
                (
                    v.name().to_string(),
                    (0..n).map(|r| flat[r * n_out + o]).collect(),
                )
            })
            .collect())
    }

    /// Firing strength of every rule (after weights), in rule order — e.g. for
    /// ANFIS-style least-squares fitting. IT2 systems report the interval midpoint.
    pub fn rule_strengths<I, K, V>(&self, inputs: I) -> Result<Vec<f64>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        Ok(self
            .rule_strengths_interval(inputs)?
            .into_iter()
            .map(midpoint)
            .collect())
    }

    /// Firing interval `(lower, upper)` of every rule (`(s, s)` for type-1 systems).
    pub fn rule_strengths_interval<I, K, V>(&self, inputs: I) -> Result<Vec<(f64, f64)>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        let values = self.bind_inputs(inputs)?;
        let c = self.compiled()?;
        Ok(c.rules.iter().map(|r| self.fire(c, r, &values)).collect())
    }

    /// The aggregated output set of a type-1 Mamdani system on its sampling grid,
    /// `(xs, mus)` — what the defuzzifier sees. Useful for plots.
    pub fn output_set<I, K, V>(&self, inputs: I, output: &str) -> Result<(Vec<f64>, Vec<f64>)>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        let (xs, lower, upper) = self.output_fou(inputs, output)?;
        if self.is_it2() {
            return Err(Error::InvalidModel(
                "output_set is for type-1 systems; use output_fou for interval type-2".into(),
            ));
        }
        debug_assert_eq!(lower, upper);
        Ok((xs, upper))
    }

    /// The aggregated footprint of uncertainty `(xs, lower, upper)` of an output on its
    /// sampling grid (lower == upper for type-1 Mamdani systems).
    pub fn output_fou<I, K, V>(
        &self,
        inputs: I,
        output: &str,
    ) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>)>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        let values = self.bind_inputs(inputs)?;
        let c = self.compiled()?;
        if c.sugeno {
            return Err(Error::InvalidModel(
                "a Sugeno (weighted_average) system has no output fuzzy set to sample".into(),
            ));
        }
        let o = self
            .outputs
            .iter()
            .position(|v| v.name() == output)
            .ok_or_else(|| {
                Error::InvalidInput(format!(
                    "unknown output '{}'{}; outputs are: {}",
                    output,
                    did_you_mean(output, self.output_names()),
                    quoted_list(self.output_names())
                ))
            })?;
        let fired = self.fire_all(c, &values);
        let sets = active_sets(c, &fired, o);
        let (lower, upper) = self.sample_fou(c, o, &sets);
        Ok((c.grids[o].clone(), lower, upper))
    }

    fn input_index_checked(&self, name: &str) -> Result<usize> {
        self.inputs
            .iter()
            .position(|v| v.name() == name)
            .ok_or_else(|| {
                Error::UnknownInput(format!(
                    "unknown input '{}'{}; inputs are: {}",
                    name,
                    did_you_mean(name, self.input_names()),
                    quoted_list(self.input_names())
                ))
            })
    }

    fn missing(&self, i: usize) -> Error {
        Error::MissingInput(format!(
            "missing input '{}' (every input must be given: {})",
            self.inputs[i].name(),
            quoted_list(self.input_names())
        ))
    }

    fn prepare_value(&self, i: usize, x: f64) -> Result<f64> {
        if x.is_nan() {
            return Err(Error::InvalidInput(format!(
                "input '{}' is NaN",
                self.inputs[i].name()
            )));
        }
        Ok(if self.clamp_inputs {
            self.inputs[i].clamp(x)
        } else {
            x
        })
    }

    fn bind_inputs<I, K, V>(&self, inputs: I) -> Result<Vec<f64>>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Borrow<f64>,
    {
        let mut values: Vec<Option<f64>> = vec![None; self.inputs.len()];
        for (name, value) in inputs {
            let i = self.input_index_checked(name.as_ref())?;
            values[i] = Some(self.prepare_value(i, *value.borrow())?);
        }
        values
            .into_iter()
            .enumerate()
            .map(|(i, v)| v.ok_or_else(|| self.missing(i)))
            .collect()
    }

    fn name_outputs<T>(&self, values: impl IntoIterator<Item = T>) -> HashMap<String, T> {
        self.outputs
            .iter()
            .map(|v| v.name().to_string())
            .zip(values)
            .collect()
    }
}

impl fmt::Display for FuzzySystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_it2() {
            format!("interval type-2 (type_reduction={})", self.type_reduction)
        } else if self.defuzz == Defuzz::WeightedAverage {
            "Sugeno".to_string()
        } else {
            format!(
                "Mamdani (defuzz={}, implication={})",
                self.defuzz, self.implication
            )
        };
        write!(
            f,
            "FuzzySystem: {}; and={}, or={}; inputs [{}] → outputs [{}]; {} rules",
            kind,
            self.and_method,
            self.or_method,
            self.input_names().join(", "),
            self.output_names().join(", "),
            self.rules.len()
        )
    }
}

// ── Compilation ──────────────────────────────────────────────────────────────

/// Hedge / NOT applied to a term's membership.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Modifier {
    power: Option<f64>,
    negated: bool,
}

impl Modifier {
    fn of(c: &Clause) -> Self {
        Self {
            power: c.hedge().map(|h: Hedge| h.power()),
            negated: c.negated,
        }
    }

    fn is_identity(self) -> bool {
        self.power.is_none() && !self.negated
    }

    #[inline]
    fn apply(self, mu: f64) -> f64 {
        let mu = self.power.map_or(mu, |p| mu.powf(p));
        if self.negated {
            1.0 - mu
        } else {
            mu
        }
    }

    #[inline]
    fn apply_interval(self, (l, u): (f64, f64)) -> (f64, f64) {
        let (l, u) = self.power.map_or((l, u), |p| (l.powf(p), u.powf(p)));
        if self.negated {
            (1.0 - u, 1.0 - l)
        } else {
            (l, u)
        }
    }
}

#[derive(Clone, Debug)]
struct CClause {
    input: usize,
    term: usize,
    modifier: Modifier,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Target {
    /// Index of a term in the output variable.
    Term(usize),
    /// Index into `FuzzySystem::consequents`.
    Linear(usize),
}

#[derive(Clone, Debug)]
struct CConsequent {
    output: usize,
    target: Target,
    modifier: Modifier,
}

#[derive(Clone, Debug)]
struct CRule {
    groups: Vec<Vec<CClause>>,
    consequents: Vec<CConsequent>,
    weight: f64,
}

#[derive(Clone, Debug)]
struct Compiled {
    it2: bool,
    sugeno: bool,
    rules: Vec<CRule>,
    /// Linear consequents as `(bias, [(input, coeff)])`, indexed like `consequents`.
    linear: Vec<(f64, Vec<(usize, f64)>)>,
    /// Sampling grid and step per output.
    grids: Vec<Vec<f64>>,
    steps: Vec<f64>,
}

fn rule_label(i: usize, rule: &Rule) -> String {
    format!("rule #{} `{}`", i + 1, rule)
}

impl FuzzySystem {
    fn compiled(&self) -> Result<&Compiled> {
        self.compiled
            .get_or_init(|| self.compile())
            .as_ref()
            .map_err(Clone::clone)
    }

    fn compile(&self) -> Result<Compiled> {
        let mut errors: Vec<String> = Vec::new();
        let it2 = self.is_it2();
        let sugeno = self.defuzz == Defuzz::WeightedAverage;

        if self.inputs.is_empty() {
            errors.push("no input variables (use add_input)".into());
        }
        if self.outputs.is_empty() {
            errors.push("no output variables (use add_output)".into());
        }
        if self.rules.is_empty() {
            errors.push("no rules (use add_rule / add_rule_str)".into());
        }
        for v in &self.inputs {
            if self.output(v.name()).is_some() {
                errors.push(format!(
                    "'{}' is registered as both an input and an output",
                    v.name()
                ));
            }
        }
        if self.inference == InferenceType::Type1 {
            for v in self
                .inputs
                .iter()
                .chain(&self.outputs)
                .filter(|v| v.is_it2())
            {
                errors.push(format!(
                    "variable '{}' has interval type-2 terms but inference is type-1; \
                     use inference 'auto' or 'it2'",
                    v.name()
                ));
            }
        }
        if it2 && self.defuzz != Defuzz::Centroid {
            errors.push(format!(
                "interval type-2 systems are reduced with type_reduction (km, cos, height), \
                 not defuzz '{}'; leave defuzz at 'centroid'",
                self.defuzz
            ));
        }
        if !sugeno && !self.consequents.is_empty() {
            errors.push(
                "add_consequent (linear TSK consequents) needs a Sugeno system: \
                 defuzz 'weighted_average'"
                    .into(),
            );
        }

        let mut linear = Vec::with_capacity(self.consequents.len());
        for lc in &self.consequents {
            if self.output(&lc.out_var).is_none() {
                errors.push(format!(
                    "consequent for '{} IS {}': unknown output '{}'{}; outputs are: {}",
                    lc.out_var,
                    lc.term,
                    lc.out_var,
                    did_you_mean(&lc.out_var, self.output_names()),
                    quoted_list(self.output_names())
                ));
            }
            let mut coeffs = Vec::with_capacity(lc.coeffs.len());
            for (var, c) in &lc.coeffs {
                match self.inputs.iter().position(|v| v.name() == var) {
                    Some(i) => coeffs.push((i, *c)),
                    None => errors.push(format!(
                        "consequent for '{} IS {}': coefficient for unknown input '{}'{}; inputs are: {}",
                        lc.out_var,
                        lc.term,
                        var,
                        did_you_mean(var, self.input_names()),
                        quoted_list(self.input_names())
                    )),
                }
            }
            linear.push((lc.bias, coeffs));
        }

        let mut rules = Vec::with_capacity(self.rules.len());
        for (ri, rule) in self.rules.iter().enumerate() {
            let label = rule_label(ri, rule);
            if !(0.0..=1.0).contains(&rule.weight) {
                errors.push(format!(
                    "{}: weight {} is outside [0, 1]",
                    label, rule.weight
                ));
            }
            if rule.antecedent.is_empty() || rule.antecedent.iter().any(Vec::is_empty) {
                errors.push(format!("{}: empty condition", label));
            }
            if rule.consequent.is_empty() {
                errors.push(format!("{}: no conclusion", label));
            }

            let mut groups = Vec::with_capacity(rule.antecedent.len());
            for group in &rule.antecedent {
                let mut cg = Vec::with_capacity(group.len());
                for cl in group {
                    let Some(input) = self.inputs.iter().position(|v| v.name() == cl.var) else {
                        let hint = if self.output(&cl.var).is_some() {
                            " — it is an output; conditions (before THEN) must use inputs"
                                .to_string()
                        } else {
                            did_you_mean(&cl.var, self.input_names())
                        };
                        errors.push(format!(
                            "{}: unknown input variable '{}'{}; inputs are: {}",
                            label,
                            cl.var,
                            hint,
                            quoted_list(self.input_names())
                        ));
                        continue;
                    };
                    let var = &self.inputs[input];
                    let Some(term) = var.term_index(&cl.term) else {
                        errors.push(format!(
                            "{}: input '{}' has no term '{}'{}; its terms are: {}",
                            label,
                            cl.var,
                            cl.term,
                            did_you_mean(&cl.term, var.term_names()),
                            quoted_list(var.term_names())
                        ));
                        continue;
                    };
                    cg.push(CClause {
                        input,
                        term,
                        modifier: Modifier::of(cl),
                    });
                }
                groups.push(cg);
            }

            let mut consequents = Vec::with_capacity(rule.consequent.len());
            let mut seen = HashSet::new();
            for cl in &rule.consequent {
                if !seen.insert(cl.var.as_str()) {
                    errors.push(format!(
                        "{}: output '{}' is concluded twice in the same rule",
                        label, cl.var
                    ));
                }
                let Some(output) = self.outputs.iter().position(|v| v.name() == cl.var) else {
                    let hint = if self.input(&cl.var).is_some() {
                        " — it is an input; conclusions (after THEN) must use outputs".to_string()
                    } else {
                        did_you_mean(&cl.var, self.output_names())
                    };
                    errors.push(format!(
                        "{}: unknown output variable '{}'{}; outputs are: {}",
                        label,
                        cl.var,
                        hint,
                        quoted_list(self.output_names())
                    ));
                    continue;
                };
                let var = &self.outputs[output];
                let lin = self
                    .consequents
                    .iter()
                    .position(|c| c.out_var == cl.var && c.term == cl.term);
                let target = match (sugeno, lin, var.term_index(&cl.term)) {
                    (true, Some(k), _) => Target::Linear(k),
                    (true, None, Some(t)) if var.term_at(t).as_type1().is_none() && !it2 => {
                        errors.push(format!(
                            "{}: Sugeno output term '{}' of '{}' must be type-1",
                            label, cl.term, cl.var
                        ));
                        continue;
                    }
                    (_, _, Some(t)) => Target::Term(t),
                    (_, _, None) => {
                        let extra = if sugeno {
                            " (or register it with add_consequent)"
                        } else {
                            ""
                        };
                        errors.push(format!(
                            "{}: output '{}' has no term '{}'{}{}; its terms are: {}",
                            label,
                            cl.var,
                            cl.term,
                            did_you_mean(&cl.term, var.term_names()),
                            extra,
                            quoted_list(var.term_names())
                        ));
                        continue;
                    }
                };
                consequents.push(CConsequent {
                    output,
                    target,
                    modifier: Modifier::of(cl),
                });
            }
            rules.push(CRule {
                groups,
                consequents,
                weight: rule.weight,
            });
        }

        if !errors.is_empty() {
            let mut msg = format!(
                "invalid fuzzy system ({} problem{}):",
                errors.len(),
                if errors.len() == 1 { "" } else { "s" }
            );
            for e in &errors {
                msg.push_str("\n  - ");
                msg.push_str(e);
            }
            return Err(Error::InvalidModel(msg));
        }

        let (grids, steps) = self
            .outputs
            .iter()
            .map(|v| {
                let step = (v.max_value() - v.min_value()) / self.resolution as f64;
                let xs = (0..=self.resolution)
                    .map(|i| v.min_value() + i as f64 * step)
                    .collect();
                (xs, step)
            })
            .unzip();
        Ok(Compiled {
            it2,
            sugeno,
            rules,
            linear,
            grids,
            steps,
        })
    }
}

// ── Inference ────────────────────────────────────────────────────────────────

/// `(output, target, modifier)` of a fired consequent with its firing interval.
type Fired = Vec<(usize, Target, Modifier, (f64, f64))>;

fn midpoint((l, u): (f64, f64)) -> f64 {
    (l + u) / 2.0
}

/// Fired sets of one output: max firing per distinct (target, modifier).
fn active_sets(c: &Compiled, fired: &Fired, output: usize) -> Vec<(Target, Modifier, (f64, f64))> {
    let _ = c;
    let mut sets: Vec<(Target, Modifier, (f64, f64))> = Vec::new();
    for &(o, t, m, (l, u)) in fired {
        if o != output {
            continue;
        }
        match sets.iter_mut().find(|(t2, m2, _)| *t2 == t && *m2 == m) {
            Some((_, _, f)) => *f = (f.0.max(l), f.1.max(u)),
            None => sets.push((t, m, (l, u))),
        }
    }
    sets
}

impl FuzzySystem {
    /// Firing interval of one compiled rule (degenerate for type-1 terms).
    fn fire(&self, c: &Compiled, rule: &CRule, values: &[f64]) -> (f64, f64) {
        let and = self.and_method;
        let or = self.or_method;
        let mut strength: Option<(f64, f64)> = None;
        for group in &rule.groups {
            let mut g = (1.0, 1.0);
            for cl in group {
                let x = values[cl.input];
                let iv = match self.inputs[cl.input].term_at(cl.term) {
                    Term::Type1(mf) if !c.it2 => {
                        let mu = cl.modifier.apply(mf.membership(x));
                        (mu, mu)
                    }
                    t => cl.modifier.apply_interval(t.interval(x)),
                };
                g = and.apply_interval(g, iv);
            }
            strength = Some(match strength {
                None => g,
                Some(s) => or.apply_interval(s, g),
            });
        }
        let (l, u) = strength.unwrap_or((0.0, 0.0));
        (
            (l * rule.weight).clamp(0.0, 1.0),
            (u * rule.weight).clamp(0.0, 1.0),
        )
    }

    fn fire_all(&self, c: &Compiled, values: &[f64]) -> Fired {
        let mut fired = Vec::new();
        for rule in &c.rules {
            let f = self.fire(c, rule, values);
            if f.1 <= 0.0 {
                continue;
            }
            for cons in &rule.consequents {
                fired.push((cons.output, cons.target, cons.modifier, f));
            }
        }
        fired
    }

    /// One `(lower, upper)` result per output; NaN when nothing fired.
    fn infer(&self, c: &Compiled, values: &[f64]) -> Vec<(f64, f64)> {
        let fired = self.fire_all(c, values);
        let nan = (f64::NAN, f64::NAN);

        if c.sugeno && !c.it2 {
            let mut num = vec![0.0; self.outputs.len()];
            let mut den = vec![0.0; self.outputs.len()];
            for &(o, target, _, (s, _)) in &fired {
                let z = match target {
                    Target::Linear(k) => {
                        let (bias, coeffs) = &c.linear[k];
                        bias + coeffs.iter().map(|&(i, w)| w * values[i]).sum::<f64>()
                    }
                    Target::Term(t) => self.outputs[o].term_at(t).center(),
                };
                num[o] += s * z;
                den[o] += s;
            }
            return (0..self.outputs.len())
                .map(|o| {
                    if den[o] > 0.0 {
                        (num[o] / den[o], num[o] / den[o])
                    } else {
                        nan
                    }
                })
                .collect();
        }

        (0..self.outputs.len())
            .map(|o| {
                let sets = active_sets(c, &fired, o);
                if sets.is_empty() {
                    return nan;
                }
                let var = &self.outputs[o];
                if !c.it2 {
                    let (_, mus) = self.sample_fou(c, o, &sets);
                    let y = Sampled {
                        xs: &c.grids[o],
                        mus: &mus,
                        step: c.steps[o],
                    }
                    .defuzzify(self.defuzz, var);
                    return (y, y);
                }
                let reduced = match self.type_reduction {
                    TypeReduction::KarnikMendel => {
                        let (lower, upper) = self.sample_fou(c, o, &sets);
                        it2::km_type_reduce(&c.grids[o], &lower, &upper)
                    }
                    TypeReduction::CenterOfSets | TypeReduction::Height => {
                        let mut centers = Vec::with_capacity(sets.len());
                        let mut f_lower = Vec::with_capacity(sets.len());
                        let mut f_upper = Vec::with_capacity(sets.len());
                        for &(t, m, (fl, fu)) in &sets {
                            if let Some(center) = self.consequent_center(c, o, t, m) {
                                centers.push(center);
                                f_lower.push(fl);
                                f_upper.push(fu);
                            }
                        }
                        if self.type_reduction == TypeReduction::CenterOfSets {
                            it2::center_of_sets(&centers, &f_lower, &f_upper)
                        } else {
                            it2::height(&centers, &f_lower, &f_upper)
                        }
                    }
                };
                reduced.unwrap_or(nan)
            })
            .collect()
    }

    /// Aggregated (lower, upper) membership of an output on its grid.
    fn sample_fou(
        &self,
        c: &Compiled,
        o: usize,
        sets: &[(Target, Modifier, (f64, f64))],
    ) -> (Vec<f64>, Vec<f64>) {
        let var = &self.outputs[o];
        let xs = &c.grids[o];
        let mut lower = vec![0.0; xs.len()];
        let mut upper = vec![0.0; xs.len()];
        for &(target, m, (fl, fu)) in sets {
            let Target::Term(t) = target else { continue };
            let term = var.term_at(t);
            for (i, &x) in xs.iter().enumerate() {
                let (ml, mu) = m.apply_interval(term.interval(x));
                lower[i] = f64::max(lower[i], self.implication.apply(ml, fl));
                upper[i] = f64::max(upper[i], self.implication.apply(mu, fu));
            }
        }
        (lower, upper)
    }

    /// Center of a consequent set; hedged/negated sets are integrated numerically.
    fn consequent_center(&self, c: &Compiled, o: usize, t: Target, m: Modifier) -> Option<f64> {
        let Target::Term(t) = t else { return None };
        let term = self.outputs[o].term_at(t);
        if m.is_identity() {
            return Some(term.center());
        }
        let (mut num, mut den) = (0.0, 0.0);
        for &x in &c.grids[o] {
            let mid = midpoint(m.apply_interval(term.interval(x)));
            num += x * mid;
            den += mid;
        }
        (den > 0.0).then(|| num / den)
    }
}

fn upsert(vars: &mut Vec<Variable>, variable: Variable) {
    match vars.iter_mut().find(|v| v.name() == variable.name()) {
        Some(slot) => *slot = variable,
        None => vars.push(variable),
    }
}

/// The aggregated output MF sampled on the grid; each defuzzifier reduces over it.
struct Sampled<'a> {
    xs: &'a [f64],
    mus: &'a [f64],
    step: f64,
}

impl Sampled<'_> {
    fn defuzzify(&self, method: Defuzz, var: &Variable) -> f64 {
        match method {
            Defuzz::Bisector => self.bisector(var),
            Defuzz::MeanOfMaximum => self.mom(var),
            Defuzz::SmallestOfMaximum => self.som(var),
            Defuzz::LargestOfMaximum => self.lom(var),
            Defuzz::Centroid | Defuzz::WeightedAverage => self.centroid(var),
        }
    }

    fn midpoint(var: &Variable) -> f64 {
        (var.min_value() + var.max_value()) / 2.0
    }

    fn peak(&self) -> f64 {
        self.mus.iter().copied().fold(0.0, f64::max)
    }

    fn at_peak(&self) -> impl DoubleEndedIterator<Item = f64> + '_ {
        let peak = self.peak();
        self.xs
            .iter()
            .zip(self.mus)
            .filter(move |(_, &mu)| (mu - peak).abs() < 1e-9)
            .map(|(&x, _)| x)
    }

    fn centroid(&self, var: &Variable) -> f64 {
        let (mut num, mut den) = (0.0, 0.0);
        for (&x, &mu) in self.xs.iter().zip(self.mus) {
            num += x * mu;
            den += mu;
        }
        if den > 0.0 {
            num / den
        } else {
            Self::midpoint(var)
        }
    }

    fn bisector(&self, var: &Variable) -> f64 {
        let half = self.mus.iter().sum::<f64>() * self.step / 2.0;
        let mut cum = 0.0;
        for (&x, &mu) in self.xs.iter().zip(self.mus) {
            cum += mu * self.step;
            if cum >= half {
                return x;
            }
        }
        Self::midpoint(var)
    }

    fn mom(&self, var: &Variable) -> f64 {
        if self.peak() == 0.0 {
            return Self::midpoint(var);
        }
        let (sum, count) = self
            .at_peak()
            .fold((0.0, 0usize), |(s, c), x| (s + x, c + 1));
        if count > 0 {
            sum / count as f64
        } else {
            Self::midpoint(var)
        }
    }

    fn som(&self, var: &Variable) -> f64 {
        if self.peak() == 0.0 {
            return var.min_value();
        }
        self.at_peak().next().unwrap_or(var.min_value())
    }

    fn lom(&self, var: &Variable) -> f64 {
        if self.peak() == 0.0 {
            return var.max_value();
        }
        self.at_peak().next_back().unwrap_or(var.max_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mf;

    fn tri(a: f64, b: f64, c: f64) -> crate::Type1MF {
        mf::triangle(a, b, c).unwrap()
    }

    fn one_input_system(sys: FuzzySystem) -> FuzzySystem {
        let x = Variable::new("x", 0.0, 10.0)
            .unwrap()
            .with_term("low", tri(0.0, 0.0, 10.0))
            .with_term("high", tri(0.0, 10.0, 10.0));
        let y = Variable::new("y", 0.0, 10.0)
            .unwrap()
            .with_term("low", tri(0.0, 0.0, 10.0))
            .with_term("high", tri(0.0, 10.0, 10.0));
        let mut sys = sys;
        sys.add_input(x).add_output(y);
        sys.add_rules_str(["IF x IS low THEN y IS low", "IF x IS high THEN y IS high"])
            .unwrap();
        sys
    }

    fn y(sys: &FuzzySystem, x: f64) -> f64 {
        sys.compute([("x", x)]).unwrap()["y"]
    }

    #[test]
    fn symmetric_system_is_identity_at_the_middle_and_monotone() {
        let sys = one_input_system(FuzzySystem::new());
        assert!((y(&sys, 5.0) - 5.0).abs() < 1e-9);
        let mut last = f64::NEG_INFINITY;
        for i in 0..=10 {
            let v = y(&sys, i as f64);
            assert!(v >= last - 1e-12);
            last = v;
        }
    }

    #[test]
    fn out_of_range_inputs_are_clamped() {
        let sys = one_input_system(FuzzySystem::new());
        assert_eq!(y(&sys, -5.0), y(&sys, 0.0));
        assert_eq!(y(&sys, 50.0), y(&sys, 10.0));
        let raw = one_input_system(FuzzySystem::new().with_input_clamping(false));
        assert!(y(&raw, -5.0).is_nan());
    }

    #[test]
    fn sugeno_uses_centers_and_linear_consequents() {
        let mut sys = one_input_system(FuzzySystem::sugeno());
        assert!((y(&sys, 3.0) - 3.0).abs() < 1e-12);
        sys.add_consequent("y", "high", [("x", 2.0)], 1.0);
        // z_high = 1 + 2·3 = 7 → 0.7·0 + 0.3·7.
        assert!((y(&sys, 3.0) - 2.1).abs() < 1e-12);
        // A rule can conclude a term that only exists as a linear consequent.
        sys.add_consequent("y", "r3", [("x", 0.0)], 4.0);
        sys.add_rule_str("IF x IS low THEN y IS r3").unwrap();
        assert!(sys.check().is_ok());
    }

    #[test]
    fn input_errors() {
        let sys = one_input_system(FuzzySystem::new());
        let e = sys.compute(Vec::<(&str, f64)>::new()).unwrap_err();
        assert!(matches!(e, Error::MissingInput(_)), "{}", e);
        let e = sys.compute([("X", 1.0)]).unwrap_err();
        assert!(matches!(e, Error::UnknownInput(_)));
        assert!(e.to_string().contains("did you mean 'x'"), "{}", e);
        assert!(matches!(
            sys.compute([("x", f64::NAN)]),
            Err(Error::InvalidInput(_))
        ));
        assert!(sys.evaluate(&[1.0, 2.0]).is_err());
        assert_eq!(sys.evaluate(&[5.0]).unwrap()[0], y(&sys, 5.0));
    }

    #[test]
    fn model_errors_name_the_problem() {
        let mut sys = one_input_system(FuzzySystem::new());
        sys.add_rule_str("IF x IS hgih THEN y IS lwo WITH 0.5")
            .unwrap();
        sys.add_rule_str("IF z IS low THEN x IS low").unwrap();
        let e = sys.compute([("x", 1.0)]).unwrap_err().to_string();
        assert!(e.contains("4 problems"), "{}", e);
        assert!(e.contains("no term 'hgih' (did you mean 'high'?)"), "{}", e);
        assert!(e.contains("no term 'lwo' (did you mean 'low'?)"), "{}", e);
        assert!(e.contains("unknown input variable 'z'"), "{}", e);
        assert!(e.contains("it is an input"), "{}", e);
        assert!(e.contains("rule #3"), "{}", e);

        let mut sys = one_input_system(FuzzySystem::new());
        sys.add_consequent("y", "high", [("x", 1.0)], 0.0);
        assert!(sys
            .check()
            .unwrap_err()
            .to_string()
            .contains("needs a Sugeno system"));

        let mut sys = one_input_system(FuzzySystem::sugeno());
        sys.add_consequent("y", "high", [("xx", 1.0)], 0.0);
        assert!(sys
            .check()
            .unwrap_err()
            .to_string()
            .contains("did you mean 'x'"));

        let mut sys = one_input_system(FuzzySystem::new());
        sys.rules_mut()[0].weight = 1.5;
        assert!(sys
            .check()
            .unwrap_err()
            .to_string()
            .contains("outside [0, 1]"));
    }

    #[test]
    fn it2_requires_compatible_settings() {
        let fou = mf::it2(tri(0.0, 0.0, 10.0), tri(0.0, 0.0, 8.0)).unwrap();
        let mut sys = one_input_system(FuzzySystem::new().with_inference(InferenceType::Type1));
        sys.input_mut("x").unwrap().add_term("low", fou);
        assert!(sys.check().unwrap_err().to_string().contains("type-1"));
        sys.set_inference(InferenceType::Auto)
            .set_defuzz(Defuzz::Bisector);
        assert!(sys
            .check()
            .unwrap_err()
            .to_string()
            .contains("type_reduction"));
        sys.set_defuzz(Defuzz::Centroid);
        assert!(sys.is_it2());
        let (l, u) = sys.compute_interval([("x", 3.0)]).unwrap()["y"];
        assert!(l <= u);
        assert!((y(&sys, 3.0) - (l + u) / 2.0).abs() < 1e-12);
    }

    #[test]
    fn every_output_is_reported_nan_when_unfired() {
        let mut sys = one_input_system(FuzzySystem::new());
        sys.add_output(
            Variable::new("z", 0.0, 1.0)
                .unwrap()
                .with_term("t", tri(0.0, 0.5, 1.0)),
        );
        sys.add_rule_str("IF x IS NOT low THEN z IS t").unwrap();
        let out = sys.compute([("x", 0.0)]).unwrap();
        assert!(out["z"].is_nan());
        assert!(!out["y"].is_nan());
        let batch = sys.compute_batch([("x", [0.0, 5.0])]).unwrap();
        assert!(batch["z"][0].is_nan() && !batch["z"][1].is_nan());
    }

    #[test]
    fn batch_matches_single_rows() {
        let sys = one_input_system(FuzzySystem::new());
        let xs: Vec<f64> = (0..=20).map(|i| i as f64 / 2.0).collect();
        let out = sys.compute_batch([("x", &xs)]).unwrap();
        for (i, &x) in xs.iter().enumerate() {
            assert_eq!(out["y"][i], y(&sys, x));
        }
        let bad = sys.compute_batch([("x", vec![1.0, 2.0]), ("z", vec![1.0])]);
        assert!(matches!(bad, Err(Error::UnknownInput(_))));
        let missing = sys.compute_batch(Vec::<(&str, Vec<f64>)>::new());
        assert!(matches!(missing, Err(Error::MissingInput(_))));
    }

    #[test]
    fn it2_with_degenerate_fou_matches_type1() {
        let t1 = one_input_system(FuzzySystem::new());
        let it2 = one_input_system(FuzzySystem::new().with_inference(InferenceType::IntervalType2));
        for x in [1.0, 3.0, 5.0, 8.0] {
            assert!((y(&t1, x) - y(&it2, x)).abs() < 1e-6);
        }
    }

    #[test]
    fn introspection() {
        let sys = one_input_system(FuzzySystem::new());
        assert_eq!(sys.rule_strengths([("x", 3.0)]).unwrap(), [0.7, 0.3]);
        let (xs, mus) = sys.output_set([("x", 3.0)], "y").unwrap();
        assert_eq!(xs.len(), 201);
        assert!(mus.iter().all(|m| (0.0..=0.7).contains(m)));
        assert!(sys
            .output_set([("x", 3.0)], "yy")
            .unwrap_err()
            .to_string()
            .contains("did you mean 'y'"));
        assert!(one_input_system(FuzzySystem::sugeno())
            .output_set([("x", 1.0)], "y")
            .is_err());
        assert!(sys.to_string().contains("Mamdani"));
    }

    #[test]
    fn validate_reports_errors_and_warnings() {
        let mut sys = one_input_system(FuzzySystem::new());
        assert!(sys.validate().is_empty());
        sys.add_rule_str("IF x IS low THEN y IS high").unwrap();
        sys.add_rule_str("IF x IS low THEN y IS low").unwrap();
        sys.add_rule_str("IF x IS medium THEN y IS low").unwrap();
        let issues = sys.validate().join("\n");
        assert!(issues.contains("error: rule #5"), "{}", issues);
        assert!(
            issues.contains("warning: rules #1 and #4 are duplicates"),
            "{}",
            issues
        );
        assert!(
            issues.contains("conclude 'y IS low' and 'y IS high'"),
            "{}",
            issues
        );
    }

    #[test]
    fn singleton_outputs_are_not_reported_as_empty() {
        let mut sys = one_input_system(FuzzySystem::sugeno());
        sys.output_mut("y")
            .unwrap()
            .add_term("low", mf::singleton(6.0));
        assert!(
            sys.validate()
                .iter()
                .all(|w| !w.contains("zero everywhere")),
            "{:?}",
            sys.validate()
        );
        sys.output_mut("y")
            .unwrap()
            .add_term("far", tri(50.0, 60.0, 70.0));
        assert!(sys
            .validate()
            .iter()
            .any(|w| w.contains("'far' of 'y' is zero everywhere")));
    }

    #[test]
    fn settings_invalidate_the_cache() {
        let mut sys = one_input_system(FuzzySystem::new());
        let a = y(&sys, 3.0);
        sys.set_defuzz(Defuzz::MeanOfMaximum);
        assert_ne!(a, y(&sys, 3.0));
        assert_eq!(Defuzz::ALL.len(), 6);
        assert_eq!("LOM".parse::<Defuzz>().unwrap(), Defuzz::LargestOfMaximum);
        assert!("centriod"
            .parse::<Defuzz>()
            .unwrap_err()
            .to_string()
            .contains("did you mean 'centroid'"));
    }
}
