// Interpreter check, free of the `vscode` module so it runs (and is tested) under plain node.

const { execFile } = require("child_process");
const fs = require("fs");
const path = require("path");

/** Oldest ferrifuzz whose wheel ships the type stubs editors read. */
const MIN_VERSION = "0.2.0";

/** Matches `import ferrifuzz`, `import ferrifuzz as ff`, `from ferrifuzz import ...`, `from ferrifuzz.fts import ...`. */
const IMPORT_RE = /^[ \t]*(?:import[ \t]+ferrifuzz\b|from[ \t]+ferrifuzz\b)/m;

// Reads metadata only, never imports the native module; exit 3 = not installed.
const PROBE = [
  "import sys",
  "try:",
  "    from importlib.metadata import version, PackageNotFoundError",
  "except ImportError:",
  "    sys.exit(4)",
  "try:",
  "    print(version('ferrifuzz'))",
  "except PackageNotFoundError:",
  "    sys.exit(3)",
].join("\n");

function importsFerrifuzz(text) {
  return IMPORT_RE.test(text);
}

/** Leading numeric release parts: "0.2.0rc1" → [0, 2, 0]. */
function releaseParts(v) {
  return String(v)
    .split(/[^0-9]+/)
    .filter((s) => s !== "")
    .slice(0, 3)
    .map(Number);
}

/** Negative, zero or positive like a comparator; compares release numbers only. */
function compareVersions(a, b) {
  const x = releaseParts(a);
  const y = releaseParts(b);
  for (let i = 0; i < 3; i++) {
    const d = (x[i] || 0) - (y[i] || 0);
    if (d !== 0) return d;
  }
  return 0;
}

/** Which ferrifuzz `python` has: {status: ok | old | missing | error, version?, detail?}. */
function probe(python, { timeoutMs = 10000, minVersion = MIN_VERSION } = {}) {
  return new Promise((resolve) => {
    execFile(python, ["-I", "-c", PROBE], { timeout: timeoutMs, windowsHide: true }, (err, stdout, stderr) => {
      if (err) {
        if (err.code === 3) return resolve({ status: "missing" });
        if (err.code === 4) return resolve({ status: "error", detail: "Python 3.8 or newer is needed" });
        return resolve({ status: "error", detail: (stderr || err.message).trim() });
      }
      const version = stdout.trim();
      resolve({ status: compareVersions(version, minVersion) < 0 ? "old" : "ok", version });
    });
  });
}

/** Python executables of the conventional project venvs (`.venv`, `venv`) in `folder`. */
function localVenvPythons(folder) {
  const rel = process.platform === "win32" ? ["Scripts", "python.exe"] : ["bin", "python"];
  return [".venv", "venv"]
    .map((name) => path.join(folder, name, ...rel))
    .filter((p) => fs.existsSync(p));
}

/** First project venv in `folder` with a usable ferrifuzz, as {python, version}, or null. */
async function findLocalVenv(folder, opts) {
  for (const python of localVenvPythons(folder)) {
    const r = await probe(python, opts);
    if (r.status === "ok") return { python, version: r.version };
  }
  return null;
}

module.exports = {
  MIN_VERSION,
  importsFerrifuzz,
  compareVersions,
  probe,
  localVenvPythons,
  findLocalVenv,
};
