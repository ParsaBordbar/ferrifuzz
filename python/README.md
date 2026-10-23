# ferrifuzz

**Fast fuzzy logic for Python, with a Rust core.** Mamdani, Sugeno (TSK) and interval
type-2 inference, linguistic hedges, six t-norms / t-conorms, fuzzy time series, and
plotting helpers — with error messages that tell you what you got wrong.

```sh
pip install ferrifuzz                 # core, no dependencies
pip install "ferrifuzz[plot]"         # + numpy and matplotlib for ferrifuzz.plot
```

Prebuilt wheels for Windows, macOS and Linux, Python 3.9+. No Rust needed.

**Documentation:** [ferrifuzz.bubblelamp.space](https://ferrifuzz.bubblelamp.space) ([GitHub Pages mirror](https://parsabordbar.github.io/ferrifuzz_docs/)) · **VS Code extension:** [Ferrifuzz](https://marketplace.visualstudio.com/items?itemName=ParsaBordbar2003.ferrifuzz)

## A system in 15 lines

```python
import ferrifuzz as ff

service = ff.Variable("service", 0, 10)
service["poor"] = ff.mf.trapezoid(0, 0, 2, 5)
service["good"] = ff.mf.triangle(2, 5, 8)
service["excellent"] = ff.mf.trapezoid(5, 8, 10, 10)

tip = ff.Variable("tip", 0, 30).automf(["low", "medium", "high"])   # evenly spaced terms

system = ff.FuzzySystem()                  # Mamdani, min/max, centroid
system.add_input(service)
system.add_output(tip)
system += "IF service IS poor THEN tip IS low"
system += "IF service IS good THEN tip IS medium"
system += "IF service IS excellent OR service IS very good THEN tip IS high"

print(system({"service": 7.5}))            # {'tip': 20.82...}
```

## Rule syntax

```text
IF <condition> THEN <output> IS <term> [AND <output> IS <term> ...] [WITH <weight>]

IF temp IS hot AND humidity IS NOT low THEN fan IS fast
IF (temp IS hot OR temp IS very warm) AND wind IS calm THEN fan IS fast AND alarm IS on WITH 0.8
```

- Keywords `IF THEN AND OR IS NOT WITH` are case-insensitive; names are single words (use `_`).
- `AND` binds tighter than `OR`; parentheses are allowed.
- Hedges after `IS [NOT]`: `very` (μ²), `extremely` (μ³), `somewhat` / `more_or_less` / `slightly` (√μ), `plus` (μ^1.25), `minus` (μ^0.75).
- `WITH w` scales the rule's firing strength (0 ≤ w ≤ 1).

## When something is wrong, it says so

```python
system += "IF service IS gud THEN tip IS hihg"
system({"service": 7.5})
```
```text
ferrifuzz.ModelError: invalid fuzzy system (2 problems):
  - rule #4 `IF service IS gud THEN tip IS hihg`: input 'service' has no term 'gud' (did you mean 'good'?); its terms are: 'poor', 'good', 'excellent'
  - rule #4 `IF service IS gud THEN tip IS hihg`: output 'tip' has no term 'hihg' (did you mean 'high'?); its terms are: 'low', 'medium', 'high'
```

| Exception (all are `ValueError`) | When |
|---|---|
| `ff.ModelError` | a rule names an unknown variable or term, weight outside [0, 1], `add_consequent` on a non-Sugeno system, IT2 settings that conflict |
| `ff.InputError` | missing input, unknown input name, NaN, batch columns of different lengths |
| `ff.RuleSyntaxError` | a rule string can't be parsed (the message points at the word) |
| `ff.FuzzyError` | invalid MF parameter, unknown option like `defuzz_method="centriod"` |

`system.validate()` lists errors *and* warnings (unused inputs, duplicate or conflicting
rules, terms that are zero over their whole range) without raising.

Inputs outside a variable's range are clamped to it (`FuzzySystem(clamp_inputs=False)`
to turn that off). Every output is always returned — `nan` if no rule fired.

