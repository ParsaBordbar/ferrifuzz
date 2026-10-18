"""Ferrifuzz: fast fuzzy logic for Python, written in Rust.

Quick start::

    import ferrifuzz as ff

    temp = ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"])
    fan = ff.Variable("fan", 0, 100).automf(["slow", "medium", "fast"])

    system = ff.FuzzySystem()
    system.add_input(temp)
    system.add_output(fan)
    system += "IF temp IS cold THEN fan IS slow"
    system += "IF temp IS warm THEN fan IS medium"
    system += "IF temp IS hot THEN fan IS fast"

    system({"temp": 30})      # {'fan': 56.07...}

Submodules: ``ferrifuzz.fts`` (fuzzy time series), ``ferrifuzz.metrics`` (RMSE/MAE/MAPE),
``ferrifuzz.plot`` (matplotlib helpers, optional).
"""

from typing import TYPE_CHECKING, Any

from ._core import (
    FuzzySystem,
    Variable,
    Rule,
    Clause,
    Operator,
    Hedge,
    mf,
    Type1MF,
    IT2MF,
    TriangularMF,
    TrapezoidalMF,
    GaussianMF,
    SingletonMF,
    BellMF,
    SigmoidMF,
    FuzzyError,
    InputError,
    ModelError,
    RuleSyntaxError,
)

from . import fts
from . import metrics

if TYPE_CHECKING:
    # Editors and type checkers see `ff.plot`; at runtime it loads lazily (below).
    from . import plot as plot

try:
    from importlib.metadata import version as _version

    __version__ = _version("ferrifuzz")
except Exception:  # pragma: no cover - running from a source tree without metadata
    __version__ = "unknown"


def __getattr__(name: str) -> Any:
    # `ff.plot` imports matplotlib only when first used.
    if name == "plot":
        import importlib

        return importlib.import_module(".plot", __name__)
    raise AttributeError(f"module 'ferrifuzz' has no attribute {name!r}")


__all__ = [
    "FuzzySystem",
    "Variable",
    "Rule",
    "Clause",
    "Operator",
    "Hedge",
    "mf",
    "Type1MF",
    "IT2MF",
    "TriangularMF",
    "TrapezoidalMF",
    "GaussianMF",
    "SingletonMF",
    "BellMF",
    "SigmoidMF",
    "FuzzyError",
    "InputError",
    "ModelError",
    "RuleSyntaxError",
    "fts",
    "metrics",
    "plot",
]
