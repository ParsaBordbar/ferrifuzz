// Interpreter check for ferrifuzz, plus completion, checks and quick fixes inside rule strings.

const vscode = require("vscode");
const path = require("path");
const check = require("./check");
const rules = require("./rules");

const MUTED_KEY = "ferrifuzz.mutedInterpreters";

/** Interpreters already reported (or found fine) this session, so each is probed once. */
const seen = new Set();
let pythonApi;

function enabled(setting = "interpreterCheck.enabled") {
  return vscode.workspace.getConfiguration("ferrifuzz").get(setting, true);
}

async function getPythonApi() {
  if (pythonApi) return pythonApi;
  const ext = vscode.extensions.getExtension("ms-python.python");
  if (!ext) return undefined;
  if (!ext.isActive) await ext.activate();
  pythonApi = ext.exports && ext.exports.environments;
  return pythonApi;
}

/** Executable of the interpreter the Python extension uses for `uri`. */
async function activeInterpreter(api, uri) {
  const envPath = api.getActiveEnvironmentPath(uri);
  if (!envPath) return undefined;
  const env = await api.resolveEnvironment(envPath);
  return (env && env.executable && env.executable.uri && env.executable.uri.fsPath) || envPath.path;
}

function shouldCheck(doc) {
  return doc.languageId === "python" && (doc.uri.scheme === "file" || doc.uri.scheme === "untitled") && check.importsFerrifuzz(doc.getText());
}

/** Check `doc`'s interpreter; `manual` (from the command) always reports, even when fine or muted. */
async function checkDocument(context, doc, manual) {
  if (!manual && (!enabled() || !shouldCheck(doc))) return;
  const api = await getPythonApi();
  if (!api) {
    if (manual) vscode.window.showWarningMessage("Ferrifuzz: the Python extension (ms-python.python) is needed to check the interpreter.");
    return;
  }
  const python = await activeInterpreter(api, doc.uri);
  if (!python) return;
  const muted = context.globalState.get(MUTED_KEY, []);
  if (!manual && (seen.has(python) || muted.includes(python))) return;
  seen.add(python);

  const result = await check.probe(python);
  if (result.status === "ok") {
    if (manual) vscode.window.showInformationMessage(`Ferrifuzz ${result.version} is installed in ${python}.`);
    return;
  }

  let problem;
  if (result.status === "missing") problem = `ferrifuzz is not installed in the selected Python interpreter (${python})`;
  else if (result.status === "old") problem = `the selected Python interpreter (${python}) has ferrifuzz ${result.version}, older than ${check.MIN_VERSION}`;
  else problem = `could not check the selected Python interpreter (${python}): ${result.detail}`;

  const folder = vscode.workspace.getWorkspaceFolder(doc.uri);
  const local = folder && (await check.findLocalVenv(folder.uri.fsPath));
  const useLocal = local && local.python !== python ? `Use ${path.basename(path.dirname(path.dirname(local.python)))}` : undefined;
  const actions = [useLocal, "Select Interpreter", manual ? undefined : "Don't Show Again"].filter(Boolean);
  const hint = useLocal ? ` ${path.relative(folder.uri.fsPath, local.python)} has ferrifuzz ${local.version}.` : "";

  const choice = await vscode.window.showWarningMessage(
    `Ferrifuzz: ${problem}, so completions and hover docs will not work.${hint}`,
    ...actions
  );
  if (choice === useLocal) {
    await api.updateActiveEnvironmentPath(local.python, folder.uri);
  } else if (choice === "Select Interpreter") {
    await vscode.commands.executeCommand("python.setInterpreter");
  } else if (choice === "Don't Show Again") {
    await context.globalState.update(MUTED_KEY, [...muted, python]);
  }
}

// ─── Rule strings ────────────────────────────────────────────────────────────

const COMPLETION_KIND = {
  keyword: vscode.CompletionItemKind.Keyword,
  variable: vscode.CompletionItemKind.Variable,
  hedge: vscode.CompletionItemKind.Operator,
  term: vscode.CompletionItemKind.EnumMember,
};

/** scanModel result per document version, so typing does not rescan on every keystroke. */
const models = new WeakMap();
function modelOf(doc) {
  const hit = models.get(doc);
  if (hit && hit.version === doc.version) return hit.model;
  const model = rules.scanModel(doc.getText());
  models.set(doc, { version: doc.version, model });
  return model;
}

