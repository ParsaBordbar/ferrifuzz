# Ferrifuzz for VS Code

Editor support for [ferrifuzz](https://github.com/ParsaBordbar/ferrifuzz), fuzzy logic for Python
with a Rust core: rule-string highlighting, completion and checks, snippets, and a check that
VS Code's Python interpreter can see ferrifuzz.

Needs the [Python extension](https://marketplace.visualstudio.com/items?itemName=ms-python.python)
and ferrifuzz 0.2.0 or newer in the interpreter VS Code uses (`pip install ferrifuzz`).

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

## Rule completion and checks

Inside a rule string the editor suggests what can come next, read from the `ff.Variable(...)`
definitions, `var["term"] = ...` / `.automf(...)` terms and `add_input` / `add_output` calls
in the same file:

| After | Suggests |
|---|---|
| `IF` / `AND` / `OR` / `(` | input variables |
| `THEN` / `AND` after THEN | output variables |
| `<var>` | `IS` |
| `IS` / `IS NOT` | that variable's terms, `NOT`, hedges (`very`, `somewhat`, ...) |
| a hedge | that variable's terms |
| a term | `AND` `OR` `THEN` (before THEN), `AND` `WITH` (after) |

Keywords follow the rule's own case (`if` → `is`, `then`, ...). Type a space to see the next
suggestions.

Rule strings that go into a system in the same file (`system += "..."`, `add_rule("...")`,
`add_rules([...])`, or a constant later passed to those) are checked as you type:

- a term the variable does not have — `'service' has no term 'excelent'. Did you mean 'excellent'?`
- a misspelled variable — `Unknown variable 'servise'. Did you mean 'service'?`
- an input after THEN or an output before it

"Did you mean" warnings come with a quick fix (<kbd>Cmd</kbd>/<kbd>Ctrl</kbd>+<kbd>.</kbd>). The
checks stay quiet about anything the file does not pin down — variables imported from another
module, terms added in a loop, rules parsed on their own with `Rule.from_string` — and
suggestions use the same edit-distance limits as ferrifuzz's own error messages.

Turn them off with `ferrifuzz.ruleCompletion.enabled` / `ferrifuzz.ruleDiagnostics.enabled`.

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

## Interpreter check

Completions and hover docs come from the type stubs inside the installed `ferrifuzz`
package, so they only work when the interpreter selected in VS Code has ferrifuzz 0.2.0 or
newer. When a Python file imports ferrifuzz and the selected interpreter has no ferrifuzz,
or an older one, a warning offers:

- **Use .venv** — when the workspace's `.venv` / `venv` has a usable ferrifuzz, switch to it;
- **Select Interpreter** — open the Python extension's picker;
- **Don't Show Again** — stop warning about that interpreter.

The check reads package metadata only (`importlib.metadata`), never imports ferrifuzz, and
runs once per interpreter per session. It needs the Python extension (`ms-python.python`).

| Command / setting | |
|---|---|
| *Ferrifuzz: Check Interpreter for Completions* | check the active file's interpreter now |
| *Ferrifuzz: Reset Interpreter Warnings* | undo every "Don't Show Again" |
| `ferrifuzz.interpreterCheck.enabled` | turn the automatic check off |

## Install

Search for **Ferrifuzz** in the Extensions view: the VS Code Marketplace, or Open VSX in
Cursor, VSCodium and Windsurf. From a `.vsix` file: *Extensions → ⋯ → Install from VSIX…*, or
`code --install-extension ferrifuzz-<version>.vsix`.

## Development

```sh
npm install
npm run package          # → ferrifuzz-<version>.vsix
npm run build:snippets   # regenerate snippets/python.code-snippets from tools/build_snippets.py
npm run test:snippets    # expand every snippet with its defaults and execute it (needs ferrifuzz, numpy, matplotlib)
npm run test:grammar     # tokenize tools/grammar_sample.py with VS Code's Python grammar + the injection
npm test                 # the three below (CI runs this)
npm run test:check       # interpreter check against throwaway venvs (needs python3)
npm run test:rules       # rule completion / checks, plus a no-false-alarm scan of examples/ and tests/
npm run test:extension   # extension.js against a fake vscode module
```

Edit snippets in `tools/build_snippets.py`, not in the generated JSON.

### Publishing

Bump `version` here and in `CHANGELOG.md`, run `npm test`, then:

```sh
npx vsce login ParsaBordbar2003  # once; Azure DevOps token with Marketplace → Manage scope
npm run publish:vsce             # VS Code Marketplace
OVSX_PAT=... npm run publish:ovsx   # Open VSX (Cursor, VSCodium, Windsurf); token from open-vsx.org
```
