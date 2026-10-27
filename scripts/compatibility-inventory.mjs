// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Inventory evidence is a coverage index, not an equivalence verdict.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { readdir, readFile, writeFile, mkdir, access } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const require = createRequire(new URL('../tools/compatibility/package.json', import.meta.url));
const ts = require('typescript');
const args = process.argv.slice(2);
assert(args.every(a => a === '--check' || a.startsWith('--out=') || a.startsWith('--mappings=')), 'Usage: compatibility-inventory.mjs [--check] [--out=tmp/name] [--mappings=tmp/name.json]');
const checking = args.includes('--check');
const outputDir = args.find(a => a.startsWith('--out='))?.slice(6) ?? 'results/compatibility';
const mappingsPath = args.find(a => a.startsWith('--mappings='))?.slice(11) ?? 'tools/compatibility/mappings.json';
assert(outputDir === 'results/compatibility' || /^tmp\/[\w/-]+$/.test(outputDir), 'Alternative output must be under tmp/');
assert(mappingsPath === 'tools/compatibility/mappings.json' || /^tmp\/[\w/-]+\.json$/.test(mappingsPath), 'Alternative mappings must be under tmp/');
const read = p => readFile(path.join(root, p), 'utf8');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const excluded = new Set(['.git', 'node_modules', 'out', 'out-test', 'out-esbuild', 'out-esbuild-test', 'lib', 'coverage']);
async function walk(dir) {
  const files = [];
  for (const entry of (await readdir(path.join(root, dir), { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name, 'en'))) {
    const name = `${dir}/${entry.name}`;
    if (entry.isSymbolicLink()) throw new Error(`Reference symlink requires explicit policy: ${name}`);
    if (entry.isDirectory()) { if (!excluded.has(entry.name)) files.push(...await walk(name)); }
    else if (entry.isFile()) files.push(name);
  }
  return files.sort();
}
const family = p => p.startsWith('source_repo/addons/') ? p.split('/').slice(1, 3).join('/')
  : p.startsWith('source_repo/src/') ? p.split('/').slice(2, 3).join('/')
    : p.split('/')[1];
const parse = (p, text) => {
  const sf = ts.createSourceFile(p, text, ts.ScriptTarget.Latest, true);
  assert.equal(sf.parseDiagnostics.length, 0, `TypeScript parse diagnostics in ${p}`);
  return sf;
};
const lineAt = (sf, node) => sf.getLineAndCharacterOfPosition(node.getStart(sf)).line + 1;
const docOf = node => (node.jsDoc ?? []).map(d => d.getText()).join('\n');
const modifiers = node => (node.modifiers ?? []).map(m => m.getText());
const nameOf = (sf, node) => node.name?.getText(sf) ?? (ts.isConstructorDeclaration(node) ? 'constructor' : ts.isCallSignatureDeclaration(node) ? '<call>' : ts.isConstructSignatureDeclaration(node) ? '<new>' : '<index>');
const referenceFiles = await walk('source_repo');
const entries = JSON.parse(await read(mappingsPath));
const behaviorRules = JSON.parse(await read('tools/compatibility/behavior-criteria.json'));
for (const rule of behaviorRules) {
  assert(referenceFiles.includes(rule.source), `Unknown behavior rule source: ${rule.source}`);
  assert(rule.criterionIds.length && rule.criterionIds.every(id => Number.isInteger(id) && id > 0));
  await access(path.join(root, rule.document));
}
const acceptanceFor = (source, title = '') => {
  const rules = behaviorRules.filter(r => r.source === source && (!r.pattern || new RegExp(r.pattern).test(title)));
  return { behaviorCriteria: [...new Set(rules.flatMap(r => r.criterionIds))].sort((a, b) => a - b),
    acceptanceDocuments: [...new Set(rules.map(r => r.document))],
    acceptanceGap: rules.length ? null : 'No focused behavior acceptance criterion mapped. Inventory criteria do not prove behavior coverage.' };
};
assert.equal(new Set(entries.modules.map(m => m.source)).size, entries.modules.length, 'Duplicate module mapping');
const moduleMappings = new Map(entries.modules.map(m => [m.source, m]));
const evidencePaths = new Set();
for (const entry of [...entries.modules, ...entries.apiOwners]) {
  for (const p of [...entry.implementation, ...entry.tests, ...entry.evidence]) { await access(path.join(root, p)); evidencePaths.add(p); }
  assert(entry.binding && entry.gap && entry.command, 'Mappings need binding, gap, and executable command');
}
const originalBaselinePath = 'results/upstream-baseline/summary.json';
const originalBaseline = JSON.parse(await read(originalBaselinePath));
evidencePaths.add(originalBaselinePath);
const originalBySource = new Map(originalBaseline.suites.map(s => [`source_repo/${s.source}`, s]));
const evidenceManifest = [];
for (const p of [...evidencePaths].sort()) {
  const bytes = await readFile(path.join(root, p));
  evidenceManifest.push({ path: p, bytes: bytes.length, sha256: digest(bytes) });
}
const manifest = [];
for (const p of referenceFiles) {
  const bytes = await readFile(path.join(root, p));
  manifest.push({ path: p, bytes: bytes.length, sha256: digest(bytes) });
}
const production = referenceFiles.filter(p => /\.(ts|tsx|css)$/.test(p) && !/\.(test|benchmark)\.ts$/.test(p)
  && (/^source_repo\/src\//.test(p) || /^source_repo\/addons\/[^/]+\/(src|typings)\//.test(p) || /^source_repo\/(typings|css)\//.test(p)));
for (const p of moduleMappings.keys()) assert(production.includes(p), `Unknown module mapping: ${p}`);
const mapped = entry => entry ? { status: 'partial', implementation: entry.implementation, tests: entry.tests,
  evidence: entry.evidence, command: entry.command, binding: entry.binding, gap: entry.gap }
  : { status: 'missing', implementation: [], tests: [], evidence: [], command: null,
    binding: null, gap: 'No Rust implementation or executable equivalence evidence mapped.' };
const modules = production.map(p => ({ source: p, family: family(p), ...acceptanceFor(p), ...mapped(moduleMappings.get(p)) }));
const declarations = production.filter(p => p.endsWith('.d.ts'));
const runtimeContracts = [];
for (const kind of ['browser', 'headless']) {
  const source = `source_repo/src/${kind}/public/Terminal.ts`;
  const sf = parse(source, await read(source));
  const proposedMembers = [];
  let constructorOnlyOptions = [];
  function visit(node) {
    if (node.name?.getText(sf) === 'CONSTRUCTOR_ONLY_OPTIONS' && node.initializer && ts.isArrayLiteralExpression(node.initializer)) constructorOnlyOptions = node.initializer.elements.map(n => n.text);
    if ((ts.isMethodDeclaration(node) || ts.isGetAccessorDeclaration(node) || ts.isSetAccessorDeclaration(node)) && node.body?.getText(sf).includes('this._checkProposedApi()')) proposedMembers.push(nameOf(sf, node));
    ts.forEachChild(node, visit);
  }
  visit(sf);
  runtimeContracts.push({ source, module: kind === 'browser' ? '@xterm/xterm' : '@xterm/headless', proposedMembers, constructorOnlyOptions, status: 'missing', gap: 'No Rust public Terminal option mutation or proposed-API gating implementation mapped.' });
}
const apis = [];
for (const p of declarations) {
  const sf = parse(p, await read(p));
  function visit(node, scope = [], inheritedDocs = '') {
    if (ts.isModuleDeclaration(node)) { if (node.body) visit(node.body, [...scope, node.name.text]); return; }
    if (ts.isSourceFile(node) || ts.isModuleBlock(node)) { node.statements.forEach(n => visit(n, scope)); return; }
    if (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) return;
    if (ts.isVariableStatement(node)) { node.declarationList.declarations.forEach(n => visit(n, scope, docOf(node))); return; }
    const name = nameOf(sf, node);
    const owner = scope.slice(1).join('.');
    const declaration = scope.length === 1;
    const docs = `${inheritedDocs}\n${docOf(node)}`.trim();
    const match = entries.apiOwners.find(e => e.owner === (declaration ? name : scope[1]));
    const kind = ts.SyntaxKind[node.kind];
    const category = declaration ? 'declaration' : /^on[A-Z]/.test(name) || /\bIEvent\s*</.test(node.type?.getText(sf) ?? '') ? 'event'
      : /Options$/.test(scope[1]) ? 'option' : 'member';
    apis.push({ id: `${p}:${lineAt(sf, node)}:${[...scope, name].join('.')}`, source: p, line: lineAt(sf, node),
      module: scope[0], owner, name, kind, category, modifiers: modifiers(node), optional: Boolean(node.questionToken),
      proposedDocumentation: /\b(experimental|proposed)\b/i.test(docs),
      proposedRuntimeGate: scope[1] === 'Terminal' ? runtimeContracts.find(r => r.module === scope[0])?.proposedMembers.includes(name) ?? null : null, documentation: docs,
      signature: node.getText(sf), ...acceptanceFor(p, [...scope, name].join('.')), ...mapped(match) });
    if (node.members) node.members.forEach(n => visit(n, [...scope, name], docs));
    if (ts.isTypeAliasDeclaration(node) && ts.isTypeLiteralNode(node.type)) node.type.members.forEach(n => visit(n, [...scope, name], docs));
    if (ts.isVariableDeclaration(node) && node.type && ts.isTypeLiteralNode(node.type)) node.type.members.forEach(n => visit(n, [...scope, name], docs));
  }
  visit(sf);
}
const testFiles = referenceFiles.filter(p => p.endsWith('.ts') && (/\.(test|benchmark)\.ts$/.test(p) || p.includes('/test/')));
for (const source of originalBySource.keys()) assert(testFiles.includes(source), `Baseline suite omitted by inventory: ${source}`);
const suites = [];
function calleeNames(node, sf) {
  if (ts.isParenthesizedExpression(node)) return calleeNames(node.expression, sf);
  if (ts.isConditionalExpression(node)) return [...calleeNames(node.whenTrue, sf), ...calleeNames(node.whenFalse, sf)];
  return [node.getText(sf).replace(/\s/g, '')];
}
for (const p of testFiles) {
  const sf = parse(p, await read(p));
  const definitions = [];
  function visit(node, parents = []) {
    if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
      const callees = calleeNames(node.expression, sf);
      const callee = callees.find(c => /^(test\.describe|describe|context|suite|it|test|specify|perfContext|RuntimeCase)(\.(skip|only|serial|parallel|concurrent|todo))*$/.test(c));
      const args = node.arguments ?? [];
      // Playwright test.skip(condition, reason) is a runtime annotation, not registration.
      const registration = args.length > 0 && (ts.isStringLiteralLike(args[0]) || ts.isTemplateExpression(args[0]) || (args[1] && (ts.isArrowFunction(args[1]) || ts.isFunctionExpression(args[1]))));
      if (callee && registration) {
        const title = args[0].getText(sf);
        const isSuite = /^(test\.describe|describe|context|suite|perfContext)(\.|$)/.test(callee);
        definitions.push({ line: lineAt(sf, node), callee, alternativeCallees: callees, kind: isSuite ? 'suite' : callee === 'RuntimeCase' ? 'benchmark' : 'test', title, parents,
          staticTitle: ts.isStringLiteralLike(args[0]), skipped: /\.(skip|todo)\b/.test(callee), conditionalCallee: callees.length > 1,
          ...acceptanceFor(p, [...parents, title].join(' / ')) });
        ts.forEachChild(node, n => visit(n, isSuite ? [...parents, title] : parents));
        return;
      }
    }
    ts.forEachChild(node, n => visit(n, parents));
  }
  visit(sf);
  const productionPath = p.replace(/\.(test|benchmark)\.ts$/, '.ts');
  const mapping = moduleMappings.get(productionPath);
  suites.push({ source: p, family: family(p), kind: p.endsWith('.benchmark.ts') ? 'benchmark' : !p.endsWith('.test.ts') ? 'test-support' : p.includes('/playwright/') || /addons\/[^/]+\/test\//.test(p) ? 'browser-integration' : 'unit',
    definitions, staticTestDefinitions: definitions.filter(d => d.kind === 'test').length,
    execution: { original: originalBySource.has(p) ? { status: 'recorded baseline', evidence: originalBaselinePath,
      ...Object.fromEntries(['discovered', 'executed', 'passed', 'failed', 'pending', 'skipped', 'unrun'].map(key => [key, originalBySource.get(p)[key]])) }
      : { status: 'No original execution evidence mapped', evidence: null, discovered: null, executed: null, passed: null, failed: null, pending: null, skipped: null, unrun: null },
      rust: { status: 'Consult scoped mapping evidence; full suite counts not certified by inventory', discovered: null, executed: null, passed: null, failed: null, skipped: null, unrun: null } },
    ...acceptanceFor(p),
    ...mapped(mapping) });
}
const fixtures = manifest.filter(m => m.path.startsWith('source_repo/test/fixtures/')).map(m => ({ ...m, status: 'unrun', implementation: [], tests: [], gap: 'No Rust terminal fixture runner mapped.' }));
const addons = (await readdir(path.join(root, 'source_repo/addons'), { withFileTypes: true })).filter(e => e.isDirectory()).map(e => {
  const prefix = `source_repo/addons/${e.name}/`;
  return { name: e.name, declarations: declarations.filter(p => p.startsWith(prefix)), modules: modules.filter(m => m.source.startsWith(prefix)).map(m => m.source),
    suites: suites.filter(s => s.source.startsWith(prefix)).map(s => s.source), status: 'missing', gap: 'No Rust addon or Rust-backed browser integration mapped.' };
});
const countBy = (rows, key) => Object.fromEntries([...new Set(rows.map(r => r[key]))].sort().map(k => [k, rows.filter(r => r[key] === k).length]));
const summary = { schemaVersion: 1, equivalence: 'NOT PROVEN', parser: `typescript ${ts.version}`,
  sourceVersion: JSON.parse(await read('source_repo/package.json')).version, referenceManifestSha256: digest(JSON.stringify(manifest)),
  exclusionPolicy: [...excluded].sort(), counts: { referenceFiles: manifest.length, productionModules: modules.length, declarationFiles: declarations.length,
    apis: apis.length, apiCategories: countBy(apis, 'category'), apiStatuses: countBy(apis, 'status'), moduleStatuses: countBy(modules, 'status'),
    testFiles: suites.length, suiteKinds: countBy(suites, 'kind'), staticTestDefinitions: suites.reduce((sum, s) => sum + s.staticTestDefinitions, 0), fixtures: fixtures.length, addons: addons.length,
    modulesWithoutFocusedCriteria: modules.filter(m => m.acceptanceGap).length, apisWithoutFocusedCriteria: apis.filter(a => a.acceptanceGap).length,
    casesWithoutFocusedCriteria: suites.flatMap(s => s.definitions).filter(d => d.kind === 'test' && d.acceptanceGap).length },
  families: Object.keys(countBy(modules, 'family')).map(f => ({ family: f, modules: modules.filter(m => m.family === f).length, partial: modules.filter(m => m.family === f && m.status === 'partial').length,
    missing: modules.filter(m => m.family === f && m.status === 'missing').length, suites: suites.filter(s => s.family === f).length })),
  limitations: ['Static test-definition counts are not runtime case/assertion counts; loops and helpers may generate cases.', 'Source hashes establish this local reference, not a claimed public revision.',
    'Original counts are copied from explicit per-source upstream-baseline results; Rust counts remain unassessed, not inferred from file presence.', 'Partial API mappings describe internal Rust equivalents only; no compatible public Terminal/browser binding is implemented.',
    'proposedDocumentation is a lexical documentation flag, not proof of runtime API gating.', 'Type signatures are retained verbatim, including nested callback and object contracts.', 'Test-support files include shared registrations and runner configurations; conditional test callees retain both alternatives.'] };
const output = { 'manifest.json': manifest, 'evidence-manifest.json': evidenceManifest, 'modules.json': modules, 'apis.json': apis, 'suites.json': suites, 'fixtures.json': fixtures, 'addons.json': addons, 'runtime-contracts.json': runtimeContracts, 'summary.json': summary };
await mkdir(path.join(root, outputDir), { recursive: true });
for (const [name, value] of Object.entries(output)) {
  const p = `${outputDir}/${name}`;
  const json = `${JSON.stringify(value, null, 2)}\n`;
  if (checking) assert((await read(p)) === json, `Stale inventory: ${p}`);
  else await writeFile(path.join(root, p), json);
}
console.log(JSON.stringify({ mode: checking ? 'check' : 'generate', node: process.version, ...summary }, null, 2));
