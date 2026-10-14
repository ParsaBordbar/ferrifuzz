//! Fuzzy time series — Chen (1996) first-order (FOFTS) and high-order (HOFTS) models.
//!
//! ```
//! use ferrifuzz::fts::FuzzyTimeSeries;
//!
//! let series = [10.0, 12.0, 13.0, 12.0, 15.0, 16.0, 18.0, 17.0, 19.0, 21.0];
//! let mut model = FuzzyTimeSeries::new(7, 1)?;
//! model.fit(&series)?;
//! let next = model.predict(&series, 3)?;
//! assert_eq!(next.len(), 3);
//! # Ok::<(), ferrifuzz::Error>(())
//! ```

use std::collections::HashMap;
use std::fmt;

use crate::error::{invalid, Error, Result};
use crate::membership::{mf, Type1MF};
use crate::metrics::{self, Scores};

/// Shape of the equal-width partitions (an alias of [`crate::Shape`]).
///
/// Chen's method assigns each value to its highest-membership partition, and both
/// shapes peak at the same evenly spaced centers, so triangle and gaussian partitions
/// produce the same model; the shape only matters when you plot or inspect partitions.
pub use crate::membership::Shape as PartitionShape;

/// One linguistic term of the universe, e.g. `A3`.
#[derive(Clone, Debug, PartialEq)]
pub struct Partition {
    pub label: String,
    pub mf: Type1MF,
    pub center: f64,
}

impl Partition {
    pub fn membership(&self, x: f64) -> f64 {
        self.mf.membership(x)
    }
}

/// Fuzzy logical relationship group: an antecedent window of partition indices
/// and every partition that followed it in the training data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flrg {
    pub antecedent: Vec<usize>,
    pub consequents: Vec<usize>,
}

impl fmt::Display for Flrg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = |v: &[usize]| {
            v.iter()
                .map(|i| format!("A{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ")
        };
        write!(
            f,
            "FLRG({} -> {})",
            label(&self.antecedent),
            label(&self.consequents)
        )
    }
}

/// `n` evenly spaced overlapping partitions over `[lo, hi]`, labelled `A1..An`.
pub fn equal_width_partitions(
    lo: f64,
    hi: f64,
    n: usize,
    shape: PartitionShape,
) -> Result<Vec<Partition>> {
    Ok(mf::evenly_spaced(lo, hi, n, shape)?
        .into_iter()
        .enumerate()
        .map(|(i, (center, mf))| Partition {
            label: format!("A{}", i + 1),
            mf,
            center,
        })
        .collect())
}

/// Chen's fuzzy time series: partition the universe, fuzzify each value to its
/// best-matching partition, learn FLRGs, and forecast by averaging consequent centers.
#[derive(Clone, Debug)]
pub struct FuzzyTimeSeries {
    n_partitions: usize,
    order: usize,
    shape: PartitionShape,
    margin: f64,
    fixed_universe: Option<(f64, f64)>,
    partitions: Vec<Partition>,
    flrgs: Vec<Flrg>,
    index: HashMap<Vec<usize>, usize>,
    universe: (f64, f64),
    fitted: bool,
}

impl FuzzyTimeSeries {
    /// `n_partitions ≥ 2` terms and an antecedent window of `order ≥ 1` steps.
    pub fn new(n_partitions: usize, order: usize) -> Result<Self> {
        if n_partitions < 2 {
            return invalid("n_partitions must be ≥ 2");
        }
        if order < 1 {
            return invalid("order must be ≥ 1");
        }
        Ok(Self {
            n_partitions,
            order,
            shape: PartitionShape::Triangle,
            margin: 0.1,
            fixed_universe: None,
            partitions: Vec::new(),
            flrgs: Vec::new(),
            index: HashMap::new(),
            universe: (0.0, 1.0),
            fitted: false,
        })
    }

    pub fn with_shape(mut self, shape: PartitionShape) -> Self {
        self.shape = shape;
        self
    }

    /// Fraction of the data range added on each side of the universe (default 0.1).
    /// Ignored when [`FuzzyTimeSeries::with_universe`] sets the universe explicitly.
    pub fn with_margin(mut self, margin: f64) -> Self {
        self.margin = margin;
        self
    }

    /// Partition exactly `[lo, hi]` instead of the data range plus margin — e.g. Chen's
    /// enrollment example uses `U = [13000, 20000]`.
    pub fn with_universe(mut self, lo: f64, hi: f64) -> Self {
        self.fixed_universe = Some((lo, hi));
        self
    }

