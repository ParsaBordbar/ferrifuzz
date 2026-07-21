//! Interval type-2 type reduction.
//!
//! Karnik–Mendel (KM) reduces a sampled interval type-1 set — the footprint of
//! uncertainty (FOU) given as parallel lower/upper membership grids — to the
//! centroid interval `[y_l, y_r]`. The IT2 Mamdani engine then defuzzifies that
//! interval to a crisp value by averaging its endpoints.

/// KM type reduction over a sampled FOU.
///
/// `xs` are the (ascending) universe samples; `lower`/`upper` are the aggregated
/// lower and upper memberships at each sample. Returns the centroid interval
/// `[y_l, y_r]`, or `None` when the FOU is empty (no rule fired).
pub fn km_type_reduce(xs: &[f64], lower: &[f64], upper: &[f64]) -> Option<(f64, f64)> {
    if xs.is_empty() {
        return None;
    }
    let yl = km_endpoint(xs, lower, upper, true)?;
    let yr = km_endpoint(xs, lower, upper, false)?;
    Some((yl, yr))
}

/// One KM endpoint. For the left endpoint (`left = true`) samples below the
/// switch point take the upper membership and those above take the lower; the
/// right endpoint mirrors this. The switch point is found by fixed-point
/// iteration, which KM guarantees converges monotonically.
fn km_endpoint(xs: &[f64], lower: &[f64], upper: &[f64], left: bool) -> Option<f64> {
    let n = xs.len();

    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        let w = (lower[i] + upper[i]) / 2.0;
        num += xs[i] * w;
        den += w;
    }
    if den <= 0.0 {
        return None;
    }
    let mut y = num / den;

    let (below, above) = if left { (upper, lower) } else { (lower, upper) };

    for _ in 0..n {
        let mut k = 0usize;
        while k + 1 < n && xs[k + 1] <= y {
            k += 1;
        }

        let mut num = 0.0;
        let mut den = 0.0;
        for i in 0..n {
            let w = if i <= k { below[i] } else { above[i] };
            num += xs[i] * w;
            den += w;
        }
        // A zero denominator means the switch walked off the active support
        // (e.g. an all-zero lower MF at a knife-edge firing). The set's centroid
        // is unbounded on this side, so the last finite iterate is the endpoint.
        if den <= 0.0 {
            return Some(y);
        }
        let y_new = num / den;

        if (y_new - y).abs() < 1e-9 {
            return Some(y_new);
        }
        y = y_new;
    }
    Some(y)
}
