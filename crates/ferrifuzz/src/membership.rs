//! Membership functions — type-1 shapes, the [`Type1MF`] enum that unifies them,
//! and interval type-2 sets ([`IT2MF`]).

use std::fmt;

use crate::error::{invalid, Result};

/// Anything that maps a crisp value to a membership degree in `[0, 1]`.
pub trait MembershipFunction {
    /// Degree of membership at `x`.
    fn membership(&self, x: f64) -> f64;
}

/// Triangular MF — feet at `a`/`c`, peak at `b` (`a ≤ b ≤ c`).
///
/// `a == b` gives a left shoulder and `b == c` a right shoulder; both reach 1.0 at the peak.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriangularMF {
    a: f64,
    b: f64,
    c: f64,
}

impl TriangularMF {
    pub fn new(a: f64, b: f64, c: f64) -> Result<Self> {
        if !(a <= b && b <= c) {
            return invalid(format!(
                "triangle({}, {}, {}): needs a ≤ b ≤ c (feet a, c; peak b)",
                a, b, c
            ));
        }
        Ok(Self { a, b, c })
    }

    pub fn params(&self) -> (f64, f64, f64) {
        (self.a, self.b, self.c)
    }
}

impl MembershipFunction for TriangularMF {
    fn membership(&self, x: f64) -> f64 {
        if x < self.a || x > self.c {
            0.0
        } else if x == self.b {
            1.0
        } else if x < self.b {
            (x - self.a) / (self.b - self.a)
        } else {
            (self.c - x) / (self.c - self.b)
        }
    }
}

/// Trapezoidal MF — feet at `a`/`d`, plateau `b..=c` (`a ≤ b ≤ c ≤ d`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrapezoidalMF {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

impl TrapezoidalMF {
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> Result<Self> {
        if !(a <= b && b <= c && c <= d) {
            return invalid(format!(
                "trapezoid({}, {}, {}, {}): needs a ≤ b ≤ c ≤ d",
                a, b, c, d
            ));
        }
        Ok(Self { a, b, c, d })
    }

    pub fn params(&self) -> (f64, f64, f64, f64) {
        (self.a, self.b, self.c, self.d)
    }
}

impl MembershipFunction for TrapezoidalMF {
    fn membership(&self, x: f64) -> f64 {
        if x < self.a || x > self.d {
            0.0
        } else if x >= self.b && x <= self.c {
            1.0
        } else if x < self.b {
            (x - self.a) / (self.b - self.a)
        } else {
            (self.d - x) / (self.d - self.c)
        }
    }
}

/// Gaussian MF — peaked at `center` with spread `sigma > 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaussianMF {
    center: f64,
    sigma: f64,
}

impl GaussianMF {
    pub fn new(center: f64, sigma: f64) -> Result<Self> {
        if sigma.is_nan() || sigma <= 0.0 {
            return invalid(format!(
                "gaussian({}, {}): sigma must be > 0",
                center, sigma
            ));
        }
        Ok(Self { center, sigma })
    }

    pub fn params(&self) -> (f64, f64) {
        (self.center, self.sigma)
    }
}

impl MembershipFunction for GaussianMF {
    fn membership(&self, x: f64) -> f64 {
        let d = (x - self.center) / self.sigma;
        (-0.5 * d * d).exp()
    }
}

/// Singleton MF — μ = 1 exactly at `x`, 0 elsewhere (Sugeno spike).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SingletonMF {
    x: f64,
}

impl SingletonMF {
    pub fn new(x: f64) -> Self {
        Self { x }
    }

    pub fn value(&self) -> f64 {
        self.x
    }
}

impl MembershipFunction for SingletonMF {
    fn membership(&self, x: f64) -> f64 {
        if (x - self.x).abs() < f64::EPSILON {
            1.0
        } else {
            0.0
        }
    }
}

/// Generalized bell MF — `1 / (1 + |(x - center)/width|^(2·slope))`, `width, slope > 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BellMF {
    center: f64,
    width: f64,
    slope: f64,
}

