"""
Type stubs for the ferrifuzz Rust extension (_core).

These stubs power VS Code IntelliSense, Pylance, and mypy.
Do not import from this file directly — import from ``ferrifuzz``.
"""

from __future__ import annotations

from typing import Any, Sequence

# ─── Membership function classes ─────────────────────────────────────────────

class TriangularMF:
    """Triangular membership function.

    Parameters
    ----------
    a : float  Left foot — membership = 0.
    b : float  Peak — membership = 1.
    c : float  Right foot — membership = 0.  Must satisfy a ≤ b ≤ c.
    """
    def __init__(self, a: float, b: float, c: float) -> None: ...
    def membership(self, x: float) -> float:
        """Return the membership degree at ``x`` ∈ [0, 1]."""
        ...

class TrapezoidalMF:
    """Trapezoidal membership function.

    Parameters
    ----------
    a : float  Left foot.
    b : float  Left shoulder (plateau starts).
    c : float  Right shoulder (plateau ends).
    d : float  Right foot.  Must satisfy a ≤ b ≤ c ≤ d.
    """
    def __init__(self, a: float, b: float, c: float, d: float) -> None: ...
    def membership(self, x: float) -> float:
        """Return the membership degree at ``x``."""
        ...

class GaussianMF:
    """Gaussian membership function.

    Parameters
    ----------
    center : float  Peak location.
    sigma : float   Standard deviation — controls width.  Must be > 0.
    """
    def __init__(self, center: float, sigma: float) -> None: ...
    def membership(self, x: float) -> float:
        """Return the membership degree at ``x``."""
        ...

class SingletonMF:
    """Singleton — membership = 1 only exactly at ``x``, 0 elsewhere.

    Parameters
    ----------
    x : float  The single point of full membership.
    """
    def __init__(self, x: float) -> None: ...
    def membership(self, x: float) -> float: ...

class BellMF:
    """Generalized bell-shaped membership function.

    Formula: ``1 / (1 + |(x - center) / width|^(2 * slope))``

    Parameters
    ----------
    center : float  Peak center.
    width : float   Half-width at crossover (must be > 0).
    slope : float   Side steepness — higher = sharper walls (must be > 0).

    Examples
    --------
    >>> mf = ff.BellMF(center=50.0, width=15.0, slope=2.0)
    >>> mf.membership(50.0)   # 1.0
    >>> mf.membership(65.0)   # 0.5
    """
    def __init__(self, center: float, width: float, slope: float) -> None: ...
    def membership(self, x: float) -> float:
        """Return the membership degree at ``x``."""
        ...

class SigmoidMF:
    """Sigmoid membership function.

    Formula: ``1 / (1 + exp(-slope * (x - center)))``

    Parameters
    ----------
    center : float  Inflection point (membership = 0.5).
    slope : float   Steepness — positive → rising, negative → falling.

    Examples
    --------
    >>> mf = ff.SigmoidMF(center=50.0, slope=0.2)   # rising
    >>> mf = ff.SigmoidMF(center=50.0, slope=-0.2)  # falling
    """
    def __init__(self, center: float, slope: float) -> None: ...
    def membership(self, x: float) -> float:
        """Return the membership degree at ``x``."""
        ...

class Type1MF:
    """Unified Type-1 membership function wrapper (created via ``ff.mf.*``).

    Examples
    --------
    >>> hot = ff.mf.triangle(20.0, 35.0, 40.0)
    >>> hot.membership(30.0)
    """
    def membership(self, x: float) -> float:
        """Return the membership degree at ``x``."""
        ...
    def center(self) -> float:
        """Analytic center (representative point) of the MF — O(1), exact.

        Triangle → peak, trapezoid → plateau midpoint, gaussian/bell/sigmoid →
        ``center``, singleton → its value. Used as ``z`` in Sugeno inference.
        """
        ...
    def shape(self) -> str:
        """Return the shape name: ``"Triangular"``, ``"Gaussian"``, etc."""
        ...
    def __repr__(self) -> str: ...

