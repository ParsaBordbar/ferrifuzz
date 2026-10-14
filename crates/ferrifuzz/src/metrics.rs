//! Forecasting error metrics. Pairs are zipped, so the longer slice is truncated.

/// Root mean square error; `NaN` on empty input.
pub fn rmse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    mean(y_true.iter().zip(y_pred).map(|(a, b)| (a - b).powi(2))).sqrt()
}

/// Mean absolute error; `NaN` on empty input.
pub fn mae(y_true: &[f64], y_pred: &[f64]) -> f64 {
    mean(y_true.iter().zip(y_pred).map(|(a, b)| (a - b).abs()))
}

/// Mean absolute percentage error (in %); zero-valued truths are skipped.
pub fn mape(y_true: &[f64], y_pred: &[f64]) -> f64 {
    100.0
        * mean(
            y_true
                .iter()
                .zip(y_pred)
                .filter(|(a, _)| **a != 0.0)
                .map(|(a, b)| ((a - b) / a).abs()),
        )
}

/// RMSE, MAE and MAPE in one go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scores {
    pub rmse: f64,
    pub mae: f64,
    pub mape: f64,
}

pub fn score(y_true: &[f64], y_pred: &[f64]) -> Scores {
    Scores {
        rmse: rmse(y_true, y_pred),
        mae: mae(y_true, y_pred),
        mape: mape(y_true, y_pred),
    }
}

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, n) = values.fold((0.0, 0usize), |(s, n), v| (s + v, n + 1));
    if n == 0 {
        f64::NAN
    } else {
        sum / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        let t = [1.0, 2.0, 4.0];
        let p = [1.0, 3.0, 2.0];
        assert!((rmse(&t, &p) - (5.0f64 / 3.0).sqrt()).abs() < 1e-12);
        assert_eq!(mae(&t, &p), 1.0);
        assert!((mape(&t, &p) - 100.0 * (0.5 + 0.5) / 3.0).abs() < 1e-12);
    }

    #[test]
    fn edge_cases() {
        assert!(rmse(&[], &[]).is_nan());
        assert!(mape(&[0.0, 0.0], &[1.0, 2.0]).is_nan());
        assert_eq!(mape(&[0.0, 2.0], &[5.0, 1.0]), 50.0);
        assert_eq!(mae(&[1.0, 2.0, 3.0], &[1.0]), 0.0);
    }
}
