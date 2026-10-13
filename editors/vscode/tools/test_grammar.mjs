// Tokenize tools/grammar_sample.py with VS Code's Python grammar + the ferrifuzz injection
// and print every ferrifuzz-scoped token. Run: npm install && npm run test:grammar
// PYTHON_GRAMMAR overrides the path to MagicPython.tmLanguage.json.
import fs from 'fs';
import path from 'path';
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import vsctm from 'vscode-textmate';
import oniguruma from 'vscode-oniguruma';

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const PY = process.env.PYTHON_GRAMMAR ??
  '/Applications/Visual Studio Code.app/Contents/Resources/app/extensions/python/syntaxes/MagicPython.tmLanguage.json';
const INJ = path.join(here, '..', 'syntaxes', 'ferrifuzz-rule.injection.json');

await oniguruma.loadWASM(fs.readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm')).buffer);
const registry = new vsctm.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (p) => new oniguruma.OnigScanner(p),
    createOnigString: (s) => new oniguruma.OnigString(s),
  }),
  loadGrammar: async (scope) => {
    const file = { 'source.python': PY, 'ferrifuzz.rule.injection': INJ }[scope];
    return file ? vsctm.parseRawGrammar(fs.readFileSync(file, 'utf8'), file) : null;
  },
  getInjections: (scope) => (scope === 'source.python' ? ['ferrifuzz.rule.injection'] : []),
});

const grammar = await registry.loadGrammar('source.python');
let stack = vsctm.INITIAL;
for (const line of fs.readFileSync(path.join(here, 'grammar_sample.py'), 'utf8').split('\n')) {
  const { tokens, ruleStack } = grammar.tokenizeLine(line, stack);
  stack = ruleStack;
  console.log('> ' + line);
  for (const t of tokens) {
    const text = line.slice(t.startIndex, t.endIndex);
    const scopes = t.scopes.filter((s) => s.endsWith('.ferrifuzz') && s !== 'meta.rule.ferrifuzz');
    if (scopes.length && text.trim()) console.log(`    ${JSON.stringify(text).padEnd(16)} ${scopes.join(' ')}`);
  }
}
