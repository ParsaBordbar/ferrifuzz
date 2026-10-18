// Drives extension.js against a fake `vscode` module. Run: npm run test:extension

const assert = require("assert/strict");
const Module = require("module");

// ─── fake vscode ─────────────────────────────────────────────────────────────

class Position {
  constructor(line, character) {
    Object.assign(this, { line, character });
  }
}
class Range {
  constructor(start, end) {
    Object.assign(this, { start, end });
  }
}

function makeDoc(text, uri = "file:///t.py") {
  const lines = () => text.split("\n");
  const doc = {
    uri: { toString: () => uri, scheme: "file", fsPath: "/t.py" },
    languageId: "python",
    version: 1,
    getText: (range) => {
      if (!range) return text;
      return text.slice(doc.offsetAt(range.start), doc.offsetAt(range.end));
    },
    offsetAt: (p) => lines().slice(0, p.line).reduce((n, l) => n + l.length + 1, 0) + p.character,
    positionAt: (o) => {
      const before = text.slice(0, o).split("\n");
      return new Position(before.length - 1, before[before.length - 1].length);
    },
    getWordRangeAtPosition: (p, re) => {
      const line = lines()[p.line];
      const g = new RegExp(re.source, "g");
      for (let m; (m = g.exec(line)); ) {
        if (m.index <= p.character && p.character <= m.index + m[0].length)
          return new Range(new Position(p.line, m.index), new Position(p.line, m.index + m[0].length));
      }
      return undefined;
    },
  };
  return doc;
}

const registered = { completion: null, actions: null };
const diagnosticsByUri = new Map();
const vscode = {
  Position,
  Range,
  CompletionItem: class {
    constructor(label, kind) {
      Object.assign(this, { label, kind });
    }
  },
  CompletionItemKind: { Keyword: 14, Variable: 6, Operator: 24, EnumMember: 20 },
  Diagnostic: class {
    constructor(range, message, severity) {
      Object.assign(this, { range, message, severity });
    }
  },
  DiagnosticSeverity: { Warning: 1 },
  CodeAction: class {
    constructor(title, kind) {
      Object.assign(this, { title, kind });
    }
  },
  CodeActionKind: { QuickFix: "quickfix" },
  WorkspaceEdit: class {
    constructor() {
      this.edits = [];
    }
    replace(uri, range, text) {
      this.edits.push({ uri, range, text });
    }
  },
  languages: {
    createDiagnosticCollection: () => ({
      set: (uri, d) => diagnosticsByUri.set(uri.toString(), d),
      delete: (uri) => diagnosticsByUri.delete(uri.toString()),
      dispose() {},
    }),
    registerCompletionItemProvider: (_sel, p, ...triggers) => {
      registered.completion = { provider: p, triggers };
      return { dispose() {} };
    },
    registerCodeActionsProvider: (_sel, p) => {
      registered.actions = p;
      return { dispose() {} };
    },
  },
  workspace: {
    textDocuments: [],
    getConfiguration: () => ({ get: (_k, d) => d }),
    onDidOpenTextDocument: () => ({ dispose() {} }),
    onDidChangeTextDocument: () => ({ dispose() {} }),
    onDidCloseTextDocument: () => ({ dispose() {} }),
    onDidSaveTextDocument: () => ({ dispose() {} }),
    onDidChangeConfiguration: () => ({ dispose() {} }),
    getWorkspaceFolder: () => undefined,
  },
  window: { activeTextEditor: undefined, showWarningMessage() {}, showInformationMessage() {} },
  commands: { registerCommand: () => ({ dispose() {} }), executeCommand() {} },
  extensions: { getExtension: () => undefined },
};

const resolve = Module._resolveFilename;
Module._resolveFilename = function (request, ...rest) {
  return request === "vscode" ? "vscode" : resolve.call(this, request, ...rest);
};
require.cache.vscode = { id: "vscode", filename: "vscode", loaded: true, exports: vscode };

// ─── tests ───────────────────────────────────────────────────────────────────

const SRC = `import ferrifuzz as ff
temp = ff.Variable("temp", 0, 40).automf(["cold", "warm", "hot"])
fan = ff.Variable("fan", 0, 100).automf(3)
system = ff.FuzzySystem()
system.add_input(temp)
system.add_output(fan)
system += "IF temp IS hott THEN fan IS high"
system += "IF temp IS wa"
`;

async function main() {
  const doc = makeDoc(SRC);
  vscode.workspace.textDocuments = [doc];
  const ext = require("../extension");
  ext.activate({ subscriptions: [], globalState: { get: (_k, d) => d, update() {} } });
  assert.deepEqual(registered.completion.triggers, [" ", '"', "'", "("]);

  // completion right after `"IF temp IS wa` (before the auto-closed quote)
  const lineNo = SRC.split("\n").findIndex((l) => l.endsWith('"IF temp IS wa"'));
  const pos = new Position(lineNo, SRC.split("\n")[lineNo].length - 1);
  const items = registered.completion.provider.provideCompletionItems(doc, pos);
  assert.deepEqual(items.slice(0, 3).map((i) => i.label), ["cold", "warm", "hot"]);
  assert.equal(items[0].kind, vscode.CompletionItemKind.EnumMember);
  assert.equal(items[0].detail, "term of temp");
  assert.equal(items[0].range.start.character, pos.character - 2, "replaces the typed 'wa'");

  // outside a rule string: nothing
  assert.equal(registered.completion.provider.provideCompletionItems(doc, new Position(1, 3)), undefined);

  // diagnostics run on activation (after a 0 ms timer)
  await new Promise((r) => setTimeout(r, 10));
  const diags = diagnosticsByUri.get(doc.uri.toString());
  assert.equal(diags.length, 1, JSON.stringify(diags));
  assert.equal(doc.getText(diags[0].range), "hott");
  assert.equal(diags[0].source, "ferrifuzz");

  // quick fix
  const [fix] = registered.actions.provideCodeActions(doc, diags[0].range, { diagnostics: diags });
  assert.equal(fix.title, "Change to 'hot'");
  assert.equal(fix.edit.edits[0].text, "hot");
  assert.equal(doc.getText(fix.edit.edits[0].range), "hott");

  console.log("extension.js: all tests passed");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