class IT2MF:
    """Interval Type-2 membership function.

    Holds an upper MF and a lower MF, evaluates to an interval [lower, upper].

    Parameters
    ----------
    upper : Type1MF  Upper membership function.
    lower : Type1MF  Lower membership function (≤ upper everywhere).

    Examples
    --------
    >>> umf = ff.mf.triangle(0, 5, 10)
    >>> lmf = ff.mf.triangle(2, 5, 8)
    >>> it2 = ff.mf.it2(umf, lmf)
    >>> it2.membership(5.0)
    (1.0, 1.0)
    """
    def __init__(self, upper: Type1MF, lower: Type1MF) -> None: ...
    def membership(self, x: float) -> tuple[float, float]:
        """Return ``(lower_mu, upper_mu)`` at ``x``."""
        ...
    def shapes(self) -> tuple[str, str]:
        """Return ``(upper_shape, lower_shape)`` names."""
        ...

# ─── ff.mf submodule ─────────────────────────────────────────────────────────

class mf:
    """Factory functions for membership functions — ``ff.mf.triangle()``, etc."""

    @staticmethod
    def triangle(a: float, b: float, c: float) -> Type1MF:
        """Create a triangular MF — peak at ``b``, zero at ``a`` and ``c``.

        Parameters
        ----------
        a : float  Left foot.
        b : float  Peak (must satisfy a ≤ b ≤ c).
        c : float  Right foot.
        """
        ...

    @staticmethod
    def trapezoid(a: float, b: float, c: float, d: float) -> Type1MF:
        """Create a trapezoidal MF — flat plateau between ``b`` and ``c``.

        Parameters
        ----------
        a, b, c, d : float  Consecutive breakpoints (a ≤ b ≤ c ≤ d).
        """
        ...

    @staticmethod
    def gaussian(center: float, sigma: float) -> Type1MF:
        """Create a Gaussian MF — bell curve with peak at ``center``.

        Parameters
        ----------
        center : float  Peak location.
        sigma : float   Standard deviation (must be > 0).
        """
        ...

    @staticmethod
    def singleton(x: float) -> Type1MF:
        """Create a singleton MF — membership = 1 only exactly at ``x``."""
        ...

    @staticmethod
    def bell(center: float, width: float, slope: float) -> Type1MF:
        """Create a generalized bell MF.

        Parameters
        ----------
        center : float  Peak location.
        width : float   Half-width at crossover point (> 0).
        slope : float   Side steepness (> 0; higher = sharper walls).

        Examples
        --------
        >>> mf = ff.mf.bell(50.0, 15.0, 2.0)
        >>> mf.membership(50.0)   # 1.0
        """
        ...

    @staticmethod
    def sigmoid(center: float, slope: float) -> Type1MF:
        """Create a sigmoid MF.

        Parameters
        ----------
        center : float  Inflection point (membership = 0.5).
        slope : float   Steepness (positive → rising, negative → falling).

        Examples
        --------
        >>> ff.mf.sigmoid(50.0,  0.3)   # rising sigmoid
        >>> ff.mf.sigmoid(50.0, -0.3)   # falling sigmoid
        """
        ...

    @staticmethod
    def it2(upper: Type1MF, lower: Type1MF) -> IT2MF:
        """Create an Interval Type-2 MF from an upper and lower Type-1 MF."""
        ...

# ─── Variable ────────────────────────────────────────────────────────────────

