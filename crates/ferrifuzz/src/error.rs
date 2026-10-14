use std::fmt;

/// Everything that can go wrong when building or running a fuzzy model.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A constructor argument is outside its valid domain (e.g. `a ≤ b ≤ c` violated).
    InvalidParameter(String),
    /// A rule string could not be parsed.
    Parse(String),
    /// Inputs have the wrong shape or value (e.g. NaN, batch columns of different lengths).
    InvalidInput(String),
    /// An input variable the system needs was not supplied.
    MissingInput(String),
    /// An input name that is not a registered input variable was supplied.
    UnknownInput(String),
    /// The model itself is inconsistent (a rule names an unknown variable or term, ...).
    InvalidModel(String),
    /// A model that needs fitting was used before `fit`.
    NotFitted,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidParameter(msg)
            | Error::Parse(msg)
            | Error::InvalidInput(msg)
            | Error::MissingInput(msg)
            | Error::UnknownInput(msg)
            | Error::InvalidModel(msg) => f.write_str(msg),
            Error::NotFitted => {
                f.write_str("Model is not fitted yet. Call fit() before predict() or forecast().")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn invalid<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::InvalidParameter(msg.into()))
}

/// `" (did you mean 'hot'?)"` when a candidate is a close spelling of `name`, else `""`.
///
/// Close means a case-insensitive match, or an edit distance (swaps count once) of at
/// most 1 for names up to 2 chars, 2 up to 5 chars, and a third of the length beyond.
pub(crate) fn did_you_mean<'a>(
    name: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> String {
    let lower = name.to_lowercase();
    let len = lower.chars().count();
    let max_d = match len {
        0..=2 => 1,
        3..=5 => 2,
        _ => len / 3,
    };
    let best = candidates
        .into_iter()
        .map(|c| (edit_distance(&lower, &c.to_lowercase()), c))
        .filter(|(d, c)| *d == 0 || (*d <= max_d && *d < len.max(c.chars().count())))
        .min_by_key(|(d, _)| *d);
    match best {
        Some((_, c)) => format!(" (did you mean '{}'?)", c),
        None => String::new(),
    }
}

/// Error for an unrecognised option name, listing the valid ones.
pub(crate) fn unknown_name(what: &str, name: &str, options: &[&str]) -> Error {
    Error::InvalidParameter(format!(
        "unknown {} '{}'{}; valid options: {}",
        what,
        name,
        did_you_mean(name, options.iter().copied()),
        quoted_list(options.iter().copied())
    ))
}

/// `'a', 'b', 'c'` for error messages; `(none)` when empty.
pub(crate) fn quoted_list<'a>(items: impl IntoIterator<Item = &'a str>) -> String {
    let v: Vec<String> = items.into_iter().map(|s| format!("'{}'", s)).collect();
    if v.is_empty() {
        "(none)".to_string()
    } else {
        v.join(", ")
    }
}

/// Optimal-string-alignment distance: Levenshtein plus adjacent swaps ("hgih" → "high" = 1).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions() {
        assert_eq!(
            did_you_mean("hott", ["cold", "hot"]),
            " (did you mean 'hot'?)"
        );
        assert_eq!(
            did_you_mean("Hot", ["cold", "hot"]),
            " (did you mean 'hot'?)"
        );
        assert_eq!(did_you_mean("zzzzzz", ["cold", "hot"]), "");
        assert_eq!(
            did_you_mean("hgih", ["low", "high"]),
            " (did you mean 'high'?)"
        );
        assert_eq!(
            did_you_mean("centriod", ["centroid", "bisector"]),
            " (did you mean 'centroid'?)"
        );
        assert_eq!(quoted_list(["a", "b"]), "'a', 'b'");
    }
}
