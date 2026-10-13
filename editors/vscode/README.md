# Ferrifuzz for VS Code

Editor support for the [ferrifuzz](https://github.com/ParsaBordbar/ferrifuzz) Python API.

## Rule highlighting

Rule strings inside Python are highlighted — keywords, input/output variables, `NOT`,
hedges, terms, parentheses and `WITH` weights each get their own color:

```python
system += "IF temp IS NOT very hot OR hum IS high THEN fan IS slow AND alarm IS on"
system += "if (temp is hot or temp is very warm) and wind is calm then fan is fast with 0.8"

RULES = """
IF service IS poor THEN tip IS low
IF service IS excellent THEN tip IS high
"""
```

Only strings (or lines of a triple-quoted string) that start with `IF <var> IS` (any case,
optionally after `(`) and contain `THEN` are touched, so ordinary strings and docstrings keep
their normal color. f-strings are left alone.

| Part | Scope |
|---|---|
| `IF`, `THEN` | `keyword.control.conditional.ferrifuzz` |
| `AND`, `OR` | `keyword.operator.logical.ferrifuzz` |
| `IS`, `WITH` | `keyword.operator.word.ferrifuzz` |
| weight number | `constant.numeric.ferrifuzz` |
| `(`, `)` | `punctuation.section.parens.ferrifuzz` |
| `NOT` | `keyword.operator.logical.negation.ferrifuzz` |
| variable | `variable.other.input.ferrifuzz` |
| hedge (`very`, `somewhat`, ...) | `storage.modifier.hedge.ferrifuzz` |
| term | `entity.name.type.term.ferrifuzz` |

Recolor with `editor.tokenColorCustomizations` → `textMateRules` if your theme is too quiet.

## Snippets

Type the prefix in a Python file and press <kbd>Tab</kbd>. Dropdowns let you pick
t-norms, defuzzifiers, type reduction and so on.

| Prefix | Inserts |
|---|---|
| `ffimport` | `import ferrifuzz as ff` |
| `ffmamdani` | complete Mamdani system (operator / defuzz pickers) |
| `ffsugeno` | complete Sugeno/TSK system with a linear consequent |
| `ffit2sys` | complete interval type-2 system with an FOU helper and `compute_interval` |
| `ffvar` | variable with low/medium/high terms |
| `ffautovar` | `ff.Variable(...).automf(3 \| 5 \| 7, shape=...)` |
| `ffterm` | `var["term"] = ff.mf.triangle(...)` |
| `fftri` `fftrap` `ffgauss` `ffbell` `ffsig` `ffsingleton` `ffit2` | membership functions |
| `ffrule` `ffrule2` `ffrules` `ffruleobj` | one rule, two-clause rule, rule list, weighted `Rule(...)` |
| `ffcompute` `ffbatch` `ffloop` | one row, a NumPy column, a closed-loop simulation with `evaluate` |
| `ffvalidate` `fftry` `ffstrengths` | list errors/warnings, catch `ModelError`/`InputError`, print rule firing strengths |
| `ffhedge` `ffand` `ffor` | hedge / t-norm / t-conorm on raw degrees |
| `ffplotvar` `ffplotout` `ffplotcurve` `ffsurface` `ffshow` | `ferrifuzz.plot`: terms, output set, response curve, 3-D surface, `plt.show()` |
| `fffts` | fuzzy time series forecast + `evaluate` on the Alabama enrollment data |

Method-name arguments (`defuzz_method=`, `and_method=`, ...) also autocomplete through the
type stubs shipped with the `ferrifuzz` package.

## Install

```sh
cd editors/vscode
npm install
npm run package                                  # → ferrifuzz-vscode-<version>.vsix
code --install-extension ferrifuzz-vscode-*.vsix
```

## Development

```sh
npm run build:snippets   # regenerate snippets/python.code-snippets from tools/build_snippets.py
npm run test:snippets    # expand every snippet with its defaults and execute it (needs ferrifuzz, numpy, matplotlib)
npm run test:grammar     # tokenize tools/grammar_sample.py with VS Code's Python grammar + the injection
```

Edit snippets in `tools/build_snippets.py`, not in the generated JSON.
