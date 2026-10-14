"""
Example 02 — Variables and Rules
==================================
Linguistic variables, term assignment, fuzzification, and rule parsing.
"""

import ferrifuzz as ff

# ── 1. Creating a Variable ────────────────────────────────────────────────────
temp = ff.Variable("Temperature", min_value=0.0, max_value=100.0)
print(repr(temp))   # Variable('Temperature', [0.0, 100.0], 0 terms)

# ── 2. Adding terms with dict-style syntax ────────────────────────────────────
temp["Cold"] = ff.mf.triangle(0.0,  0.0,  40.0)
temp["Warm"] = ff.mf.triangle(20.0, 50.0, 80.0)
temp["Hot"]  = ff.mf.triangle(60.0, 100.0, 100.0)
print(repr(temp))   # Variable('Temperature', [0.0, 100.0], 3 terms)

# ── 3. Listing terms ──────────────────────────────────────────────────────────
print("\nTerms:", temp.get_terms())   # ['Cold', 'Warm', 'Hot'] (order may vary)

# ── 4. Fuzzification — three equivalent syntaxes ─────────────────────────────
value = 65.0
print(f"\nFuzzify {value}°C:")
print("  .fuzzify():", temp.fuzzify(value))   # {'Cold': 0.0, 'Warm': 0.375, 'Hot': 0.125}
print("  callable:  ", temp(value))           # same, shorthand

# ── 5. Retrieving a single term MF ────────────────────────────────────────────
hot_mf = temp["Hot"]
print(f"\ntemp['Hot'].membership(80) = {hot_mf.membership(80.0):.2f}")  # 0.50

# ── 6. Rules — from_string ────────────────────────────────────────────────────
print("\n=== Rules from strings ===")

r1 = ff.Rule.from_string("IF Temperature IS Hot THEN fan IS Fast")
r2 = ff.Rule.from_string("IF Temperature IS Cold THEN fan IS Slow")
r3 = ff.Rule.from_string(
    "IF Temperature IS Warm AND humidity IS High THEN fan IS Medium"
)
print(r1)   # IF Temperature IS Hot THEN fan IS Fast
print(r2)
print(r3)

# ── 7. Rules — direct construction ───────────────────────────────────────────
r4 = ff.Rule(
    conditions=[("Temperature", "Hot"), ("humidity", "High")],
    conclusion=("fan", "Fast"),
)
print("\nDirect rule:", r4)

# ── 8. Rule introspection ─────────────────────────────────────────────────────
print("\nr1 conditions :", r1.conditions)   # [('Temperature', 'Hot')]
print("r1 conclusion :", r1.conclusion)    # ('fan', 'Fast')
print("repr          :", repr(r1))         # <Rule: 'IF Temperature IS Hot THEN fan IS Fast'>

# ── 9. Rule equality ──────────────────────────────────────────────────────────
r1b = ff.Rule.from_string("IF Temperature IS Hot THEN fan IS Fast")
print(f"\nr1 == r1b : {r1 == r1b}")   # True
print(f"r1 == r2  : {r1 == r2}")    # False
print(f"r1 != r2  : {r1 != r2}")    # True

# ── 10. Multiple input variables ──────────────────────────────────────────────
print("\n=== Multi-variable fuzzification ===")
humidity = ff.Variable("humidity", 0.0, 100.0)
humidity["Low"]    = ff.mf.trapezoid(0.0,  0.0,  30.0, 50.0)
humidity["Medium"] = ff.mf.triangle(30.0,  50.0, 70.0)
humidity["High"]   = ff.mf.trapezoid(50.0, 70.0, 100.0, 100.0)

print("humidity at 80%:", humidity(80.0))
