"""
Example 05 — Fuzzy Time Series & Metrics
==========================================
First-order (FOFTS) and high-order (HOFTS) forecasting, plus every
metric in ferrifuzz.metrics.
"""

import math
import ferrifuzz as ff
from ferrifuzz.fts import FuzzyTimeSeries
from ferrifuzz.metrics import rmse, mae, mape, score

# ─────────────────────────────────────────────────────────────────────────────
# Synthetic dataset — a noisy sine wave
# ─────────────────────────────────────────────────────────────────────────────

N = 60
series = [50 + 30 * math.sin(2 * math.pi * i / 20) + (i % 7) * 0.5 for i in range(N)]
train, test = series[:48], series[48:]

# ─────────────────────────────────────────────────────────────────────────────
# 1. First-Order FTS (order=1, default)
# ─────────────────────────────────────────────────────────────────────────────

print("=== First-Order FTS ===")
fofts = FuzzyTimeSeries(n_partitions=9, order=1)
fofts.fit(train)

print(fofts.summary())

# One-step-ahead forecasts over the test split
y_pred_fo = fofts.forecast(test)
y_true    = test[fofts.order:]
print(f"\nTest ground truth : {[round(v, 2) for v in y_true]}")
print(f"FOFTS predictions : {[round(v, 2) for v in y_pred_fo]}")

# ─────────────────────────────────────────────────────────────────────────────
# 2. High-Order FTS (order=3)
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== High-Order FTS (order=3) ===")
hofts = FuzzyTimeSeries(n_partitions=9, order=3)
hofts.fit(train)

print(hofts.summary())

y_pred_ho = hofts.forecast(test)
y_true_ho = test[hofts.order:]
print(f"\nTest ground truth : {[round(v, 2) for v in y_true_ho]}")
print(f"HOFTS predictions : {[round(v, 2) for v in y_pred_ho]}")

# ─────────────────────────────────────────────────────────────────────────────
# 3. Multi-step ahead forecasting
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== Multi-step ahead (5 steps) ===")
history = train[-5:]   # last 5 known values
future  = fofts.predict(history=history, steps=5)
print(f"Seed history : {[round(v, 2) for v in history]}")
print(f"Next 5 steps : {[round(v, 2) for v in future]}")

# ─────────────────────────────────────────────────────────────────────────────
# 4. Gaussian MF partitioning
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== Gaussian partitioned FTS ===")
gauss_fts = FuzzyTimeSeries(n_partitions=7, order=1, mf_type="gaussian")
gauss_fts.fit(train)
print(gauss_fts.summary())

# ─────────────────────────────────────────────────────────────────────────────
# 5. Inspect fitted model
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== Partitions (FOFTS) ===")
for p in fofts.get_partitions():
    print(f"  {p}")

print(f"\n=== FLRGs (first 5) ===")
for flrg in list(fofts.get_flrgs())[:5]:
    print(f"  {flrg}")

# ─────────────────────────────────────────────────────────────────────────────
# 6. Metrics
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== Metrics ===")

# Individual metric functions
print(f"RMSE  : {rmse(y_true, y_pred_fo):.4f}")
print(f"MAE   : {mae(y_true, y_pred_fo):.4f}")
print(f"MAPE  : {mape(y_true, y_pred_fo):.2f}%")

# Convenience wrapper — all three at once
report = score(y_true, y_pred_fo)
print(f"\nscore() report: {report}")

# Compare FOFTS vs HOFTS (where test lengths match)
min_len = min(len(y_pred_fo), len(y_pred_ho))
print("\n=== FOFTS vs HOFTS comparison ===")
for label, preds, truth in [
    ("FOFTS", y_pred_fo[:min_len], y_true[:min_len]),
    ("HOFTS", y_pred_ho[:min_len], y_true_ho[:min_len]),
]:
    s = score(truth, preds)
    print(f"  {label}  RMSE={s['rmse']:.3f}  MAE={s['mae']:.3f}  MAPE={s['mape']:.2f}%")
