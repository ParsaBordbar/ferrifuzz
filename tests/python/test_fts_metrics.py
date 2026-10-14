import math

import pytest

from ferrifuzz import metrics
from ferrifuzz.fts import FuzzyTimeSeries


def test_periodic_series_is_learned_exactly():
    series = [[0.0, 5.0, 10.0][i % 3] for i in range(30)]
    m = FuzzyTimeSeries(n_partitions=3, order=1, margin=0.0).fit(series)
    assert len(m.get_flrgs()) == 3
    assert m.predict([0.0], steps=4) == [5.0, 10.0, 0.0, 5.0]
    assert m.forecast(series) == series[1:]


def test_high_order_and_errors():
    m = FuzzyTimeSeries(n_partitions=5, order=2)
    with pytest.raises(RuntimeError):
        m.predict([1.0, 2.0])
    m.fit([1, 2, 3, 2, 1, 2, 3, 2, 1])
    assert all(len(g.antecedent) == 2 for g in m.get_flrgs())
    with pytest.raises(ValueError):
        m.predict([1.0])
    with pytest.raises(ValueError):
        FuzzyTimeSeries(n_partitions=1)


def test_values_outside_universe_use_nearest_partition():
    data = [13055, 13563, 13867, 14696, 15460, 15311, 15603, 15861, 16807, 16919, 16388,
            15433, 15497, 15145, 15163, 15984, 16859, 18150, 18970, 19328, 19337, 18876]
    m = FuzzyTimeSeries(n_partitions=7).fit(data[:16])
    assert m.predict([19337])[0] > 16000


def test_universe_evaluate_and_validation():
    data = [1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0]
    m = FuzzyTimeSeries(n_partitions=3, universe=(0, 4)).fit(data)
    assert [p.center for p in m.get_partitions()] == [0.0, 2.0, 4.0]
    s = m.evaluate(data)
    assert set(s) == {"rmse", "mae", "mape"}
    with pytest.raises(ValueError):
        FuzzyTimeSeries().fit([1.0, float("nan"), 2.0])
    with pytest.raises(ValueError, match="mf_type"):
        FuzzyTimeSeries(mf_type="trapezoid")
    with pytest.raises(ValueError, match="partitioner"):
        FuzzyTimeSeries(partitioner="cmeans")


def test_metrics():
    t, p = [1.0, 2.0, 4.0], [1.0, 3.0, 2.0]
    assert metrics.rmse(t, p) == pytest.approx(math.sqrt(5 / 3))
    assert metrics.mae(t, p) == 1.0
    assert metrics.mape(t, p) == pytest.approx(100 * 1.0 / 3)
    assert set(metrics.score(t, p)) == {"rmse", "mae", "mape"}
    assert math.isnan(metrics.rmse([], []))
