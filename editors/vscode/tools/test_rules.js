// Tests rules.js (rule-string completion and checks). Run: npm run test:rules

const assert = require("assert/strict");
const fs = require("fs");
const path = require("path");
const r = require("../rules");

const SRC = `
import ferrifuzz as ff

service = ff.Variable("service", 0, 10)
service["poor"] = ff.mf.trapezoid(0, 0, 2, 5)
service["good"] = ff.mf.triangle(2, 5, 8)
service["excellent"] = ff.mf.trapezoid(5, 8, 10, 10)
food = ff.Variable("food", 0, 10).automf(["rancid", "ok", "delicious"])
tip = ff.Variable("tip", 0, 30).automf(3)
err = ff.Variable(name="err", min_value=-1, max_value=1)
err.automf(7, shape="gaussian")
ext = ff.Variable("ext", 0, 1)          # terms set somewhere else

system = ff.FuzzySystem()
system.add_input(service)
system.add_input(food)
system.add_input(ff.Variable("hum", 0, 100).automf(5))
system.add_output(tip)
system += "IF service IS poor OR food IS rancid THEN tip IS low"
system += "IF service IS excelent THEN tip IS hgh"   # two typos
system += "IF tip IS low THEN service IS good"
system += "if hum is very very_high then tip is high with 0.5"
system += f"IF service IS {x} THEN tip IS high"
system += "IF ext IS whatever THEN tip IS high"
system += "IF servise IS good THEN tip IS medium"
RULES = """
IF food IS delicious THEN tip IS high
IF food IS yummy THEN tip IS high
"""
system.add_rules(RULES.strip().splitlines())
# system += "IF food IS nope THEN tip IS high"
`;

