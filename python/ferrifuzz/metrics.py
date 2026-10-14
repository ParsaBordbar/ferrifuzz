"""ferrifuzz.metrics — forecasting error metrics over any ``Sequence[float]``."""

from __future__ import annotations

import math
from typing import Dict, Sequence


def rmse(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    """Root Mean Square Error — lower is better, ``nan`` on empty input."""
    pairs = list(zip(y_true, y_pred))
    if not pairs:
        return float("nan")
    return math.sqrt(sum((a - b) ** 2 for a, b in pairs) / len(pairs))


def mae(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    """Mean Absolute Error — lower is better, ``nan`` on empty input."""
    pairs = list(zip(y_true, y_pred))
    if not pairs:
        return float("nan")
    return sum(abs(a - b) for a, b in pairs) / len(pairs)


def mape(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    """Mean Absolute Percentage Error — zero-valued truths are skipped."""
    valid = [(a, b) for a, b in zip(y_true, y_pred) if a != 0]
    if not valid:
        return float("nan")
    return 100.0 * sum(abs((a - b) / a) for a, b in valid) / len(valid)


def score(
    y_true: Sequence[float],
    y_pred: Sequence[float],
) -> Dict[str, float]:
    """Compute RMSE, MAE, and MAPE in a single call."""
    return {
        "rmse": rmse(y_true, y_pred),
        "mae":  mae(y_true, y_pred),
        "mape": mape(y_true, y_pred),
    }
