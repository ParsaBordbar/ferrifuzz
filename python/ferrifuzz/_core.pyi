"""
Type stubs for the ferrifuzz Rust extension (_core).

These stubs power editor hover docs and autocompletion (VS Code / Pylance, PyCharm)
and type checking (mypy). Import from ``ferrifuzz``, not from this module.
"""

from __future__ import annotations

from typing import Any, Iterator, Literal, Mapping, Sequence, Union, overload

# ─── Option names ────────────────────────────────────────────────────────────
# The Literal part drives editor completion and lists canonical names only; ``| str``
# keeps aliases and names held in plain ``str`` variables type-correct. Accepted aliases:
#   t-norm: godel / zadeh / standard (min), prod / algebraic (product), luk / bounded (lukasiewicz)
#   t-conorm: godel / zadeh / standard (max), prob / probabilistic / algebraic (probor), luk / bounded
#   defuzz: cog / coa (centroid), sugeno / wtaver (weighted_average), mean_of_maximum, ...
#   inference: t1 / type-1 / mamdani / sugeno (type1), type2 / type-2 / interval (it2)
#   implication: min (mamdani), product / prod (larsen); complement: zadeh (standard)
#   type reduction: karnik_mendel (km), center_of_sets (cos); shape: triangular, gauss, ...
# At runtime an unknown name raises FuzzyError listing the valid options (and suggesting
# the closest one).

TNormName = Union[Literal["min", "product", "lukasiewicz", "drastic", "einstein", "hamacher"], str]
TConormName = Union[Literal["max", "probor", "lukasiewicz", "drastic", "einstein", "hamacher"], str]
DefuzzName = Union[Literal["centroid", "bisector", "mom", "som", "lom", "weighted_average"], str]
InferenceName = Union[Literal["auto", "type1", "it2"], str]
ImplicationName = Union[Literal["mamdani", "larsen"], str]
TypeReductionName = Union[Literal["km", "cos", "height"], str]
ComplementName = Union[Literal["standard", "sugeno", "yager"], str]
ShapeName = Union[Literal["triangle", "gaussian"], str]
HedgeWord = Literal["very", "extremely", "somewhat", "more_or_less", "slightly", "plus", "minus"]

AnyType1MF = Union["Type1MF", "TriangularMF", "TrapezoidalMF", "GaussianMF", "SingletonMF", "BellMF", "SigmoidMF"]
ArrayLike = Union[Sequence[float], Any]
"""A list / tuple / NumPy array of floats."""

# ─── Exceptions ──────────────────────────────────────────────────────────────

class FuzzyError(ValueError):
    """Base class of every ferrifuzz error (a ``ValueError``), e.g. an invalid MF parameter
    or an unknown option name such as ``defuzz_method="centriod"``."""

class InputError(FuzzyError):
    """Bad input to ``compute``: a missing input, an unknown input name (with a
    "did you mean" hint), a NaN, or batch columns of different lengths."""

class ModelError(FuzzyError):
    """The system is inconsistent — a rule names an unknown variable or term, a weight is
    outside [0, 1], ``add_consequent`` is used on a non-Sugeno system, ... The message
    lists every problem with the rule number (from 1) and suggestions."""

class RuleSyntaxError(FuzzyError):
    """A rule string could not be parsed; the message points at the offending word."""

# ─── Membership function classes ─────────────────────────────────────────────

class _Shape:
    """Common interface of the type-1 membership-function classes."""
    @overload
    def membership(self, x: float) -> float: ...
    @overload
    def membership(self, x: ArrayLike) -> Any:
        """Degree of membership in [0, 1] — a float for a float, an array for a
        list / NumPy array (a list when NumPy is not installed)."""
        ...

class TriangularMF(_Shape):
    """Triangular MF — feet at ``a`` and ``c``, peak at ``b`` (a ≤ b ≤ c).
    Prefer ``ff.mf.triangle``; use this class for ``isinstance`` checks."""
    def __init__(self, a: float, b: float, c: float) -> None: ...

