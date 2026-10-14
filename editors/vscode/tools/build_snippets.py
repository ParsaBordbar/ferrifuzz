"""Generate snippets/python.code-snippets. Run: python tools/build_snippets.py

Defaults are chosen so every snippet expands to runnable code: skeletons stand
alone, fragments assume the names from `ffmamdani` (system, temp -> fan,
cold/hot -> slow/fast). tools/test_snippets.py executes them all.
"""

import json
from pathlib import Path

TNORMS = "min,product,lukasiewicz,drastic,einstein,hamacher"
TCONORMS = "max,probor,lukasiewicz,drastic,einstein,hamacher"
DEFUZZ = "centroid,bisector,mom,som,lom"

S = {}


def snip(name, prefix, description, body):
    S[name] = {"prefix": prefix, "description": description, "body": body.strip("\n").split("\n")}


snip("Import ferrifuzz", "ffimport", "import ferrifuzz as ff", "import ferrifuzz as ff\n$0")

# ── Membership functions ──────────────────────────────────────────────────────
snip("Triangle MF", "fftri", "ff.mf.triangle(a, b, c) — feet a/c, peak b",
     "ff.mf.triangle(${1:0.0}, ${2:5.0}, ${3:10.0})$0")
snip("Trapezoid MF", "fftrap", "ff.mf.trapezoid(a, b, c, d) — plateau b..c",
     "ff.mf.trapezoid(${1:0.0}, ${2:2.0}, ${3:8.0}, ${4:10.0})$0")
snip("Gaussian MF", "ffgauss", "ff.mf.gaussian(center, sigma)",
     "ff.mf.gaussian(${1:5.0}, ${2:1.5})$0")
snip("Bell MF", "ffbell", "ff.mf.bell(center, width, slope)",
     "ff.mf.bell(${1:5.0}, ${2:2.0}, ${3:3.0})$0")
snip("Sigmoid MF", "ffsig", "ff.mf.sigmoid(center, slope) — slope > 0 rises, < 0 falls",
     "ff.mf.sigmoid(${1:5.0}, ${2:1.0})$0")
snip("Singleton MF", "ffsingleton", "ff.mf.singleton(x) — Sugeno constant output",
     "ff.mf.singleton(${1:50.0})$0")
snip("Interval type-2 MF", "ffit2", "ff.mf.it2(upper, lower) — triangle with a footprint of uncertainty",
     "ff.mf.it2(ff.mf.triangle(${1:0.0}, ${2:5.0}, ${3:10.0}), ff.mf.triangle(${4:2.0}, ${2}, ${5:8.0}))$0")

# ── Variables ─────────────────────────────────────────────────────────────────
snip("Linguistic variable (3 terms)", "ffvar", "Variable with low/medium/high triangular terms", r"""
${1:speed} = ff.Variable("${1}", ${2:0.0}, ${3:100.0})
${1}["${4:low}"] = ff.mf.triangle(${2}, ${2}, (${2} + ${3}) / 2)
${1}["${5:medium}"] = ff.mf.triangle(${2}, (${2} + ${3}) / 2, ${3})
${1}["${6:high}"] = ff.mf.triangle((${2} + ${3}) / 2, ${3}, ${3})
$0""")
snip("Linguistic variable (evenly spaced terms)", "ffautovar",
     "Variable with evenly spaced, overlapping terms (automf)",
     '${1:error} = ff.Variable("${1}", ${2:-10.0}, ${3:10.0}).automf(${4|3,5,7|}, shape="${5|triangle,gaussian|}")$0')
snip("Add term", "ffterm", "var[\"term\"] = ff.mf.triangle(...)",
     '${1:temp}["${2:warm}"] = ff.mf.triangle(${3:10.0}, ${4:20.0}, ${5:30.0})$0')

# ── Rules ─────────────────────────────────────────────────────────────────────
snip("Rule", "ffrule", "system += ff.Rule.from_string(\"IF ... THEN ...\")",
     '${1:system} += ff.Rule.from_string("IF ${2:temp} IS ${3:cold} THEN ${4:fan} IS ${5:slow}")$0')
snip("Rule with two conditions", "ffrule2", "IF a IS x AND|OR b IS y THEN ...",
     '${1:system} += ff.Rule.from_string("IF ${2:temp} IS ${3:hot} ${4|AND,OR|} ${2} IS NOT ${5:very} ${6:cold} THEN ${7:fan} IS ${8:fast}")$0')
snip("Rule base", "ffrules", "Add a list of rule strings", r"""
for _rule in [
    "IF ${2:temp} IS ${3:cold} THEN ${4:fan} IS ${5:slow}",
    "IF ${2} IS ${6:hot} THEN ${4} IS ${7:fast}",$0
]:
    ${1:system} += ff.Rule.from_string(_rule)""")