function main() {
  // automf names
  assert.deepEqual(r.automfTerms(""), ["low", "medium", "high"]);
  assert.deepEqual(r.automfTerms("5"), ["very_low", "low", "medium", "high", "very_high"]);
  assert.deepEqual(r.automfTerms("4, shape='gaussian'"), ["A1", "A2", "A3", "A4"]);
  assert.deepEqual(r.automfTerms("names=['a', \"b\"]"), ["a", "b"]);
  assert.deepEqual(r.automfTerms("shape='gaussian'"), ["low", "medium", "high"]);
  assert.equal(r.automfTerms("labels"), null);

  // the model
  const m = r.scanModel(SRC);
  assert.deepEqual(m.vars.get("service"), { terms: ["poor", "good", "excellent"], known: true });
  assert.deepEqual(m.vars.get("food").terms, ["rancid", "ok", "delicious"]);
  assert.deepEqual(m.vars.get("tip").terms, ["low", "medium", "high"]);
  assert.deepEqual(m.vars.get("err").terms, ["NB", "NM", "NS", "ZE", "PS", "PM", "PB"]);
  assert.deepEqual(m.vars.get("hum").terms, ["very_low", "low", "medium", "high", "very_high"]);
  assert.equal(m.vars.get("ext").known, false);
  assert.deepEqual([...m.inputs], ["service", "food", "hum"]);
  assert.deepEqual([...m.outputs], ["tip"]);

  // strings
  const strs = r.findStrings(`a = "x\\"y" # "not a string"\nb = r'z' + '''t\nu'''\nc = "open`);
  assert.deepEqual(strs.map((s) => s.content), ['x\\"y', "z", "t\nu", "open"]);
  assert.equal(strs[1].prefix, "r");

  // checks
  const problems = r.checkText(SRC, m).map((p) => [SRC.slice(p.start, p.end), p.message]);
  const words = problems.map((p) => p[0]);
  assert.deepEqual(words, ["excelent", "hgh", "tip", "service", "servise", "yummy"], JSON.stringify(problems, null, 1));
  assert.match(problems[0][1], /Did you mean 'excellent'/);
  assert.match(problems[1][1], /Did you mean 'high'/);
  assert.match(problems[2][1], /'tip' is an output; before THEN use an input/);
  assert.match(problems[3][1], /'service' is an input; after THEN use an output \(tip\)/);
  assert.match(problems[4][1], /Unknown variable 'servise'. Did you mean 'service'\?/);
  assert.match(problems[5][1], /Terms: rancid, ok, delicious/);
  assert.deepEqual(r.checkText(`x = "IF a IS b THEN c IS d"`), [], "no variables in the file: stay quiet");
  const LISTED = `${SRC}\nsystem.add_rules([\n    "IF food IS ok THEN tip IS low",\n    "IF food IS okk THEN tip IS low",\n])\nr = ff.Rule.from_string("IF food IS nah THEN tip IS low")\n`;
  assert.deepEqual(r.checkText(LISTED).map((p) => LISTED.slice(p.start, p.end)).slice(-1), ["okk"], "add_rules list checked, from_string not");
  assert.deepEqual(r.checkText(`v = ff.Variable("a", 0, 1)\nr = ff.Rule.from_string("IF a IS b THEN c IS d")`), [], "no system: stay quiet");
  assert.deepEqual(r.checkText(`from x import a\ns.add_input(a)\ns += "IF b IS c THEN d IS e"`), [], "unresolved input: stay quiet");
  assert.equal(m.closed, true);

  // did-you-mean limits match the Rust core
  assert.equal(r.closest("hott", ["cold", "hot"]), "hot");
  assert.equal(r.closest("gud", ["poor", "good"]), "good");
  assert.equal(r.closest("hgih", ["low", "high"]), "high");
  assert.equal(r.closest("zzzzzz", ["cold", "hot"]), null);

  // completion context
  const ctx = (p) => r.ruleContext(p).kind;
  assert.equal(ctx("IF "), "variable");
  assert.equal(ctx("IF serv"), "variable");
  assert.equal(ctx("IF service "), "is");
  assert.equal(ctx("IF service IS "), "term");
  assert.equal(ctx("IF service IS NOT "), "term");
  assert.equal(ctx("IF service IS very "), "term");
  assert.equal(ctx("IF service IS very go"), "term");
  assert.equal(ctx("IF service IS good "), "after-term");
  assert.equal(ctx("IF (service IS good OR food IS ok) "), "after-term");
  assert.equal(ctx("IF (service IS good OR "), "variable");
  assert.equal(ctx("IF service IS good THEN "), "variable");
  assert.equal(ctx("IF service IS good THEN tip IS high WITH "), "weight");

  const labels = (p) => r.completions(p, m).map((c) => c.label);
  assert.deepEqual(labels("IF "), ["service", "food", "hum"]);
  assert.deepEqual(labels("IF service IS good THEN "), ["tip"]);
  assert.deepEqual(labels("IF service "), ["IS"]);
  assert.deepEqual(labels("if service "), ["is"]);
  assert.deepEqual(labels("IF service IS ").slice(0, 4), ["poor", "good", "excellent", "NOT"]);
  assert.ok(labels("IF service IS ").includes("very"));
  assert.deepEqual(labels("IF service IS very "), ["poor", "good", "excellent"]);
  assert.deepEqual(labels("IF service IS good THEN tip IS "), ["low", "medium", "high"]);
  assert.deepEqual(labels("IF service IS good "), ["AND", "OR", "THEN"]);
  assert.deepEqual(labels("IF service IS good THEN tip IS low "), ["AND", "WITH"]);

  // where the cursor is
  const line = 'system += "IF service IS ';
  const doc = `${line}"\n`;
  assert.equal(r.rulePrefixAt(doc, line.length), "IF service IS ");
  assert.equal(r.rulePrefixAt(doc, 3), null, "outside a string");
  assert.equal(r.rulePrefixAt('x = "hello ', 9), null, "not a rule");
  assert.equal(r.rulePrefixAt('x = f"IF a IS ', 14), null, "f-string");
  const tq = 'RULES = """\nIF food IS good THEN tip IS high\nIF food IS ';
  assert.equal(r.rulePrefixAt(tq, tq.length), "IF food IS ");

  // No false alarms on the repo's own examples and tests (they all run).
  const root = path.join(__dirname, "..", "..", "..");
  const files = ["examples/python", "tests/python"].flatMap((d) =>
    fs.existsSync(path.join(root, d))
      ? fs.readdirSync(path.join(root, d)).filter((f) => f.endsWith(".py")).map((f) => path.join(root, d, f))
      : []
  );
  for (const f of files) {
    const text = fs.readFileSync(f, "utf8");
    const found = r.checkText(text).map((p) => `${path.relative(root, f)}: ${text.slice(p.start, p.end)}: ${p.message}`);
    // Tests that check error messages use bad names on purpose; list them for review.
    if (!f.includes("tests")) assert.deepEqual(found, [], "false positives");
    if (found.length) console.log(found.join("\n"));
  }
  console.log(`rules.js: all tests passed (${files.length} repo files scanned)`);
}

main();