class TrapezoidalMF(_Shape):
    """Trapezoidal MF — feet at ``a`` and ``d``, plateau from ``b`` to ``c``. Prefer ``ff.mf.trapezoid``."""
    def __init__(self, a: float, b: float, c: float, d: float) -> None: ...

class GaussianMF(_Shape):
    """Gaussian MF ``exp(-½((x - center)/sigma)²)``. Prefer ``ff.mf.gaussian``."""
    def __init__(self, center: float, sigma: float) -> None: ...

class SingletonMF(_Shape):
    """Singleton MF — 1 exactly at ``x``, 0 elsewhere. Prefer ``ff.mf.singleton``."""
    def __init__(self, x: float) -> None: ...

class BellMF(_Shape):
    """Generalized bell MF ``1 / (1 + |(x - center)/width|^(2·slope))``. Prefer ``ff.mf.bell``."""
    def __init__(self, center: float, width: float, slope: float) -> None: ...

class SigmoidMF(_Shape):
    """Sigmoid MF ``1 / (1 + exp(-slope·(x - center)))``. Prefer ``ff.mf.sigmoid``."""
    def __init__(self, center: float, slope: float) -> None: ...

class Type1MF(_Shape):
    """A type-1 membership function, as returned by ``ff.mf.triangle(...)`` and friends.

    ``repr`` shows the constructor call, e.g. ``triangle(0, 5, 10)``. Calling the object
    is the same as ``.membership``.
    """
    @overload
    def __call__(self, x: float) -> float: ...
    @overload
    def __call__(self, x: ArrayLike) -> Any: ...
    def center(self) -> float:
        """Representative point: the peak (triangle, gaussian, bell, singleton), plateau
        midpoint (trapezoid) or inflection point (sigmoid). This is the constant a
        zero-order Sugeno rule outputs. It is *not* the centroid:
        ``triangle(0, 0, 5).center()`` is 0, its centroid is 5/3."""
        ...
    def shape(self) -> str:
        """Shape name: ``"Triangular"``, ``"Trapezoidal"``, ``"Gaussian"``, ``"Singleton"``,
        ``"Bell"`` or ``"Sigmoid"``."""
        ...
    def key_points(self) -> list[float]:
        """Points where the shape changes (feet, peaks, center ± k·sigma, ...)."""
        ...
    def __eq__(self, other: object) -> bool: ...

class IT2MF:
    """Interval type-2 membership function: a band (footprint of uncertainty, FOU)
    between an upper and a lower type-1 MF. Build with ``ff.mf.it2(upper, lower)``.

    >>> m = ff.mf.it2(ff.mf.triangle(0, 5, 10), ff.mf.triangle(2, 5, 8))
    >>> m.membership(3.0)
    (0.333..., 0.6)
    """
    def __init__(self, upper: AnyType1MF, lower: AnyType1MF, lower_height: float = 1.0) -> None: ...
    @overload
    def membership(self, x: float) -> tuple[float, float]: ...
    @overload
    def membership(self, x: ArrayLike) -> tuple[Any, Any]:
        """``(lower, upper)`` membership at ``x``; for a list / array, a pair of arrays."""
        ...
    @overload
    def __call__(self, x: float) -> tuple[float, float]: ...
    @overload
    def __call__(self, x: ArrayLike) -> tuple[Any, Any]: ...
    @property
    def upper(self) -> Type1MF:
        """Upper MF (UMF) — outer edge of the FOU."""
        ...
    @property
    def lower(self) -> Type1MF:
        """Lower MF (LMF) — inner edge of the FOU, before ``lower_height`` scaling."""
        ...
    @property
    def lower_height(self) -> float:
        """Factor in (0, 1] applied to the lower MF."""
        ...
    def center(self) -> float:
        """Midpoint of the upper and lower centers."""
        ...
    def shapes(self) -> tuple[str, str]:
        """``(upper_shape, lower_shape)`` names."""
        ...

# ─── ff.mf ───────────────────────────────────────────────────────────────────

