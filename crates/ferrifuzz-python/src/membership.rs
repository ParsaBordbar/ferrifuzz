use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use ferrifuzz::membership as core;
use ferrifuzz::MembershipFunction;

use crate::{map_values, to_array, to_py_err};

macro_rules! shape_class {
    ($py:ident, $core:ident, $name:literal, $doc:literal, ($($arg:ident),*), fallible) => {
        shape_class!(@class $py, $core, $name, $doc);
        #[pymethods]
        impl $py {
            #[new]
            fn new($($arg: f64),*) -> PyResult<Self> {
                core::$core::new($($arg),*).map(Self).map_err(to_py_err)
            }
            /// Degree of membership at `x` — a float, or an array for a sequence / array input.
            fn membership(&self, py: Python<'_>, x: &Bound<'_, PyAny>) -> PyResult<PyObject> {
                map_values(py, x, |v| self.0.membership(v))
            }
            fn __repr__(&self) -> String {
                core::Type1MF::from(self.0).to_string()
            }
        }
    };
    ($py:ident, $core:ident, $name:literal, $doc:literal, ($($arg:ident),*)) => {
        shape_class!(@class $py, $core, $name, $doc);
        #[pymethods]
        impl $py {
            #[new]
            fn new($($arg: f64),*) -> Self {
                Self(core::$core::new($($arg),*))
            }
            /// Degree of membership at `x` — a float, or an array for a sequence / array input.
            fn membership(&self, py: Python<'_>, x: &Bound<'_, PyAny>) -> PyResult<PyObject> {
                map_values(py, x, |v| self.0.membership(v))
            }
            fn __repr__(&self) -> String {
                core::Type1MF::from(self.0).to_string()
            }
        }
    };
    (@class $py:ident, $core:ident, $name:literal, $doc:literal) => {
        #[doc = $doc]
        #[pyclass(name = $name, module = "ferrifuzz")]
        #[derive(Clone)]
        pub struct $py(pub core::$core);
    };
}

shape_class!(
    PyTriangularMF,
    TriangularMF,
    "TriangularMF",
    "Triangular MF struct — feet at `a`/`c`, peak at `b`; prefer `ff.mf.triangle`.",
    (a, b, c),
    fallible
);
shape_class!(
    PyTrapezoidalMF,
    TrapezoidalMF,
    "TrapezoidalMF",
    "Trapezoidal MF struct — feet at `a`/`d`, plateau `b`..`c`; prefer `ff.mf.trapezoid`.",
    (a, b, c, d),
    fallible
);
shape_class!(
    PyGaussianMF,
    GaussianMF,
    "GaussianMF",
    "Gaussian MF struct — peaked at `center`, spread `sigma`; prefer `ff.mf.gaussian`.",
    (center, sigma),
    fallible
);
shape_class!(
    PySingletonMF,
    SingletonMF,
    "SingletonMF",
    "Singleton MF struct — μ = 1 at `x`, 0 elsewhere; prefer `ff.mf.singleton`.",
    (x)
);
shape_class!(
    PyBellMF,
    BellMF,
    "BellMF",
    "Generalized bell MF struct — `center`, `width`, `slope`; prefer `ff.mf.bell`.",
    (center, width, slope),
    fallible
);
shape_class!(
    PySigmoidMF,
    SigmoidMF,
    "SigmoidMF",
    "Sigmoid MF struct — inflection at `center`, signed `slope`; prefer `ff.mf.sigmoid`.",
    (center, slope)
);

/// Type-1 membership function — `.membership(x)`, `.center()`, `.shape()`.
#[pyclass(name = "Type1MF", module = "ferrifuzz")]
#[derive(Clone)]
pub struct PyType1MF(pub core::Type1MF);

#[pymethods]
impl PyType1MF {
    /// Degree of membership at `x` — a float, or an array for a sequence / array input.
    fn membership(&self, py: Python<'_>, x: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        map_values(py, x, |v| self.0.membership(v))
    }

    fn __call__(&self, py: Python<'_>, x: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        self.membership(py, x)
    }

    /// Peak / plateau midpoint / inflection point — the constant a zero-order Sugeno
    /// rule outputs. Not the centroid.
    fn center(&self) -> f64 {
        self.0.center()
    }

