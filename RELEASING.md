# Releasing ferrifuzz

Your checklist, then step-by-step: TestPyPI → PyPI → getting testers (and your professor) on it.

> **Heads-up:** anything uploaded to TestPyPI or PyPI is publicly downloadable, and a version
> number can never be uploaded twice (not even after deleting it). The wheels contain the
> compiled library and the Python files, not the Rust source. Uploading an sdist would publish
> the source too, so these steps upload **wheels only**.

---

## 1. TODO check

### Before anything goes online
- [ ] Read `CHANGELOG.md` — it lists the breaking changes since 0.1.0.
- [ ] Confirm the license (Apache-2.0 + `NOTICE` + `CITATION.cff`). Changing it later is messy.
- [ ] Commit the work (nothing is committed yet), on a branch in both repos:
  ```sh
  git switch -c release/0.2.0 && git add -A && git commit -m "Release 0.2.0: Rust core, Python bindings, docs"
  cd ../ferrifuzzDocsWebsite && git switch -c docs/0.2.0 && git add -A && git commit -m "Docs for 0.2.0"
  ```
- [ ] Rebuild and re-check the wheels from the committed code:
  ```sh
  scripts/build_wheels.sh                 # → dist/*.whl (macOS universal2, Linux x86_64/arm64, Windows x64)
  uvx twine check --strict dist/*.whl
  ```
- [ ] Optional but recommended: push the branch to your **private** GitHub repo and run
  *Actions → Release wheels → Run workflow* (publish: `none`). It builds and tests the wheels
  on real Linux, macOS and Windows machines. The Linux and Windows wheels have only been built,
  never run, on your Mac.

### TestPyPI (section 2)
- [ ] TestPyPI account with 2FA → API token.
- [ ] Upload wheels, install in a clean venv, run the 5-minute check.
- [ ] Have at least one Windows or Linux user run the same check.

### PyPI (section 3)
- [ ] PyPI account with 2FA → API token (start with an account-wide token; after the first
  upload, replace it with one scoped to the `ferrifuzz` project).
- [ ] Upload, check https://pypi.org/project/ferrifuzz/, `pip install ferrifuzz` in a clean venv.
- [ ] Tag: `git tag v0.2.1 && git push --tags`.

### Testers (section 4)
- [ ] Docs: pick hosting, set `url` in `ferrifuzzDocsWebsite/docusaurus.config.ts`, deploy
  (or send a built copy).
- [ ] Send `TRY_IT.md` and the `.vsix` editor extension to testers / your professor.
- [ ] Collect feedback (form or email), turn it into issues, ship fixes as 0.2.1.

### Optional
- [ ] Rust crate on crates.io: `cargo publish -p ferrifuzz --dry-run`, then without `--dry-run`.
  This publishes the Rust source.
- [ ] VS Code Marketplace / Open VSX for the extension (needs a publisher account). Until then,
  share `editors/vscode/ferrifuzz-vscode-0.2.1.vsix`.

---

## 2. Test on TestPyPI

