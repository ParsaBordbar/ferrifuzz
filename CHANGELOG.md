# Changelog

All notable changes to ferrifuzz. Versions follow [semver](https://semver.org); while the
version is 0.x, minor releases may change the API.

## 0.2.0 — unreleased

### Highlights
- **Pure-Rust crate.** The engine is now a dependency-free Rust library (`crates/ferrifuzz`),
  usable directly from Rust; the Python package is a thin layer over it.
- **Errors instead of silently wrong numbers.** Misspelled variables / terms, missing or
  unknown inputs, NaN inputs, invalid weights and conflicting settings raise clear errors
  with "did you mean ...?" suggestions.
- **New for coursework:** `ferrifuzz.plot`, `Variable.automf`, vectorised `membership`,
  `compute_interval`, `rule_strengths`, `output_set` / `output_fou`, `evaluate`,
  parentheses and `WITH` weights in rule strings, `lower_height` for IT2 sets.
- **Faster:** rules are compiled once (≈3–5× faster `compute`), `compute_batch` uses all cores.

### Fixed
- Shoulder membership functions (`triangle(0, 0, 20)`, `trapezoid(0, 10, 20, 20)`) returned 0
  at their own peak, so outputs were missing at the edges of the input range.
- Fuzzy time series mapped values outside the fitted range to the *lowest* partition
  (forecasts of ~13 000 for 19 000-level data); they now map to the nearest partition.
- `SingletonMF.membership` was missing in Python.
- A term could exist as type-1 and interval type-2 at the same time.
- Term, input and output order is now insertion order (was random).

### Breaking changes and how to migrate
| Before | Now | Migrate |
|---|---|---|
| Missing / unknown inputs were ignored | `ff.InputError` | pass every input, spelled exactly |
| Unknown terms in rules counted as μ = 0 | `ff.ModelError` | fix the name the message suggests |
| Unknown option names fell back to the default | `ff.FuzzyError` | use a listed name |
| Inputs outside the range gave `{}` | clamped to the range | `clamp_inputs=False` for the old behaviour |
| An output no rule fired was absent | present as `nan` | check `math.isnan(...)` |
| `str(rule)` printed `(weight=0.5)` | prints `WITH 0.5` (parseable) | — |
| Multi-word terms (`IS really poor`) | `ff.RuleSyntaxError` | `really_poor` |
| `repr(mf)` was `<MembershipFunction: Triangular>` | `triangle(0, 5, 10)` | — |
| `validate()` messages `rule 0: ...` | `error: rule #1 ...` / `warning: ...` | — |
| FLRGs printed as indices `FLRG((0,) → [0, 1])` | labels `FLRG(A1 -> A1, A2)` | — |
| Rust: `compute(..) -> HashMap` | `-> Result<HashMap>` | add `?` |
| Rust: `mf::it2(..) -> IT2MF` | `-> Result<IT2MF>` | add `?` |
| `FuzzyTimeSeries(mf_type="trapezoid")` silently used triangles | `ValueError` | `"triangle"` or `"gaussian"` |

### Added
- Python: `ff.FuzzyError`, `InputError`, `ModelError`, `RuleSyntaxError` (all `ValueError`).
- `FuzzySystem`: `clamp_inputs=`, `add_rules`, string rules in `add_rule` / `+=`, `check()`,
  `evaluate(values)`, `compute_interval`, `rule_strengths(_interval)`, `output_set`,
  `output_fou`, `input(name)` / `output(name)`, setting properties, `is_it2`, informative `repr`.
- `validate()` warns about conflicting rules, terms that are zero over their range and unused
  linear consequents.
- `add_consequent` terms no longer need to exist in the output variable (one linear output per rule).
- `Variable.automf(n | names, shape)`, `Variable.clamp`, `str(variable)` with parameters,
  `fuzzify` includes IT2 terms as `(lower, upper)`.
- `mf.membership(array)` returns an array; `IT2MF.membership(array)` a pair of arrays.
- `ff.mf.it2(upper, lower, lower_height=1.0)` checks lower ≤ upper.
- `Hedge.extremely()`, `Hedge.slightly()`.
- Invalid membership-function parameters name the call: `triangle(5, 0, 10): needs a ≤ b ≤ c (feet a, c; peak b)`.
- `FuzzyTimeSeries(universe=(lo, hi))`, `.evaluate(series)`; NaN rejected.
- `ferrifuzz.plot`: `plot_variable`, `plot_output`, `plot_response`, `plot_surface`
  (`pip install "ferrifuzz[plot]"`).
- Type stubs cover the full API with option-name completion; `py.typed`.
- Apache-2.0 license, `NOTICE`, `CITATION.cff`.
- VS Code extension (`editors/vscode`): rule-string highlighting and snippets.
- Requires Python ≥ 3.9 (was 3.13); wheels for Windows, macOS and Linux.

## 0.1.0
- First TestPyPI release (macOS arm64 only).
