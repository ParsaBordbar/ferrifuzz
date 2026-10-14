"""
Example 06 — OR Rules & Sugeno Inference
==========================================
Two capabilities beyond basic Mamdani:

  1. OR in rule antecedents (AND binds tighter than OR).
  2. Sugeno / TSK inference — fast, sampling-free defuzzification.
"""

import ferrifuzz as ff

# ─────────────────────────────────────────────────────────────────────────────
# 1. OR in rules
# ─────────────────────────────────────────────────────────────────────────────

# AND binds tighter than OR → (temp hot AND humidity high) OR (temp extreme)
rule = ff.Rule.from_string(
    "IF temp IS hot AND humidity IS high OR temp IS extreme THEN fan IS fast"
)
print("=== OR rule parsing ===")
print("  str        :", rule)
print("  antecedent :", rule.antecedent)   # DNF: list of AND-groups, OR'd
print("  group_count:", rule.group_count())  # 2

# Direct DNF construction (no string parsing)
rule2 = ff.Rule(
    antecedent=[[("temp", "hot")], [("temp", "extreme")]],
    conclusion=("fan", "fast"),
)
print("  direct DNF :", rule2)

# ─────────────────────────────────────────────────────────────────────────────
# 2. OR inference — max vs probabilistic OR
# ─────────────────────────────────────────────────────────────────────────────

temp = ff.Variable("temp", 0.0, 100.0)
temp["hot"]     = ff.mf.triangle(50.0, 80.0, 100.0)
temp["extreme"] = ff.mf.triangle(85.0, 100.0, 100.0)

fan = ff.Variable("fan", 0.0, 100.0)
fan["fast"] = ff.mf.triangle(60.0, 100.0, 100.0)

print("\n=== OR method comparison ===")
for or_method in ["max", "probor"]:
    s = ff.FuzzySystem(or_method=or_method)
    s.add_input(temp)
    s.add_output(fan)
    s += ff.Rule.from_string("IF temp IS hot OR temp IS extreme THEN fan IS fast")
    print(f"  or_method={or_method:7s} → {s({'temp': 88.0})}")

# ─────────────────────────────────────────────────────────────────────────────
# 3. Sugeno / TSK — singleton consequents, weighted-average output
# ─────────────────────────────────────────────────────────────────────────────

t = ff.Variable("t", 0.0, 100.0)
t["cold"] = ff.mf.trapezoid(0.0, 0.0, 20.0, 40.0)
t["warm"] = ff.mf.triangle(20.0, 50.0, 80.0)
t["hot"]  = ff.mf.trapezoid(60.0, 80.0, 100.0, 100.0)

# Output terms are singletons → their center IS the constant consequent value.
power = ff.Variable("power", 0.0, 100.0)
power["low"]  = ff.mf.singleton(10.0)
power["mid"]  = ff.mf.singleton(50.0)
power["high"] = ff.mf.singleton(90.0)

sugeno = ff.FuzzySystem(defuzz_method="sugeno")
sugeno.add_input(t)
sugeno.add_output(power)
sugeno += ff.Rule.from_string("IF t IS cold THEN power IS low")
sugeno += ff.Rule.from_string("IF t IS warm THEN power IS mid")
sugeno += ff.Rule.from_string("IF t IS hot  THEN power IS high")

print("\n=== Sugeno inference (singleton consequents) ===")
print(f"  {'t':>4}  {'power':>8}")
for tv in range(0, 101, 10):
    out = sugeno({"t": float(tv)})
    print(f"  {tv:>4}  {out.get('power', float('nan')):>8.2f}")

# Analytic centers — O(1), exact (this is what Sugeno weights)
print("\n=== MF analytic centers ===")
print("  triangle(0,5,10)       →", ff.mf.triangle(0.0, 5.0, 10.0).center())
print("  trapezoid(10,30,70,90) →", ff.mf.trapezoid(10.0, 30.0, 70.0, 90.0).center())
print("  singleton(42)          →", ff.mf.singleton(42.0).center())
