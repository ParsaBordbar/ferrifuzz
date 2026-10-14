//! Fuzzy IF-THEN rules and the rule-string parser.

use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::hedge::Hedge;

/// One clause: `var IS [NOT] [hedge] term`.
#[derive(Clone, Debug, PartialEq)]
pub struct Clause {
    pub var: String,
    pub term: String,
    /// True for `var IS NOT term`.
    pub negated: bool,
    /// Hedge word applied to the term, e.g. `"very"`.
    pub hedge: Option<String>,
}

impl Clause {
    /// Plain `var IS term`.
    pub fn new(var: impl Into<String>, term: impl Into<String>) -> Self {
        Self {
            var: var.into(),
            term: term.into(),
            negated: false,
            hedge: None,
        }
    }

    /// Negate the clause (`var IS NOT term`).
    pub fn negate(mut self) -> Self {
        self.negated = true;
        self
    }

    /// Apply a hedge word (`very`, `somewhat`, ...); errors on an unknown word.
    pub fn with_hedge(mut self, word: &str) -> Result<Self> {
        if Hedge::from_word(word).is_none() {
            return Err(Error::InvalidParameter(format!(
                "unknown hedge '{}' (try very/extremely/somewhat/slightly/plus/minus)",
                word
            )));
        }
        self.hedge = Some(word.to_lowercase());
        Ok(self)
    }

    pub fn hedge(&self) -> Option<Hedge> {
        self.hedge.as_deref().and_then(Hedge::from_word)
    }

    /// Hedge then negate a raw membership degree.
    pub fn modify(&self, mu: f64) -> f64 {
        let mu = self.hedge().map_or(mu, |h| h.apply(mu));
        if self.negated {
            1.0 - mu
        } else {
            mu
        }
    }

    /// Interval form of [`Clause::modify`]; negation swaps the bounds.
    pub fn modify_interval(&self, iv: (f64, f64)) -> (f64, f64) {
        let (l, u) = self.hedge().map_or(iv, |h| h.apply_interval(iv));
        if self.negated {
            (1.0 - u, 1.0 - l)
        } else {
            (l, u)
        }
    }

    /// True when NOT or a hedge reshapes the term.
    pub fn is_modified(&self) -> bool {
        self.negated || self.hedge.is_some()
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} IS ", self.var)?;
        if self.negated {
            f.write_str("NOT ")?;
        }
        if let Some(h) = &self.hedge {
            write!(f, "{} ", h)?;
        }
        f.write_str(&self.term)
    }
}

impl From<(&str, &str)> for Clause {
    fn from((var, term): (&str, &str)) -> Self {
        Clause::new(var, term)
    }
}

impl From<(String, String)> for Clause {
    fn from((var, term): (String, String)) -> Self {
        Clause::new(var, term)
    }
}

/// A fuzzy rule. The antecedent is in DNF — an OR of AND-groups.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub antecedent: Vec<Vec<Clause>>,
    /// A rule may drive several output variables at once.
    pub consequent: Vec<Clause>,
    /// Firing-strength multiplier (default 1.0).
    pub weight: f64,
}

impl Rule {
    /// Build from an explicit DNF antecedent and one or more consequents.
    pub fn new(antecedent: Vec<Vec<Clause>>, consequent: Vec<Clause>) -> Result<Self> {
        if consequent.is_empty() {
            return Err(Error::InvalidParameter(
                "a rule needs at least one consequent".into(),
            ));
        }
        if antecedent.is_empty() || antecedent.iter().any(Vec::is_empty) {
            return Err(Error::InvalidParameter(
                "a rule needs at least one condition".into(),
            ));
        }
        Ok(Self {
            antecedent,
            consequent,
            weight: 1.0,
        })
    }

    /// `IF c1 AND c2 ... THEN conclusion` from `(var, term)` pairs.
    pub fn when<C: Into<Clause>>(
        conditions: impl IntoIterator<Item = C>,
        conclusion: impl Into<Clause>,
    ) -> Result<Self> {
        Self::new(
            vec![conditions.into_iter().map(Into::into).collect()],
            vec![conclusion.into()],
        )
    }