impl BellMF {
    pub fn new(center: f64, width: f64, slope: f64) -> Result<Self> {
        if width.is_nan() || width <= 0.0 {
            return invalid(format!(
                "bell({}, {}, {}): width must be > 0",
                center, width, slope
            ));
        }
        if slope.is_nan() || slope <= 0.0 {
            return invalid(format!(
                "bell({}, {}, {}): slope must be > 0",
                center, width, slope
            ));
        }
        Ok(Self {
            center,
            width,
            slope,
        })
    }

    pub fn params(&self) -> (f64, f64, f64) {
        (self.center, self.width, self.slope)
    }
}

impl MembershipFunction for BellMF {
    fn membership(&self, x: f64) -> f64 {
        1.0 / (1.0
            + ((x - self.center) / self.width)
                .abs()
                .powf(2.0 * self.slope))
    }
}

/// Sigmoid MF — `1 / (1 + exp(-slope·(x - center)))`; slope > 0 rises, < 0 falls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SigmoidMF {
    center: f64,
    slope: f64,
}

impl SigmoidMF {
    pub fn new(center: f64, slope: f64) -> Self {
        Self { center, slope }
    }

    pub fn params(&self) -> (f64, f64) {
        (self.center, self.slope)
    }
}

impl MembershipFunction for SigmoidMF {
    fn membership(&self, x: f64) -> f64 {
        1.0 / (1.0 + (-self.slope * (x - self.center)).exp())
    }
}

/// Any type-1 membership function. Build one with the [`crate::mf`] helpers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Type1MF {
    Triangular(TriangularMF),
    Trapezoidal(TrapezoidalMF),
    Gaussian(GaussianMF),
    Singleton(SingletonMF),
    Bell(BellMF),
    Sigmoid(SigmoidMF),
}

impl Type1MF {
    /// Degree of membership at `x`.
    pub fn membership(&self, x: f64) -> f64 {
        match self {
            Self::Triangular(mf) => mf.membership(x),
            Self::Trapezoidal(mf) => mf.membership(x),
            Self::Gaussian(mf) => mf.membership(x),
            Self::Singleton(mf) => mf.membership(x),
            Self::Bell(mf) => mf.membership(x),
            Self::Sigmoid(mf) => mf.membership(x),
        }
    }

    /// Representative point: the peak (triangle, gaussian, bell, singleton), plateau
    /// midpoint (trapezoid) or inflection point (sigmoid).
    ///
    /// This is the constant `z` a zero-order Sugeno rule outputs and the consequent
    /// center for IT2 center-of-sets / height reduction. It is *not* the centroid:
    /// `triangle(0, 0, 5).center()` is 0, while its centroid is 5/3.
    pub fn center(&self) -> f64 {
        match self {
            Self::Triangular(mf) => mf.b,
            Self::Trapezoidal(mf) => (mf.b + mf.c) / 2.0,
            Self::Gaussian(mf) => mf.center,
            Self::Singleton(mf) => mf.x,
            Self::Bell(mf) => mf.center,
            Self::Sigmoid(mf) => mf.center,
        }
    }

    /// Points where the shape changes (feet, peaks, ±3σ, ...) — useful for plotting and checks.
    pub fn key_points(&self) -> Vec<f64> {
        match *self {
            Self::Triangular(m) => vec![m.a, m.b, m.c],
            Self::Trapezoidal(m) => vec![m.a, m.b, m.c, m.d],
            Self::Gaussian(m) => (-3..=3).map(|k| m.center + k as f64 * m.sigma).collect(),
            Self::Singleton(m) => vec![m.x],
            Self::Bell(m) => (-4..=4)
                .map(|k| m.center + k as f64 * m.width / 2.0)
                .collect(),
            Self::Sigmoid(m) => {
                let w = if m.slope == 0.0 {
                    1.0
                } else {
                    4.0 / m.slope.abs()
                };
                vec![m.center - w, m.center, m.center + w]
            }
        }
    }

    /// Shape name, e.g. `"Triangular"`.
    pub fn shape(&self) -> &'static str {
        match self {
            Self::Triangular(_) => "Triangular",
            Self::Trapezoidal(_) => "Trapezoidal",
            Self::Gaussian(_) => "Gaussian",
            Self::Singleton(_) => "Singleton",
            Self::Bell(_) => "Bell",
            Self::Sigmoid(_) => "Sigmoid",
        }
    }
}

impl MembershipFunction for Type1MF {
    fn membership(&self, x: f64) -> f64 {
        Type1MF::membership(self, x)
    }
}

