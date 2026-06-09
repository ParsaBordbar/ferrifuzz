"""
ferrifuzz.fts — Fuzzy Time Series forecasting
==============================================

Implements First-Order (FOFTS) and High-Order (HOFTS) Fuzzy Time Series
following the Chen (1996) / Song & Chissom framework.

Quick start
-----------
>>> from ferrifuzz.fts import FuzzyTimeSeries
>>> model = FuzzyTimeSeries(n_partitions=7, order=1)
>>> model.fit(train_series)
>>> preds = model.predict(history=train_series[-3:], steps=5)

>>> from ferrifuzz.fts import FuzzyTimeSeries
>>> hofts = FuzzyTimeSeries(n_partitions=9, order=3)
>>> hofts.fit(train_series)
>>> one_step_forecasts = hofts.forecast(test_series)
"""

from __future__ import annotations

import math
from collections import defaultdict
from typing import Dict, List, Optional, Sequence, Tuple

import ferrifuzz as ff


# ─── Partition (one linguistic term) ─────────────────────────────────────────

class Partition:
    """A single fuzzy linguistic term in the FTS universe.

    Attributes
    ----------
    label : str
        Name of this partition (e.g. ``"A3"``).
    mf : Type1MF
        Membership function (triangular or Gaussian).
    center : float
        Peak/centroid used during defuzzification.
    """

    def __init__(self, label: str, mf, center: float) -> None:
        self.label = label
        self.mf = mf
        self.center = center

    def membership(self, x: float) -> float:
        """Return the membership degree of ``x`` in this partition."""
        return self.mf.membership(x)

    def __repr__(self) -> str:
        return f"Partition({self.label!r}, center={self.center:.4f})"


# ─── Partitioning strategies ──────────────────────────────────────────────────

def _equal_width_partitions(
    lo: float,
    hi: float,
    n: int,
    mf_type: str = "triangle",
) -> List[Partition]:
    """Return *n* equal-width overlapping triangular (or Gaussian) partitions.

    Parameters
    ----------
    lo, hi : float
        Universe of discourse boundaries.
    n : int
        Number of partitions (≥ 2).
    mf_type : str
        ``"triangle"`` (default) or ``"gaussian"``.
    """
    if n < 2:
        raise ValueError("n must be ≥ 2")

    step = (hi - lo) / (n - 1)
    partitions: List[Partition] = []

    for i in range(n):
        center = lo + i * step
        label = f"A{i + 1}"

        if mf_type == "gaussian":
            sigma = step / 2.0 if step > 0 else 1e-6
            mf = ff.mf.gaussian(center, sigma)
        else:
            a = center - step
            b = center
            c = center + step
            # Clamp so extreme partitions don't violate a ≤ b ≤ c
            if a >= b:
                a = b - 1e-9
            if c <= b:
                c = b + 1e-9
            mf = ff.mf.triangle(a, b, c)

        partitions.append(Partition(label, mf, center))

    return partitions


# ─── FLRG ─────────────────────────────────────────────────────────────────────

class FLRG:
    """Fuzzy Logical Rule Group.

    Maps a tuple of antecedent partition indices to a list of possible
    consequent partition indices discovered in training.

    Attributes
    ----------
    antecedent : tuple[int, ...]
        Ordered partition indices for the antecedent (length = model order).
    consequents : list[int]
        Partition indices that followed this antecedent in training data.
    """

    def __init__(self, antecedent: Tuple[int, ...]) -> None:
        self.antecedent = antecedent
        self.consequents: List[int] = []

    def add_consequent(self, idx: int) -> None:
        """Register a consequent if not already present."""
        if idx not in self.consequents:
            self.consequents.append(idx)

    def __repr__(self) -> str:
        return f"FLRG({self.antecedent} → {self.consequents})"


# ─── FuzzyTimeSeries ──────────────────────────────────────────────────────────