const ruleCompletion = {
  provideCompletionItems(doc, position) {
    if (!enabled("ruleCompletion.enabled")) return undefined;
    const text = doc.getText();
    const prefix = rules.rulePrefixAt(text, doc.offsetAt(position));
    if (prefix === null) return undefined;
    const range = doc.getWordRangeAtPosition(position, /[\w]+/) || new vscode.Range(position, position);
    return rules.completions(prefix, modelOf(doc)).map((c, i) => {
      const item = new vscode.CompletionItem(c.label, COMPLETION_KIND[c.kind]);
      item.detail = c.detail;
      item.range = new vscode.Range(range.start, position);
      item.sortText = String(i).padStart(3, "0");
      return item;
    });
  },
};

const DID_YOU_MEAN = /Did you mean '([^']+)'\?/;

const ruleFixes = {
  provideCodeActions(doc, _range, ctx) {
    return ctx.diagnostics
      .filter((d) => d.source === "ferrifuzz" && DID_YOU_MEAN.test(d.message))
      .map((d) => {
        const fix = DID_YOU_MEAN.exec(d.message)[1];
        const action = new vscode.CodeAction(`Change to '${fix}'`, vscode.CodeActionKind.QuickFix);
        action.edit = new vscode.WorkspaceEdit();
        action.edit.replace(doc.uri, d.range, fix);
        action.diagnostics = [d];
        action.isPreferred = true;
        return action;
      });
  },
};

function updateDiagnostics(collection, doc) {
  if (doc.languageId !== "python") return;
  if (!enabled("ruleDiagnostics.enabled") || !check.importsFerrifuzz(doc.getText())) {
    collection.delete(doc.uri);
    return;
  }
  const found = rules.checkText(doc.getText(), modelOf(doc)).map((p) => {
    const range = new vscode.Range(doc.positionAt(p.start), doc.positionAt(p.end));
    const d = new vscode.Diagnostic(range, p.message, vscode.DiagnosticSeverity.Warning);
    d.source = "ferrifuzz";
    return d;
  });
  collection.set(doc.uri, found);
}

/** @param {vscode.ExtensionContext} context */
function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection("ferrifuzz");
  const pending = new Map();
  const refresh = (doc, delay = 0) => {
    clearTimeout(pending.get(doc.uri.toString()));
    pending.set(doc.uri.toString(), setTimeout(() => updateDiagnostics(diagnostics, doc), delay));
  };
  const python = { language: "python" };
  context.subscriptions.push(
    diagnostics,
    vscode.languages.registerCompletionItemProvider(python, ruleCompletion, " ", '"', "'", "("),
    vscode.languages.registerCodeActionsProvider(python, ruleFixes, { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] }),
    vscode.workspace.onDidOpenTextDocument((doc) => refresh(doc)),
    vscode.workspace.onDidChangeTextDocument((e) => refresh(e.document, 300)),
    vscode.workspace.onDidCloseTextDocument((doc) => diagnostics.delete(doc.uri)),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("ferrifuzz")) vscode.workspace.textDocuments.forEach((doc) => refresh(doc));
    })
  );
  vscode.workspace.textDocuments.forEach((doc) => refresh(doc));

  const run = (doc) => checkDocument(context, doc, false).catch((e) => console.error("ferrifuzz interpreter check:", e));

  context.subscriptions.push(
    vscode.commands.registerCommand("ferrifuzz.checkInterpreter", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return vscode.window.showInformationMessage("Ferrifuzz: open a Python file first.");
      return checkDocument(context, editor.document, true);
    }),
    vscode.commands.registerCommand("ferrifuzz.resetInterpreterWarnings", () => {
      seen.clear();
      return context.globalState.update(MUTED_KEY, []);
    }),
    vscode.workspace.onDidOpenTextDocument(run),
    vscode.workspace.onDidSaveTextDocument(run)
  );

  // A newly selected interpreter (e.g. after "Select Interpreter") gets a fresh check.
  getPythonApi().then((api) => {
    if (!api) return;
    context.subscriptions.push(
      api.onDidChangeActiveEnvironmentPath(() => {
        seen.clear();
        vscode.workspace.textDocuments.forEach(run);
      })
    );
  });

  vscode.workspace.textDocuments.forEach(run);
}

function deactivate() {}

module.exports = { activate, deactivate };