snip("Weighted rule object", "ffruleobj", "ff.Rule(conditions=[...], conclusion=(...), weight=w)",
     '${1:system} += ff.Rule(conditions=[("${2:temp}", "${3:hot}")], conclusion=("${4:fan}", "${5:fast}"), weight=${6:0.8})$0')

# ── Systems ───────────────────────────────────────────────────────────────────
snip("Mamdani system", "ffmamdani", "Complete Mamdani system skeleton", rf"""
import ferrifuzz as ff

${{1:temp}} = ff.Variable("${{1}}", ${{2:0.0}}, ${{3:40.0}})
${{1}}["cold"] = ff.mf.triangle(${{2}}, ${{2}}, (${{2}} + ${{3}}) / 2)
${{1}}["hot"] = ff.mf.triangle((${{2}} + ${{3}}) / 2, ${{3}}, ${{3}})

${{4:fan}} = ff.Variable("${{4}}", ${{5:0.0}}, ${{6:100.0}})
${{4}}["slow"] = ff.mf.triangle(${{5}}, ${{5}}, (${{5}} + ${{6}}) / 2)
${{4}}["fast"] = ff.mf.triangle((${{5}} + ${{6}}) / 2, ${{6}}, ${{6}})

${{7:system}} = ff.FuzzySystem(
    and_method="${{8|{TNORMS}|}}",
    or_method="${{9|{TCONORMS}|}}",
    defuzz_method="${{10|{DEFUZZ}|}}",
    implication="${{11|mamdani,larsen|}}",
)
${{7}}.add_input(${{1}})
${{7}}.add_output(${{4}})
${{7}} += ff.Rule.from_string("IF ${{1}} IS cold THEN ${{4}} IS slow")
${{7}} += ff.Rule.from_string("IF ${{1}} IS hot THEN ${{4}} IS fast")
assert not ${{7}}.validate(), ${{7}}.validate()

print(${{7}}({{"${{1}}": ${{12:30.0}}}}))
$0""")
snip("Sugeno system", "ffsugeno", "Complete Sugeno/TSK system skeleton (constant + linear consequents)", r"""
import ferrifuzz as ff

${1:x} = ff.Variable("${1}", ${2:0.0}, ${3:10.0})
${1}["low"] = ff.mf.triangle(${2}, ${2}, ${3})
${1}["high"] = ff.mf.triangle(${2}, ${3}, ${3})

${4:y} = ff.Variable("${4}", ${5:0.0}, ${6:100.0})
${4}["small"] = ff.mf.singleton(${5})  # zero-order: z = constant
${4}["large"] = ff.mf.singleton(${6})

${7:system} = ff.FuzzySystem(defuzz_method="weighted_average")
${7}.add_input(${1})
${7}.add_output(${4})
${7} += ff.Rule.from_string("IF ${1} IS low THEN ${4} IS small")
${7} += ff.Rule.from_string("IF ${1} IS high THEN ${4} IS large")
# first-order (TSK): z = bias + coeff * ${1}
${7}.add_consequent("${4}", "large", {"${1}": ${8:5.0}}, bias=${9:50.0})

print(${7}({"${1}": ${10:5.0}}))
$0""")
snip("Interval type-2 system", "ffit2sys", "Complete interval type-2 Mamdani skeleton", r"""
import ferrifuzz as ff


def it2_triangle(a, b, c, fou=${1:0.2}):
    # Upper MF (a, b, c); lower MF narrower by `fou` of the base and only 80 % tall.
    w = fou * (c - a) / 2
    lower = ff.mf.triangle(min(a + w, b), b, max(c - w, b))
    return ff.mf.it2(ff.mf.triangle(a, b, c), lower, lower_height=0.8)


${2:temp} = ff.Variable("${2}", 0.0, 40.0)
${2}["cold"] = it2_triangle(0.0, 0.0, 25.0)
${2}["hot"] = it2_triangle(15.0, 40.0, 40.0)

${3:fan} = ff.Variable("${3}", 0.0, 100.0)
${3}["slow"] = it2_triangle(0.0, 0.0, 60.0)
${3}["fast"] = it2_triangle(40.0, 100.0, 100.0)

${4:system} = ff.FuzzySystem(type="it2", type_reduction="${5|km,cos,height|}")
${4}.add_input(${2})
${4}.add_output(${3})
${4} += ff.Rule.from_string("IF ${2} IS cold THEN ${3} IS slow")
${4} += ff.Rule.from_string("IF ${2} IS hot THEN ${3} IS fast")

print(${4}({"${2}": ${6:30.0}}))
print(${4}.compute_interval({"${2}": ${6}}))  # type-reduced interval (y_l, y_r)
$0""")