class mf:
    """Membership-function constructors: ``ff.mf.triangle(0, 5, 10)``, ...

    Invalid parameters raise ``ff.FuzzyError`` (a ``ValueError``).
    """

    @staticmethod
    def triangle(a: float, b: float, c: float) -> Type1MF:
        """Triangle with feet ``a``, ``c`` and peak ``b`` (a ≤ b ≤ c).

        ``a == b`` makes a left shoulder, ``b == c`` a right shoulder; both are 1 at the
        edge — e.g. ``triangle(0, 0, 20)`` for "cold" on [0, 40].
        """
        ...
    @staticmethod
    def trapezoid(a: float, b: float, c: float, d: float) -> Type1MF:
        """Trapezoid rising on a→b, 1 on b..c, falling on c→d (a ≤ b ≤ c ≤ d)."""
        ...
    @staticmethod
    def gaussian(center: float, sigma: float) -> Type1MF:
        """Gaussian bell peaked at ``center`` with spread ``sigma`` > 0."""
        ...
    @staticmethod
    def singleton(x: float) -> Type1MF:
        """1 exactly at ``x``, 0 elsewhere — a constant Sugeno output."""
        ...
    @staticmethod
    def bell(center: float, width: float, slope: float) -> Type1MF:
        """Generalized bell: flat top of half-width ``width``, steeper for larger ``slope`` (both > 0)."""
        ...
    @staticmethod
    def sigmoid(center: float, slope: float) -> Type1MF:
        """S-curve through 0.5 at ``center``; ``slope`` > 0 rises, < 0 falls."""
        ...
    @staticmethod
    def it2(upper: AnyType1MF, lower: AnyType1MF, lower_height: float = 1.0) -> IT2MF:
        """Interval type-2 MF from an ``upper`` (outer) and ``lower`` (inner) type-1 MF.

        ``lower_height`` in (0, 1] scales the lower MF down (the textbook "shorter inner
        triangle"). Raises if the lower MF rises above the upper one — usually the two
        arguments are swapped.
        """
        ...

# ─── Variable ────────────────────────────────────────────────────────────────

class Variable:
    """Linguistic variable — a named range with fuzzy terms.

    >>> temp = ff.Variable("temp", 0, 40)
    >>> temp["cold"] = ff.mf.triangle(0, 0, 20)
    >>> temp["hot"] = ff.mf.triangle(20, 40, 40)
    >>> temp.fuzzify(30)
    {'cold': 0.0, 'hot': 0.5}

    or let ferrifuzz space the terms evenly:

    >>> temp = ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"])

    ``FuzzySystem.add_input`` / ``add_output`` store a *copy*: finish the terms first.
    """

    name: str
    """Variable name, as used in rule strings and ``compute`` dicts."""
    min_value: float
    """Lower bound of the universe."""
    max_value: float
    """Upper bound of the universe."""

    def __init__(self, name: str, min_value: float, max_value: float) -> None: ...
    def __setitem__(self, name: str, mf: AnyType1MF | IT2MF) -> None:
        """Add or replace a term: ``var["hot"] = ff.mf.triangle(20, 40, 40)``."""
        ...
    def __getitem__(self, name: str) -> Type1MF | IT2MF:
        """A term's MF; ``KeyError`` listing the terms when missing."""
        ...
    def __contains__(self, name: str) -> bool: ...
    def __len__(self) -> int: ...
    def __iter__(self) -> Iterator[str]: ...
    def __call__(self, value: float) -> dict[str, float | tuple[float, float]]:
        """Shorthand for ``fuzzify``."""
        ...
    def fuzzify(self, value: float) -> dict[str, float | tuple[float, float]]:
        """Membership of ``value`` in every term, in insertion order. Type-1 terms give a
        float, interval type-2 terms a ``(lower, upper)`` tuple.

        >>> temp = ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"])
        >>> temp.fuzzify(30)
        {'cold': 0.0, 'warm': 0.5, 'hot': 0.5}
        """
        ...
    def automf(self, names: int | Sequence[str] = 3, shape: ShapeName = "triangle") -> Variable:
        """Replace all terms with evenly spaced, overlapping terms; returns the variable.

        ``names``: a list of names, or a count — 2 → low/high, 3 → low/medium/high,
        5 → very_low/low/medium/high/very_high, 7 → NB NM NS ZE PS PM PB, otherwise
        A1..An. The first and last terms peak at ``min_value`` / ``max_value``;
        triangles sum to 1 everywhere inside the range. ``shape``: "triangle" or "gaussian".

        >>> ff.Variable("speed", 0, 100).automf(3).get_terms()
        ['low', 'medium', 'high']
        >>> ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"], shape="gaussian")
        """
        ...
    def get_terms(self) -> list[str]:
        """Term names, in insertion order."""
        ...
    def clamp(self, value: float) -> float:
        """``value`` clipped into ``[min_value, max_value]``."""
        ...
    def __str__(self) -> str:
        """All terms with parameters: ``temp [0, 40]: cold = triangle(0, 0, 20), ...``."""
        ...