TestPyPI is a separate sandbox index (https://test.pypi.org) with its own accounts. Nothing there
reaches `pip install ferrifuzz`.

**Account and token**
1. Register at https://test.pypi.org/account/register/ and turn on 2FA.
2. *Account settings → API tokens → Add API token*, scope "Entire account" → copy the
   `pypi-…` token (it is shown once).

**Upload** (from the repo root; `uvx` needs no install):
```sh
uvx twine upload --repository-url https://test.pypi.org/legacy/ \
    -u __token__ -p 'pypi-YOUR-TESTPYPI-TOKEN' dist/*.whl
```
Check https://test.pypi.org/project/ferrifuzz/: version 0.2.1, the README rendered, four wheels.

**Install it like a user**, in a fresh environment. `--extra-index-url` lets pip get numpy and
matplotlib from the real PyPI, because TestPyPI doesn't host them:
```sh
python3 -m venv /tmp/ff-test && source /tmp/ff-test/bin/activate
pip install -i https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ \
    "ferrifuzz[plot]==0.2.1"
```

**5-minute check**
```sh
python - <<'EOF'
import ferrifuzz as ff
print(ff.__version__)                                   # 0.2.1
temp = ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"])
fan = ff.Variable("fan", 0, 100).automf(["slow", "medium", "fast"])
s = ff.FuzzySystem(); s.add_input(temp); s.add_output(fan)
s += "IF temp IS cold THEN fan IS slow"
s += "IF temp IS warm THEN fan IS medium"
s += "IF temp IS hot THEN fan IS fast"
print(s({"temp": 30}))                                  # {'fan': 56.07...}
try:
    s += "IF temp IS hott THEN fan IS fast"; s({"temp": 30})
except ff.ModelError as e:
    print(e)                                            # ... (did you mean 'hot'?) ...
EOF
```
For the full test suite against the installed wheel: copy `tests/python` from the repo and run
`pip install pytest && pytest tests/python`.

**Found a bug?** Fix it, bump the version (`version = "0.2.2"` in `Cargo.toml`), rebuild and
upload again. A version already uploaded can't be replaced. To rehearse without spending real numbers, use
pre-release versions on TestPyPI (`0.2.2-rc.1` in Cargo.toml becomes `0.2.2rc1`; install with
`pip install --pre ...`).

---

## 3. Deploy to PyPI

Same steps against the real index:
1. Register at https://pypi.org/account/register/, turn on 2FA, create an API token.
2. Upload:
   ```sh
   uvx twine upload -u __token__ -p 'pypi-YOUR-PYPI-TOKEN' dist/*.whl
   ```
3. Check https://pypi.org/project/ferrifuzz/, then in a clean venv:
   `pip install "ferrifuzz[plot]"` and run the 5-minute check above.
4. Replace the account-wide token with a project-scoped one. Or skip tokens entirely: set up
   **trusted publishing** (PyPI → project → *Publishing* → add GitHub repo
   `ParsaBordbar/ferrifuzz`, workflow `release.yml`, environment `pypi`). From then on, release
   from GitHub with *Actions → Release wheels → Run workflow → publish: pypi*. The workflow tests
   every wheel before it uploads. Use environment `testpypi` the same way for TestPyPI.
5. `git tag v0.2.1 && git push --tags` and move the CHANGELOG entry from "unreleased" to the date.

**Platforms covered:** Windows x64, macOS (Intel and Apple Silicon), Linux x86_64 and ARM64
(manylinux2014, i.e. glibc ≥ 2.17), CPython 3.9 and newer. Not covered: Windows ARM, Alpine/musl
Linux, PyPy. Those users would need an sdist (Rust toolchain), which these steps don't upload.

---

## 4. Let others test it (and your professor)

What to send:
- **`TRY_IT.md`** — install, a 10-minute tour, what feedback you want.
- **The docs.** Run `npm run build` in `ferrifuzzDocsWebsite`, then either:
  - host it on Netlify / Vercel / GitHub Pages (that makes it public), or
  - zip `build/` and send it; the reader runs `npx serve build`.
- **The VS Code extension** — `editors/vscode/ferrifuzz-vscode-0.2.1.vsix`
  (install with *Extensions → … → Install from VSIX*).
- **Feedback channel** — your repo stays private, so use a Google Form or email. Ask for:
  what they tried, what confused them, any wrong results, and what they would need to use it in
  a course.

**Email template for your professor**

> Subject: ferrifuzz — fuzzy logic library for Python, would value your feedback
>
> Dear Professor ___,
>
> I have been building **ferrifuzz**, an open-source fuzzy logic library for Python with a Rust
> core. It supports Mamdani, Sugeno/TSK and interval type-2 inference, linguistic hedges, six
> t-norms/t-conorms, and fuzzy time series (Chen's first- and high-order models). It was
> designed with students in mind: rules are written as plain text ("IF temp IS hot AND humidity
> IS NOT low THEN fan IS fast"), and mistakes produce explanations ("input 'temp' has no term
> 'hott' — did you mean 'hot'?") rather than silently wrong numbers.
>
> It installs with `pip install ferrifuzz` on Windows, macOS and Linux (Python 3.9+). The
> attached guide has a 10-minute tour and the documentation includes tutorials such as the
> tipping problem, a tank controller and type-1 vs interval type-2 under noise.
>
> I would be grateful for your opinion on two things: whether the methods match the definitions
> you teach (operators, defuzzification, Karnik–Mendel type reduction, FTS), and whether it would
> be useful for course projects. Any comment, however short, would help.
>
> Best regards,
> Parsa Bordbar

**After feedback:** turn each point into a GitHub issue (private repo), fix, release 0.2.1 the
same way, and reply with what changed. People keep testing when they see their feedback used.
