"""
Example 03 — Operators and Hedges
===================================
T-norms, T-conorms, complement, and linguistic hedges — for both
Type-1 (float) and Interval Type-2 ((float, float)) membership degrees.
"""

import ferrifuzz as ff

# ─────────────────────────────────────────────────────────────────────────────
# OPERATORS
# ─────────────────────────────────────────────────────────────────────────────

mu_a = 0.8
mu_b = 0.5
it2_a = (0.6, 0.9)   # (lower, upper)
it2_b = (0.3, 0.7)

print("=== T-norm (AND) — Type-1 ===")
print(f"  min      (default): {ff.Operator.t_norm(mu_a, mu_b):.2f}")              # 0.50
print(f"  min      (explicit): {ff.Operator.t_norm(mu_a, mu_b, kind='min'):.2f}") # 0.50
print(f"  godel    (alias):    {ff.Operator.t_norm(mu_a, mu_b, kind='godel'):.2f}")
print(f"  product  (Larsen):   {ff.Operator.t_norm(mu_a, mu_b, kind='product'):.2f}")  # 0.40
print(f"  algebraic (alias):   {ff.Operator.t_norm(mu_a, mu_b, kind='algebraic'):.2f}")

print("\n=== T-norm (AND) — IT2 ===")
print(f"  min:     {ff.Operator.t_norm(it2_a, it2_b)}")              # (0.3, 0.7)
print(f"  product: {ff.Operator.t_norm(it2_a, it2_b, kind='product')}")

print("\n=== T-conorm (OR) — Type-1 ===")
print(f"  max         (default): {ff.Operator.t_conorm(mu_a, mu_b):.2f}")               # 0.80
print(f"  max         (explicit): {ff.Operator.t_conorm(mu_a, mu_b, kind='max'):.2f}")
print(f"  godel       (alias):    {ff.Operator.t_conorm(mu_a, mu_b, kind='godel'):.2f}")
print(f"  prob/algebraic (sum):   {ff.Operator.t_conorm(mu_a, mu_b, kind='prob'):.2f}") # 0.90
print(f"  probabilistic (alias):  {ff.Operator.t_conorm(mu_a, mu_b, kind='probabilistic'):.2f}")

print("\n=== T-conorm (OR) — IT2 ===")
print(f"  max:  {ff.Operator.t_conorm(it2_a, it2_b)}")
print(f"  prob: {ff.Operator.t_conorm(it2_a, it2_b, kind='prob')}")

print("\n=== Complement (NOT) ===")
print(f"  NOT {mu_a}          = {ff.Operator.complement(mu_a):.2f}")        # 0.20
print(f"  NOT {it2_a} = {ff.Operator.complement(it2_a)}")  # (0.1, 0.4) — bounds swap

# ─────────────────────────────────────────────────────────────────────────────
# HEDGES
# ─────────────────────────────────────────────────────────────────────────────

mu   = 0.8
it2  = (0.6, 0.9)

print("\n=== Standard hedges — Type-1 ===")
very     = ff.Hedge.very()       # power = 2.0 — concentrates membership
somewhat = ff.Hedge.somewhat()   # power = 0.5 — dilates membership
plus     = ff.Hedge.plus()       # power = 1.25
minus    = ff.Hedge.minus()      # power = 0.75

print(f"  very({mu})     = {very(mu):.4f}")       # 0.6400
print(f"  somewhat({mu}) = {somewhat(mu):.4f}")   # 0.8944
print(f"  plus({mu})     = {plus(mu):.4f}")       # 0.7578
print(f"  minus({mu})    = {minus(mu):.4f}")      # 0.8409

print("\n=== Standard hedges — IT2 ===")
print(f"  very({it2})     = {very(it2)}")
print(f"  somewhat({it2}) = {somewhat(it2)}")
print(f"  plus({it2})     = {plus(it2)}")
print(f"  minus({it2})    = {minus(it2)}")

print("\n=== Custom hedges ===")
extremely = ff.Hedge(3.0)   # mu^3 — very strong concentration
slightly  = ff.Hedge(0.2)   # mu^0.2 — strong dilation

print(f"  extremely({mu})  = {extremely(mu):.4f}")   # 0.5120
print(f"  slightly({mu})   = {slightly(mu):.4f}")    # 0.9565
print(f"  extremely({it2}) = {extremely(it2)}")
print(f"  slightly({it2})  = {slightly(it2)}")

# ─────────────────────────────────────────────────────────────────────────────
# Combining operators and hedges in a manual rule evaluation
# ─────────────────────────────────────────────────────────────────────────────
print("\n=== Manual rule: 'IF temp IS very hot AND humidity IS high' ===")

temp_mf = ff.mf.triangle(60.0, 100.0, 100.0)
hum_mf  = ff.mf.trapezoid(50.0, 70.0, 100.0, 100.0)

mu_hot  = temp_mf.membership(85.0)      # 0.625
mu_high = hum_mf.membership(80.0)       # 1.0

mu_very_hot = ff.Hedge.very()(mu_hot)   # hedge applied before AND
strength    = ff.Operator.t_norm(mu_very_hot, mu_high, kind="product")

print(f"  mu_hot         = {mu_hot:.3f}")
print(f"  very(mu_hot)   = {mu_very_hot:.3f}")
print(f"  mu_high        = {mu_high:.3f}")
print(f"  rule strength  = {strength:.3f}")