# ─── Rules ───────────────────────────────────────────────────────────────────

class Clause:
    """One clause ``var IS [NOT] [hedge] term``.

    >>> ff.Clause("temp", "hot", negated=True, hedge="very")   # temp IS NOT very hot
    """

    var: str
    """Variable name the clause tests."""
    term: str
    """Term name the clause tests."""
    negated: bool
    hedge: str | None
    hedge_power: float | None

    def __init__(self, var: str, term: str, negated: bool = False, hedge: HedgeWord | None = None) -> None: ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, other: object) -> bool: ...

_Conclusion = Union[tuple[str, str], list[tuple[str, str]], Clause, list[Clause]]

class Rule:
    """A fuzzy IF-THEN rule. Usually written as a string::

        IF temp IS hot AND humidity IS NOT low THEN fan IS fast
        IF (temp IS hot OR temp IS very warm) AND wind IS calm THEN fan IS fast AND alarm IS on WITH 0.8

    Syntax: keywords (IF THEN AND OR IS NOT WITH) are case-insensitive; variable and term
    names are single words (use underscores) and case-sensitive. AND binds tighter than
    OR; parentheses are allowed. Hedges after IS [NOT]: very (μ²), extremely (μ³),
    somewhat / more_or_less / slightly (√μ), plus (μ^1.25), minus (μ^0.75).
    ``WITH w`` sets the rule weight in [0, 1]. Several outputs are joined with AND after THEN.

    Internally the condition is an OR of AND-groups (``antecedent``); parentheses are
    expanded into that form, which is exact for min/max operators.
    """

    antecedent: list[list[tuple[str, str]]]
    """OR of AND-groups as ``(var, term)`` pairs (NOT / hedges dropped; see ``clauses``)."""
    conditions: list[tuple[str, str]]
    """Flat AND list; ``ValueError`` for rules that use OR."""
    conclusion: tuple[str, str]
    """The single ``(output, term)``; ``ValueError`` for multi-output rules."""
    conclusions: list[tuple[str, str]]
    weight: float
    """Rule weight in [0, 1]; scales the firing strength."""
    """Firing-strength multiplier in [0, 1] (checked when the system runs)."""
    @property
    def clauses(self) -> list[list[Clause]]: ...
    @property
    def consequent_clauses(self) -> list[Clause]: ...

    def __init__(
        self,
        conditions: list[tuple[str, str]] | None = None,
        conclusion: _Conclusion | None = None,
        antecedent: list[list[tuple[str, str]]] | None = None,
        weight: float = 1.0,
    ) -> None:
        """Build a rule from ``conditions`` (AND list) *or* ``antecedent`` (OR of AND
        lists), plus a ``conclusion``. Strings via ``Rule.from_string`` are usually easier."""
        ...
    @classmethod
    def from_string(cls, rule_str: str) -> Rule:
        """Parse a rule string (see the class docs). Raises ``ff.RuleSyntaxError`` with a
        caret under the offending word."""
        ...
    def group_count(self) -> int:
        """Number of OR-groups (1 for an AND-only rule)."""
        ...
    def consequent_count(self) -> int:
        """Number of outputs the rule concludes."""
        ...
    def __str__(self) -> str:
        """The rule as a parseable string (weights as ``WITH w``)."""
        ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, other: object) -> bool: ...