class Variable:
    """Linguistic variable — a named universe with fuzzy terms.

    Parameters
    ----------
    name : str         Human-readable name (e.g. ``"Temperature"``).
    min_value : float  Lower bound of the universe of discourse.
    max_value : float  Upper bound of the universe of discourse.

    Examples
    --------
    >>> temp = ff.Variable("Temperature", min_value=0.0, max_value=100.0)
    >>> temp["Cold"] = ff.mf.triangle(0.0,   0.0,  40.0)
    >>> temp["Warm"] = ff.mf.triangle(20.0,  50.0,  80.0)
    >>> temp["Hot"]  = ff.mf.triangle(60.0, 100.0, 100.0)
    >>> temp.fuzzify(75.0)
    {'Cold': 0.0, 'Warm': 0.166..., 'Hot': 0.375}
    """

    name: str
    min_value: float
    max_value: float

    def __init__(self, name: str, min_value: float, max_value: float) -> None: ...

    def __setitem__(self, name: str, mf: Type1MF) -> None:
        """Add or replace a term: ``var["Hot"] = ff.mf.triangle(...)``."""
        ...

    def __getitem__(self, name: str) -> Type1MF:
        """Retrieve a term's MF by name: ``var["Hot"]``."""
        ...

    def __call__(self, value: float) -> dict[str, float]:
        """Shorthand for :meth:`fuzzify`."""
        ...

    def fuzzify(self, value: float) -> dict[str, float]:
        """Compute membership degrees for ``value`` across all terms.

        Parameters
        ----------
        value : float  Crisp input within the universe of discourse.

        Returns
        -------
        dict[str, float]
            ``{term_name: degree}`` for every registered term.

        Examples
        --------
        >>> temp.fuzzify(75.0)
        {'Cold': 0.0, 'Warm': 0.166..., 'Hot': 0.375}
        """
        ...

    def get_terms(self) -> list[str]:
        """Return names of all registered terms."""
        ...

    def __repr__(self) -> str: ...

# ─── Rule ────────────────────────────────────────────────────────────────────

class Rule:
    """A fuzzy IF-THEN rule. The antecedent supports AND and OR.

    The antecedent is stored in disjunctive normal form (DNF): an OR of
    AND-groups, e.g. ``(a AND b) OR c`` → ``[[a, b], [c]]``. AND binds tighter
    than OR.

    Attributes
    ----------
    antecedent : list[list[tuple[str, str]]]
        DNF antecedent — outer list is OR, inner lists are AND-groups.
    conditions : list[tuple[str, str]]
        Backward-compatible flat AND list. Only valid for single-group
        (AND-only) rules; raises ``ValueError`` if the rule uses OR.
    conclusion : tuple[str, str]
        ``(output_variable_name, term_name)`` — consequent.

    Examples
    --------
    >>> r = ff.Rule.from_string(
    ...     "IF temp IS hot AND humidity IS high OR temp IS extreme THEN fan IS fast"
    ... )
    >>> r.antecedent
    [[('temp', 'hot'), ('humidity', 'high')], [('temp', 'extreme')]]
    """

    antecedent: list[list[tuple[str, str]]]
    conditions: list[tuple[str, str]]
    conclusion: tuple[str, str]

    def __init__(
        self,
        conditions: list[tuple[str, str]] | None = None,
        conclusion: tuple[str, str] | None = None,
        antecedent: list[list[tuple[str, str]]] | None = None,
    ) -> None:
        """Construct a rule.

        Pass exactly one of ``conditions`` (a flat AND list) or ``antecedent``
        (an explicit DNF for OR rules). ``conclusion`` is required.
        """
        ...

    @classmethod
    def from_string(cls, rule_str: str) -> "Rule":
        """Parse a rule from a human-readable string.

        Format
        ------
        ``IF <var> IS <term> [(AND|OR) <var> IS <term> ...] THEN <var> IS <term>``

        ``AND`` binds tighter than ``OR``.

        Parameters
        ----------
        rule_str : str  Rule string (variable/term names are case-sensitive).

        Examples
        --------
        >>> rule = ff.Rule.from_string("IF temp IS hot OR temp IS extreme THEN fan IS fast")
        """
        ...

    def group_count(self) -> int:
        """Number of OR-groups in the antecedent (1 for a plain AND rule)."""
        ...

    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __ne__(self, other: object) -> bool: ...

# ─── Operator ────────────────────────────────────────────────────────────────