## What's in the box

| | |
|---|---|
| Membership functions | `ff.mf.triangle`, `trapezoid`, `gaussian`, `bell`, `sigmoid`, `singleton`, `it2(upper, lower, lower_height=1)` — `mf.membership(x)` takes a float or an array |
| Variables | `var["term"] = mf`, `var.automf(3 / 5 / 7 / [names])`, `var.fuzzify(x)` |
| Operators | `ff.Operator.t_norm / t_conorm / complement`; `ff.Hedge.very()`, ... |
| Systems | `FuzzySystem(and_method, or_method, defuzz_method, implication, type, type_reduction, resolution, clamp_inputs)` |
| Defuzzification | `centroid`, `bisector`, `mom`, `som`, `lom`, `weighted_average` (Sugeno) |
| Sugeno / TSK | `defuzz_method="weighted_average"`; constant outputs = term centers (use `ff.mf.singleton`), linear outputs via `system.add_consequent(out, term, {"x": 2.0}, bias=1.0)` |
| Interval type-2 | IT2 terms switch the engine automatically; `type_reduction="km" / "cos" / "height"`; `system.compute_interval(...)` gives `(y_l, y_r)` |
| Running | `system(inputs)`, `system.compute_batch({"x": array})` (all CPU cores), `system.evaluate([x1, x2])` (fast, positional) |
| Looking inside | `system.rule_strengths(inputs)`, `system.output_set(inputs, "out")`, `system.output_fou(...)`, `system.validate()` |
| Plots (`ferrifuzz.plot`) | `plot_variable`, `plot_output`, `plot_response`, `plot_surface` |
| Time series (`ferrifuzz.fts`) | Chen (1996) first / high-order: `FuzzyTimeSeries(n_partitions, order, universe=...)`, `.fit`, `.predict`, `.forecast`, `.evaluate` |
| Metrics (`ferrifuzz.metrics`) | `rmse`, `mae`, `mape`, `score` |

```python
import ferrifuzz.plot as fp
fp.plot_variable(service)
fp.plot_output(system, {"service": 7.5}, "tip")     # aggregated set + centroid
fp.plot_response(system, "service", "tip")
```

Time series:

```python
from ferrifuzz.fts import FuzzyTimeSeries

enrollments = [13055, 13563, 13867, 14696, 15460, 15311, 15603, 15861, 16807, 16919, 16388,
               15433, 15497, 15145, 15163, 15984, 16859, 18150, 18970, 19328, 19337, 18876]
model = FuzzyTimeSeries(n_partitions=7, order=1, universe=(13000, 20000)).fit(enrollments)
print(model.evaluate(enrollments))      # {'rmse': ..., 'mae': ..., 'mape': ...}
print(model.predict(enrollments[-1:], steps=2))
```

## Editor support

Type hints and docstrings ship with the package, so hover help and autocompletion of option
names (`defuzz_method="` → `centroid`, `bisector`, ...) work in VS Code, PyCharm and Jupyter.

The **Ferrifuzz** extension ([VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=ParsaBordbar2003.ferrifuzz); [Open VSX](https://open-vsx.org/?search=ferrifuzz) for Cursor,
VSCodium and Windsurf) adds, inside rule strings, highlighting, completion of variables, terms
and hedges, and "did you mean" warnings with quick fixes; plus `ff…` snippets (`ffmamdani`,
`ffsugeno`, `ffit2sys`, ...) and a warning when the editor's Python interpreter has no ferrifuzz.

## Citing

If ferrifuzz helps your coursework, thesis or paper, please cite it:

```bibtex
@software{bordbar_ferrifuzz,
  author  = {Bordbar, Parsa},
  title   = {ferrifuzz: fast fuzzy logic for Python and Rust},
  version = {0.2.1},
  year    = {2026},
  url     = {https://github.com/ParsaBordbar/ferrifuzz}
}
```

## License

Apache-2.0. If you redistribute ferrifuzz or build on it, keep the `NOTICE` file
(attribution to Parsa Bordbar), as the license requires.