# ── Running ───────────────────────────────────────────────────────────────────
snip("Compute", "ffcompute", "Run inference on one input row",
     '${1:out} = ${2:system}.compute({"${3:temp}": ${4:30.0}})$0')
snip("Compute batch", "ffbatch", "Vectorised inference over a NumPy column", r"""
import numpy as np

${1:xs} = np.linspace(${2:0.0}, ${3:40.0}, ${4:100})
${5:ys} = ${6:system}.compute_batch({"${7:temp}": ${1}})["${8:fan}"]
$0""")
snip("Validate rule base", "ffvalidate", "Print errors and warnings found by system.validate()", r"""
for _issue in ${1:system}.validate():
    print(_issue)
$0""")
snip("Catch model errors", "fftry", "Run inference and show ferrifuzz's error message", r"""
try:
    ${1:out} = ${2:system}.compute({"${3:temp}": ${4:30.0}})
except ff.ModelError as e:   # unknown variable / term in a rule, conflicting settings
    print(e)
except ff.InputError as e:   # missing / unknown / NaN input
    print(e)
$0""")
snip("Rule strengths", "ffstrengths", "Firing strength of every rule for one input row", r"""
for _rule, _w in zip(${1:system}.rules_str(), ${1}.rule_strengths({"${2:temp}": ${3:30.0}})):
    print(f"{_w:.3f}  {_rule}")
$0""")
snip("Simulation loop", "ffloop", "Closed-loop simulation with the fast positional evaluate()", r"""
${1:state} = ${2:0.0}
for _step in range(${3:1000}):
    (${4:u},) = ${5:system}.evaluate([${1}])   # inputs in system.input_names() order
    ${1} += ${6:0.01} * ${4}
$0""")
snip("Hedge", "ffhedge", "Apply a linguistic hedge to a degree",
     "ff.Hedge.${1|very,somewhat,plus,minus|}()(${2:0.8})$0")
snip("T-norm", "ffand", "Fuzzy AND of two degrees",
     f'ff.Operator.t_norm(${{1:0.7}}, ${{2:0.4}}, kind="${{3|{TNORMS}|}}")$0')
snip("T-conorm", "ffor", "Fuzzy OR of two degrees",
     f'ff.Operator.t_conorm(${{1:0.7}}, ${{2:0.4}}, kind="${{3|{TCONORMS}|}}")$0')

# ── Plotting (matplotlib) ─────────────────────────────────────────────────────
snip("Plot variable", "ffplotvar", "Plot every term of a Variable (IT2 terms drawn as a band)", r"""
import ferrifuzz.plot as fp

fp.plot_variable(${1:temp})
$0""")
snip("Plot output set", "ffplotout", "Plot the aggregated output set and the crisp value for one input row", r"""
import ferrifuzz.plot as fp

fp.plot_output(${1:system}, {"${2:temp}": ${3:30.0}}, "${4:fan}")
$0""")
snip("Plot response curve", "ffplotcurve", "Plot an output against one input", r"""
import ferrifuzz.plot as fp

fp.plot_response(${1:system}, "${2:temp}", "${3:fan}")
$0""")
snip("Plot control surface", "ffsurface", "3-D control surface of a two-input system", r"""
import ferrifuzz.plot as fp

fp.plot_surface(${1:system}, "${2:x}", "${3:y}", "${4:out}")
$0""")
snip("Show plots", "ffshow", "matplotlib.pyplot.show()", "import matplotlib.pyplot as plt\n\nplt.show()\n$0")

# ── Time series ───────────────────────────────────────────────────────────────
snip("Fuzzy time series forecast", "fffts", "Chen FTS train/test split, forecast and metrics", r"""
from ferrifuzz.fts import FuzzyTimeSeries

# Default data: University of Alabama enrollments 1971-1992 (Song & Chissom / Chen benchmark).
${1:series} = [${2:13055, 13563, 13867, 14696, 15460, 15311, 15603, 15861, 16807, 16919, 16388, 15433, 15497, 15145, 15163, 15984, 16859, 18150, 18970, 19328, 19337, 18876}]
_split = int(len(${1}) * ${3:0.7})
_train, _test = ${1}[:_split], ${1}[_split:]

${4:model} = FuzzyTimeSeries(n_partitions=${5:7}, order=${6:1}, mf_type="${7|triangle,gaussian|}").fit(_train)
print(${4}.summary())
print(${4}.evaluate(_train[-${6}:] + _test))  # one-step-ahead RMSE / MAE / MAPE on the test years
print(${4}.predict(${1}[-${6}:], steps=${8:3}))
$0""")

out = Path(__file__).resolve().parent.parent / "snippets" / "python.code-snippets"
out.write_text(json.dumps(S, indent=2, ensure_ascii=False) + "\n")
print(f"wrote {len(S)} snippets to {out}")