/// Prints the constructor call, e.g. `triangle(0, 5, 10)`.
impl fmt::Display for Type1MF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Triangular(m) => write!(f, "triangle({}, {}, {})", m.a, m.b, m.c),
            Self::Trapezoidal(m) => write!(f, "trapezoid({}, {}, {}, {})", m.a, m.b, m.c, m.d),
            Self::Gaussian(m) => write!(f, "gaussian({}, {})", m.center, m.sigma),
            Self::Singleton(m) => write!(f, "singleton({})", m.x),
            Self::Bell(m) => write!(f, "bell({}, {}, {})", m.center, m.width, m.slope),
            Self::Sigmoid(m) => write!(f, "sigmoid({}, {})", m.center, m.slope),
        }
    }
}

macro_rules! impl_from_shape {
    ($($shape:ident => $variant:ident),*) => {
        $(impl From<$shape> for Type1MF {
            fn from(mf: $shape) -> Self { Type1MF::$variant(mf) }
        })*
    };
}

impl_from_shape!(
    TriangularMF => Triangular,
    TrapezoidalMF => Trapezoidal,
    GaussianMF => Gaussian,
    SingletonMF => Singleton,
    BellMF => Bell,
    SigmoidMF => Sigmoid
);

/// Interval type-2 MF bounded by an `upper` and a `lower` type-1 MF.
///
/// The region between them is the footprint of uncertainty (FOU). The lower MF may be
/// scaled down by `lower_height ∈ (0, 1]`, giving the textbook "shorter inner triangle".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IT2MF {
    upper: Type1MF,
    lower: Type1MF,
    lower_height: f64,
}

impl IT2MF {
    /// Errors if the lower MF rises above the upper one (checked at both shapes' key
    /// points and a grid between them) — usually a sign the arguments are swapped.
    pub fn new(upper: Type1MF, lower: Type1MF) -> Result<Self> {
        Self::with_lower_height(upper, lower, 1.0)
    }

    /// Like [`IT2MF::new`] with the lower MF scaled by `lower_height ∈ (0, 1]`.
    pub fn with_lower_height(upper: Type1MF, lower: Type1MF, lower_height: f64) -> Result<Self> {
        if lower_height.is_nan() || lower_height <= 0.0 || lower_height > 1.0 {
            return invalid(format!(
                "lower_height must be in (0, 1], got {}",
                lower_height
            ));
        }
        let mf = Self {
            upper,
            lower,
            lower_height,
        };
        let mut pts = upper.key_points();
        pts.extend(lower.key_points());
        let lo = pts.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = pts.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        pts.extend((0..=128).map(|i| lo + (hi - lo) * i as f64 / 128.0));
        for x in pts {
            let (l, u) = mf.membership(x);
            if l > u + 1e-9 {
                return invalid(format!(
                    "IT2 lower MF {} is above upper MF {} at x={} ({:.4} > {:.4}); \
                     the order is it2(upper, lower) — are they swapped?",
                    lower, upper, x, l, u
                ));
            }
        }
        Ok(mf)
    }

    /// Membership interval `(lower, upper)` at `x`.
    pub fn membership(&self, x: f64) -> (f64, f64) {
        (
            self.lower_height * self.lower.membership(x),
            self.upper.membership(x),
        )
    }

    /// Scale applied to the lower MF (1.0 unless built with `with_lower_height`).
    pub fn lower_height(&self) -> f64 {
        self.lower_height
    }

    /// Upper MF (UMF) — outer edge of the FOU.
    pub fn upper(&self) -> Type1MF {
        self.upper
    }

    /// Lower MF (LMF) — inner edge of the FOU.
    pub fn lower(&self) -> Type1MF {
        self.lower
    }

    /// Midpoint of the upper and lower centers.
    pub fn center(&self) -> f64 {
        (self.upper.center() + self.lower.center()) / 2.0
    }
}

/// Short constructors returning [`Type1MF`] / [`IT2MF`] — `mf::triangle(0.0, 5.0, 10.0)?`.
pub mod mf {
    use super::*;

    /// Triangular MF — peak at `b`, zero at `a` and `c` (`a ≤ b ≤ c`).
    pub fn triangle(a: f64, b: f64, c: f64) -> Result<Type1MF> {
        TriangularMF::new(a, b, c).map(Into::into)
    }

