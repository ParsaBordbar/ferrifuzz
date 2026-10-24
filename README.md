<p align="center">
  <img src="docs/assets/logo.svg" alt="Ferrifuzz logo" width="120">
</p>

<h1 align="center">Ferrifuzz</h1>

<p align="center"><strong>Built for uncertainty. Engineered for speed.</strong></p>

<p align="center">
  <a href="https://pypi.org/project/ferrifuzz/"><img src="https://img.shields.io/pypi/v/ferrifuzz?color=7c83ff" alt="PyPI version"></a>
  <a href="https://pypi.org/project/ferrifuzz/"><img src="https://img.shields.io/pypi/pyversions/ferrifuzz" alt="Python versions"></a>
  <a href="https://github.com/ParsaBordbar/ferrifuzz/actions/workflows/ci.yml"><img src="https://github.com/ParsaBordbar/ferrifuzz/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://parsabordbar.github.io/ferrifuzz_docs/"><img src="https://img.shields.io/badge/docs-GitHub%20Pages-a68cff" alt="Documentation"></a>
  <a href="https://marketplace.visualstudio.com/items?itemName=ParsaBordbar2003.ferrifuzz"><img src="https://img.shields.io/visual-studio-marketplace/v/ParsaBordbar2003.ferrifuzz?label=VS%20Code" alt="VS Code extension"></a>
  <img src="https://img.shields.io/badge/core-Rust-dea584?logo=rust" alt="Rust core">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="License: Apache-2.0"></a>
</p>

<p align="center">
  <a href="https://parsabordbar.github.io/ferrifuzz_docs/"><b>Documentation</b></a> ·
  <a href="https://parsabordbar.github.io/ferrifuzz_docs/docs/getting-started/installation">Installation</a> ·
  <a href="https://parsabordbar.github.io/ferrifuzz_docs/docs/tutorials/tipping">Tutorials</a> ·
  <a href="https://parsabordbar.github.io/ferrifuzz_docs/docs/python-api/overview">Python API</a> ·
  <a href="https://parsabordbar.github.io/ferrifuzz_docs/docs/rust-api/overview">Rust API</a> ·
  <a href="https://pypi.org/project/ferrifuzz/">PyPI</a>
</p>

<p align="center">
  <img src="docs/assets/demo.gif" alt="Ferrifuzz demo" width="720">
</p>

**Ferrifuzz** is a fuzzy logic library with a pure-Rust core and a Python package on top.
Both expose the same engine, so you can prototype in a notebook and ship the same model in
Rust. It was written with fuzzy-systems courses in mind: rules are plain English, results
are reproducible, and mistakes produce error messages that tell you what went wrong.

## What you get

- **Membership functions**: triangle, trapezoid, gaussian, generalized bell, sigmoid,
  singleton, and interval type-2 sets built from any two of them.
- **Linguistic variables** with dict-style terms, hedges (`very`, `extremely`, `somewhat`, ...)
  and `NOT`.
- **Fuzzy operators**: 6 t-norms, 6 t-conorms, 3 complements.
- **Rules as strings**: `IF temp IS NOT very cold OR humidity IS high THEN fan IS fast AND alarm IS on`,
  with parentheses and rule weights.
- **Inference**: Mamdani, Sugeno/TSK (constant or linear consequents) and interval type-2
  (Karnik–Mendel, center-of-sets, height type reduction). Six defuzzification methods.
- **Fuzzy time series** forecasting (Chen 1996, first and high order) with RMSE/MAE/MAPE metrics.
- **Clear errors** with "did you mean" hints instead of silently wrong numbers.
- **Batch inference** over NumPy columns, optional **matplotlib plots**, and a
  [**VS Code extension**](https://marketplace.visualstudio.com/items?itemName=ParsaBordbar2003.ferrifuzz)
  that completes, checks and highlights rule strings.

```
crisp input ──▶ fuzzify ──▶ fire rules ──▶ aggregate ──▶ defuzzify ──▶ crisp output
                (μ per       (AND / OR     (max per       (centroid,
                 term)        t-norms)      output term)   bisector, …)
```

## Python

```sh
pip install ferrifuzz                 # no required dependencies
pip install "ferrifuzz[plot]"         # + numpy and matplotlib for ferrifuzz.plot
```

Prebuilt wheels for Windows, macOS and Linux, Python 3.9+. No Rust needed.

```python
import ferrifuzz as ff

service = ff.Variable("service", 0, 10)
service["poor"] = ff.mf.trapezoid(0, 0, 2, 5)
service["good"] = ff.mf.triangle(2, 5, 8)
service["excellent"] = ff.mf.trapezoid(5, 8, 10, 10)

tip = ff.Variable("tip", 0, 30)
tip["low"] = ff.mf.triangle(0, 5, 10)
tip["medium"] = ff.mf.triangle(10, 15, 20)
tip["high"] = ff.mf.triangle(20, 25, 30)

system = ff.FuzzySystem()          # defuzz_method="centroid", and_method="min", ...
system.add_input(service)
system.add_output(tip)
system += ff.Rule.from_string("IF service IS poor THEN tip IS low")
system += ff.Rule.from_string("IF service IS good THEN tip IS medium")
system += ff.Rule.from_string("IF service IS excellent THEN tip IS high")

print(system({"service": 7.5}))     # {'tip': 22.6...}
print(system.validate())           # [] — or "error: ..." / "warning: ..." lines
```

More in [`examples/python/`](examples/python). The Python package's own README (shown on
PyPI) is [`python/README.md`](python/README.md).

