"""Runtime docstrings exist for every public object and the stubs list the same names."""

import ast
import inspect
import pathlib

import pytest

import ferrifuzz as ff
from ferrifuzz import _core

STUB = pathlib.Path(_core.__file__).with_name("_core.pyi")


def _public_objects():
    for name in ff.__all__:
        if name == "plot":  # needs matplotlib; covered by test_plot.py
            continue
        obj = getattr(ff, name)
        yield name, obj
        if inspect.isclass(obj) or obj is ff.mf:
            for attr, member in vars(obj).items():
                if not attr.startswith("_"):
                    yield f"{name}.{attr}", member


def _stub_members():
    """{class name: set of public member names, including stub base classes} from ``_core.pyi``."""
    tree = ast.parse(STUB.read_text(encoding="utf-8"))
    members = {}
    for node in tree.body:
        if isinstance(node, ast.ClassDef):
            names = set()
            for base in node.bases:
                if isinstance(base, ast.Name):
                    names |= members.get(base.id, set())
            for item in node.body:
                if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    names.add(item.name)
                elif isinstance(item, ast.AnnAssign) and isinstance(item.target, ast.Name):
                    names.add(item.target.id)
            members[node.name] = {n for n in names if not n.startswith("_")}
    return members


@pytest.mark.parametrize("name, obj", list(_public_objects()), ids=lambda v: v if isinstance(v, str) else "")
def test_runtime_docstring(name, obj):
    assert (getattr(obj, "__doc__", None) or "").strip(), f"{name} has no docstring"


def test_stub_matches_runtime():
    stub = _stub_members()
    problems = []
    for name in ff.__all__:
        obj = getattr(ff, name, None)
        if not (inspect.isclass(obj) or obj is ff.mf) or name not in stub:
            continue
        runtime = {a for a in vars(obj) if not a.startswith("_")}
        inherited = set().union(*(stub.get(b.__name__, set()) for b in getattr(obj, "__mro__", ())[1:]))
        if missing := runtime - stub[name] - inherited:
            problems.append(f"{name}: in the extension but not in _core.pyi: {sorted(missing)}")
        if extra := stub[name] - runtime - {a for a in dir(obj) if not a.startswith("_")}:
            problems.append(f"{name}: in _core.pyi but not in the extension: {sorted(extra)}")
    assert not problems, "\n".join(problems)