    /// Trapezoidal MF — plateau between `b` and `c` (`a ≤ b ≤ c ≤ d`).
    pub fn trapezoid(a: f64, b: f64, c: f64, d: f64) -> Result<Type1MF> {
        TrapezoidalMF::new(a, b, c, d).map(Into::into)
    }

    /// Gaussian MF peaked at `center` with spread `sigma > 0`.
    pub fn gaussian(center: f64, sigma: f64) -> Result<Type1MF> {
        GaussianMF::new(center, sigma).map(Into::into)
    }

    /// Singleton MF at `x`.
    pub fn singleton(x: f64) -> Type1MF {
        SingletonMF::new(x).into()
    }

    /// Generalized bell MF (`width, slope > 0`).
    pub fn bell(center: f64, width: f64, slope: f64) -> Result<Type1MF> {
        BellMF::new(center, width, slope).map(Into::into)
    }

    /// Sigmoid MF; `slope > 0` rises, `< 0` falls.
    pub fn sigmoid(center: f64, slope: f64) -> Type1MF {
        SigmoidMF::new(center, slope).into()
    }

    /// Interval type-2 MF from an `upper` and a `lower` type-1 MF (checked lower ≤ upper).
    pub fn it2(upper: Type1MF, lower: Type1MF) -> Result<IT2MF> {
        IT2MF::new(upper, lower)
    }

    /// `n ≥ 2` evenly spaced, overlapping terms over `[lo, hi]` as `(center, mf)` pairs.
    ///
    /// Triangles peak at each center and reach zero at the neighbouring centers (so the
    /// memberships sum to 1 inside the range); gaussians use `sigma = step / 2`.
    pub fn evenly_spaced(lo: f64, hi: f64, n: usize, shape: Shape) -> Result<Vec<(f64, Type1MF)>> {
        if n < 2 {
            return invalid("n must be ≥ 2");
        }
        if lo.is_nan() || hi.is_nan() || lo >= hi {
            return invalid(format!("need lo < hi, got [{}, {}]", lo, hi));
        }
        let step = (hi - lo) / (n - 1) as f64;
        (0..n)
            .map(|i| {
                let c = lo + i as f64 * step;
                let mf = match shape {
                    Shape::Triangle => triangle(c - step, c, c + step)?,
                    Shape::Gaussian => gaussian(c, step / 2.0)?,
                };
                Ok((c, mf))
            })
            .collect()
    }
}

/// Shape used by generated term sets ([`mf::evenly_spaced`], `Variable::automf`, FTS).
///
/// Names: `triangle` (`triangular`, `trimf`), `gaussian` (`gauss`, `gaussmf`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Shape {
    #[default]
    Triangle,
    Gaussian,
}

named_enum!(Shape, "shape", {
    Triangle => "triangle" | "triangular" | "trimf",
    Gaussian => "gaussian" | "gauss" | "gaussmf",
});

#[cfg(test)]
mod tests {
    use super::mf::*;
    use super::{Shape, IT2MF};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn triangle_shape() {
        let t = triangle(0.0, 5.0, 10.0).unwrap();
        assert_eq!(t.membership(-1.0), 0.0);
        assert_eq!(t.membership(0.0), 0.0);
        assert!(close(t.membership(2.5), 0.5));
        assert_eq!(t.membership(5.0), 1.0);
        assert!(close(t.membership(7.5), 0.5));
        assert_eq!(t.membership(10.0), 0.0);
        assert_eq!(t.membership(11.0), 0.0);
        assert_eq!(t.center(), 5.0);
    }

    #[test]
    fn shoulders_reach_one_at_their_edge() {
        let left = triangle(0.0, 0.0, 20.0).unwrap();
        let right = triangle(20.0, 40.0, 40.0).unwrap();
        assert_eq!(left.membership(0.0), 1.0);
        assert!(close(left.membership(10.0), 0.5));
        assert_eq!(right.membership(40.0), 1.0);

        let left = trapezoid(0.0, 0.0, 10.0, 20.0).unwrap();
        let right = trapezoid(0.0, 10.0, 20.0, 20.0).unwrap();
        assert_eq!(left.membership(0.0), 1.0);
        assert_eq!(right.membership(20.0), 1.0);
        assert!(close(right.membership(5.0), 0.5));
    }

