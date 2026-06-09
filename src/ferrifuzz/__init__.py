"""
Ferrifuzz: High-performance fuzzy logic library for Python, written in Rust.

Quick start
-----------
>>> import ferrifuzz as ff
>>>
>>> # Membership functions
>>> hot = ff.mf.triangle(20.0, 35.0, 40.0)
>>> hot.membership(30.0)
>>>
>>> # Linguistic variable
>>> temp = ff.Variable("Temperature", 0.0, 100.0)
>>> temp["Cold"] = ff.mf.triangle(0.0, 0.0, 40.0)
>>> temp["Hot"]  = ff.mf.triangle(60.0, 100.0, 100.0)
>>> temp.fuzzify(75.0)
>>>
>>> # Fuzzy inference system
>>> system = ff.FuzzySystem()
>>> system.add_input(temp)
>>> ...
>>> system += ff.Rule.from_string("IF Temperature IS Hot THEN fan IS Fast")
>>> result = system({"Temperature": 80.0})
>>>
>>> # Fuzzy Time Series
>>> from ferrifuzz.fts import FuzzyTimeSeries
>>> model = FuzzyTimeSeries(n_partitions=7, order=1)
>>> model.fit(train_data)
>>> preds = model.predict(train_data[-3:], steps=5)
>>>
>>> # Metrics
>>> from ferrifuzz.metrics import score
>>> score(y_true, y_pred)
"""

from ._core import (
    # Inference system
    FuzzySystem,
    # Variables and rules
    Variable,
    Rule,
    # Operators and hedges
    Operator,
    Hedge,
    # MF submodule (ff.mf.triangle, etc.)
    mf,
    # Individual MF classes (for isinstance checks / type hints)
    Type1MF,
    IT2MF,
    TriangularMF,
    TrapezoidalMF,
    GaussianMF,
    SingletonMF,
    BellMF,
    SigmoidMF,
)

# Submodules — available as ff.fts and ff.metrics after import
from . import fts
from . import metrics

__version__ = "0.2.0"

__all__ = [
    # System
    "FuzzySystem",
    # Core
    "Variable",
    "Rule",
    "Operator",
    "Hedge",
    # MF factory
    "mf",
    # MF classes
    "Type1MF",
    "IT2MF",
    "TriangularMF",
    "TrapezoidalMF",
    "GaussianMF",
    "SingletonMF",
    "BellMF",
    "SigmoidMF",
    # Submodules
    "fts",
    "metrics",
]