# ─── Operators & hedges ──────────────────────────────────────────────────────

class Operator:
    """Fuzzy operators on degrees — a float, or ``(lower, upper)`` for interval type-2.

    >>> ff.Operator.t_norm(0.7, 0.4)                    # AND → 0.4
    >>> ff.Operator.t_conorm(0.7, 0.4, kind="probor")   # OR  → 0.82
    >>> ff.Operator.complement(0.3)                     # NOT → 0.7
    """

    @staticmethod
    def t_norm(mu1: float | tuple[float, float], mu2: float | tuple[float, float], kind: TNormName = "min") -> float | tuple[float, float]:
        """Fuzzy AND. ``kind``: min (a∧b), product (ab), lukasiewicz (max(0, a+b-1)),
        drastic, einstein (ab / (2-(a+b-ab))), hamacher (ab / (a+b-ab))."""
        ...
    @staticmethod
    def t_conorm(mu1: float | tuple[float, float], mu2: float | tuple[float, float], kind: TConormName = "max") -> float | tuple[float, float]:
        """Fuzzy OR. ``kind``: max, probor (a+b-ab), lukasiewicz (min(1, a+b)), drastic,
        einstein ((a+b) / (1+ab)), hamacher ((a+b-2ab) / (1-ab))."""
        ...
    @staticmethod
    def complement(mu: float | tuple[float, float], kind: ComplementName = "standard", param: float = 1.0) -> float | tuple[float, float]:
        """Fuzzy NOT. ``standard``: 1-μ; ``sugeno``: (1-μ)/(1+λμ) with λ = param > -1;
        ``yager``: (1-μ^w)^(1/w) with w = param > 0. Interval bounds stay ordered."""
        ...

class Hedge:
    """Linguistic hedge ``μ ↦ μ^power`` (> 1 concentrates, < 1 dilates).

    >>> ff.Hedge.very()(0.8)     # 0.64
    >>> ff.Hedge(3.0)(0.8)       # 0.512
    """

    def __init__(self, power: float) -> None: ...
    @property
    def power(self) -> float:
        """Exponent of the hedge: ``μ ↦ μ^power``."""
        ...
    def __call__(self, mu: float | tuple[float, float]) -> float | tuple[float, float]: ...
    @staticmethod
    def very() -> Hedge:
        """power 2.0"""
        ...
    @staticmethod
    def extremely() -> Hedge:
        """power 3.0"""
        ...
    @staticmethod
    def somewhat() -> Hedge:
        """power 0.5"""
        ...
    @staticmethod
    def slightly() -> Hedge:
        """power 0.5 (same as somewhat, matching the rule-string word)"""
        ...
    @staticmethod
    def plus() -> Hedge:
        """power 1.25"""
        ...
    @staticmethod
    def minus() -> Hedge:
        """power 0.75"""
        ...

# ─── FuzzySystem ─────────────────────────────────────────────────────────────