    /// Parse a rule string.
    ///
    /// ```text
    /// IF <condition> THEN <clause> [AND <clause> ...] [WITH <weight>]
    /// condition := <clause> | <condition> AND <condition> | <condition> OR <condition> | ( <condition> )
    /// clause    := <var> IS [NOT] [<hedge>] <term>
    /// ```
    ///
    /// Keywords are case-insensitive; variable and term names are single words and
    /// case-sensitive. AND binds tighter than OR. Parentheses are expanded into an OR of
    /// AND-groups, which is exact for min/max operators. Hedges: `very`, `extremely`,
    /// `somewhat`, `more_or_less`, `slightly`, `plus`, `minus`. The weight must be in `[0, 1]`.
    ///
    /// ```
    /// use ferrifuzz::Rule;
    /// let r = Rule::parse("if (temp is hot or hum is high) and wind is NOT strong then fan is fast with 0.8")?;
    /// assert_eq!(r.antecedent.len(), 2);
    /// assert_eq!(r.weight, 0.8);
    /// # Ok::<(), ferrifuzz::Error>(())
    /// ```
    pub fn parse(s: &str) -> Result<Self> {
        Parser::new(s).rule()
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

impl FromStr for Rule {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Rule::parse(s)
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let join = |clauses: &[Clause]| {
            clauses
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" AND ")
        };
        let groups: Vec<String> = self.antecedent.iter().map(|g| join(g)).collect();
        write!(
            f,
            "IF {} THEN {}",
            groups.join(" OR "),
            join(&self.consequent)
        )?;
        if self.weight != 1.0 {
            write!(f, " WITH {}", self.weight)?;
        }
        Ok(())
    }
}

const KEYWORDS: [&str; 7] = ["IF", "THEN", "AND", "OR", "IS", "NOT", "WITH"];
const HEDGES: &str = "very, extremely, somewhat, more_or_less, slightly, plus, minus";
/// Guard against `(a OR b) AND (c OR d) AND ...` expanding into an enormous DNF.
const MAX_GROUPS: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Word(String),
    Open,
    Close,
    End,
}