    /// Learn partitions and FLRGs; the series must be longer than `order`.
    pub fn fit(&mut self, series: &[f64]) -> Result<&mut Self> {
        if series.len() <= self.order {
            return Err(Error::InvalidInput(format!(
                "series length ({}) must be > order ({})",
                series.len(),
                self.order
            )));
        }
        if let Some(i) = series.iter().position(|x| !x.is_finite()) {
            return Err(Error::InvalidInput(format!(
                "series[{}] is {}; remove or fill missing values before fit()",
                i, series[i]
            )));
        }
        self.universe = match self.fixed_universe {
            Some((lo, hi)) if lo < hi => (lo, hi),
            Some((lo, hi)) => {
                return invalid(format!("universe needs lo < hi, got [{}, {}]", lo, hi))
            }
            None => {
                let lo = series.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = series.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let range = if hi - lo == 0.0 { 1.0 } else { hi - lo };
                let pad = range * self.margin;
                // A constant series with margin 0 would leave a zero-width universe.
                let pad = if pad == 0.0 && lo == hi { 0.5 } else { pad };
                (lo - pad, hi + pad)
            }
        };
        self.partitions = equal_width_partitions(
            self.universe.0,
            self.universe.1,
            self.n_partitions,
            self.shape,
        )?;

        let fuzz = self.fuzzify_series(series);
        self.flrgs.clear();
        self.index.clear();
        for window in fuzz.windows(self.order + 1) {
            let (antecedent, consequent) = (&window[..self.order], window[self.order]);
            let i = *self.index.entry(antecedent.to_vec()).or_insert_with(|| {
                self.flrgs.push(Flrg {
                    antecedent: antecedent.to_vec(),
                    consequents: Vec::new(),
                });
                self.flrgs.len() - 1
            });
            if !self.flrgs[i].consequents.contains(&consequent) {
                self.flrgs[i].consequents.push(consequent);
            }
        }
        self.fitted = true;
        Ok(self)
    }

    /// Forecast `steps` values past the end of `history` (needs ≥ `order` values),
    /// feeding each forecast back in as the next input.
    pub fn predict(&self, history: &[f64], steps: usize) -> Result<Vec<f64>> {
        self.check_fitted()?;
        if history.len() < self.order {
            return Err(Error::InvalidInput(format!(
                "history must contain at least order={} values",
                self.order
            )));
        }
        let mut fuzz = self.fuzzify_series(history);
        let mut out = Vec::with_capacity(steps);
        for _ in 0..steps {
            let value = self.forecast_window(&fuzz);
            out.push(value);
            fuzz.push(self.fuzzify(value));
        }
        Ok(out)
    }

    /// One-step-ahead forecasts over `series`, aligned with `series[order..]`.
    pub fn forecast(&self, series: &[f64]) -> Result<Vec<f64>> {
        self.check_fitted()?;
        let fuzz = self.fuzzify_series(series);
        Ok(fuzz
            .windows(self.order)
            .take(fuzz.len().saturating_sub(self.order))
            .map(|w| self.forecast_window(w))
            .collect())
    }

    /// Index of the partition with the highest membership at `x` (first on ties); a
    /// value outside every partition maps to the partition with the nearest center.
    pub fn fuzzify(&self, x: f64) -> usize {
        let mut best = (0, 0.0);
        for (i, p) in self.partitions.iter().enumerate() {
            let mu = p.membership(x);
            if mu > best.1 {
                best = (i, mu);
            }
        }
        if best.1 > 0.0 {
            return best.0;
        }
        let dist = |p: &Partition| (p.center - x).abs();
        (0..self.partitions.len())
            .min_by(|&a, &b| dist(&self.partitions[a]).total_cmp(&dist(&self.partitions[b])))
            .unwrap_or(0)
    }

    /// One-step-ahead forecast errors over `series` (compared with `series[order..]`).
    pub fn evaluate(&self, series: &[f64]) -> Result<Scores> {
        let forecast = self.forecast(series)?;
        Ok(metrics::score(
            &series[self.order.min(series.len())..],
            &forecast,
        ))
    }

    pub fn partitions(&self) -> &[Partition] {
        &self.partitions
    }

    /// FLRGs in order of first appearance.
    pub fn flrgs(&self) -> &[Flrg] {
        &self.flrgs
    }

    pub fn universe(&self) -> (f64, f64) {
        self.universe
    }

    pub fn is_fitted(&self) -> bool {
        self.fitted
    }