class FuzzySystem:
    """A fuzzy inference system — Mamdani, Sugeno (TSK) or interval type-2.

    >>> system = ff.FuzzySystem()                   # Mamdani, centroid
    >>> system.add_input(temp)
    >>> system.add_output(fan)
    >>> system += "IF temp IS cold THEN fan IS slow"
    >>> system += "IF temp IS hot THEN fan IS fast"
    >>> system({"temp": 30})
    {'fan': ...}

    The system is checked the first time it runs: a misspelled variable or term raises
    ``ff.ModelError`` with a "did you mean" hint instead of silently returning a number.

    Parameters
    ----------
    and_method : "min" | "product" | "lukasiewicz" | "drastic" | "einstein" | "hamacher"
        T-norm combining clauses joined by AND.
    or_method : "max" | "probor" | "lukasiewicz" | "drastic" | "einstein" | "hamacher"
        T-conorm combining OR-groups.
    defuzz_method : "centroid" | "bisector" | "mom" | "som" | "lom" | "weighted_average"
        ``weighted_average`` (alias ``sugeno``) makes a Sugeno system: each rule outputs
        its consequent term's ``center()`` (or a linear function from ``add_consequent``)
        and the result is their firing-weighted average. The other methods sample the
        aggregated set on ``resolution + 1`` points, so results lie on that grid.
    resolution : int
        Sampling steps over each output range (default 200).
    type : "auto" | "type1" | "it2"
        ``auto`` runs interval type-2 inference when any variable has an IT2 term.
    implication : "mamdani" | "larsen"
        Clip (min) or scale (product) consequents by the firing strength.
    type_reduction : "km" | "cos" | "height"
        Interval type-2 only (Karnik–Mendel, center-of-sets, height). IT2 systems ignore
        ``defuzz_method`` — leave it at "centroid".
    clamp_inputs : bool
        Clip inputs into each variable's range first (default True), so values slightly
        outside still fire the edge terms instead of producing NaN.
    """

    def __init__(
        self,
        and_method: TNormName = "min",
        or_method: TConormName = "max",
        defuzz_method: DefuzzName = "centroid",
        resolution: int = 200,
        type: InferenceName = "auto",
        implication: ImplicationName = "mamdani",
        type_reduction: TypeReductionName = "km",
        clamp_inputs: bool = True,
    ) -> None: ...

    # Building
    def add_input(self, variable: Variable) -> None:
        """Register a copy of ``variable`` as an input (replaces one with the same name)."""
        ...
    def add_output(self, variable: Variable) -> None:
        """Register a copy of ``variable`` as an output (replaces one with the same name)."""
        ...
    def add_rule(self, rule: Rule | str) -> None:
        """Append a rule — a ``Rule`` or a rule string."""
        ...
    def add_rules(self, rules: Sequence[Rule | str]) -> None:
        """Append several rules; nothing is added if one fails to parse."""
        ...
    def __iadd__(self, rule: Rule | str) -> FuzzySystem:
        """``system += "IF temp IS hot THEN fan IS fast"``"""
        ...
    def add_consequent(self, out_var: str, term: str, coeffs: Mapping[str, float], bias: float = 0.0) -> None:
        """First-order Sugeno (TSK) consequent: rules concluding ``out_var IS term`` output
        ``bias + Σ coeffs[input] · input``. Needs ``defuzz_method="weighted_average"``.

        ``term`` may be a name used only in rules, so every rule can carry its own linear
        function: ``add_consequent("y", "r1", {"x": 2.0}, bias=1.0)`` with
        ``"IF x IS low THEN y IS r1"``.
        """
        ...

    # Running
    def compute(self, inputs: Mapping[str, float]) -> dict[str, float]:
        """Run inference: ``{input: value}`` → ``{output: value}`` (every output, NaN where
        no rule fired). Every input must be given.

        Raises ``ff.InputError`` (missing / unknown / NaN input), ``ff.ModelError``
        (inconsistent rules).

        >>> system.compute({"temp": 30})
        {'fan': 56.078...}
        """
        ...
    def __call__(self, inputs: Mapping[str, float]) -> dict[str, float]:
        """Shorthand for ``compute``."""
        ...
    def compute_batch(self, inputs: Mapping[str, ArrayLike]) -> dict[str, Any]:
        """Many rows at once: ``{input: column}`` (lists or NumPy arrays, equal length) →
        ``{output: column}`` as NumPy arrays (lists without NumPy). Runs on all CPU cores.
        Rows where no rule fired are NaN.

        >>> system.compute_batch({"temp": [10, 20, 30]})
        {'fan': array([43.92..., 50.0, 56.07...])}
        """
        ...
    def evaluate(self, values: Sequence[float]) -> list[float]:
        """Positional ``compute``: inputs in ``input_names()`` order, outputs in
        ``output_names()`` order. The quickest call inside a simulation loop.

        >>> for _ in range(steps):
        ...     (fan,) = system.evaluate([temp])
        ...     temp -= 0.01 * fan
        """
        ...
    def compute_interval(self, inputs: Mapping[str, float]) -> dict[str, tuple[float, float]]:
        """Interval type-2: the type-reduced interval ``(y_l, y_r)`` per output (the crisp
        output is its midpoint). Type-1 systems return ``(y, y)``."""
        ...

    # Looking inside
    def rule_strengths(self, inputs: Mapping[str, float]) -> list[float]:
        """Firing strength of every rule, in rule order (after weights) — e.g. to fit
        Sugeno consequents by least squares (ANFIS-style).

        >>> system.rule_strengths({"temp": 30})    # cold, warm, hot
        [0.0, 0.5, 0.5]
        """
        ...
    def rule_strengths_interval(self, inputs: Mapping[str, float]) -> list[tuple[float, float]]:
        """Firing interval ``(lower, upper)`` of every rule (IT2)."""
        ...
    def output_set(self, inputs: Mapping[str, float], output: str) -> tuple[Any, Any]:
        """Aggregated output fuzzy set ``(xs, mus)`` of a type-1 Mamdani system — what the
        defuzzifier sees. See also ``ferrifuzz.plot.plot_output``."""
        ...
    def output_fou(self, inputs: Mapping[str, float], output: str) -> tuple[Any, Any, Any]:
        """Aggregated footprint of uncertainty ``(xs, lower, upper)`` (IT2 Mamdani)."""
        ...
    def check(self) -> None:
        """Raise ``ff.ModelError`` describing every problem that would make ``compute`` fail."""
        ...
    def validate(self) -> list[str]:
        """Every problem as text: ``"error: ..."`` lines (what ``check`` raises) then
        ``"warning: ..."`` lines (unused inputs, outputs no rule concludes, duplicate or
        conflicting rules, terms that are zero over their whole range). Empty when clean.

        >>> system.validate()
        ["warning: input variable 'hum' is never used by any rule"]
        """
        ...
    def input(self, name: str) -> Variable | None:
        """Copy of a registered input variable."""
        ...
    def output(self, name: str) -> Variable | None:
        """Copy of a registered output variable."""
        ...
    def input_names(self) -> list[str]:
        """Names of the registered inputs, in the order they were added."""
        ...
    def output_names(self) -> list[str]:
        """Names of the registered outputs, in the order they were added."""
        ...
    def rule_count(self) -> int:
        """Number of rules in the rule base."""
        ...
    def rules_str(self) -> list[str]:
        """All rules as parseable strings."""
        ...
    @property
    def rules(self) -> list[Rule]:
        """Copies of the rules."""
        ...

    # Settings (canonical names)
    @property
    def and_method(self) -> str:
        """T-norm combining clauses joined by AND (canonical name)."""
        ...
    @property
    def or_method(self) -> str:
        """T-conorm combining OR-groups (canonical name)."""
        ...
    @property
    def defuzz_method(self) -> str:
        """Defuzzification method (canonical name)."""
        ...
    @property
    def implication(self) -> str:
        """``"mamdani"`` (clip consequents) or ``"larsen"`` (scale them)."""
        ...
    @property
    def type(self) -> str:
        """Engine requested: ``"auto"``, ``"type1"`` or ``"it2"``."""
        ...
    @property
    def type_reduction(self) -> str:
        """IT2 type reduction: ``"km"``, ``"cos"`` or ``"height"``."""
        ...
    @property
    def resolution(self) -> int:
        """Sampling steps over each output range."""
        ...
    @property
    def clamp_inputs(self) -> bool:
        """True when inputs are clipped into each variable's range first."""
        ...
    @property
    def is_it2(self) -> bool:
        """True when the interval type-2 engine runs."""
        ...
