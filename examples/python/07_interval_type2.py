"""
Example 07 — Interval Type-2 Inference
========================================
Type-2 fuzzy sets model uncertainty *about the membership grade itself*: each
term is a band (a footprint of uncertainty, FOU) between a lower and an upper
membership function rather than a single curve.

Ferrifuzz runs a full IT2 Mamdani engine:

  1. Build IT2 terms with ``ff.mf.it2(upper, lower)``.
  2. Each rule fires an *interval* ``[lower, upper]`` strength.
  3. Karnik–Mendel type reduction collapses the aggregated FOU to a centroid
     interval; the crisp output is its midpoint.

The engine is selected with ``FuzzySystem(type="it2")``, or automatically
whenever a registered variable carries IT2 terms.
"""

import ferrifuzz as ff

# ─────────────────────────────────────────────────────────────────────────────
# 1. IT2 terms — upper (outer) and lower (inner) membership functions
# ─────────────────────────────────────────────────────────────────────────────

temp = ff.Variable("temp", 0.0, 100.0)
temp["hot"]  = ff.mf.it2(ff.mf.triangle(30.0, 70.0, 100.0), ff.mf.triangle(50.0, 70.0, 90.0))
temp["mild"] = ff.mf.it2(ff.mf.triangle(0.0, 30.0, 70.0),   ff.mf.triangle(10.0, 30.0, 50.0))

fan = ff.Variable("fan", 0.0, 100.0)
fan["fast"] = ff.mf.it2(ff.mf.triangle(40.0, 80.0, 100.0), ff.mf.triangle(60.0, 80.0, 100.0))
fan["slow"] = ff.mf.it2(ff.mf.triangle(0.0, 20.0, 60.0),   ff.mf.triangle(0.0, 20.0, 40.0))

# The membership of an IT2 term is the (lower, upper) interval at x.
print("=== IT2 term membership at temp=60 ===")
print("  hot :", temp["hot"].membership(60.0))
print("  mild:", temp["mild"].membership(60.0))

# ─────────────────────────────────────────────────────────────────────────────
# 2. Build the IT2 system
# ─────────────────────────────────────────────────────────────────────────────

# type="it2" forces the IT2 engine; it is also auto-selected because the
# variables carry IT2 terms, so type= could be omitted here.
system = ff.FuzzySystem(type="it2")
system.add_input(temp)
system.add_output(fan)
system += ff.Rule.from_string("IF temp IS hot THEN fan IS fast")
system += ff.Rule.from_string("IF temp IS mild THEN fan IS slow")

print("\n=== IT2 inference sweep ===")
print(f"  {'temp':>6}   fan")
for t in range(0, 101, 10):
    out = system.compute({"temp": float(t)})
    fan_val = out.get("fan", float("nan"))
    print(f"  {t:>6}   {fan_val:6.2f}")

# ─────────────────────────────────────────────────────────────────────────────
# 3. FOU width controls how much a term "blurs"
# ─────────────────────────────────────────────────────────────────────────────

# A zero-width FOU (upper == lower) is just a Type-1 set: the IT2 engine then
# reduces to ordinary centroid defuzzification.
crisp_temp = ff.Variable("temp", 0.0, 100.0)
crisp_temp["hot"]  = ff.mf.it2(ff.mf.triangle(40.0, 70.0, 100.0), ff.mf.triangle(40.0, 70.0, 100.0))
crisp_temp["mild"] = ff.mf.it2(ff.mf.triangle(0.0, 30.0, 60.0),   ff.mf.triangle(0.0, 30.0, 60.0))
crisp_fan = ff.Variable("fan", 0.0, 100.0)
crisp_fan["fast"] = ff.mf.it2(ff.mf.triangle(50.0, 80.0, 100.0), ff.mf.triangle(50.0, 80.0, 100.0))
crisp_fan["slow"] = ff.mf.it2(ff.mf.triangle(0.0, 20.0, 50.0),   ff.mf.triangle(0.0, 20.0, 50.0))

degenerate = ff.FuzzySystem(type="it2")
degenerate.add_input(crisp_temp)
degenerate.add_output(crisp_fan)
degenerate += ff.Rule.from_string("IF temp IS hot THEN fan IS fast")
degenerate += ff.Rule.from_string("IF temp IS mild THEN fan IS slow")

reference = ff.FuzzySystem()  # Type-1 centroid
ref_temp = ff.Variable("temp", 0.0, 100.0)
ref_temp["hot"]  = ff.mf.triangle(40.0, 70.0, 100.0)
ref_temp["mild"] = ff.mf.triangle(0.0, 30.0, 60.0)
ref_fan = ff.Variable("fan", 0.0, 100.0)
ref_fan["fast"] = ff.mf.triangle(50.0, 80.0, 100.0)
ref_fan["slow"] = ff.mf.triangle(0.0, 20.0, 50.0)
reference.add_input(ref_temp)
reference.add_output(ref_fan)
reference += ff.Rule.from_string("IF temp IS hot THEN fan IS fast")
reference += ff.Rule.from_string("IF temp IS mild THEN fan IS slow")

print("\n=== Zero-width FOU matches Type-1 centroid ===")
print(f"  {'temp':>6}   {'it2(deg)':>9}   {'type-1':>8}")
for temp_c in (25.0, 55.0, 75.0):
    a = degenerate.compute({"temp": temp_c}).get("fan", float("nan"))
    b = reference.compute({"temp": temp_c}).get("fan", float("nan"))
    print(f"  {temp_c:>6}   {a:>9.4f}   {b:>8.4f}")
