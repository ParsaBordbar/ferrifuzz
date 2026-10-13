# Ferrifuzz

Built for uncertainty. Engineered for speed.

Ferrifuzz is a fuzzy logic library with a pure-Rust core and a Python package on top.
Both expose the same engine:

- Membership functions: triangle, trapezoid, gaussian, bell, sigmoid, singleton, interval type-2
- Linguistic variables, hedges (`very`, `somewhat`, ...) and fuzzy operators (6 t-norms, 6 t-conorms, 3 complements)
- Rules as plain strings: `IF temp IS NOT very cold OR humidity IS high THEN fan IS fast AND alarm IS on`
- Mamdani, Sugeno/TSK (constant or linear consequents) and interval type-2 inference
  (Karnik–Mendel, center-of-sets, height type reduction)
- Fuzzy time series forecasting (Chen 1996, first and high order) and RMSE/MAE/MAPE metrics
- Clear errors with "did you mean" hints instead of silently wrong numbers; plotting helpers

The Python package's own README (shown on PyPI) is [`python/README.md`](python/README.md).

## Python

```sh
pip install ferrifuzz
```

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

More in [`examples/python/`](examples/python). Editor support (rule highlighting + snippets)
lives in [`editors/vscode/`](editors/vscode).

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
| `editors/vscode` | VS Code extension: rule-string highlighting and snippets |

## License & citation

Apache-2.0 — keep the [`NOTICE`](NOTICE) file when redistributing. Please cite ferrifuzz in
academic work ([`CITATION.cff`](CITATION.cff)).