    pub fn n_partitions(&self) -> usize {
        self.n_partitions
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn shape(&self) -> PartitionShape {
        self.shape
    }

    pub fn margin(&self) -> f64 {
        self.margin
    }

    /// Multi-line description of the fitted model.
    pub fn summary(&self) -> Result<String> {
        self.check_fitted()?;
        Ok(format!(
            "{}\n  Universe    : [{:.4}, {:.4}]\n  Partitions  : {}  ({})\n  FLRGs found : {}",
            self,
            self.universe.0,
            self.universe.1,
            self.n_partitions,
            self.shape,
            self.flrgs.len()
        ))
    }

    fn fuzzify_series(&self, series: &[f64]) -> Vec<usize> {
        series.iter().map(|&x| self.fuzzify(x)).collect()
    }

    /// Mean center of the consequents for the last `order` indices; falls back to
    /// the last partition's center for an unseen pattern.
    fn forecast_window(&self, fuzz: &[usize]) -> f64 {
        let window = &fuzz[fuzz.len() - self.order..];
        match self.index.get(window) {
            Some(&i) => {
                let cons = &self.flrgs[i].consequents;
                cons.iter().map(|&c| self.partitions[c].center).sum::<f64>() / cons.len() as f64
            }
            None => self.partitions[window[window.len() - 1]].center,
        }
    }

    fn check_fitted(&self) -> Result<()> {
        if self.fitted {
            Ok(())
        } else {
            Err(Error::NotFitted)
        }
    }
}

impl fmt::Display for FuzzyTimeSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FuzzyTimeSeries(n_partitions={}, order={}, mf_type='{}', ",
            self.n_partitions, self.order, self.shape
        )?;
        if self.fitted {
            write!(f, "fitted, {} FLRGs)", self.flrgs.len())
        } else {
            f.write_str("not fitted)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partitions_are_evenly_spaced() {
        let p = equal_width_partitions(0.0, 10.0, 5, PartitionShape::Triangle).unwrap();
        let centers: Vec<f64> = p.iter().map(|p| p.center).collect();
        assert_eq!(centers, [0.0, 2.5, 5.0, 7.5, 10.0]);
        assert_eq!(p[1].membership(2.5), 1.0);
        assert_eq!(p[1].membership(3.75), 0.5);
        assert_eq!(p[4].label, "A5");
        assert!(equal_width_partitions(0.0, 1.0, 1, PartitionShape::Triangle).is_err());
    }

    #[test]
    fn learns_flrgs_and_forecasts() {
        // Strictly periodic pattern → every transition is deterministic.
        let series: Vec<f64> = (0..30).map(|i| [0.0, 5.0, 10.0][i % 3]).collect();
        let mut m = FuzzyTimeSeries::new(3, 1).unwrap().with_margin(0.0);
        m.fit(&series).unwrap();
        assert_eq!(m.flrgs().len(), 3);
        assert!(m.flrgs().iter().all(|g| g.consequents.len() == 1));
        assert_eq!(m.predict(&[0.0], 4).unwrap(), [5.0, 10.0, 0.0, 5.0]);
        let one_step = m.forecast(&series).unwrap();
        assert_eq!(one_step.len(), series.len() - 1);
        assert_eq!(one_step, series[1..]);
    }

    #[test]
    fn high_order_window() {
        let series = [1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0, 2.0, 1.0];
        let mut m = FuzzyTimeSeries::new(5, 2).unwrap();
        m.fit(&series).unwrap();
        assert!(m.flrgs().iter().all(|g| g.antecedent.len() == 2));
        assert_eq!(m.forecast(&series).unwrap().len(), series.len() - 2);
        assert!(m.predict(&[1.0], 1).is_err());
    }

    #[test]
    fn errors_before_fit_and_on_short_series() {
        let m = FuzzyTimeSeries::new(7, 1).unwrap();
        assert_eq!(m.predict(&[1.0], 1), Err(Error::NotFitted));
        let mut m = FuzzyTimeSeries::new(7, 3).unwrap();
        assert!(m.fit(&[1.0, 2.0, 3.0]).is_err());
        assert!(FuzzyTimeSeries::new(1, 1).is_err());
        assert!(FuzzyTimeSeries::new(3, 0).is_err());
    }

    #[test]
    fn values_outside_the_universe_use_the_nearest_partition() {
        let series: Vec<f64> = (0..16).map(|i| 100.0 + i as f64).collect();
        let mut m = FuzzyTimeSeries::new(5, 1).unwrap();
        m.fit(&series).unwrap();
        assert_eq!(m.fuzzify(1e6), 4);
        assert_eq!(m.fuzzify(-1e6), 0);
        let next = m.predict(&[200.0], 1).unwrap()[0];
        assert!(next > 110.0, "{}", next);
    }

    #[test]
    fn explicit_universe_nan_and_evaluate() {
        let series = [1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0];
        let mut m = FuzzyTimeSeries::new(3, 1).unwrap().with_universe(0.0, 4.0);
        m.fit(&series).unwrap();
        assert_eq!(m.universe(), (0.0, 4.0));
        assert_eq!(m.partitions()[1].center, 2.0);
        let s = m.evaluate(&series).unwrap();
        assert!(s.rmse.is_finite() && s.mae <= s.rmse);
        assert!(FuzzyTimeSeries::new(3, 1)
            .unwrap()
            .fit(&[1.0, f64::NAN, 2.0])
            .is_err());
        assert!(FuzzyTimeSeries::new(3, 1)
            .unwrap()
            .with_universe(2.0, 1.0)
            .fit(&series)
            .is_err());
    }

    #[test]
    fn constant_series_still_fits() {
        let mut m = FuzzyTimeSeries::new(4, 1)
            .unwrap()
            .with_shape(PartitionShape::Gaussian);
        m.fit(&[5.0; 6]).unwrap();
        assert_eq!(m.predict(&[5.0], 1).unwrap().len(), 1);
        assert!(m.summary().unwrap().contains("gaussian"));
    }
}