class Operator:
    """Static fuzzy set operators: T-norm, T-conorm, complement.

    Works with both Type-1 (``float``) and IT2 (``tuple[float, float]``)
    membership degrees.

    Examples
    --------
    >>> ff.Operator.t_norm(0.7, 0.5)             # 0.5  (min)
    >>> ff.Operator.t_norm(0.7, 0.5, "product")  # 0.35
    >>> ff.Operator.t_conorm(0.3, 0.6)           # 0.6  (max)
    >>> ff.Operator.complement(0.7)              # 0.3
    """

    @staticmethod
    def t_norm(
        mu1: float | tuple[float, float],
        mu2: float | tuple[float, float],
        kind: str = "min",
    ) -> float | tuple[float, float]:
        """Fuzzy AND (T-norm / intersection).

        Parameters
        ----------
        mu1, mu2 : float or tuple[float, float]
            Membership degrees (Type-1 float or IT2 interval).
        kind : str
            ``"min"`` (Gödel, default) or ``"product"`` (algebraic).
        """
        ...

    @staticmethod
    def t_conorm(
        mu1: float | tuple[float, float],
        mu2: float | tuple[float, float],
        kind: str = "max",
    ) -> float | tuple[float, float]:
        """Fuzzy OR (T-conorm / union).

        Parameters
        ----------
        mu1, mu2 : float or tuple[float, float]
            Membership degrees.
        kind : str
            ``"max"`` (Gödel, default) or ``"prob"`` (probabilistic sum).
        """
        ...

    @staticmethod
    def complement(
        mu: float | tuple[float, float],
    ) -> float | tuple[float, float]:
        """Fuzzy NOT — standard complement ``1 - μ``.

        Parameters
        ----------
        mu : float or tuple[float, float]  Membership degree.
        """
        ...

# ─── Hedge ───────────────────────────────────────────────────────────────────

class Hedge:
    """Linguistic hedge — modifies membership via a power function: ``μ^power``.

    Parameters
    ----------
    power : float  Exponent.  ``> 1`` concentrates (steepens), ``< 1`` dilates.

    Examples
    --------
    >>> very     = ff.Hedge.very()      # power = 2.0
    >>> somewhat = ff.Hedge.somewhat()  # power = 0.5
    >>> very(0.8)        # 0.64
    >>> somewhat(0.64)   # 0.8
    >>> ff.Hedge(3.0)(0.8)  # 0.512  — "extremely"
    """

    def __init__(self, power: float) -> None: ...

    def __call__(
        self, mu: float | tuple[float, float]
    ) -> float | tuple[float, float]:
        """Apply the hedge to a membership degree."""
        ...

    @staticmethod
    def very() -> "Hedge":
        """Predefined *very* hedge (power = 2.0) — concentrates membership."""
        ...

    @staticmethod
    def somewhat() -> "Hedge":
        """Predefined *somewhat* hedge (power = 0.5) — dilates membership."""
        ...

    @staticmethod
    def plus() -> "Hedge":
        """Predefined *plus* hedge (power = 1.25)."""
        ...

    @staticmethod
    def minus() -> "Hedge":
        """Predefined *minus* hedge (power = 0.75)."""
        ...

# ─── FuzzySystem ─────────────────────────────────────────────────────────────