/// Recursive-descent parser over `(token, byte offset)` pairs.
struct Parser<'a> {
    src: &'a str,
    toks: Vec<(Tok, usize)>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        let mut toks = Vec::new();
        let mut word_start: Option<usize> = None;
        for (i, ch) in src.char_indices() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                if let Some(st) = word_start.take() {
                    toks.push((Tok::Word(src[st..i].to_string()), st));
                }
                if ch == '(' {
                    toks.push((Tok::Open, i));
                } else if ch == ')' {
                    toks.push((Tok::Close, i));
                }
            } else if word_start.is_none() {
                word_start = Some(i);
            }
        }
        if let Some(st) = word_start {
            toks.push((Tok::Word(src[st..].to_string()), st));
        }
        toks.push((Tok::End, src.len()));
        Self { src, toks, pos: 0 }
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].0
    }

    fn peek_at(&self, ahead: usize) -> &Tok {
        &self.toks[(self.pos + ahead).min(self.toks.len() - 1)].0
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.pos].0.clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn is_kw(tok: &Tok, kw: &str) -> bool {
        matches!(tok, Tok::Word(w) if w.eq_ignore_ascii_case(kw))
    }

    fn is_name(tok: &Tok) -> bool {
        matches!(tok, Tok::Word(w) if !KEYWORDS.iter().any(|k| w.eq_ignore_ascii_case(k)))
    }

    fn describe(tok: &Tok) -> String {
        match tok {
            Tok::Word(w) => format!("'{}'", w),
            Tok::Open => "'('".into(),
            Tok::Close => "')'".into(),
            Tok::End => "end of rule".into(),
        }
    }

    /// Parse error pointing at the current token with a caret under the rule text.
    fn error(&self, msg: impl Into<String>) -> Error {
        let at = self.toks[self.pos].1;
        let caret = " ".repeat(self.src[..at].chars().count());
        Error::Parse(format!("{}\n  {}\n  {}^", msg.into(), self.src, caret))
    }

    fn expect_kw(&mut self, kw: &str, context: &str) -> Result<()> {
        if Self::is_kw(self.peek(), kw) {
            self.bump();
            Ok(())
        } else {
            Err(self.error(format!(
                "expected {} {}, found {}",
                kw,
                context,
                Self::describe(self.peek())
            )))
        }
    }

    fn rule(mut self) -> Result<Rule> {
        if !Self::is_kw(self.peek(), "IF") {
            return Err(self.error(
                "a rule must start with IF, e.g. 'IF temp IS hot AND hum IS high THEN fan IS fast'",
            ));
        }
        self.bump();
        let antecedent = self.or_expr()?;
        self.expect_kw("THEN", "after the condition")?;

        let mut consequent = vec![self.clause()?];
        loop {
            match self.peek() {
                t if Self::is_kw(t, "AND") => {
                    self.bump();
                    consequent.push(self.clause()?);
                }
                t if Self::is_kw(t, "OR") => {
                    return Err(self
                        .error("OR is not allowed after THEN; combine several outputs with AND"))
                }
                _ => break,
            }
        }

        let mut weight = 1.0;
        if Self::is_kw(self.peek(), "WITH") {
            self.bump();
            let w = match self.peek() {
                Tok::Word(w) => w.parse::<f64>().ok(),
                _ => None,
            };
            match w {
                Some(w) if (0.0..=1.0).contains(&w) => {
                    weight = w;
                    self.bump();
                }
                _ => return Err(self.error("WITH must be followed by a weight between 0 and 1")),
            }
        }
        if *self.peek() != Tok::End {
            return Err(self.error(format!(
                "unexpected {} after the conclusion",
                Self::describe(self.peek())
            )));
        }
        Ok(Rule {
            antecedent,
            consequent,
            weight,
        })
    }

    /// `and_expr (OR and_expr)*` — union of the AND-groups.
    fn or_expr(&mut self) -> Result<Vec<Vec<Clause>>> {
        let mut groups = self.and_expr()?;
        while Self::is_kw(self.peek(), "OR") {
            self.bump();
            groups.extend(self.and_expr()?);
        }
        Ok(groups)
    }

    /// `factor (AND factor)*` — cross product distributes AND over OR.
    fn and_expr(&mut self) -> Result<Vec<Vec<Clause>>> {
        let mut groups = self.factor()?;
        while Self::is_kw(self.peek(), "AND") {
            self.bump();
            let rhs = self.factor()?;
            if groups.len() * rhs.len() > MAX_GROUPS {
                return Err(self.error("condition too large once parentheses are expanded"));
            }
            groups = groups
                .iter()
                .flat_map(|g| {
                    rhs.iter()
                        .map(move |h| g.iter().chain(h).cloned().collect())
                })
                .collect();
        }
        Ok(groups)
    }

    fn factor(&mut self) -> Result<Vec<Vec<Clause>>> {
        if *self.peek() == Tok::Open {
            self.bump();
            let inner = self.or_expr()?;
            if *self.peek() != Tok::Close {
                return Err(self.error(format!(
                    "expected ')' to close the parenthesis, found {}",
                    Self::describe(self.peek())
                )));
            }
            self.bump();
            return Ok(inner);
        }
        Ok(vec![vec![self.clause()?]])
    }

    /// `var IS [NOT] [hedge] term`.
    fn clause(&mut self) -> Result<Clause> {
        let var = match self.peek() {
            t if Self::is_name(t) => match self.bump() {
                Tok::Word(w) => w,
                _ => unreachable!(),
            },
            t => {
                return Err(self.error(format!(
                    "expected a variable name, found {}",
                    Self::describe(t)
                )))
            }
        };
        self.expect_kw("IS", &format!("after variable '{}'", var))?;

        let mut negated = false;
        if Self::is_kw(self.peek(), "NOT") {
            self.bump();
            negated = true;
        }

        // A hedge word only counts as a hedge when a term follows it, so a term may
        // itself be called e.g. "plus".
        let mut hedge = None;
        if let Tok::Word(w) = self.peek() {
            if Hedge::from_word(w).is_some() && Self::is_name(self.peek_at(1)) {
                hedge = Some(w.to_lowercase());
                self.bump();
            }
        }

        let term = match self.peek() {
            t if Self::is_name(t) => match self.bump() {
                Tok::Word(w) => w,
                _ => unreachable!(),
            },
            t => {
                return Err(self.error(format!(
                    "expected a term name after '{} IS', found {}",
                    var,
                    Self::describe(t)
                )))
            }
        };

        match self.peek() {
            Tok::Word(w) if Self::is_name(self.peek()) => {
                return Err(self.error(if Hedge::from_word(&term).is_none() && hedge.is_none() {
                    format!(
                        "unexpected '{}' after '{} IS {}': terms are single words. If '{}' is meant \
                         as a hedge, use one of: {}; for a multi-word term use underscores ('{}_{}')",
                        w, var, term, term, HEDGES, term, w
                    )
                } else {
                    format!("unexpected '{}' after '{} IS {}': terms are single words", w, var, term)
                }))
            }
            t if Self::is_kw(t, "NOT") => {
                return Err(self.error(format!(
                    "NOT must come right after IS: write '{} IS NOT {} ...'",
                    var, term
                )))
            }
            _ => {}
        }
        Ok(Clause {
            var,
            term,
            negated,
            hedge,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_rule() {
        let r = Rule::parse("IF temp IS hot AND hum IS high THEN fan IS fast").unwrap();
        assert_eq!(
            r.antecedent,
            vec![vec![Clause::new("temp", "hot"), Clause::new("hum", "high")]]
        );
        assert_eq!(r.consequent, vec![Clause::new("fan", "fast")]);
        assert_eq!(r.weight, 1.0);
    }

    #[test]
    fn and_binds_tighter_than_or() {
        let r: Rule = "IF a IS x OR b IS y AND c IS z THEN o IS p"
            .parse()
            .unwrap();
        assert_eq!(r.antecedent.len(), 2);
        assert_eq!(r.antecedent[0].len(), 1);
        assert_eq!(r.antecedent[1].len(), 2);
    }

    #[test]
    fn negation_hedges_and_multi_consequents() {
        let r = Rule::parse("IF t IS NOT very hot THEN fan IS slow AND alarm IS off").unwrap();
        let c = &r.antecedent[0][0];
        assert!(c.negated);
        assert_eq!(c.hedge.as_deref(), Some("very"));
        assert_eq!(c.term, "hot");
        assert_eq!(r.consequent.len(), 2);
        assert!((c.modify(0.5) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn single_word_after_is_is_the_term() {
        let r = Rule::parse("IF x IS plus THEN y IS minus").unwrap();
        assert_eq!(r.antecedent[0][0].term, "plus");
        assert_eq!(r.antecedent[0][0].hedge, None);
    }

    #[test]
    fn display_round_trips() {
        for s in [
            "IF t IS hot THEN f IS fast",
            "IF t IS NOT very hot OR h IS low AND t IS cold THEN f IS slow AND a IS on",
        ] {
            let r = Rule::parse(s).unwrap();
            assert_eq!(r.to_string(), s);
            assert_eq!(Rule::parse(&r.to_string()).unwrap(), r);
        }
        let w = Rule::parse("IF t IS hot THEN f IS fast")
            .unwrap()
            .with_weight(0.5);
        assert_eq!(w.to_string(), "IF t IS hot THEN f IS fast WITH 0.5");
        assert_eq!(Rule::parse(&w.to_string()).unwrap(), w);
    }

    #[test]
    fn malformed_rules_error() {
        for s in [
            "t IS hot THEN f IS fast",
            "IF t IS hot",
            "IF t hot THEN f IS fast",
            "IF t IS hot THEN f IS fast OR g IS slow",
            "IF t IS NOT THEN f IS fast",
            "IF t IS really hot THEN f IS fast",
            "IF t IS very NOT hot THEN f IS fast",
            "IF (t IS hot THEN f IS fast",
            "IF t IS hot) THEN f IS fast",
            "IF t IS hot THEN f IS fast WITH 2",
            "IF t IS hot THEN f IS fast extra",
        ] {
            assert!(matches!(Rule::parse(s), Err(Error::Parse(_))), "{}", s);
        }
        let err = Rule::parse("IF service IS really poor THEN tip IS low")
            .unwrap_err()
            .to_string();
        assert!(err.contains("really_poor") && err.contains('^'), "{}", err);
        let err = Rule::parse("IF t IS very NOT hot THEN f IS fast")
            .unwrap_err()
            .to_string();
        assert!(err.contains("IS NOT"), "{}", err);
    }

    #[test]
    fn parentheses_and_case_insensitive_keywords() {
        let r = Rule::parse("if (a is x or b is y) and c is NOT z then o is p").unwrap();
        assert_eq!(
            r.antecedent,
            vec![
                vec![Clause::new("a", "x"), Clause::new("c", "z").negate()],
                vec![Clause::new("b", "y"), Clause::new("c", "z").negate()],
            ]
        );
        assert_eq!(
            r.to_string(),
            "IF a IS x AND c IS NOT z OR b IS y AND c IS NOT z THEN o IS p"
        );
        let nested = Rule::parse("IF ((a IS x)) THEN o IS p").unwrap();
        assert_eq!(nested.antecedent, vec![vec![Clause::new("a", "x")]]);
    }

    #[test]
    fn builder_helpers() {
        let r = Rule::when([("t", "hot"), ("h", "high")], ("f", "fast")).unwrap();
        assert_eq!(r.to_string(), "IF t IS hot AND h IS high THEN f IS fast");
        assert!(Clause::new("t", "hot").with_hedge("nope").is_err());
        assert!(Rule::new(vec![], vec![Clause::new("f", "x")]).is_err());
    }
}
