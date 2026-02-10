use pyo3::prelude::*;

pub trait MF {
    fn membership(&self, x: f64) -> f64;
}

#[pyclass]
#[derive(Clone)]
pub struct TriangularMF {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

#[derive(Clone)]
pub struct Term {
    pub name: String,
    pub mf: Type1MF,
}

#[pymethods]
impl TriangularMF {
    #[new]
    pub fn new(a: f64, b: f64, c: f64) -> PyResult<Self> {
        if !(a <= b && b <= c) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Must satisfy a ≤ b ≤ c",
            ));
        }
        Ok(Self { a, b, c })
    }

    pub fn membership(&self, x: f64) -> f64 {
        if x <= self.a || x >= self.c {
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

impl MF for TriangularMF {
    fn membership(&self, x: f64) -> f64 {
        self.membership(x)
    }
}


#[pyclass]
#[derive(Clone)]
pub struct TrapezoidalMF {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

#[pymethods]
impl TrapezoidalMF {
    #[new]
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> PyResult<Self> {
        if !(a <= b && b <= c && c <= d) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Must satisfy a ≤ b ≤ c ≤ d",
            ));
        }
        Ok(Self { a, b, c, d })
    }

    pub fn membership(&self, x: f64) -> f64 {
        if x <= self.a || x >= self.d {
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

impl MF for TrapezoidalMF {
    fn membership(&self, x: f64) -> f64 {
        self.membership(x)
    }
}


#[pyclass]
#[derive(Clone)]
pub struct GaussianMF {
    pub center: f64,
    pub sigma: f64,
}

#[pymethods]
impl GaussianMF {
    #[new]
    pub fn new(center: f64, sigma: f64) -> PyResult<Self> {
        if sigma <= 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Sigma must be positive",
            ));
        }
        Ok(Self { center, sigma })
    }

    pub fn membership(&self, x: f64) -> f64 {
        let d = (x - self.center) / self.sigma;
        (-0.5 * d * d).exp()
    }
}

impl MF for GaussianMF {
    fn membership(&self, x: f64) -> f64 {
        self.membership(x)
    }
}

// TYPE-1 MF ENUM (PYTHON-FRIENDLY)
#[pyclass]
#[derive(Clone)]
pub enum Type1MF {
    Triangular(TriangularMF),
    Trapezoidal(TrapezoidalMF),
    Gaussian(GaussianMF),
}

#[pymethods]
impl Type1MF {
    pub fn membership(&self, x: f64) -> f64 {
        match self {
            Self::Triangular(mf) => mf.membership(x),
            Self::Trapezoidal(mf) => mf.membership(x),
            Self::Gaussian(mf) => mf.membership(x),
        }
    }

    pub fn shape(&self) -> &'static str {
        match self {
            Self::Triangular(_) => "Triangular",
            Self::Trapezoidal(_) => "Trapezoidal",
            Self::Gaussian(_) => "Gaussian",
        }
    }
}

// INTERVAL TYPE-2 MF
#[pyclass]
#[derive(Clone)]
pub struct IT2MF {
    upper: Type1MF,
    lower: Type1MF,
}

#[pymethods]
impl IT2MF {
    #[new]
    pub fn new(upper: Type1MF, lower: Type1MF) -> Self {
        Self { upper, lower }
    }

    pub fn membership(&self, x: f64) -> (f64, f64) {
        (
            self.lower.membership(x),
            self.upper.membership(x),
        )
    }

    pub fn shapes(&self) -> (String, String) {
        (
            self.upper.shape().to_string(),
            self.lower.shape().to_string(),
        )
    }
}

// UNIFIED MEMBERSHIP VALUE
#[derive(Clone, Copy, Debug)]
pub enum MembershipValue {
    Type1(f64),
    IT2 { lower: f64, upper: f64 },
}

impl MembershipValue {
    pub fn crisp(&self) -> f64 {
        match self {
            Self::Type1(v) => *v,
            Self::IT2 { lower, upper } => (lower + upper) / 2.0,
        }
    }
}

// FINAL MF WRAPPER (TYPE-1 OR IT2)
#[derive(Clone)]
pub enum MFWrapper {
    Type1(Type1MF),
    IT2(IT2MF),
}

impl MFWrapper {
    pub fn membership(&self, x: f64) -> MembershipValue {
        match self {
            Self::Type1(mf) => MembershipValue::Type1(mf.membership(x)),
            Self::IT2(mf) => {
                let (l, u) = mf.membership(x);
                MembershipValue::IT2 { lower: l, upper: u }
            }
        }
    }

    pub fn is_it2(&self) -> bool {
        matches!(self, Self::IT2(_))
    }
}
