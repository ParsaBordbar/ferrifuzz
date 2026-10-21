"""ferrifuzz.fts — First-order (FOFTS) and high-order (HOFTS) Fuzzy Time Series.

Chen (1996): partition the universe into evenly spaced fuzzy sets, map each value to
its best-matching set, learn fuzzy logical relationship groups (FLRGs), and forecast
by averaging the centers of the matching group's consequents.

>>> from ferrifuzz.fts import FuzzyTimeSeries
>>> model = FuzzyTimeSeries(n_partitions=7, order=1).fit(train)
>>> model.predict(train[-1:], steps=3)       # three steps ahead
>>> model.evaluate(test_with_history)        # {'rmse': ..., 'mae': ..., 'mape': ...}
"""

from __future__ import annotations

import math
from typing import Dict, List, Optional, Sequence, Tuple

import ferrifuzz as ff
from . import metrics

_SHAPES = ("triangle", "gaussian")
_PARTITIONERS = ("equal_width",)


class Partition:
    """A single fuzzy linguistic term in the FTS universe."""

    def __init__(self, label: str, mf: ff.Type1MF, center: float) -> None:
        self.label = label
        self.mf = mf
        self.center = center

    def membership(self, x: float) -> float:
        """Return the membership degree of ``x`` in this partition."""
        return self.mf.membership(x)

    def __repr__(self) -> str:
        return f"Partition({self.label!r}, center={self.center:.4f})"


def _equal_width_partitions(
    lo: float,
    hi: float,
    n: int,
    mf_type: str = "triangle",
) -> List[Partition]:
    """Return *n* equal-width overlapping triangular (or Gaussian) partitions."""
    if n < 2:
        raise ValueError("n must be ≥ 2")
    if not lo < hi:
        raise ValueError(f"need lo < hi, got [{lo}, {hi}]")

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


class FLRG:
    """Fuzzy Logical Rule Group — maps an antecedent index tuple to its consequents."""

    def __init__(self, antecedent: Tuple[int, ...]) -> None:
        self.antecedent = antecedent
        self.consequents: List[int] = []

    def add_consequent(self, idx: int) -> None:
        """Register a consequent if not already present."""
        if idx not in self.consequents:
            self.consequents.append(idx)

    def __repr__(self) -> str:
        lhs = ", ".join(f"A{i + 1}" for i in self.antecedent)
        rhs = ", ".join(f"A{i + 1}" for i in self.consequents)
        return f"FLRG({lhs} -> {rhs})"