## Rust

```toml
[dependencies]
ferrifuzz = { git = "https://github.com/ParsaBordbar/ferrifuzz" }
```

```rust
use ferrifuzz::prelude::*;

fn main() -> Result<(), ferrifuzz::Error> {
    let service = Variable::new("service", 0.0, 10.0)?
        .with_term("poor", mf::trapezoid(0.0, 0.0, 2.0, 5.0)?)
        .with_term("good", mf::triangle(2.0, 5.0, 8.0)?)
        .with_term("excellent", mf::trapezoid(5.0, 8.0, 10.0, 10.0)?);
    let tip = Variable::new("tip", 0.0, 30.0)?
        .with_term("low", mf::triangle(0.0, 5.0, 10.0)?)
        .with_term("medium", mf::triangle(10.0, 15.0, 20.0)?)
        .with_term("high", mf::triangle(20.0, 25.0, 30.0)?);

    let mut system = FuzzySystem::new().with_defuzz(Defuzz::Centroid);
    system.add_input(service).add_output(tip);
    system.add_rules_str([
        "IF service IS poor THEN tip IS low",
        "IF service IS good THEN tip IS medium",
        "IF service IS excellent THEN tip IS high",
    ])?;

    let out = system.compute([("service", 7.5)])?; // errors name typos, missing inputs, ...
    println!("tip = {:.2}", out["tip"]);
    Ok(())
}
```

`cargo run -p ferrifuzz --example tipper` runs a complete example.

## Documentation

The full documentation lives at **[parsabordbar.github.io/ferrifuzz_docs](https://parsabordbar.github.io/ferrifuzz_docs/)**
(also at [ferrifuzz.bubblelamp.space](https://ferrifuzz.bubblelamp.space)); its source is in
[ParsaBordbar/ferrifuzz_docs](https://github.com/ParsaBordbar/ferrifuzz_docs).

| I want to… | Go to |
|---|---|
| Install the package | [Installation](https://parsabordbar.github.io/ferrifuzz_docs/docs/getting-started/installation) |
| Build and run a system in five minutes | [Your first system](https://parsabordbar.github.io/ferrifuzz_docs/docs/getting-started/first-system) |
| Learn the theory behind the code | [Concepts](https://parsabordbar.github.io/ferrifuzz_docs/docs/concepts/what-is-fuzzy) |
| Look up what rule strings can contain | [Rule syntax](https://parsabordbar.github.io/ferrifuzz_docs/docs/rule-syntax) |
| Work through a full course-style project | [Tutorials](https://parsabordbar.github.io/ferrifuzz_docs/docs/tutorials/tipping) |
| Look up a class or method | [Python API](https://parsabordbar.github.io/ferrifuzz_docs/docs/python-api/overview) · [Rust API](https://parsabordbar.github.io/ferrifuzz_docs/docs/rust-api/overview) |
| Fix an error | [Reading error messages](https://parsabordbar.github.io/ferrifuzz_docs/docs/getting-started/error-messages) · [FAQ](https://parsabordbar.github.io/ferrifuzz_docs/docs/faq) |
| Update code written for 0.1 | [Migrating from 0.1](https://parsabordbar.github.io/ferrifuzz_docs/docs/getting-started/migrating-from-0.1) |
| Set up the editor | [Editor support](https://parsabordbar.github.io/ferrifuzz_docs/docs/editor-support) |

## Development

```sh
cargo test -p ferrifuzz                 # Rust core: unit + doc tests (no Python needed)
uv venv && uv pip install maturin pytest numpy
.venv/bin/maturin develop --release     # build the Python extension into the venv
.venv/bin/pytest                        # Python tests
```

| Path | What |
|---|---|
| `crates/ferrifuzz` | Pure-Rust library (zero dependencies) |
| `crates/ferrifuzz-python` | PyO3 bindings, built by maturin as `ferrifuzz._core` |
| `python/ferrifuzz` | Python package: re-exports, `fts`, `metrics`, type stubs |
| `examples/python`, `tests/python` | Python examples and pytest suite |
| `editors/vscode` | VS Code extension: rule-string completion, checks, highlighting, snippets |

## License & citation

Apache-2.0 — keep the [`NOTICE`](NOTICE) file when redistributing. Please cite ferrifuzz in
academic work ([`CITATION.cff`](CITATION.cff), or see
[Cite & license](https://parsabordbar.github.io/ferrifuzz_docs/docs/cite-license)).
