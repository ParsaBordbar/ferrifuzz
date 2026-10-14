# Ferrifuzz TODO

Two tracks. **Track A** closes the Tier-1 correctness/usability gaps in the existing
engine. **Track B** makes the library a first-class citizen next to NumPy and PyTorch.
Track A items are independent of each other; Track B depends on A1 and A2 only where noted.

---

## Track A — Tier 1

### A1. Multi-consequent rules
`Rule.conclusion` is a single `(String, String)` (`src/rule.rs:96`), so a rule can only
drive one output variable. Every MIMO controller needs more.

- [ ] Change `Rule.conclusion` to `Vec<(String, String)>`; keep a `conclusion` property
      that returns the single tuple (and raises on multi) for back-compat, add
      `conclusions` as the plural accessor.
- [ ] Parse `THEN z IS c AND w IS d` in `Rule::from_string` (`src/rule.rs:from_string`).
      Reuse the antecedent clause parser so `IS NOT` / hedges work in consequents too.
- [ ] Update `Rule::__str__` to round-trip multi-consequent rules.
- [ ] Update firing loops in `FuzzySystem::infer` and `infer_it2` (`src/system.rs:379`,
      `src/system.rs:517`) to fan a rule's firing strength out to every consequent.
- [ ] Update `validate()` (`src/system.rs:289`) to check each consequent.
- [ ] Tests: single-consequent behaviour unchanged; a 2-output rule activates both;
      Sugeno consequents resolve per output var.

### A2. Inference trace / explainability
`compute` returns only the crisp dict (`src/system.rs:186`). Nothing about *why*.
This also blocks plotting (A5) and is the main debugging complaint.

- [ ] Add `compute(inputs, detail=False)`. With `detail=True` return a `Result` object
      exposing: `outputs`, `fuzzified` (`{var: {term: mu}}`), `firing`
      (`[(rule_index, rule_str, strength)]`), `activations` (`{out_var: {term: mu}}`),
      and `aggregated` (`{out_var: (xs, mus)}` — reuse `Sampled` from `sample()`,
      `src/system.rs:510`).
- [ ] IT2 path returns interval firing strengths `(lower, upper)` and the type-reduced
      interval `[y_l, y_r]` before midpointing (`src/system.rs:575`).
- [ ] `system.explain(inputs)` — human-readable string ranking rules by contribution.
- [ ] Keep the fast path allocation-free: `detail=False` must not build any of this.
- [ ] Tests: firing strengths sum/max match hand-computed values on a 2-rule system.

### A3. Serialization
No save/load exists. Systems only live as code.

- [ ] `system.to_dict()` / `FuzzySystem.from_dict()` covering variables, MFs (type-1 and
      IT2), rules, weights, consequents, and every engine setting from
      `FuzzySystem::new` (`src/system.rs:122`).
- [ ] `to_json(path=None)` / `from_json(path_or_str)` on top of the dict form.
- [ ] Version the schema (`"ferrifuzz_schema": 1`) so future loads can migrate.
- [ ] FCL (IEC 61131-7) export + import — the interchange format `jFuzzyLogic` and
      `fuzzylite` speak.
- [ ] MATLAB `.fis` reader (import only; enough to run the Fuzzy Logic Toolbox demos).
- [ ] Tests: round-trip equality for a system using every MF shape, IT2 terms, OR-rules,
      hedges, negation, Sugeno consequents.

### A4. `Variable.automf`
The partitioning logic exists but is private in `src/ferrifuzz/fts.py:26`.

