// Tests check.js against throwaway venvs with fake ferrifuzz metadata. Run: npm run test:check

const assert = require("assert/strict");
const { execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const check = require("../check");

const PYTHON = process.env.PYTHON || (process.platform === "win32" ? "python" : "python3");

/** venv at `dir` whose site-packages claims ferrifuzz `version` (none when null). */
function makeVenv(dir, version) {
  execFileSync(PYTHON, ["-m", "venv", "--without-pip", dir]);
  const python = check.localVenvPythons(path.dirname(dir)).find((p) => p.startsWith(dir + path.sep));
  if (version) {
    const site = execFileSync(python, ["-I", "-c", "import sysconfig; print(sysconfig.get_paths()['purelib'])"]).toString().trim();
    const info = path.join(site, `ferrifuzz-${version}.dist-info`);
    fs.mkdirSync(info);
    fs.writeFileSync(path.join(info, "METADATA"), `Metadata-Version: 2.1\nName: ferrifuzz\nVersion: ${version}\n`);
  }
  return python;
}

async function main() {
  // importsFerrifuzz
  for (const src of ["import ferrifuzz as ff", "from ferrifuzz import mf", "x = 1\n  from ferrifuzz.fts import FuzzyTimeSeries", "import ferrifuzz"]) {
    assert.ok(check.importsFerrifuzz(src), src);
  }
  for (const src of ["import ferrifuzzy", "# import ferrifuzz", "s = 'import ferrifuzz'", "import numpy"]) {
    assert.ok(!check.importsFerrifuzz(src), src);
  }

  // compareVersions
  assert.ok(check.compareVersions("0.1.0", "0.2.0") < 0);
  assert.ok(check.compareVersions("0.2.0", "0.2.0") === 0);
  assert.ok(check.compareVersions("0.2.0rc1", "0.2.0") === 0);
  assert.ok(check.compareVersions("0.10.0", "0.2.0") > 0);
  assert.ok(check.compareVersions("1", "0.2.0") > 0);

  // probe / findLocalVenv on real interpreters
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "ferrifuzz-check-"));
  try {
    const none = makeVenv(path.join(root, "none", ".venv"), null);
    const old = makeVenv(path.join(root, "old", ".venv"), "0.1.0");
    const good = makeVenv(path.join(root, "good", "venv"), "0.2.0");

    assert.deepEqual(await check.probe(none), { status: "missing" });
    assert.deepEqual(await check.probe(old), { status: "old", version: "0.1.0" });
    assert.deepEqual(await check.probe(good), { status: "ok", version: "0.2.0" });
    assert.equal((await check.probe(path.join(root, "no-such-python"))).status, "error");

    assert.equal(await check.findLocalVenv(path.join(root, "none")), null);
    assert.equal(await check.findLocalVenv(path.join(root, "old")), null);
    assert.deepEqual(await check.findLocalVenv(path.join(root, "good")), { python: good, version: "0.2.0" });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
  console.log("check.js: all tests passed");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
