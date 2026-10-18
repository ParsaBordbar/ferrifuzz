// Rule-string intelligence, free of the `vscode` module so it runs (and is tested) under plain node.
//
// A Python file is read with regexes, not executed: variables come from `X = ff.Variable("name", ...)`,
// terms from `X["term"] = ...` and `.automf(...)`, inputs / outputs from `add_input` / `add_output`.
// Anything the scan cannot follow (a term set from a variable, a Variable from another module)
// makes it stay quiet rather than guess.

const KEYWORDS = ["IF", "THEN", "AND", "OR", "IS", "NOT", "WITH"];
const HEDGES = ["very", "extremely", "somewhat", "more_or_less", "slightly", "plus", "minus"];
const HEDGE_WORDS = new Set([...HEDGES, "moreorless"]);

const isKeyword = (w, kw) => w.toUpperCase() === kw;
const isAnyKeyword = (w) => KEYWORDS.includes(w.toUpperCase());
const isHedge = (w) => HEDGE_WORDS.has(w.toLowerCase());

/** Same names as `default_term_names` in crates/ferrifuzz/src/variable.rs. */
function defaultTermNames(n) {
  const named = {
    2: ["low", "high"],
    3: ["low", "medium", "high"],
    5: ["very_low", "low", "medium", "high", "very_high"],
    7: ["NB", "NM", "NS", "ZE", "PS", "PM", "PB"],
  }[n];
  return named || Array.from({ length: n }, (_, i) => `A${i + 1}`);
}

// ─── Python strings ──────────────────────────────────────────────────────────

/**
 * Every string literal in `text` as {open, start, end, close, content, prefix, triple}:
 * `start`/`end` delimit the content, `open`/`close` the whole literal with prefix and quotes. An unterminated one-line string runs to the end
 * of its line, so the string being typed is found too. Comments are skipped.
 */
function findStrings(text) {
  const out = [];
  let i = 0;
  const n = text.length;
  while (i < n) {
    const c = text[i];
    if (c === "#") {
      while (i < n && text[i] !== "\n") i++;
      continue;
    }
    if (c !== '"' && c !== "'") {
      i++;
      continue;
    }
    let p = i;
    while (p > 0 && /[rRbBuUfF]/.test(text[p - 1])) p--;
    const prefix = p > 0 && /\w/.test(text[p - 1]) ? "" : text.slice(p, i).toLowerCase();
    const triple = text.startsWith(c.repeat(3), i);
    const quote = triple ? c.repeat(3) : c;
    const start = i + quote.length;
    let j = start;
    let end = -1;
    while (j < n) {
      // A backslash escapes the next character; in a raw string it still keeps a quote in.
      if (text[j] === "\\") {
        j += 2;
        continue;
      }
      if (!triple && text[j] === "\n") break;
      if (text.startsWith(quote, j)) {
        end = j;
        break;
      }
      j++;
    }
    if (end === -1) end = Math.min(j, n);
    const closed = end === j && text.startsWith(quote, j);
    out.push({ open: i - prefix.length, start, end, close: closed ? end + quote.length : end, content: text.slice(start, end), prefix, triple });
    i = closed ? end + quote.length : end + 1;
  }
  return out;
}

/**
 * Rule candidates: one-line strings, and each line of a triple-quoted string, as
 * {offset, text, toSystem}. `toSystem` is true when the string visibly goes into a system:
 * `system += "..."`, `add_rule("...")`, `add_rules([...])`, or a constant later passed to those.
 * Strings elsewhere (e.g. `Rule.from_string` in a parser test) may name variables on purpose.
 */
