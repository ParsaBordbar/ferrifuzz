"""
Example 04 — Fuzzy Inference System (Mamdani)
===============================================
Full end-to-end example: define variables, add rules, run inference.
Demonstrates every defuzzification method and both AND methods.
"""

import ferrifuzz as ff

# ─────────────────────────────────────────────────────────────────────────────
# 1. Define linguistic variables
# ─────────────────────────────────────────────────────────────────────────────

temperature = ff.Variable("temperature", 0.0, 100.0)
temperature["cold"] = ff.mf.trapezoid(0.0,  0.0,  20.0, 40.0)
temperature["warm"] = ff.mf.triangle( 20.0, 50.0, 80.0)
temperature["hot"]  = ff.mf.trapezoid(60.0, 80.0, 100.0, 100.0)

humidity = ff.Variable("humidity", 0.0, 100.0)
humidity["low"]    = ff.mf.trapezoid(0.0,  0.0,  30.0, 50.0)
humidity["medium"] = ff.mf.triangle( 30.0, 50.0, 70.0)
humidity["high"]   = ff.mf.trapezoid(50.0, 70.0, 100.0, 100.0)

fan_speed = ff.Variable("fan_speed", 0.0, 100.0)
fan_speed["slow"]   = ff.mf.triangle(0.0,  0.0,  40.0)
fan_speed["medium"] = ff.mf.triangle(20.0, 50.0, 80.0)
fan_speed["fast"]   = ff.mf.triangle(60.0, 100.0, 100.0)

# ─────────────────────────────────────────────────────────────────────────────
# 2. Build a FuzzySystem (default: AND=min, defuzz=centroid)
# ─────────────────────────────────────────────────────────────────────────────

system = ff.FuzzySystem()
system.add_input(temperature)
system.add_input(humidity)
system.add_output(fan_speed)

# ─────────────────────────────────────────────────────────────────────────────
# 3. Add rules — two syntaxes
# ─────────────────────────────────────────────────────────────────────────────

# String parsing (most readable)
system += ff.Rule.from_string("IF temperature IS cold THEN fan_speed IS slow")
system += ff.Rule.from_string("IF temperature IS warm THEN fan_speed IS medium")
system += ff.Rule.from_string("IF temperature IS hot  THEN fan_speed IS fast")
system += ff.Rule.from_string(
    "IF temperature IS hot AND humidity IS high THEN fan_speed IS fast"
)
system += ff.Rule.from_string(
    "IF temperature IS warm AND humidity IS low THEN fan_speed IS slow"
)

# Direct construction
system.add_rule(ff.Rule(
    conditions=[("temperature", "cold"), ("humidity", "low")],
    conclusion=("fan_speed", "slow"),
))

# ─────────────────────────────────────────────────────────────────────────────
# 4. Inspect the system
# ─────────────────────────────────────────────────────────────────────────────

print(repr(system))
print(f"\nInputs  : {system.input_names()}")
print(f"Outputs : {system.output_names()}")
print(f"Rules   : {system.rule_count()}")
print("\nRule base:")
for rule_str in system.rules_str():
    print(f"  {rule_str}")

# ─────────────────────────────────────────────────────────────────────────────
# 5. Run inference — two equivalent call styles
# ─────────────────────────────────────────────────────────────────────────────

inputs = {"temperature": 75.0, "humidity": 65.0}

result_call    = system(inputs)           # __call__ shorthand
result_compute = system.compute(inputs)   # explicit

print(f"\nInputs : {inputs}")
print(f"Output (centroid): fan_speed = {result_call['fan_speed']:.2f}")
print(f"Same via .compute: fan_speed = {result_compute['fan_speed']:.2f}")

# ─────────────────────────────────────────────────────────────────────────────
# 6. Compare all defuzzification methods
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== Defuzzification methods compared ===")
defuzz_methods = ["centroid", "bisector", "mom", "som", "lom", "weighted_average"]

for method in defuzz_methods:
    s = ff.FuzzySystem(defuzz_method=method)
    s.add_input(temperature)
    s.add_input(humidity)
    s.add_output(fan_speed)
    for rule_str in [
        "IF temperature IS cold THEN fan_speed IS slow",
        "IF temperature IS warm THEN fan_speed IS medium",
        "IF temperature IS hot  THEN fan_speed IS fast",
        "IF temperature IS hot AND humidity IS high THEN fan_speed IS fast",
    ]:
        s += ff.Rule.from_string(rule_str)

    out = s(inputs)
    print(f"  {method:18s} → {out['fan_speed']:.2f}")

# ─────────────────────────────────────────────────────────────────────────────
# 7. Larsen (product AND) vs Mamdani (min AND)
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== AND method comparison ===")
for and_method in ["min", "product"]:
    s = ff.FuzzySystem(and_method=and_method)
    s.add_input(temperature)
    s.add_input(humidity)
    s.add_output(fan_speed)
    s += ff.Rule.from_string(
        "IF temperature IS hot AND humidity IS high THEN fan_speed IS fast"
    )
    s += ff.Rule.from_string("IF temperature IS warm THEN fan_speed IS medium")
    out = s(inputs)
    print(f"  and_method={and_method:8s} → fan_speed = {out['fan_speed']:.2f}")

# ─────────────────────────────────────────────────────────────────────────────
# 8. Sweep over input range
# ─────────────────────────────────────────────────────────────────────────────

print("\n=== Fan speed vs temperature (humidity fixed at 50) ===")
print(f"  {'Temp':>6}  {'Fan speed':>10}")
for t in range(0, 101, 10):
    out = system({"temperature": float(t), "humidity": 50.0})
    speed = out.get("fan_speed", float("nan"))
    print(f"  {t:>6}  {speed:>10.2f}")