class FuzzySystem:
    """Mamdani fuzzy inference system.

    Parameters
    ----------
    and_method : str
        T-norm combining conditions inside an AND-group.
        ``"min"`` (Mamdani, default) or ``"product"`` (Larsen).
    or_method : str
        T-conorm combining OR-groups of a rule antecedent.
        ``"max"`` (default) or ``"probor"`` (probabilistic OR).
    defuzz_method : str
        Defuzzification strategy for output variables. Options:

        * ``"centroid"``        — Centre of Gravity (default)
        * ``"bisector"``        — Bisector of Area
        * ``"mom"``             — Mean of Maxima
        * ``"som"``             — Smallest of Maxima
        * ``"lom"``             — Largest of Maxima
        * ``"weighted_average"``— Weighted Average / 0th-order Sugeno
          (alias ``"sugeno"``); uses each term's analytic center, no sampling.

    resolution : int
        Number of sample points for numerical integration (default 200).

    Examples
    --------
    >>> import ferrifuzz as ff
    >>>
    >>> temp = ff.Variable("temp", 0.0, 100.0)
    >>> temp["cold"] = ff.mf.triangle(0.0,   0.0,  40.0)
    >>> temp["hot"]  = ff.mf.triangle(60.0, 100.0, 100.0)
    >>>
    >>> fan = ff.Variable("fan", 0.0, 100.0)
    >>> fan["slow"] = ff.mf.triangle(0.0,  0.0,  50.0)
    >>> fan["fast"] = ff.mf.triangle(50.0, 100.0, 100.0)
    >>>
    >>> system = ff.FuzzySystem()
    >>> system.add_input(temp)
    >>> system.add_output(fan)
    >>> system += ff.Rule.from_string("IF temp IS cold THEN fan IS slow")
    >>> system += ff.Rule.from_string("IF temp IS hot  THEN fan IS fast")
    >>>
    >>> system({"temp": 80.0})
    {'fan': 75.0}
    """

    def __init__(
        self,
        and_method: str = "min",
        or_method: str = "max",
        defuzz_method: str = "centroid",
        resolution: int = 200,
    ) -> None: ...

    def add_input(self, variable: Variable) -> None:
        """Register a ``Variable`` as a system input."""
        ...

    def add_output(self, variable: Variable) -> None:
        """Register a ``Variable`` as a system output."""
        ...

    def add_consequent(
        self,
        out_var: str,
        term: str,
        coeffs: dict[str, float],
        bias: float = 0.0,
    ) -> None:
        """Register a 1st-order (TSK) linear consequent for a Sugeno output term.

        The fired rule concluding ``... THEN out_var IS term`` emits
        ``z = bias + Σ coeffs[input_var] · input_value`` instead of an MF center.
        Use with ``defuzz_method="sugeno"``. Empty ``coeffs`` → a constant
        (0th-order) consequent. Sugeno outputs are weighted per rule:
        ``y = Σ(wᵢ·zᵢ) / Σ(wᵢ)``.
        """
        ...

    def add_rule(self, rule: Rule) -> None:
        """Append a rule to the rule base."""
        ...

    def __iadd__(self, rule: Rule) -> "FuzzySystem":
        """``system += rule`` — append a rule to the rule base."""
        ...

    def compute(self, inputs: dict[str, float]) -> dict[str, float]:
        """Run Mamdani inference; return defuzzified outputs.

        Parameters
        ----------
        inputs : dict[str, float]
            ``{variable_name: crisp_value}`` for every input variable.

        Returns
        -------
        dict[str, float]
            Defuzzified crisp output for each output variable.

        Examples
        --------
        >>> system.compute({"temperature": 35.0, "humidity": 60.0})
        {'fan_speed': 72.5}
        """
        ...

    def compute_batch(
        self, inputs: dict[str, Sequence[float]]
    ) -> dict[str, Any]:
        """Vectorized inference over a batch of input rows in one call.

        Columnar in, columnar out — ``{var: [values]}`` to ``{out_var: [values]}``.
        Each input column is a 1-D sequence (list or NumPy float64 array); all
        columns must share the same length ``N``. Result columns are NumPy arrays
        when NumPy is importable, otherwise plain lists. Rows where an output
        variable has no firing rule are filled with ``NaN``.

        The whole ``N``-row loop runs in Rust over the shared inference core — one
        FFI crossing and no per-row Python object churn, unlike calling
        :meth:`compute` in a Python loop.

        Parameters
        ----------
        inputs : dict[str, Sequence[float]]
            ``{variable_name: column}`` with every column of equal length ``N``.

        Returns
        -------
        dict[str, Any]
            ``{output_name: column}`` — NumPy arrays if NumPy is available,
            else lists of ``float``.

        Examples
        --------
        >>> import numpy as np
        >>> out = system.compute_batch({"temperature": np.array([20.0, 55.0, 90.0])})
        >>> out["fan"]            # np.ndarray of shape (3,)
        """
        ...

    def __call__(self, inputs: dict[str, float]) -> dict[str, float]:
        """Shorthand for :meth:`compute` — ``system({"temp": 35.0})``."""
        ...

    def input_names(self) -> list[str]:
        """Names of all registered input variables."""
        ...

    def output_names(self) -> list[str]:
        """Names of all registered output variables."""
        ...

    def rule_count(self) -> int:
        """Number of rules in the rule base."""
        ...

    def rules_str(self) -> list[str]:
        """All rules as human-readable strings."""
        ...

    def __repr__(self) -> str: ...