class FuzzyTimeSeries:
    """Chen (1996) Fuzzy Time Series — partition, fuzzify, extract FLRGs, forecast.

    Parameters
    ----------
    n_partitions : int
        Number of fuzzy sets (≥ 2), labelled ``A1..An`` from low to high.
    order : int
        How many past steps form a rule's antecedent (1 = first order).
    mf_type : {"triangle", "gaussian"}
        Partition shape. Values are assigned to their highest-membership set and both
        shapes peak at the same centers, so the shape does not change the model — it
        only matters for plots.
    partitioner : {"equal_width"}
        Partitioning scheme.
    margin : float
        Fraction of the data range added on each side of the universe (default 0.1).
        Ignored when ``universe`` is given.
    universe : (lo, hi), optional
        Partition exactly this range, e.g. ``(13000, 20000)`` for Chen's enrollment example.

    Values outside the universe are mapped to the nearest partition.
    """

    def __init__(
        self,
        n_partitions: int = 7,
        order: int = 1,
        mf_type: str = "triangle",
        partitioner: str = "equal_width",
        margin: float = 0.1,
        universe: Optional[Tuple[float, float]] = None,
    ) -> None:
        if n_partitions < 2:
            raise ValueError("n_partitions must be ≥ 2")
        if order < 1:
            raise ValueError("order must be ≥ 1")
        if mf_type not in _SHAPES:
            raise ValueError(f"unknown mf_type {mf_type!r}; valid options: {', '.join(map(repr, _SHAPES))}")
        if partitioner not in _PARTITIONERS:
            raise ValueError(
                f"unknown partitioner {partitioner!r}; valid options: {', '.join(map(repr, _PARTITIONERS))}"
            )
        if universe is not None and not universe[0] < universe[1]:
            raise ValueError(f"universe needs lo < hi, got {universe}")

        self.n_partitions = n_partitions
        self.order = order
        self.mf_type = mf_type
        self.partitioner = partitioner
        self.margin = margin
        self.universe = universe

        self._partitions: List[Partition] = []
        self._flrgs: Dict[Tuple[int, ...], FLRG] = {}
        self._fitted = False
        self._lo: float = 0.0
        self._hi: float = 1.0

    def fit(self, series: Sequence[float]) -> "FuzzyTimeSeries":
        """Fit to a training series (length must exceed ``order``); returns self."""
        series = list(series)
        if len(series) <= self.order:
            raise ValueError(
                f"series length ({len(series)}) must be > order ({self.order})"
            )

        for i, x in enumerate(series):
            if not math.isfinite(x):
                raise ValueError(f"series[{i}] is {x}; remove or fill missing values before fit()")
        if self.universe is not None:
            self._lo, self._hi = float(self.universe[0]), float(self.universe[1])
        else:
            lo, hi = min(series), max(series)
            rng = hi - lo
            if rng == 0:
                rng = 1.0
            pad = rng * self.margin
            if pad == 0 and lo == hi:  # constant series with margin 0
                pad = 0.5
            self._lo = lo - pad
            self._hi = hi + pad

        self._partitions = _equal_width_partitions(
            self._lo, self._hi, self.n_partitions, self.mf_type
        )

        fuzz = self._fuzzify_series(series)

        self._flrgs = {}
        for t in range(self.order, len(fuzz)):
            antecedent = tuple(fuzz[t - self.order : t])
            consequent = fuzz[t]
            if antecedent not in self._flrgs:
                self._flrgs[antecedent] = FLRG(antecedent)
            self._flrgs[antecedent].add_consequent(consequent)

        self._fitted = True
        return self

    def predict(self, history: Sequence[float], steps: int = 1) -> List[float]:
        """Forecast ``steps`` values ahead from the end of ``history``."""
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
        """One-step-ahead forecasts over a series, aligned to ``series[order:]``."""
        self._check_fitted()
        series = list(series)
        fuzz = self._fuzzify_series(series)
        return [
            self._forecast_one(fuzz[t - self.order : t])
            for t in range(self.order, len(fuzz))
        ]

    def evaluate(self, series: Sequence[float]) -> Dict[str, float]:
        """One-step-ahead RMSE / MAE / MAPE over ``series`` (compared with ``series[order:]``).

        Prepend the last ``order`` training values to a test set to score every test point.
        """
        series = list(series)
        return metrics.score(series[self.order :], self.forecast(series))

    def get_flrgs(self) -> List[FLRG]:
        """Return all Fuzzy Logical Rule Groups discovered during fitting."""
        self._check_fitted()
        return list(self._flrgs.values())

    def get_partitions(self) -> List[Partition]:
        """Return the fuzzy partitions (linguistic terms), ordered low to high."""
        self._check_fitted()
        return list(self._partitions)

    def summary(self) -> str:
        """Return a human-readable summary of the fitted model."""
        self._check_fitted()
        lines = [
            repr(self),
            f"  Universe    : [{self._lo:.4f}, {self._hi:.4f}]",
            f"  Partitions  : {self.n_partitions}  ({self.mf_type})",
            f"  FLRGs found : {len(self._flrgs)}",
        ]
        return "\n".join(lines)

    def _fuzzify_one(self, x: float) -> int:
        """Index of the partition with highest membership at x; nearest center if none."""
        best_i, best_mu = 0, 0.0
        for i, p in enumerate(self._partitions):
            mu = p.membership(x)
            if mu > best_mu:
                best_mu = mu
                best_i = i
        if best_mu > 0.0:
            return best_i
        return min(range(len(self._partitions)), key=lambda i: abs(self._partitions[i].center - x))

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
