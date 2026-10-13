# Try ferrifuzz — a 10-minute tour

ferrifuzz is a fuzzy logic library for Python with a Rust core: Mamdani, Sugeno (TSK) and
interval type-2 inference, linguistic hedges, six t-norms / t-conorms, fuzzy time series, and
plotting helpers. Thank you for testing it. Your feedback decides what changes next.

## 1. Install (1 minute)

Python 3.9 or newer on Windows, macOS or Linux; no Rust or compiler needed.

```sh
python3 -m venv ff && source ff/bin/activate          # Windows: ff\Scripts\activate
pip install "ferrifuzz[plot]"
```

If you were sent the **pre-release on TestPyPI** instead:

```sh
pip install -i https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ "ferrifuzz[plot]==0.2.0"
```

Optional, for VS Code: install the `.vsix` you received (*Extensions → … → Install from
VSIX*). It adds colour to rule strings and snippets (type `ffmamdani` + Tab).

## 2. A Mamdani controller (3 minutes)

```python
import ferrifuzz as ff

service = ff.Variable("service", 0, 10)
service["poor"] = ff.mf.trapezoid(0, 0, 2, 5)
service["good"] = ff.mf.triangle(2, 5, 8)
service["excellent"] = ff.mf.trapezoid(5, 8, 10, 10)
food = ff.Variable("food", 0, 10).automf(["rancid", "ok", "delicious"])
tip = ff.Variable("tip", 0, 30).automf(["low", "medium", "high"])

system = ff.FuzzySystem(and_method="min", or_method="max", defuzz_method="centroid")
system.add_input(service)
system.add_input(food)
system.add_output(tip)
system += "IF service IS poor OR food IS rancid THEN tip IS low"
system += "IF service IS good THEN tip IS medium"
system += "IF service IS excellent OR food IS very delicious THEN tip IS high"

print(system({"service": 7, "food": 8}))
print(system.validate())                    # [] when the rule base is consistent

import ferrifuzz.plot as fp, matplotlib.pyplot as plt
fp.plot_variable(service)
fp.plot_output(system, {"service": 7, "food": 8}, "tip")
fp.plot_surface(system, "service", "food", "tip")
plt.show()
```

Things worth trying:
- `defuzz_method=` `"bisector"`, `"mom"`, `"som"`, `"lom"`; `and_method="product"`;
  `implication="larsen"`
- hedges and negation in rules: `very`, `somewhat`, `extremely`, `IS NOT`, parentheses,
  `WITH 0.5` weights
- `system.rule_strengths({...})` — how strongly each rule fired

## 3. Make a mistake on purpose (1 minute)

```python
system += "IF service IS excelent THEN tip IS hgih"
system({"service": 7, "food": 8})            # → ff.ModelError naming both typos with suggestions
system({"service": 7})                       # → ff.InputError: missing input 'food'
ff.FuzzySystem(defuzz_method="centriod")     # → did you mean 'centroid'?
```

## 4. Sugeno and interval type-2 (3 minutes)

```python
x = ff.Variable("x", 0, 10).automf(["low", "high"])
y = ff.Variable("y", 0, 100)
y["small"] = ff.mf.singleton(0)

tsk = ff.FuzzySystem(defuzz_method="weighted_average")         # Sugeno
tsk.add_input(x)
tsk.add_output(y)
tsk += "IF x IS low THEN y IS small"
tsk += "IF x IS high THEN y IS line"
tsk.add_consequent("y", "line", {"x": 8.0}, bias=10.0)          # first-order: y = 10 + 8x
print(tsk({"x": 7}))

# interval type-2: a band of uncertainty around each set
temp = ff.Variable("temp", 0, 40)
temp["cold"] = ff.mf.it2(ff.mf.triangle(-20, 0, 25), ff.mf.triangle(-15, 0, 20), lower_height=0.8)
temp["hot"] = ff.mf.it2(ff.mf.triangle(15, 40, 60), ff.mf.triangle(20, 40, 55), lower_height=0.8)
fan = ff.Variable("fan", 0, 100).automf(["slow", "fast"])
it2 = ff.FuzzySystem(type_reduction="km")                      # Karnik–Mendel
it2.add_input(temp)
it2.add_output(fan)
it2 += "IF temp IS cold THEN fan IS slow"
it2 += "IF temp IS hot THEN fan IS fast"
print(it2({"temp": 28}), it2.compute_interval({"temp": 28}))   # crisp value and [y_l, y_r]
```

## 5. Fuzzy time series (2 minutes)

```python
from ferrifuzz.fts import FuzzyTimeSeries

enrollments = [13055, 13563, 13867, 14696, 15460, 15311, 15603, 15861, 16807, 16919, 16388,
               15433, 15497, 15145, 15163, 15984, 16859, 18150, 18970, 19328, 19337, 18876]
chen = FuzzyTimeSeries(n_partitions=7, order=1, universe=(13500, 19500)).fit(enrollments)
print(chen.forecast(enrollments)[:5])        # Chen (1996): 14000, 14000, 14000, 15500, 16000
print(chen.evaluate(enrollments))            # RMSE / MAE / MAPE
for g in chen.get_flrgs():
    print(g)
```

## 6. Feedback — what helps most

Please send short notes on any of these:
1. **Correctness:** do the operators, defuzzifiers, Karnik–Mendel reduction and FTS match the
   definitions you know? Any number that looks wrong?
2. **Teaching:** would you use it in a course or project? What is missing (a method, a plot,
   an example)?
3. **Usability:** where did you get stuck? Which error message didn't help?
4. **Environment:** OS, Python version, and whether `pip install` worked first time.

Everything you write — even "step 4 confused me" — is useful.

*ferrifuzz is Apache-2.0 licensed. If it helps your teaching or research, please cite it
(see `CITATION.cff`).*