function ruleTexts(text) {
  const out = [];
  const strings = findStrings(text);
  let prevClose = 0;
  let prevToSystem = false;
  for (const s of strings) {
    const lead = text.slice(prevClose, s.open);
    prevClose = s.close;
    let toSystem;
    if (/^[\s,(\[]*$/.test(lead) && prevToSystem) toSystem = true; // next item of the same list
    else if (/(?:\+=|\badd_rules?\(\s*\[?)\s*$/.test(lead)) toSystem = true;
    else {
      const name = /\b([A-Za-z_]\w*)\s*=\s*[(\[]?\s*$/.exec(lead);
      toSystem = !!name && new RegExp(`(?:\\+=|\\badd_rules?\\()\\s*${name[1]}\\b`).test(text);
    }
    prevToSystem = toSystem;
    if (s.prefix.includes("f") || s.prefix.includes("b")) continue;
    if (!s.triple) {
      out.push({ offset: s.start, text: s.content, toSystem });
      continue;
    }
    let off = s.start;
    for (const line of s.content.split("\n")) {
      out.push({ offset: off, text: line, toSystem });
      off += line.length + 1;
    }
  }
  return out;
}

/** Same test as the highlighting grammar: starts with `IF <var> IS` (maybe after `(`), has THEN. */
function looksLikeRule(s) {
  return /^\s*\(?\s*IF\s+[^\s()]+\s+IS\b/i.test(s) && /\bTHEN\b/i.test(s);
}

// ─── The model a file builds ─────────────────────────────────────────────────

/** Text inside the parentheses that open at `open` (index of "("), or null if unbalanced. */
function callArgs(text, open) {
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    const c = text[i];
    if (c === '"' || c === "'") {
      const q = c;
      for (i++; i < text.length && text[i] !== q && text[i] !== "\n"; i++) if (text[i] === "\\") i++;
    } else if (c === "(" || c === "[" || c === "{") depth++;
    else if (c === ")" || c === "]" || c === "}") {
      depth--;
      if (depth === 0) return { args: text.slice(open + 1, i), close: i };
    }
  }
  return null;
}

/** Term names from `automf(...)` arguments, or null when they cannot be read statically. */
function automfTerms(args) {
  const first = args.trim();
  if (first === "" || /^shape\s*=/.test(first)) return defaultTermNames(3);
  const m = /^(?:names\s*=\s*)?(.*)$/s.exec(first)[1];
  const count = /^(\d+)\s*(?:,|$)/.exec(m);
  if (count) return defaultTermNames(Number(count[1]));
  const list = /^[[(]([^\])]*)[\])]/.exec(m);
  if (list) {
    const items = list[1].split(",").map((s) => s.trim()).filter(Boolean);
    const names = items.map((s) => /^(["'])([^"']*)\1$/.exec(s));
    if (names.every(Boolean)) return names.map((x) => x[2]);
  }
  return null;
}

/**
 * Variables, terms, inputs and outputs a Python file defines:
 * {vars: Map(name → {terms: string[], known: boolean}), inputs: Set, outputs: Set, closed: boolean}.
 * `known` is false when some terms come from code the scan cannot read; `closed` is true when the
 * file registers inputs / outputs and the scan resolved every one of them.
 */
function scanModel(text) {
  const vars = new Map();
  const identToName = new Map();
  const ensure = (name) => {
    if (!vars.has(name)) vars.set(name, { terms: [], known: true, touched: false });
    return vars.get(name);
  };
  const setTerms = (v, terms) => {
    v.touched = true;
    if (terms) v.terms = [...terms];
    else v.known = false;
  };
  const addTerm = (v, term) => {
    v.touched = true;
    if (term === null) v.known = false;
    else if (!v.terms.includes(term)) v.terms.push(term);
  };

  // Every Variable(...) call, with an optional `ident =` before it and `.automf(...)` after it.
  const callRe = /(?:([A-Za-z_][\w.]*)\s*=\s*)?(?:\b[A-Za-z_]\w*\.)*\bVariable\(/g;
  for (let m; (m = callRe.exec(text)); ) {
    const open = m.index + m[0].length - 1;
    const call = callArgs(text, open);
    if (!call) continue;
    const name = /^\s*(?:name\s*=\s*)?(["'])([^"'\s]+)\1/.exec(call.args);
    if (!name) continue;
    const v = ensure(name[2]);
    if (m[1]) identToName.set(m[1], name[2]);
    const chained = /^\s*\.automf\(/.exec(text.slice(call.close + 1));
    if (chained) {
      const a = callArgs(text, call.close + 1 + chained[0].length - 1);
      if (a) setTerms(v, automfTerms(a.args));
    }
  }

  // ident.automf(...) and ident["term"] = ... on a known variable.
  const useRe = /\b([A-Za-z_][\w.]*?)(?:\.automf\(|\[\s*(?:(["'])([^"']*)\2|[^\]]*)\s*\]\s*=(?!=))/g;
  for (let m; (m = useRe.exec(text)); ) {
    const name = identToName.get(m[1]);
    if (!name) continue;
    const v = vars.get(name);
    if (m[0].endsWith("(")) {
      const a = callArgs(text, m.index + m[0].length - 1);
      if (a) setTerms(v, automfTerms(a.args));
    } else {
      addTerm(v, m[2] ? m[3] : null);
    }
  }

  // A Variable whose terms never appear here gets them elsewhere: do not judge its terms.
  for (const v of vars.values()) {
    if (!v.touched) v.known = false;
    delete v.touched;
  }

  const inputs = new Set();
  const outputs = new Set();
  let closed = true;
  const ioRe = /\.add_(input|output)\(\s*(?:([A-Za-z_][\w.]*)\s*\)|(?:\b[A-Za-z_]\w*\.)*Variable\(\s*(?:name\s*=\s*)?(["'])([^"'\s]+)\3)?/g;
  for (let m; (m = ioRe.exec(text)); ) {
    const name = m[2] ? identToName.get(m[2]) : m[4];
    if (name) (m[1] === "input" ? inputs : outputs).add(name);
    else closed = false;
  }
  closed = closed && inputs.size + outputs.size > 0;
  return { vars, inputs, outputs, closed };
}

// ─── Rule tokens ─────────────────────────────────────────────────────────────

/** Words and parentheses of a rule with their offsets, split like the Rust lexer. */
function tokenize(s) {
  const toks = [];
  const re = /[()]|[^\s()]+/g;
  for (let m; (m = re.exec(s)); ) toks.push({ text: m[0], start: m.index, end: m.index + m[0].length });
  return toks;
}

/** Closest candidate within a small edit distance, for "did you mean" — same limits as
 * `did_you_mean` in crates/ferrifuzz/src/error.rs. */
function closest(word, candidates) {
  const dist = (a, b) => {
    const d = Array.from({ length: a.length + 1 }, (_, i) => [i, ...Array(b.length).fill(0)]);
    for (let j = 1; j <= b.length; j++) d[0][j] = j;
    for (let i = 1; i <= a.length; i++)
      for (let j = 1; j <= b.length; j++)
        d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
    return d[a.length][b.length];
  };
  const w = word.toLowerCase();
  const maxD = w.length <= 2 ? 1 : w.length <= 5 ? 2 : Math.floor(w.length / 3);
  let best = null;
  let bestD = Infinity;
  for (const c of candidates) {
    const dd = dist(w, c.toLowerCase());
    if (dd < bestD && (dd === 0 || (dd <= maxD && dd < Math.max(w.length, c.length)))) [best, bestD] = [c, dd];
  }
  return best;
}

/** Clauses `var IS [NOT] [hedge] term` of a rule, with `consequent` set after THEN. */
function clauses(toks) {
  const out = [];
  let consequent = false;
  for (let i = 0; i < toks.length; i++) {
    const w = toks[i].text;
    if (isKeyword(w, "THEN")) consequent = true;
    if (!isKeyword(w, "IS") || i === 0) continue;
    const varTok = toks[i - 1];
    if (isAnyKeyword(varTok.text) || varTok.text === "(") continue;
    let k = i + 1;
    if (toks[k] && isKeyword(toks[k].text, "NOT")) k++;
    let hedge = null;
    const named = (t) => t && t.text !== "(" && t.text !== ")" && !isAnyKeyword(t.text);
    if (named(toks[k]) && isHedge(toks[k].text) && named(toks[k + 1])) hedge = toks[k++];
    const termTok = named(toks[k]) ? toks[k] : null;
    out.push({ varTok, hedge, termTok, consequent });
  }
  return out;
}

/**
 * Problems in one rule string as {start, end, message} (offsets into `rule`). Only what the
 * file pins down is judged: an unknown variable only when it looks like a typo of a defined one
 * and the scan followed every input and output (`closed`) — a rule may be parsed on its own
 * with names defined nowhere — and an unknown term only when every term of its variable is known.
 */
function checkRule(rule, model) {
  const problems = [];
  const names = [...model.vars.keys()];
  for (const c of clauses(tokenize(rule))) {
    const name = c.varTok.text;
    const v = model.vars.get(name);
    if (!v) {
      const guess = model.closed ? closest(name, names) : null;
      if (!guess) continue;
      problems.push({
        start: c.varTok.start,
        end: c.varTok.end,
        message: `Unknown variable '${name}'. Did you mean '${guess}'? Variables in this file: ${names.join(", ")}.`,
      });
      continue;
    }
    const side = c.consequent ? model.outputs : model.inputs;
    const other = c.consequent ? model.inputs : model.outputs;
    if (side.size && !side.has(name) && other.has(name)) {
      problems.push({
        start: c.varTok.start,
        end: c.varTok.end,
        message: c.consequent
          ? `'${name}' is an input; after THEN use an output (${[...model.outputs].join(", ")}).`
          : `'${name}' is an output; before THEN use an input (${[...model.inputs].join(", ")}).`,
      });
    }
    if (c.termTok && v.known && !v.terms.includes(c.termTok.text)) {
      const term = c.termTok.text;
      const guess = closest(term, v.terms);
      problems.push({
        start: c.termTok.start,
        end: c.termTok.end,
        message: `'${name}' has no term '${term}'.${guess ? ` Did you mean '${guess}'?` : ""} Terms: ${v.terms.join(", ") || "(none)"}.`,
      });
    }
  }
  return problems;
}

/** Problems in every rule string of a file as {start, end, message} (offsets into `text`). */
function checkText(text, model = scanModel(text)) {
  const out = [];
  for (const r of ruleTexts(text)) {
    if (!r.toSystem || !looksLikeRule(r.text)) continue;
    for (const p of checkRule(r.text, model)) out.push({ ...p, start: r.offset + p.start, end: r.offset + p.end });
  }
  return out;
}

// ─── Completion ──────────────────────────────────────────────────────────────

/** Rule text from its start up to `offset`, or null when `offset` is not inside a rule string. */
function rulePrefixAt(text, offset) {
  for (const s of findStrings(text)) {
    if (offset < s.start || offset > s.end) continue;
    if (s.prefix.includes("f") || s.prefix.includes("b")) return null;
    let from = s.start;
    if (s.triple) from = Math.max(s.start, text.lastIndexOf("\n", offset - 1) + 1);
    const prefix = text.slice(from, offset);
    return /^\s*\(?\s*IF\b/i.test(prefix) ? prefix : null;
  }
  return null;
}

/**
 * What can come next in a rule whose text so far is `prefix`:
 * {kind, partial, variable?, consequent, lower} where kind is
 * "keyword-if" | "variable" | "is" | "term" | "after-term" | "weight".
 * `partial` is the word being typed; `lower` is true when the rule writes keywords in lower case.
 */
function ruleContext(prefix) {
  const toks = tokenize(prefix);
  let partial = "";
  if (toks.length && toks[toks.length - 1].end === prefix.length && toks[toks.length - 1].text !== "(" && toks[toks.length - 1].text !== ")") {
    partial = toks.pop().text;
  }
  const words = toks.map((t) => t.text);
  const consequent = words.some((w) => isKeyword(w, "THEN"));
  const ifWord = words.find((w) => isKeyword(w, "IF")) || partial;
  const lower = ifWord !== "" && ifWord === ifWord.toLowerCase();
  const base = { partial, consequent, lower };
  const last = words[words.length - 1];
  const prev = words[words.length - 2];
  if (last === undefined || (last === "(" && !words.some((w) => isKeyword(w, "IF")))) return { ...base, kind: "keyword-if" };
  if (["IF", "AND", "OR", "THEN"].some((k) => isKeyword(last, k)) || last === "(") return { ...base, kind: "variable" };
  if (isKeyword(last, "WITH")) return { ...base, kind: "weight" };
  // Find the clause the cursor is in: back up to its IS.
  const isAt = (() => {
    for (let i = words.length - 1; i >= 0; i--) {
      const w = words[i];
      if (isKeyword(w, "IS")) return i;
      if (isAnyKeyword(w) && !isKeyword(w, "NOT")) return -1;
      if (w === "(" || w === ")") return -1;
    }
    return -1;
  })();
  if (isAt >= 1) {
    const variable = words[isAt - 1];
    const after = words.slice(isAt + 1).filter((w) => !isKeyword(w, "NOT"));
    const hedged = after.length === 1 && isHedge(after[0]);
    if (after.length === 0 || hedged) return { ...base, kind: "term", variable, hedged, negated: words.slice(isAt + 1).some((w) => isKeyword(w, "NOT")) };
    return { ...base, kind: "after-term" };
  }
  if (last === ")") return { ...base, kind: "after-term" };
  if (prev !== undefined && (["IF", "AND", "OR", "THEN"].some((k) => isKeyword(prev, k)) || prev === "(")) return { ...base, kind: "is" };
  return { ...base, kind: "after-term" };
}

/**
 * Completion items for the rule text `prefix` as {label, kind, detail}; kind is
 * "keyword" | "variable" | "hedge" | "term". Empty when nothing sensible follows.
 */
function completions(prefix, model) {
  const ctx = ruleContext(prefix);
  const kw = (w) => ({ label: ctx.lower ? w.toLowerCase() : w, kind: "keyword", detail: "rule keyword" });
  const vars = (side) => {
    const names = side.size ? [...side] : [...model.vars.keys()];
    return names.map((n) => {
      const v = model.vars.get(n);
      const terms = v && v.terms.length ? `: ${v.terms.join(", ")}` : "";
      const role = model.inputs.has(n) ? "input" : model.outputs.has(n) ? "output" : "variable";
      return { label: n, kind: "variable", detail: `${role}${terms}` };
    });
  };
  switch (ctx.kind) {
    case "keyword-if":
      return [kw("IF")];
    case "variable":
      return vars(ctx.consequent ? model.outputs : model.inputs);
    case "is":
      return [kw("IS")];
    case "term": {
      const v = model.vars.get(ctx.variable);
      const terms = (v ? v.terms : []).map((t) => ({ label: t, kind: "term", detail: `term of ${ctx.variable}` }));
      if (ctx.hedged || ctx.consequent) return terms;
      const extra = ctx.negated ? [] : [kw("NOT")];
      const hedges = HEDGES.map((h) => ({ label: h, kind: "hedge", detail: "hedge" }));
      return [...terms, ...extra, ...hedges];
    }
    case "after-term":
      return (ctx.consequent ? ["AND", "WITH"] : ["AND", "OR", "THEN"]).map(kw);
    default:
      return [];
  }
}

module.exports = {
  KEYWORDS,
  HEDGES,
  defaultTermNames,
  findStrings,
  ruleTexts,
  looksLikeRule,
  automfTerms,
  scanModel,
  tokenize,
  closest,
  checkRule,
  checkText,
  rulePrefixAt,
  ruleContext,
  completions,
};
