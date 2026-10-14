//! Interval type-2 type reduction — Karnik–Mendel, center-of-sets, and height.

/// KM type reduction over a sampled FOU — returns `[y_l, y_r]`, or `None` if empty.
pub fn km_type_reduce(xs: &[f64], lower: &[f64], upper: &[f64]) -> Option<(f64, f64)> {
    if xs.is_empty() {
        return None;
    }
    let yl = km_endpoint(xs, lower, upper, true)?;
    let yr = km_endpoint(xs, lower, upper, false)?;
    Some((yl, yr))
}

/// Center-of-sets reduction over (center, firing-interval) pairs; centers need not be sorted.
pub fn center_of_sets(centers: &[f64], f_lower: &[f64], f_upper: &[f64]) -> Option<(f64, f64)> {
    if centers.is_empty() {
        return None;
    }
    let mut idx: Vec<usize> = (0..centers.len()).collect();
    idx.sort_by(|&a, &b| {
        centers[a]
            .partial_cmp(&centers[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let xs: Vec<f64> = idx.iter().map(|&i| centers[i]).collect();
    let lo: Vec<f64> = idx.iter().map(|&i| f_lower[i]).collect();
    let up: Vec<f64> = idx.iter().map(|&i| f_upper[i]).collect();
    km_type_reduce(&xs, &lo, &up)
}

/// Height reduction — firing-weighted average of centers, computed per interval bound and
/// returned as an ordered `(low, high)` pair.
pub fn height(centers: &[f64], f_lower: &[f64], f_upper: &[f64]) -> Option<(f64, f64)> {
    let wavg = |w: &[f64]| -> Option<f64> {
        let den: f64 = w.iter().sum();
        if den <= 0.0 {
            return None;
        }
        Some(centers.iter().zip(w).map(|(c, wi)| c * wi).sum::<f64>() / den)
    };
    // Each bound is a separate weighted average, so they need not come out ordered; and
    // when every lower firing is zero only the upper average exists.
    match (wavg(f_lower), wavg(f_upper)) {
        (Some(a), Some(b)) => Some((a.min(b), a.max(b))),
        (Some(a), None) | (None, Some(a)) => Some((a, a)),
        (None, None) => None,
    }
}

/// One KM endpoint — the switch point is found by monotonically convergent fixed-point iteration.
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
        // Switch walked off the active support; last finite iterate is the endpoint.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degenerate_fou_reduces_to_the_centroid() {
        let xs: Vec<f64> = (0..=10).map(f64::from).collect();
        let mu: Vec<f64> = xs
            .iter()
            .map(|&x| 1.0 - (x - 4.0).abs() / 4.0)
            .map(|m| m.max(0.0))
            .collect();
        let (yl, yr) = km_type_reduce(&xs, &mu, &mu).unwrap();
        assert!((yl - 4.0).abs() < 1e-9 && (yr - 4.0).abs() < 1e-9);
    }

    #[test]
    fn symmetric_fou_gives_symmetric_interval() {
        let xs: Vec<f64> = (0..=100).map(|i| i as f64 / 10.0).collect();
        let upper: Vec<f64> = xs
            .iter()
            .map(|&x| (1.0 - (x - 5.0).abs() / 5.0).max(0.0))
            .collect();
        let lower: Vec<f64> = upper.iter().map(|u| 0.5 * u).collect();
        let (yl, yr) = km_type_reduce(&xs, &lower, &upper).unwrap();
        assert!(yl < 5.0 && yr > 5.0);
        assert!(((5.0 - yl) - (yr - 5.0)).abs() < 1e-6);
    }

    #[test]
    fn empty_or_zero_sets_have_no_centroid() {
        assert_eq!(km_type_reduce(&[], &[], &[]), None);
        assert_eq!(km_type_reduce(&[1.0, 2.0], &[0.0, 0.0], &[0.0, 0.0]), None);
        assert_eq!(height(&[1.0], &[0.0], &[0.0]), None);
        assert_eq!(
            height(&[1.0, 3.0], &[0.0, 0.0], &[1.0, 1.0]),
            Some((2.0, 2.0))
        );
    }

    #[test]
    fn center_of_sets_and_height() {
        let (l, r) = center_of_sets(&[10.0, 0.0], &[0.5, 0.5], &[1.0, 1.0]).unwrap();
        assert!(l < 5.0 && r > 5.0);
        assert_eq!(
            height(&[0.0, 10.0], &[1.0, 1.0], &[1.0, 3.0]),
            Some((5.0, 7.5))
        );
    }

    #[test]
    fn height_bounds_are_ordered() {
        // The lower firings favour the larger center, so the raw averages come out reversed.
        assert_eq!(
            height(&[0.0, 10.0], &[1.0, 3.0], &[1.0, 1.0]),
            Some((5.0, 7.5))
        );
    }
}
