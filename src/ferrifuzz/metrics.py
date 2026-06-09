"""
ferrifuzz.metrics — Forecasting error metrics
==============================================

All functions accept any ``Sequence[float]`` (list, tuple, numpy array, etc.)
and return plain Python floats.

Examples
--------
>>> from ferrifuzz.metrics import score
>>> report = score(y_true, y_pred)
>>> print(report)
{'rmse': 1.23, 'mae': 0.97, 'mape': 4.51}
"""

from __future__ import annotations

import math
from typing import Dict, Sequence


def rmse(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    """Root Mean Square Error.

    Parameters
    ----------
    y_true : sequence of float
        Ground-truth observations.
    y_pred : sequence of float
        Model predictions, same length as ``y_true``.

    Returns
    -------
    float
        RMSE ≥ 0.  Lower is better.

    Examples
    --------
    >>> rmse([1, 2, 3], [1.1, 2.2, 2.8])
    0.163...
    """
    pairs = list(zip(y_true, y_pred))
    if not pairs:
        return float("nan")
    return math.sqrt(sum((a - b) ** 2 for a, b in pairs) / len(pairs))


def mae(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    """Mean Absolute Error.

    Parameters
    ----------
    y_true : sequence of float
        Ground-truth observations.
    y_pred : sequence of float
        Model predictions, same length as ``y_true``.

    Returns
    -------
    float
        MAE ≥ 0.  Lower is better.

    Examples
    --------
    >>> mae([1, 2, 3], [1.1, 2.2, 2.8])
    0.166...
    """
    pairs = list(zip(y_true, y_pred))
    if not pairs:
        return float("nan")
    return sum(abs(a - b) for a, b in pairs) / len(pairs)


def mape(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    """Mean Absolute Percentage Error (%).

    Pairs where ``y_true == 0`` are silently excluded to avoid
    division-by-zero.  If all true values are zero, returns ``nan``.

    Parameters
    ----------
    y_true : sequence of float
        Ground-truth observations.
    y_pred : sequence of float
        Model predictions, same length as ``y_true``.

    Returns
    -------
    float
        MAPE in percent (0–100+).  Lower is better.

    Examples
    --------
    >>> mape([100, 200, 300], [110, 190, 310])
    5.0
    """
    valid = [(a, b) for a, b in zip(y_true, y_pred) if a != 0]
    if not valid:
        return float("nan")
    return 100.0 * sum(abs((a - b) / a) for a, b in valid) / len(valid)


def score(
    y_true: Sequence[float],
    y_pred: Sequence[float],
) -> Dict[str, float]:
    """Compute RMSE, MAE, and MAPE in a single call.

    Parameters
    ----------
    y_true : sequence of float
        Ground-truth observations.
    y_pred : sequence of float
        Model predictions, same length as ``y_true``.

    Returns
    -------
    dict with keys ``"rmse"``, ``"mae"``, ``"mape"``.

    Examples
    --------
    >>> from ferrifuzz.metrics import score
    >>> score(y_true, y_pred)
    {'rmse': 1.23, 'mae': 0.97, 'mape': 4.51}
    """
    return {
        "rmse": rmse(y_true, y_pred),
        "mae":  mae(y_true, y_pred),
        "mape": mape(y_true, y_pred),
    }