class FuzzyTimeSeries:
    """First-Order or High-Order Fuzzy Time Series model.

    Follows the Chen (1996) / Song & Chissom approach:
    partition the universe → fuzzify → extract FLRGs → forecast.

    Parameters
    ----------
    n_partitions : int
        Number of fuzzy sets that partition the universe (default 7).
    order : int
        Number of previous time steps used as the antecedent (default 1).
        order=1 → FOFTS, order≥2 → HOFTS.
    mf_type : str
        Shape of membership functions: ``"triangle"`` (default) or
        ``"gaussian"``.
    partitioner : str
        Partitioning strategy. Currently ``"equal_width"`` (default).
    margin : float
        Fraction of the data range added to both ends of the universe
        (default 0.1 = 10 %).

    Examples
    --------
    >>> model = FuzzyTimeSeries(n_partitions=7, order=1)
    >>> model.fit(train)
    >>> preds = model.predict(train[-1:], steps=10)

    >>> hofts = FuzzyTimeSeries(n_partitions=9, order=3)
    >>> hofts.fit(train)
    >>> forecasts = hofts.forecast(test)   # one-step-ahead over test set
    """

    def __init__(
        self,
        n_partitions: int = 7,
        order: int = 1,
        mf_type: str = "triangle",
        partitioner: str = "equal_width",
        margin: float = 0.1,
    ) -> None:
        if n_partitions < 2:
            raise ValueError("n_partitions must be ≥ 2")
        if order < 1:
            raise ValueError("order must be ≥ 1")

        self.n_partitions = n_partitions
        self.order = order
        self.mf_type = mf_type
        self.partitioner = partitioner
        self.margin = margin

        self._partitions: List[Partition] = []
        self._flrgs: Dict[Tuple[int, ...], FLRG] = {}
        self._fitted = False
        self._lo: float = 0.0
        self._hi: float = 1.0

    # ── fitting ───────────────────────────────────────────────────────────────

    def fit(self, series: Sequence[float]) -> "FuzzyTimeSeries":
        """Fit the model to a training time series.

        Parameters
        ----------
        series : sequence of float
            Training observations. Length must be > ``order``.

        Returns
        -------
        self
            Returns the fitted model (chainable).
        """
        series = list(series)
        if len(series) <= self.order:
            raise ValueError(
                f"series length ({len(series)}) must be > order ({self.order})"
            )

        lo, hi = min(series), max(series)
        rng = hi - lo
        if rng == 0:
            rng = 1.0
        self._lo = lo - rng * self.margin
        self._hi = hi + rng * self.margin

        # Build partitions
        self._partitions = _equal_width_partitions(
            self._lo, self._hi, self.n_partitions, self.mf_type
        )

        # Fuzzify the whole series
        fuzz = self._fuzzify_series(series)

        # Extract FLRGs
        self._flrgs = {}
        for t in range(self.order, len(fuzz)):
            antecedent = tuple(fuzz[t - self.order : t])
            consequent = fuzz[t]
            if antecedent not in self._flrgs:
                self._flrgs[antecedent] = FLRG(antecedent)
            self._flrgs[antecedent].add_consequent(consequent)

        self._fitted = True
        return self

    # ── prediction ────────────────────────────────────────────────────────────

    def predict(self, history: Sequence[float], steps: int = 1) -> List[float]:
        """Forecast ``steps`` steps ahead from the end of ``history``.

        Parameters
        ----------
        history : sequence of float
            Recent observed values. Must contain at least ``order`` points.
        steps : int
            Number of future values to forecast.

        Returns
        -------
        list[float]
            Forecasted values, length = ``steps``.

        Examples
        --------
        >>> preds = model.predict(history=series[-5:], steps=3)
        """
        self._check_fitted()
        if len(history) < self.order:
            raise ValueError(
                f"history must contain at least order={self.order} values"
            )

        fuzz_hist = self._fuzzify_series(list(history))
        predictions: List[float] = []

        for _ in range(steps):
            val = self._forecast_one(fuzz_hist)
            predictions.append(val)
            fuzz_hist.append(self._fuzzify_one(val))

        return predictions

    def forecast(self, series: Sequence[float]) -> List[float]:
        """Produce one-step-ahead forecasts over a full series.

        Uses only past in-sample observations — no look-ahead.
        Returns ``len(series) - order`` values, aligned to
        ``series[order:]``.

        Parameters
        ----------
        series : sequence of float
            Observations to forecast over (typically the test split).

        Returns
        -------
        list[float]
            Predicted values aligned to ``series[order:]``.

        Examples
        --------
        >>> y_pred = model.forecast(test_series)
        >>> y_true = test_series[model.order:]
        """
        self._check_fitted()
        series = list(series)
        fuzz = self._fuzzify_series(series)
        return [
            self._forecast_one(fuzz[t - self.order : t])
            for t in range(self.order, len(fuzz))
        ]

    # ── introspection ─────────────────────────────────────────────────────────

    def get_flrgs(self) -> List[FLRG]:
        """Return all Fuzzy Logical Rule Groups discovered during fitting.

        Returns
        -------
        list[FLRG]
            Each FLRG maps an antecedent tuple to a list of consequents.
        """
        self._check_fitted()
        return list(self._flrgs.values())

    def get_partitions(self) -> List[Partition]:
        """Return the list of fuzzy partitions (linguistic terms).

        Returns
        -------
        list[Partition]
            Ordered list of partitions from low to high.
        """
        self._check_fitted()
        return list(self._partitions)

    def summary(self) -> str:
        """Print a human-readable summary of the fitted model."""
        self._check_fitted()
        lines = [
            repr(self),
            f"  Universe    : [{self._lo:.4f}, {self._hi:.4f}]",
            f"  Partitions  : {self.n_partitions}  ({self.mf_type})",
            f"  FLRGs found : {len(self._flrgs)}",
        ]
        return "\n".join(lines)

    # ── internals ─────────────────────────────────────────────────────────────

    def _fuzzify_one(self, x: float) -> int:
        """Return the index of the partition with highest membership at x."""
        best_i, best_mu = 0, -1.0
        for i, p in enumerate(self._partitions):
            mu = p.membership(x)
            if mu > best_mu:
                best_mu = mu
                best_i = i
        return best_i

    def _fuzzify_series(self, series: List[float]) -> List[int]:
        return [self._fuzzify_one(x) for x in series]

    def _forecast_one(self, fuzz_window: List[int]) -> float:
        """Defuzzify: average center of consequent partitions for a window."""
        antecedent = tuple(fuzz_window[-self.order :])

        if antecedent in self._flrgs:
            flrg = self._flrgs[antecedent]
            centers = [self._partitions[i].center for i in flrg.consequents]
            return sum(centers) / len(centers)

        # No matching FLRG — fall back to center of the last known partition
        last_idx = fuzz_window[-1]
        return self._partitions[last_idx].center

    def _check_fitted(self) -> None:
        if not self._fitted:
            raise RuntimeError(
                "Model is not fitted yet. Call fit() before predict() or forecast()."
            )

    def __repr__(self) -> str:
        status = (
            f"fitted, {len(self._flrgs)} FLRGs" if self._fitted else "not fitted"
        )
        return (
            f"FuzzyTimeSeries(n_partitions={self.n_partitions}, "
            f"order={self.order}, mf_type={self.mf_type!r}, {status})"
        )