    #[test]
    fn trapezoid_shape() {
        let t = trapezoid(0.0, 2.0, 4.0, 6.0).unwrap();
        assert_eq!(t.membership(0.0), 0.0);
        assert!(close(t.membership(1.0), 0.5));
        assert_eq!(t.membership(3.0), 1.0);
        assert!(close(t.membership(5.0), 0.5));
        assert_eq!(t.membership(6.0), 0.0);
        assert_eq!(t.center(), 3.0);
    }

    #[test]
    fn smooth_shapes() {
        let g = gaussian(5.0, 2.0).unwrap();
        assert_eq!(g.membership(5.0), 1.0);
        assert!(close(g.membership(7.0), (-0.5f64).exp()));

        let b = bell(0.0, 2.0, 3.0).unwrap();
        assert_eq!(b.membership(0.0), 1.0);
        assert!(close(b.membership(2.0), 0.5));

        let s = sigmoid(0.0, 1.0);
        assert!(close(s.membership(0.0), 0.5));
        assert!(sigmoid(0.0, -1.0).membership(5.0) < 0.01);

        assert_eq!(singleton(3.0).membership(3.0), 1.0);
        assert_eq!(singleton(3.0).membership(3.1), 0.0);
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        assert!(triangle(5.0, 0.0, 10.0).is_err());
        assert!(trapezoid(0.0, 3.0, 2.0, 4.0).is_err());
        assert!(gaussian(0.0, 0.0).is_err());
        assert!(gaussian(0.0, f64::NAN).is_err());
        assert!(bell(0.0, -1.0, 1.0).is_err());
        assert!(bell(0.0, 1.0, 0.0).is_err());
    }

    #[test]
    fn it2_interval() {
        let m = it2(
            triangle(0.0, 5.0, 10.0).unwrap(),
            triangle(2.0, 5.0, 8.0).unwrap(),
        )
        .unwrap();
        let (l, u) = m.membership(3.0);
        assert!(close(l, 1.0 / 3.0));
        assert!(close(u, 0.6));
        assert_eq!(m.center(), 5.0);
    }

    #[test]
    fn it2_checks_order_and_scales_lower() {
        let wide = gaussian(5.0, 3.0).unwrap();
        let narrow = gaussian(5.0, 1.0).unwrap();
        let err = it2(narrow, wide).unwrap_err().to_string();
        assert!(err.contains("swapped"), "{}", err);
        let m = IT2MF::with_lower_height(wide, narrow, 0.5).unwrap();
        assert_eq!(m.membership(5.0), (0.5, 1.0));
        assert!(IT2MF::with_lower_height(wide, narrow, 0.0).is_err());
    }

    #[test]
    fn display_and_evenly_spaced() {
        assert_eq!(
            triangle(0.0, 2.5, 5.0).unwrap().to_string(),
            "triangle(0, 2.5, 5)"
        );
        let terms = evenly_spaced(0.0, 10.0, 3, Shape::Triangle).unwrap();
        assert_eq!(
            terms.iter().map(|t| t.0).collect::<Vec<_>>(),
            [0.0, 5.0, 10.0]
        );
        let sum: f64 = terms.iter().map(|(_, m)| m.membership(3.0)).sum();
        assert!((sum - 1.0).abs() < 1e-12);
        assert!(evenly_spaced(1.0, 1.0, 3, Shape::Gaussian).is_err());
        assert_eq!("Gauss".parse::<Shape>().unwrap(), Shape::Gaussian);
    }

    #[test]
    fn memberships_stay_in_unit_interval() {
        let shapes = [
            triangle(-3.0, 0.0, 4.0).unwrap(),
            trapezoid(-3.0, -1.0, 1.0, 4.0).unwrap(),
            gaussian(0.5, 1.5).unwrap(),
            bell(0.0, 1.0, 2.0).unwrap(),
            sigmoid(0.0, 3.0),
            singleton(0.0),
        ];
        for mf in shapes {
            for i in -100..=100 {
                let mu = mf.membership(i as f64 / 10.0);
                assert!((0.0..=1.0).contains(&mu), "{} gave {}", mf.shape(), mu);
            }
        }
    }
}
