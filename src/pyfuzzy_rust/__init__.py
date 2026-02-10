# src/pyfuzzy_rust/__init__.py
"""
PyFuzzy-Rust: High-performance fuzzy logic library for Python, written in Rust.
"""

from pyfuzzy_rust._core import (
    TriangularMF,
    TrapezoidalMF,
    GaussianMF,
    Type1MF,
    IT2MF,
    Variable,
    Rule,
)

__all__ = [
    "TriangularMF",
    "TrapezoidalMF",
    "GaussianMF",
    "Type1MF",
    "IT2MF",
    "Variable",
    "Rule",
]

__version__ = "0.1.0"