- [ ] `var.automf(n, shape="triangle", names=None, overlap=0.5)` — evenly spaced terms
      over `[min_value, max_value]`; default names for n = 3/5/7
      (`poor/average/good`, etc., matching scikit-fuzzy's convention).
- [ ] Support `shape` in `triangle | trapezoid | gaussian | bell`.
- [ ] `automf_it2(..., fou=0.1)` — same grid, footprint scaled by `fou`.
- [ ] Refactor `fts.py:_equal_width_partitions` to call it (single implementation).

### A5. Plot helpers (`ferrifuzz.plot`, optional matplotlib)
- [ ] `plot_variable(var, ax=None)` — all terms; IT2 draws the shaded FOU.
- [ ] `plot_output(system, inputs, out_var)` — aggregated MF + defuzzified marker.
      Consumes A2's `aggregated`.
- [ ] `plot_surface(system, x_var, y_var, out_var, n=50)` — 3D control surface.
- [ ] `plot_rules(system, inputs)` — per-rule activation bars. Consumes A2's `firing`.
- [ ] Import matplotlib lazily inside functions; raise a clear message when missing.

### A6. Configurable output aggregation
`aggregate` hardcodes `agg.max(...)` (`src/system.rs:490`); `or_method` is stored but
never used for aggregation.

- [ ] Add `aggregation=` to `FuzzySystem::new`: `max | sum | probor | bounded`.
      Default `max` (no behaviour change).
- [ ] Route through `crate::operator::t_conorm_fn` instead of a new lookup.
- [ ] Apply the same choice in the IT2 FOU aggregation path.
- [ ] Clamp `sum` to 1.0 and document that `sum` is not idempotent.

### A7. Tests, benches, CI
`grep -c cfg(test) src/` = 0. `tests/` holds three hand-run scripts with no assertions.

- [ ] Rust unit tests per module: `membership.rs` (each MF at boundary/peak/outside),
      `operator.rs` (t-norm axioms: commutativity, associativity, 1-identity, monotonic),
      `rule.rs` (parser round-trips + malformed-input errors), `it2.rs` (KM against a
      published worked example).
- [ ] Rust integration test: the canonical tipper system, asserted against known outputs.
- [ ] Convert `tests/*.py` to pytest with assertions; add `pytest` as a dev dep.
- [ ] Property tests (`proptest`): membership always in `[0,1]`; defuzzified output always
      within `[min_value, max_value]`; IT2 lower ≤ upper for every MF at every x.
- [ ] `criterion` benches: single `compute`, `compute_batch` at 10k rows, IT2 vs type-1.
- [ ] Verify `.github/workflows` actually runs `cargo test`, `cargo clippy -- -D warnings`,
      and pytest on the built wheel — not just a build.

### A8. Make the crate usable from pure Rust — DONE (workspace split)
Core is `crates/ferrifuzz` (no pyo3), bindings are `crates/ferrifuzz-python`.
Remaining: pick a license and publish to crates.io.

<details><summary>original plan</summary>

`Cargo.toml` declares `crate-type = ["cdylib", "rlib"]` but `pyo3/extension-module` is
unconditional, so the rlib does not link standalone and the crate cannot be published.

- [ ] Add a `python` feature; gate `pyo3` and every `#[pyclass]`/`#[pymethods]` behind it.
- [ ] Split core math into plain Rust types; keep PyO3 wrappers in a thin layer.
- [ ] `default = ["python"]` so maturin keeps working with no flags.
- [ ] Verify `cargo build --no-default-features` and `cargo test --no-default-features`.
- [ ] Add crates.io metadata (description, license, repository, keywords, categories).
</details>

### A9. Input handling policy
Missing inputs are silently dropped from the `crisp` map (`src/system.rs:186`); values
outside `[min_value, max_value]` pass through unchecked.

- [ ] `on_missing="error" | "ignore"` (default `error` — silent wrong answers are worse).
- [ ] `on_out_of_range="clamp" | "error" | "allow"` (default `clamp`).
- [ ] Set the policy on `FuzzySystem::new`; apply in both `compute` and `compute_batch`.

---

## Track B — NumPy & PyTorch interop

Design rule: **Rust core = fast inference. Torch layer = training.** Do not try to make
the Rust engine differentiable — mirror it in torch ops instead, and make the two
convertible in both directions (B5). That gets autograd, GPU, and batching for free.

### B1. Zero-copy NumPy
`compute_batch` takes `HashMap<String, Vec<f64>>` (`src/system.rs:203`) — PyO3 extracts
element by element, so a 1M-row array is fully copied into Vecs before any math runs.
Output does the reverse via `np.asarray`.

- [ ] Add the `numpy` crate (rust-numpy, PyO3-compatible version).
- [ ] Accept `PyReadonlyArray1<f64>` per column — borrow the buffer, no copy.
- [ ] Also accept a single 2-D array plus a column-name list: `compute_array(arr, names)`
      → `(out_arr, out_names)`, the layout ML code actually has.
- [ ] Return `PyArray1`/`PyArray2` built directly from the result Vec (`into_pyarray`),
      dropping the `np.asarray` round-trip.
- [ ] Accept `f32` input too (cast on read) — torch defaults to float32.
- [ ] Keep the list-of-floats path working when NumPy is absent.
- [ ] Bench: 1M-row `compute_batch` before/after. Target ≥5x.

### B2. Vectorized membership evaluation
`mf.membership(x)` is scalar-only. Every plotting, partitioning, and preprocessing use
site loops in Python.

- [ ] `mf.membership(x)` accepts a scalar or an array; returns the same shape.
- [ ] `var.fuzzify(x)` accepts an array → `{term: array}`.
- [ ] `IT2MF.membership(arr)` → `(lower_arr, upper_arr)`.
- [ ] Implement over borrowed NumPy slices; no per-element Python round-trip.

### B3. Release the GIL and parallelize batch
Rust is the selling point and the batch path never realizes it.

- [ ] Wrap the `compute_batch` inference loop in `py.allow_threads`.
- [ ] Add `rayon`; parallelize the row loop behind a `parallel` cargo feature.
- [ ] `n_jobs=` parameter (`0`/`None` = serial, `-1` = all cores).
- [ ] Verify no `Py<...>` handles cross the `allow_threads` boundary.

### B4. Buffer/array protocol acceptance
So torch CPU tensors, pandas Series, JAX arrays, and anything else array-like just work.

- [ ] On input, try in order: NumPy array → buffer protocol → `__array__()` →
      `__dlpack__()` → sequence fallback.
- [ ] Detect CUDA/MPS tensors and raise a clear error naming `.cpu()` — never silently
      sync a device tensor.
- [ ] Require contiguous input; call `np.ascontiguousarray` on the Python side when not.
- [ ] Tests with `torch.tensor`, `pd.Series`, and a plain `array.array`.

### B5. `ferrifuzz.torch` — differentiable fuzzy layer
The real reason to want torch: tuning MF parameters from data (ANFIS, neuro-fuzzy),
and using a fuzzy system as one layer inside a larger network.

- [ ] `FuzzyLayer(nn.Module)` — MF parameters (centers, widths, slopes) as
      `nn.Parameter`; forward runs fuzzify → fire → aggregate → defuzzify in torch ops
      only, batched over dim 0, device- and dtype-agnostic.
- [ ] MF shapes in torch: triangular, trapezoidal, gaussian, bell, sigmoid, singleton —
      numerically matching `src/membership.rs` to ~1e-6.
- [ ] Differentiable t-norms: `min` (subgradient, fine), `product`, plus **soft** variants
      (`softmin`/`logsumexp` with a temperature) for smoother gradients. Document that
      `min`/`max` give sparse gradients and soft variants usually train better.
- [ ] Differentiable defuzzification: centroid over the sampled grid is already smooth;
      Sugeno/weighted-average is exact and cheapest — make Sugeno the recommended
      training mode. Warn on `mom`/`som`/`lom` (piecewise-constant, no useful gradient).
- [ ] Parameter constraints: enforce `a ≤ b ≤ c` (triangle), `sigma > 0` via
      reparameterization (softplus / cumulative offsets), not post-hoc clamping.
- [ ] `FuzzyLayer.from_system(sys)` and `layer.to_system()` — train in torch, export to
      the Rust engine for fast inference. This is the payoff; do not ship B5 without it.
- [ ] Fixed rule structure only in v1; learning the rule base is a separate item (Tier 3).
- [ ] Tests: `gradcheck` on each MF; `from_system → forward` matches
      `system.compute_batch` to 1e-6; `to_system → from_system` round-trips.
- [ ] Torch is an optional dep (`[project.optional-dependencies] torch`); importing
      `ferrifuzz` without torch installed must not fail.

### B6. ANFIS on top of B5
- [ ] `ANFIS(nn.Module)` — grid or clustering-initialized Sugeno system, linear
      consequents as a `nn.Linear`, hybrid learning (least squares for consequents +
      gradient for premises), with a pure-gradient fallback.
- [ ] `fit(X, y)` / `predict(X)` convenience wrapper for people who do not want the
      training loop.

### B7. sklearn estimator API
- [ ] `FuzzyRegressor` / `FuzzyClassifier` implementing `fit`/`predict`/`get_params`/
      `set_params` so the library drops into `Pipeline` and `GridSearchCV`.
- [ ] Backed by B6 when torch is present; by a non-gradient tuner when it is not.
- [ ] Run sklearn's `check_estimator`.

### B8. Packaging & typing for the interop story
- [ ] Extras: `numpy`, `torch`, `plot`, `all`. Keep zero hard runtime deps.
- [ ] `requires-python` is `>=3.13` in `pyproject.toml` but the wheel is built `abi3-py39`
      (`Cargo.toml`) — lower it to `>=3.9` or the abi3 build is pointless.
- [ ] Update `src/ferrifuzz/_core.pyi` for every new signature; add array overloads
      (`@overload` scalar vs `npt.NDArray`) so type checkers see the vectorized forms.
- [ ] `pyproject.toml` still says `description = "Add your description here"` — fix.
- [ ] `version` disagrees across `Cargo.toml` (0.1.0), `pyproject.toml` (0.1.0), and
      `__init__.py` (0.2.0). Pick one source of truth.

---

## Suggested order

1. A7 (tests first — everything below changes engine behaviour)
2. A1, A2 (engine semantics; A2 unblocks A5)
3. B1, B2, B3 (NumPy path — self-contained, big measurable win)
4. A3, A4, A5, A6, A9
5. A8 (refactor; easier once tests exist)
6. B4, B5 (torch layer), then B6, B7
