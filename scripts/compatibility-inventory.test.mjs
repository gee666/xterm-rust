// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir, mkdir, writeFile, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import test from 'node:test';
const root = fileURLToPath(new URL('../', import.meta.url));
const generator = path.join(root, 'scripts/compatibility-inventory.mjs');
const output = 'tmp/compatibility-validation';
const load = async name => JSON.parse(await readFile(path.join(root, `results/compatibility/${name}.json`), 'utf8'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const run = (args, cwd = root) => spawnSync(process.execPath, [generator, ...args], { cwd, encoding: 'utf8', timeout: 60_000, maxBuffer: 4 * 1024 * 1024 });
const ok = result => { assert.ifError(result.error); assert.equal(result.status, 0, result.stderr); };
const bad = (result, expression) => { assert.ifError(result.error); assert.notEqual(result.status, 0); assert.match(result.stderr, expression); };
async function diskFiles(relative) {
  const paths = [];
  for (const item of await readdir(path.join(root, relative), { withFileTypes: true })) {
    const p = `${relative}/${item.name}`;
    if (item.isDirectory() && !['.git', 'node_modules', 'out', 'out-test', 'out-esbuild', 'out-esbuild-test', 'lib', 'coverage'].includes(item.name)) paths.push(...await diskFiles(p));
    if (item.isFile()) paths.push(p);
  }
  return paths.sort();
}

test('every reference path/hash, production module, fixture and addon is indexed', async () => {
  const manifest = await load('manifest');
  assert.deepEqual(manifest.map(m => m.path), await diskFiles('source_repo'));
  for (const row of manifest) {
    const bytes = await readFile(path.join(root, row.path));
    assert.equal(row.bytes, bytes.length, row.path);
    assert.equal(row.sha256, hash(bytes), row.path);
  }
  const expected = manifest.map(m => m.path).filter(p => /\.(ts|tsx|css)$/.test(p) && !/\.(test|benchmark)\.ts$/.test(p)
    && (p.startsWith('source_repo/src/') || /^source_repo\/addons\/[^/]+\/(src|typings)\//.test(p) || /^source_repo\/(css|typings)\//.test(p)));
  assert.deepEqual((await load('modules')).map(m => m.source), expected);
  assert.deepEqual((await load('fixtures')).map(f => f.path), manifest.filter(m => m.path.startsWith('source_repo/test/fixtures/')).map(m => m.path));
  const addons = await load('addons');
  assert.deepEqual(addons.map(a => a.name).sort(), (await readdir(path.join(root, 'source_repo/addons'), { withFileTypes: true })).filter(d => d.isDirectory()).map(d => d.name).sort());
  assert.equal(addons.length, 14);
});

test('all API signatures point to exact source, overload identities and categories survive', async () => {
  const apis = await load('apis');
  assert.equal(new Set(apis.map(a => a.id)).size, apis.length);
  assert.equal(apis.length, 770, 'Reference changes require reviewing this declaration-count anchor');
  const sources = new Map();
  for (const api of apis) {
    if (!sources.has(api.source)) sources.set(api.source, await readFile(path.join(root, api.source), 'utf8'));
    assert(sources.get(api.source).includes(api.signature), api.id);
    assert(api.line > 0 && api.name && api.kind);
    assert(['declaration', 'event', 'option', 'member'].includes(api.category));
    if (api.status === 'missing') assert.deepEqual(api.implementation, []);
    assert(api.gap, `Unproven contract must retain its gap: ${api.id}`);
  }
  assert.equal(sources.size, 16);
  for (const module of ['@xterm/xterm', '@xterm/headless']) {
    const terminal = apis.filter(a => a.module === module && a.owner === 'Terminal');
    assert(terminal.some(a => a.name === 'constructor'));
    assert(terminal.some(a => a.name === 'write' && a.signature.includes('callback')));
    assert(terminal.some(a => a.name === 'unicode' && a.proposedRuntimeGate === true));
    assert(terminal.filter(a => a.name !== 'unicode').every(a => a.proposedRuntimeGate === false));
    assert(apis.some(a => a.module === module && a.owner === 'ITerminalOptions' && a.name === 'allowProposedApi' && a.category === 'option'));
    assert(terminal.some(a => a.name === 'onData' && a.category === 'event'));
  }
  assert(apis.some(a => a.name === '<call>' && a.kind === 'CallSignature'));
});

test('suite support, conditional renderer cases and skip annotation semantics survive', async () => {
  const suites = await load('suites');
  const expected = (await diskFiles('source_repo')).filter(p => p.endsWith('.ts') && (/\.(test|benchmark)\.ts$/.test(p) || p.includes('/test/')));
  assert.deepEqual(suites.map(s => s.source), expected);
  const renderer = suites.find(s => s.source.endsWith('/SharedRendererTests.ts'));
  assert(renderer && renderer.kind === 'test-support');
  assert(renderer.staticTestDefinitions > 10);
  assert(renderer.definitions.some(d => d.conditionalCallee && d.alternativeCallees.includes('test.skip')));
  assert(renderer.definitions.some(d => d.callee === 'test.describe.skip'));
  const terminal = suites.find(s => s.source === 'source_repo/test/playwright/Terminal.test.ts');
  assert(terminal.definitions.some(d => d.parents.length > 1));
  assert(terminal.definitions.every(d => d.callee !== 'test.fail'));
  const eventBenchmark = suites.find(s => s.source.endsWith('/Event.benchmark.ts'));
  assert(eventBenchmark.definitions.some(d => d.kind === 'benchmark'));
  for (const suite of suites) {
    assert.equal(suite.execution.rust.passed, null, 'Inventory must not invent Rust runtime results');
    const original = suite.execution.original;
    if (original.status === 'recorded baseline') {
      assert.equal(original.discovered, original.executed + original.pending + original.skipped + original.unrun);
      assert.equal(original.executed, original.passed + original.failed);
    } else assert.equal(original.passed, null);
    for (const definition of suite.definitions) {
      assert(definition.behaviorCriteria.length || definition.acceptanceGap);
      assert(definition.title && definition.line > 0);
    }
  }
});

test('mapping/evidence hashes and public gating explicitly preserve unproven status', async () => {
  for (const item of await load('evidence-manifest')) {
    const bytes = await readFile(path.join(root, item.path));
    assert.equal(item.sha256, hash(bytes), item.path);
  }
  for (const row of [...await load('modules'), ...await load('apis')]) {
    assert(['partial', 'missing'].includes(row.status));
    assert(row.gap && (row.behaviorCriteria.length || row.acceptanceGap));
    if (row.status === 'partial') assert(row.implementation.length && row.command && row.binding);
  }
  for (const contract of await load('runtime-contracts')) {
    assert.deepEqual(contract.proposedMembers, ['unicode']);
    assert.deepEqual(contract.constructorOnlyOptions, ['cols', 'rows']);
    assert.equal(contract.status, 'missing');
  }
  assert.equal((await load('summary')).equivalence, 'NOT PROVEN');
});

test('non-root generation/check works and every changed artifact fails stale checks', async () => {
  await mkdir(path.join(root, output), { recursive: true });
  // Freeze mapped evidence in this private sandbox so concurrent owner reruns do
  // not invalidate the negative tests. The canonical evidence check stays live.
  const mapping = JSON.parse(await readFile(path.join(root, 'tools/compatibility/mappings.json'), 'utf8'));
  const paths = new Map();
  for (const row of [...mapping.modules, ...mapping.apiOwners]) for (const key of ['implementation', 'tests', 'evidence']) {
    for (let i = 0; i < row[key].length; i++) {
      const source = row[key][i];
      if (!paths.has(source)) {
        const destination = `${output}/evidence-${paths.size}.txt`;
        await writeFile(path.join(root, destination), await readFile(path.join(root, source)));
        paths.set(source, destination);
      }
      row[key][i] = paths.get(source);
    }
  }
  const mappingPath = `${output}/stable-mappings.json`;
  await writeFile(path.join(root, mappingPath), JSON.stringify(mapping));
  const args = [`--out=${output}`, `--mappings=${mappingPath}`];
  ok(run(args, path.join(root, 'tools/compatibility')));
  ok(run([...args, '--check']));
  for (const filename of ['manifest.json', 'modules.json', 'apis.json', 'suites.json', 'fixtures.json', 'summary.json', 'evidence-manifest.json']) {
    const file = path.join(root, output, filename);
    const original = await readFile(file, 'utf8');
    await writeFile(file, `${original} `);
    bad(run([...args, '--check']), /Stale inventory:/);
    await writeFile(file, original);
  }
  ok(run([...args, '--check']));
});

test('duplicate mappings, unknown source, absent evidence and output escape are rejected', async () => {
  const original = JSON.parse(await readFile(path.join(root, 'tools/compatibility/mappings.json'), 'utf8'));
  const mapPath = `${output}/bad-mappings.json`;
  const reject = async (mapping, expression) => {
    await writeFile(path.join(root, mapPath), JSON.stringify(mapping));
    bad(run([`--out=${output}`, `--mappings=${mapPath}`]), expression);
  };
  await reject({ ...original, modules: [...original.modules, original.modules[0]] }, /Duplicate module mapping/);
  const unknown = structuredClone(original);
  unknown.modules[0].source = 'source_repo/src/nonexistent.ts';
  await reject(unknown, /Unknown module mapping/);
  const absent = structuredClone(original);
  absent.modules[0].evidence = ['tmp/compatibility-validation/no-such-evidence.json'];
  await reject(absent, /ENOENT/);
  bad(run(['--out=../outside']), /Alternative output must be under tmp/);
  bad(run(['--mappings=../outside.json']), /Alternative mappings must be under tmp/);
  await rm(path.join(root, output), { recursive: true });
});
