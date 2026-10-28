// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, readdir, mkdir, writeFile, rm, symlink } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import test from 'node:test';
import { loadBaseline } from '../tools/compatibility/baseline.mjs';
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
    if (item.isDirectory() && p !== 'source_repo/demo/dist' && !['.git', 'node_modules', 'out', 'out-test', 'out-esbuild', 'out-esbuild-test', 'lib', 'coverage'].includes(item.name)) paths.push(...await diskFiles(p));
    if (item.isFile()) paths.push(p);
  }
  return paths.sort();
}

test('every reference path/hash, production module, fixture and addon is indexed', async () => {
  const manifest = await load('manifest');
  assert.equal(manifest.length, 776, 'Generated outputs must not alter the retained source anchor');
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
  const throughput = suites.find(s => s.source.endsWith('/Terminal.benchmark.ts'));
  assert(throughput.definitions.some(d => d.kind === 'benchmark' && d.callee === 'ThroughputRuntimeCase'));
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
  const latest = JSON.parse(await readFile(path.join(root, 'results/upstream-baseline/latest.json'), 'utf8'));
  const baseline = await readFile(path.join(root, 'results/upstream-baseline', latest.summary));
  await writeFile(path.join(root, output, 'baseline-summary.json'), baseline);
  const baselinePath = `${output}/latest.json`;
  await writeFile(path.join(root, baselinePath), JSON.stringify({ ...latest, summary: 'baseline-summary.json' }));
  const args = [`--out=${output}`, `--mappings=${mappingPath}`, `--baseline=${baselinePath}`];
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
  for (const invalid of ['../outside', '/outside', 'C:/outside', 'a\\b', './file', 'a//b']) {
    const escaped = structuredClone(original);
    escaped.modules[0].evidence = [invalid];
    await reject(escaped, /Invalid mapped path/);
  }
  await mkdir(path.join(root, output, 'evidence-target'));
  await writeFile(path.join(root, output, 'evidence-target/file.txt'), 'private fixture');
  await symlink(path.join(root, output, 'evidence-target'), path.join(root, output, 'evidence-linked'), process.platform === 'win32' ? 'junction' : 'dir');
  const linked = structuredClone(original);
  linked.modules[0].evidence = [`${output}/evidence-linked/file.txt`];
  await reject(linked, /Mapped file must not be linked/);
  const project = path.join(root, output, 'root-project');
  await mkdir(path.join(project, 'scripts'), { recursive: true });
  await mkdir(path.join(project, 'tools'));
  await writeFile(path.join(project, 'scripts/compatibility-inventory.mjs'), await readFile(generator));
  const linkType = process.platform === 'win32' ? 'junction' : 'dir';
  await symlink(path.join(root, 'tools/compatibility'), path.join(project, 'tools/compatibility'), linkType);
  await symlink(path.join(root, 'source_repo'), path.join(project, 'source_repo'), linkType);
  bad(spawnSync(process.execPath, ['scripts/compatibility-inventory.mjs'], { cwd: project, encoding: 'utf8', timeout: 60_000 }), /Reference directory must not be linked/);
  bad(run(['--out=../outside']), /Alternative output must be under tmp/);
  bad(run(['--mappings=../outside.json']), /Alternative mappings must be under tmp/);
  await rm(path.join(root, output), { recursive: true });
});

test('failed, incomplete, altered and stale baseline evidence cannot advertise recorded passes', async () => {
  const dir = 'tmp/compatibility-baseline-validation';
  await mkdir(path.join(root, dir), { recursive: true });
  const latest = JSON.parse(await readFile(path.join(root, 'results/upstream-baseline/latest.json'), 'utf8'));
  const original = JSON.parse(await readFile(path.join(root, 'results/upstream-baseline', latest.summary), 'utf8'));
  const manifest = await load('manifest');
  const pointerPath = `${dir}/latest.json`;
  const writeFixture = async (summary, pointer = { ...latest, summary: 'summary.json' }) => {
    await writeFile(path.join(root, dir, 'summary.json'), JSON.stringify(summary));
    await writeFile(path.join(root, pointerPath), JSON.stringify(pointer));
  };
  await writeFixture(original);
  await loadBaseline(root, pointerPath, manifest);
  const mutations = [
    [s => { s.status = 'failed'; }, /summary is not passed/],
    [s => { s.runId = 'stale'; }, /runId mismatch/],
    [s => { s.commands[0].exitCode = 1; }, /command failed/],
    [s => { s.commands[0].timedOut = true; }, /timed out/],
    [s => { s.commands[0].log = `${dir}/missing-command.log`; }, /ENOENT/],
    [s => { s.reference.after.mismatches.push('changed'); }, /reference mismatch/],
    [s => { s.reference.manifestSha256 = '0'.repeat(64); }, /manifest hash mismatch/],
    [s => { s.rawReportSha256 = '0'.repeat(64); }, /raw report hash mismatch/],
    [s => { s.suites[0].passed++; }, /execution mismatch/],
    [s => { s.suites.push(s.suites[0]); }, /Duplicate baseline suite/],
    [s => { s.suites.pop(); }, /suite set incomplete/]
  ];
  for (const [mutate, expected] of mutations) {
    const summary = structuredClone(original);
    mutate(summary);
    await writeFixture(summary);
    await assert.rejects(loadBaseline(root, pointerPath, manifest), expected);
  }
  await writeFixture(original, { ...latest, status: 'failed', summary: 'summary.json' });
  await assert.rejects(loadBaseline(root, pointerPath, manifest), /Latest baseline is not passed/);
  await writeFixture(original, { ...latest, summary: '../outside.json' });
  await assert.rejects(loadBaseline(root, pointerPath, manifest), /Invalid baseline pointer/);
  const duplicateManifest = JSON.parse(await readFile(path.join(root, original.reference.manifest), 'utf8'));
  duplicateManifest[1] = duplicateManifest[0];
  const manifestBytes = JSON.stringify(duplicateManifest);
  const duplicateSummary = structuredClone(original);
  duplicateSummary.reference.manifest = `${dir}/duplicate-manifest.json`;
  duplicateSummary.reference.manifestSha256 = hash(manifestBytes);
  await writeFile(path.join(root, duplicateSummary.reference.manifest), manifestBytes);
  await writeFixture(duplicateSummary);
  await assert.rejects(loadBaseline(root, pointerPath, manifest), /Duplicate baseline manifest path/);
  await writeFixture(original);
  const alteredSource = structuredClone(manifest);
  alteredSource[0].sha256 = '0'.repeat(64);
  await assert.rejects(loadBaseline(root, pointerPath, alteredSource), /source hash mismatch/);
  await rm(path.join(root, dir), { recursive: true });
});
