"""ferrifuzz.plot — matplotlib helpers for variables, outputs and control surfaces.

Needs matplotlib (``pip install "ferrifuzz[plot]"``); NumPy comes with it.
Every function draws on ``ax`` (a new figure when omitted) and returns the axes.

>>> import ferrifuzz.plot as fp
>>> fp.plot_variable(temp)
>>> fp.plot_output(system, {"temp": 30}, "fan")
>>> fp.plot_response(system, "temp", "fan")
>>> fp.plot_surface(system, "service", "food", "tip")
"""

from __future__ import annotations

from typing import Any, Mapping, Optional

import ferrifuzz as ff

__all__ = ["plot_variable", "plot_output", "plot_response", "plot_surface"]


def _pyplot():
    try:
        import matplotlib.pyplot as plt
        import numpy as np
    except ImportError as e:  # pragma: no cover - depends on the environment
        raise ImportError(
            "ferrifuzz.plot needs matplotlib and numpy: pip install \"ferrifuzz[plot]\""
        ) from e
    return plt, np


def _axes(ax, projection: Optional[str] = None):
    plt, _ = _pyplot()
    if ax is not None:
        return ax
    fig = plt.figure()
    return fig.add_subplot(projection=projection) if projection else fig.add_subplot()


def _defaults(system: "ff.FuzzySystem", fixed: Optional[Mapping[str, float]]) -> dict:
    """Every input at its range midpoint, overridden by ``fixed``."""
    values = {}
    for name in system.input_names():
        var = system.input(name)
        assert var is not None
        values[name] = (var.min_value + var.max_value) / 2
    unknown = set(fixed or {}) - set(values)
    if unknown:
        raise ff.InputError(f"unknown inputs in fixed=: {sorted(unknown)}; inputs are: {list(values)}")
    values.update(fixed or {})
    return values


def plot_variable(var: "ff.Variable", ax: Any = None, n: int = 400, legend: bool = True):
    """Plot every term of a variable; interval type-2 terms are drawn as a shaded band."""
    _, np = _pyplot()
    ax = _axes(ax)
    xs = np.linspace(var.min_value, var.max_value, n)
    for name in var.get_terms():
        mf = var[name]
        if isinstance(mf, ff.IT2MF):
            lower, upper = mf.membership(xs)
            line = ax.plot(xs, upper, label=name)[0]
            ax.fill_between(xs, lower, upper, alpha=0.3, color=line.get_color())
            ax.plot(xs, lower, color=line.get_color(), linewidth=0.8)
        else:
            ax.plot(xs, mf.membership(xs), label=name)
    ax.set_xlim(var.min_value, var.max_value)
    ax.set_ylim(-0.02, 1.05)
    ax.set_xlabel(var.name)
    ax.set_ylabel("membership")
    if legend:
        ax.legend()
    return ax


def plot_output(system: "ff.FuzzySystem", inputs: Mapping[str, float], output: str, ax: Any = None):
    """Show how an output is formed for one input row.

    Mamdani: the output terms (faint), the aggregated set (filled) and the crisp value.
    Interval type-2: the aggregated footprint of uncertainty and the interval ``[y_l, y_r]``.
    Sugeno: the output terms and the crisp value.
    """
    var = system.output(output)
    if var is None:
        raise ff.InputError(f"unknown output {output!r}; outputs are: {system.output_names()}")
    ax = _axes(ax)
    plot_variable(var, ax=ax, legend=False)
    for line in ax.get_lines():
        line.set_alpha(0.35)
    y = system.compute(inputs)[output]
    if system.is_it2:
        xs, lower, upper = system.output_fou(inputs, output)
        ax.fill_between(xs, lower, upper, alpha=0.5, label="aggregated FOU")
        yl, yr = system.compute_interval(inputs)[output]
        ax.axvspan(yl, yr, alpha=0.15, color="k", label=f"[y_l, y_r] = [{yl:.3g}, {yr:.3g}]")
    elif system.defuzz_method != "weighted_average":
        xs, mus = system.output_set(inputs, output)
        ax.fill_between(xs, 0, mus, alpha=0.5, label="aggregated set")
    ax.axvline(y, color="k", linestyle="--", label=f"{output} = {y:.4g}")
    ax.set_title(", ".join(f"{k}={v:g}" for k, v in inputs.items()))
    ax.legend()
    return ax


def plot_response(
    system: "ff.FuzzySystem",
    input: str,
    output: str,
    fixed: Optional[Mapping[str, float]] = None,
    n: int = 200,
    ax: Any = None,
):
    """Plot ``output`` against one input; the other inputs sit at ``fixed`` (default: mid-range)."""
    _, np = _pyplot()
    values = _defaults(system, fixed)
    var = system.input(input)
    if var is None:
        raise ff.InputError(f"unknown input {input!r}; inputs are: {system.input_names()}")
    xs = np.linspace(var.min_value, var.max_value, n)
    columns = {k: np.full(n, float(v)) for k, v in values.items()}
    columns[input] = xs
    ys = system.compute_batch(columns)[output]
    ax = _axes(ax)
    ax.plot(xs, ys)
    ax.set_xlabel(input)
    ax.set_ylabel(output)
    others = {k: v for k, v in values.items() if k != input}
    if others:
        ax.set_title(", ".join(f"{k}={v:g}" for k, v in others.items()))
    return ax


def plot_surface(
    system: "ff.FuzzySystem",
    x: str,
    y: str,
    output: str,
    fixed: Optional[Mapping[str, float]] = None,
    n: int = 40,
    ax: Any = None,
    cmap: str = "viridis",
):
    """3-D control surface of ``output`` over two inputs (others at ``fixed`` / mid-range)."""
    _, np = _pyplot()
    values = _defaults(system, fixed)
    vx, vy = system.input(x), system.input(y)
    if vx is None or vy is None:
        raise ff.InputError(f"unknown input {x if vx is None else y!r}; inputs are: {system.input_names()}")
    X, Y = np.meshgrid(np.linspace(vx.min_value, vx.max_value, n), np.linspace(vy.min_value, vy.max_value, n))
    columns = {k: np.full(X.size, float(v)) for k, v in values.items()}
    columns[x], columns[y] = X.ravel(), Y.ravel()
    Z = np.asarray(system.compute_batch(columns)[output]).reshape(X.shape)
    ax = _axes(ax, projection="3d")
    ax.plot_surface(X, Y, Z, cmap=cmap)
    ax.set_xlabel(x)
    ax.set_ylabel(y)
    ax.set_zlabel(output)
    return ax
