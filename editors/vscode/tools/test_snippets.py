"""Expand every snippet with its defaults and execute it. Needs ferrifuzz, numpy, matplotlib.

Run: MPLBACKEND=Agg python tools/test_snippets.py
"""

import json
import re
import sys
from pathlib import Path

SNIPPETS = json.loads((Path(__file__).resolve().parent.parent / "snippets" / "python.code-snippets").read_text())

PLACEHOLDER = re.compile(r"\$\{(\d+):([^}]*)\}")
CHOICE = re.compile(r"\$\{(\d+)\|([^|]*)\|\}")
MIRROR = re.compile(r"\$\{(\d+)\}|\$(\d+)")


def expand(body):
    text = "\n".join(body)
    values = {}
    for n, default in PLACEHOLDER.findall(text):
        values.setdefault(n, default)
    for n, choices in CHOICE.findall(text):
        values.setdefault(n, choices.split(",")[0])
    text = PLACEHOLDER.sub(lambda m: m.group(2), text)
    text = CHOICE.sub(lambda m: m.group(2).split(",")[0], text)
    return MIRROR.sub(lambda m: values.get(m.group(1) or m.group(2), ""), text)


code = {s["prefix"]: expand(s["body"]) for s in SNIPPETS.values()}
mamdani = code["ffmamdani"]
two_inputs = """
import ferrifuzz as ff
x = ff.Variable("x", 0, 10); x["lo"] = ff.mf.triangle(0, 0, 10); x["hi"] = ff.mf.triangle(0, 10, 10)
y = ff.Variable("y", 0, 10); y["lo"] = ff.mf.triangle(0, 0, 10); y["hi"] = ff.mf.triangle(0, 10, 10)
out = ff.Variable("out", 0, 1); out["a"] = ff.mf.triangle(0, 0, 1); out["b"] = ff.mf.triangle(0, 1, 1)
system = ff.FuzzySystem(); system.add_input(x); system.add_input(y); system.add_output(out)
system += ff.Rule.from_string("IF x IS lo AND y IS lo THEN out IS a")
system += ff.Rule.from_string("IF x IS hi OR y IS hi THEN out IS b")
"""
standalone = {"ffimport", "ffmamdani", "ffsugeno", "ffit2sys", "fffts", "ffvar", "ffautovar", "ffshow"}
expressions = {"fftri", "fftrap", "ffgauss", "ffbell", "ffsig", "ffsingleton", "ffit2", "ffhedge", "ffand", "ffor"}

failed = 0
for prefix, src in code.items():
    if prefix in standalone:
        program = "import ferrifuzz as ff\n" + src
    elif prefix in expressions:
        program = f"import ferrifuzz as ff\n_result = {src}\nassert _result is not None"
    elif prefix == "ffsurface":
        program = two_inputs + src
    else:
        program = mamdani + "\n" + src
    try:
        exec(compile(program, prefix, "exec"), {"__name__": "__snippet__"})
        print(f"ok    {prefix}")
    except Exception as e:  # noqa: BLE001
        failed += 1
        print(f"FAIL  {prefix}: {type(e).__name__}: {e}\n{src}\n")
sys.exit(1 if failed else 0)
