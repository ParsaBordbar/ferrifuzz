use pyo3::prelude::*;

// Triangular membership function
#[pyclass]
#[derive(Clone)]
pub struct TriangularMF {
    #[pyo3(get)]
    pub a: f64,
    #[pyo3(get)]
    pub b: f64,
    #[pyo3(get)]
    pub c: f64,
}

#[pymethods]
impl TriangularMF {
    #[new]
    pub fn new(a: f64, b: f64, c: f64) -> PyResult<Self> {
        if !(a <= b && b <= c) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Must satisfy a ≤ b ≤ c"
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

    fn __repr__(&self) -> String {
        format!("TriangularMF({}, {}, {})", self.a, self.b, self.c)
    }
}

// Trapezoidal membership function
#[pyclass]
#[derive(Clone)]
pub struct TrapezoidalMF {
    #[pyo3(get)]
    pub a: f64,
    #[pyo3(get)]
    pub b: f64,
    #[pyo3(get)]
    pub c: f64,
    #[pyo3(get)]
    pub d: f64,
}

#[pymethods]
impl TrapezoidalMF {
    #[new]
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> PyResult<Self> {
        if !(a <= b && b <= c && c <= d) {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Must satisfy a ≤ b ≤ c ≤ d"
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

    fn __repr__(&self) -> String {
        format!("TrapezoidalMF({}, {}, {}, {})", self.a, self.b, self.c, self.d)
    }
}

// Internal enum for holding different MF types
#[derive(Clone)]
pub enum MFType {
    Triangular(TriangularMF),
    Trapezoidal(TrapezoidalMF),
}

impl MFType {
    pub fn membership(&self, x: f64) -> f64 {
        match self {
            MFType::Triangular(mf) => mf.membership(x),
            MFType::Trapezoidal(mf) => mf.membership(x),
        }
    }
}

#[derive(Clone)]
pub struct Term {
    pub name: String,
    pub mf: MFType,
}