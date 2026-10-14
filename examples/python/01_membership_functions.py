"""
Example 01 — Membership Functions
==================================
Covers every MF shape available in ferrifuzz, both Type-1 and IT2.
"""

import ferrifuzz as ff

# ── 1. Triangular ─────────────────────────────────────────────────────────────
tri = ff.mf.triangle(0.0, 5.0, 10.0)
print("=== Triangular (0, 5, 10) ===")
print(f"  x=0  → {tri.membership(0.0):.2f}")   # 0.00  (left foot)
print(f"  x=2  → {tri.membership(2.0):.2f}")   # 0.40
print(f"  x=5  → {tri.membership(5.0):.2f}")   # 1.00  (peak)
print(f"  x=8  → {tri.membership(8.0):.2f}")   # 0.40
print(f"  x=10 → {tri.membership(10.0):.2f}")  # 0.00  (right foot)

# Right-shoulder trick: set b == c to get a ramp that stays at 1.0
hot = ff.mf.triangle(20.0, 40.0, 40.0)
print("\n=== Right-shoulder triangle (20, 40, 40) — 'hot' ===")
print(f"  20°C → {hot.membership(20.0):.2f}")  # 0.00
print(f"  30°C → {hot.membership(30.0):.2f}")  # 0.50
print(f"  40°C → {hot.membership(40.0):.2f}")  # 1.00

# ── 2. Trapezoidal ────────────────────────────────────────────────────────────
trap = ff.mf.trapezoid(10.0, 30.0, 70.0, 90.0)
print("\n=== Trapezoidal (10, 30, 70, 90) ===")
print(f"  x=10 → {trap.membership(10.0):.2f}")   # 0.00
print(f"  x=20 → {trap.membership(20.0):.2f}")   # 0.50
print(f"  x=50 → {trap.membership(50.0):.2f}")   # 1.00  (plateau)
print(f"  x=80 → {trap.membership(80.0):.2f}")   # 0.50
print(f"  x=90 → {trap.membership(90.0):.2f}")   # 0.00

# ── 3. Gaussian ───────────────────────────────────────────────────────────────
gauss = ff.mf.gaussian(center=50.0, sigma=10.0)
print("\n=== Gaussian (center=50, σ=10) ===")
print(f"  x=50 → {gauss.membership(50.0):.4f}")   # 1.0000  (peak)
print(f"  x=60 → {gauss.membership(60.0):.4f}")   # 0.6065  (1σ away)
print(f"  x=70 → {gauss.membership(70.0):.4f}")   # 0.1353  (2σ away)

# ── 4. Singleton ──────────────────────────────────────────────────────────────
single = ff.mf.singleton(42.0)
print("\n=== Singleton (x=42) ===")
print(f"  x=42   → {single.membership(42.0):.1f}")   # 1.0
print(f"  x=42.1 → {single.membership(42.1):.1f}")   # 0.0
print(f"  x=41.9 → {single.membership(41.9):.1f}")   # 0.0

# ── 5. Bell ───────────────────────────────────────────────────────────────────
bell = ff.mf.bell(center=50.0, width=15.0, slope=2.0)
print("\n=== Bell (center=50, width=15, slope=2) ===")
print(f"  x=50 → {bell.membership(50.0):.4f}")   # 1.0000
print(f"  x=65 → {bell.membership(65.0):.4f}")   # 0.5000  (half-width point)
print(f"  x=35 → {bell.membership(35.0):.4f}")   # 0.5000

# ── 6. Sigmoid ────────────────────────────────────────────────────────────────
sig = ff.mf.sigmoid(center=50.0, slope=0.3)
print("\n=== Sigmoid (center=50, slope=0.3) ===")
print(f"  x=50 → {sig.membership(50.0):.4f}")   # 0.5000
print(f"  x=60 → {sig.membership(60.0):.4f}")   # approaches 1
print(f"  x=40 → {sig.membership(40.0):.4f}")   # approaches 0

# ── 7. Accessing the shape name ───────────────────────────────────────────────
for name, mf in [("triangle", tri), ("trapezoid", trap), ("gaussian", gauss),
                  ("singleton", single), ("bell", bell), ("sigmoid", sig)]:
    print(f"  {name:12s} → shape = '{mf.shape()}'")

# ── 8. Direct class construction (for isinstance checks) ─────────────────────
raw_tri = ff.TriangularMF(0.0, 5.0, 10.0)
print(f"\nDirect TriangularMF(0, 5, 10).membership(5) = {raw_tri.membership(5.0)}")
print(f"isinstance check: {isinstance(raw_tri, ff.TriangularMF)}")  # True

# ── 9. Interval Type-2 (IT2) MF ──────────────────────────────────────────────
upper = ff.mf.triangle(0.0, 5.0, 10.0)    # wider upper bound
lower = ff.mf.triangle(2.0, 5.0, 8.0)     # narrower lower bound
it2   = ff.mf.it2(upper, lower)

print("\n=== IT2 MF ===")
# Returns (lower_membership, upper_membership) at x
print(f"  x=5  → {it2.membership(5.0)}")    # (1.0, 1.0)  — peak
print(f"  x=3  → {it2.membership(3.0)}")    # (0.5, 0.6)
print(f"  x=9  → {it2.membership(9.0)}")    # (lower < upper)
print(f"  shapes → {it2.shapes()}")         # ('Triangular', 'Triangular')
