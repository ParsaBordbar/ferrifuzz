//! The Rust FTS port must reproduce the reference Python implementation
//! (`python/ferrifuzz/fts.py`). Expected values were produced by that module.

use ferrifuzz::fts::{FuzzyTimeSeries, PartitionShape};
use ferrifuzz::metrics::rmse;

const ENROLLMENTS: [f64; 22] = [
    13055.0, 13563.0, 13867.0, 14696.0, 15460.0, 15311.0, 15603.0, 15861.0, 16807.0, 16919.0,
    16388.0, 15433.0, 15497.0, 15145.0, 15163.0, 15984.0, 16859.0, 18150.0, 18970.0, 19328.0,
    19337.0, 18876.0,
];

fn check(
    n: usize,
    order: usize,
    shape: PartitionShape,
    rmse_ref: f64,
    next_ref: f64,
    flrgs: usize,
) {
    let mut m = FuzzyTimeSeries::new(n, order).unwrap().with_shape(shape);
    m.fit(&ENROLLMENTS).unwrap();
    let f = m.forecast(&ENROLLMENTS).unwrap();
    assert!((rmse(&ENROLLMENTS[order..], &f) - rmse_ref).abs() < 1e-6);
    let next = m
        .predict(&ENROLLMENTS[ENROLLMENTS.len() - order..], 3)
        .unwrap();
    assert!(
        next.iter().all(|v| (v - next_ref).abs() < 1e-6),
        "{:?}",
        next
    );
    assert_eq!(m.flrgs().len(), flrgs);
}

#[test]
fn matches_python_reference() {
    check(
        7,
        1,
        PartitionShape::Triangle,
        535.8129755445497,
        18708.8,
        5,
    );
    check(
        9,
        2,
        PartitionShape::Triangle,
        349.6219256210912,
        19022.9,
        13,
    );
    check(
        5,
        3,
        PartitionShape::Gaussian,
        895.3735806619314,
        18080.6,
        11,
    );
}