    /// Shape name, e.g. `"Triangular"`.
    fn shape(&self) -> &'static str {
        self.0.shape()
    }

    /// Points where the shape changes (feet, peaks, ±3σ, ...).
    fn key_points(&self) -> Vec<f64> {
        self.0.key_points()
    }

    fn __repr__(&self) -> String {
        self.0.to_string()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Interval type-2 MF — `membership(x)` returns the `(lower, upper)` interval.
#[pyclass(name = "IT2MF", module = "ferrifuzz")]
#[derive(Clone)]
pub struct PyIT2MF(pub core::IT2MF);

#[pymethods]
impl PyIT2MF {
    #[new]
    #[pyo3(signature = (upper, lower, lower_height=1.0))]
    fn new(upper: Type1Arg, lower: Type1Arg, lower_height: f64) -> PyResult<Self> {
        core::IT2MF::with_lower_height(upper.0, lower.0, lower_height)
            .map(Self)
            .map_err(to_py_err)
    }

    /// `(lower, upper)` at `x`; for a sequence / array, a pair of arrays.
    fn membership(&self, py: Python<'_>, x: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(v) = x.extract::<f64>() {
            return Ok(self.0.membership(v).into_py(py));
        }
        let xs: Vec<f64> = x.extract().map_err(|_| {
            PyTypeError::new_err("expected a float or a sequence / array of floats")
        })?;
        let (lo, up): (Vec<f64>, Vec<f64>) = xs.iter().map(|&v| self.0.membership(v)).unzip();
        Ok((to_array(py, lo)?, to_array(py, up)?).into_py(py))
    }

    fn __call__(&self, py: Python<'_>, x: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        self.membership(py, x)
    }

    /// Upper MF (UMF) — outer edge of the footprint of uncertainty.
    #[getter]
    fn upper(&self) -> PyType1MF {
        PyType1MF(self.0.upper())
    }

    /// Lower MF (LMF) — inner edge of the footprint of uncertainty (before `lower_height`).
    #[getter]
    fn lower(&self) -> PyType1MF {
        PyType1MF(self.0.lower())
    }

    /// Scale applied to the lower MF.
    #[getter]
    fn lower_height(&self) -> f64 {
        self.0.lower_height()
    }

    /// Midpoint of the upper and lower centers.
    fn center(&self) -> f64 {
        self.0.center()
    }

    fn shapes(&self) -> (&'static str, &'static str) {
        (self.0.upper().shape(), self.0.lower().shape())
    }

    fn __repr__(&self) -> String {
        if self.0.lower_height() == 1.0 {
            format!("it2({}, {})", self.0.upper(), self.0.lower())
        } else {
            format!(
                "it2({}, {}, lower_height={})",
                self.0.upper(),
                self.0.lower(),
                self.0.lower_height()
            )
        }
    }
}

/// A `Type1MF` or any raw shape class (`TriangularMF`, ...), accepted wherever an MF is expected.
pub struct Type1Arg(pub core::Type1MF);

impl<'py> FromPyObject<'py> for Type1Arg {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        macro_rules! try_extract {
            ($($t:ty),*) => {
                $(if let Ok(m) = ob.extract::<$t>() {
                    return Ok(Type1Arg(m.0.into()));
                })*
            };
        }
        if let Ok(m) = ob.extract::<PyType1MF>() {
            return Ok(Type1Arg(m.0));
        }
        try_extract!(
            PyTriangularMF,
            PyTrapezoidalMF,
            PyGaussianMF,
            PySingletonMF,
            PyBellMF,
            PySigmoidMF
        );
        Err(PyTypeError::new_err(format!(
            "expected a type-1 membership function (ff.mf.triangle/gaussian/...), got {}",
            ob.get_type().name()?
        )))
    }
}

/// Triangular MF — peak at `b`, zero at `a` and `c` (a ≤ b ≤ c). `a == b` / `b == c` give shoulders.
#[pyfunction]
pub fn triangle(a: f64, b: f64, c: f64) -> PyResult<PyType1MF> {
    core::mf::triangle(a, b, c)
        .map(PyType1MF)
        .map_err(to_py_err)
}

/// Trapezoidal MF — flat plateau between `b` and `c` (a ≤ b ≤ c ≤ d).
#[pyfunction]
pub fn trapezoid(a: f64, b: f64, c: f64, d: f64) -> PyResult<PyType1MF> {
    core::mf::trapezoid(a, b, c, d)
        .map(PyType1MF)
        .map_err(to_py_err)
}

/// Gaussian MF — bell curve peaked at `center` with spread `sigma` (> 0).
#[pyfunction]
pub fn gaussian(center: f64, sigma: f64) -> PyResult<PyType1MF> {
    core::mf::gaussian(center, sigma)
        .map(PyType1MF)
        .map_err(to_py_err)
}

/// Singleton MF — μ = 1 exactly at `x`, 0 elsewhere (Sugeno constant output).
#[pyfunction]
pub fn singleton(x: f64) -> PyType1MF {
    PyType1MF(core::mf::singleton(x))
}

/// Generalized bell MF — `1 / (1 + |(x - center)/width|^(2·slope))`, width/slope > 0.
#[pyfunction]
pub fn bell(center: f64, width: f64, slope: f64) -> PyResult<PyType1MF> {
    core::mf::bell(center, width, slope)
        .map(PyType1MF)
        .map_err(to_py_err)
}

/// Sigmoid MF — `1 / (1 + exp(-slope·(x - center)))`; slope > 0 rises, < 0 falls.
#[pyfunction]
pub fn sigmoid(center: f64, slope: f64) -> PyType1MF {
    PyType1MF(core::mf::sigmoid(center, slope))
}

/// Interval type-2 MF from an `upper` and `lower` type-1 MF; the lower MF is scaled by
/// `lower_height ∈ (0, 1]`. Raises if lower rises above upper (arguments swapped?).
#[pyfunction]
#[pyo3(signature = (upper, lower, lower_height=1.0))]
pub fn it2(upper: Type1Arg, lower: Type1Arg, lower_height: f64) -> PyResult<PyIT2MF> {
    PyIT2MF::new(upper, lower, lower_height)
}
