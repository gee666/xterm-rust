import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { mkdir, mkdtemp, readFile, writeFile, rm, rename, symlink } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { runInNewContext } from 'node:vm';
import { fileURLToPath } from 'node:url';
import { FAMILIES, buildCases, validateCaseContracts, verifyCaseSnapshot } from './acceptance-case-index.mjs';
import { assertJsonSafe, portablePath, regularFile, referenceFile, validateCaptures, validateResult } from './acceptance-validation.mjs';
import { parseArguments, runAttempt } from './acceptance-oracle.mjs';
import { runCommand } from './quality-process.mjs';
import { runCase as routeTerminal } from './acceptance-adapters/terminal.mjs';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const hash = value => createHash('sha256').update(value).digest('hex');
const copy = value => JSON.parse(JSON.stringify(value));
const enoughDisk = async () => ({ availableBytes: '4294967296', minimumBytes: '1073741824', sufficient: true });
function contract(family = 'move') {
  return { id: `${family}-01`, criterionId: 1, title: 'Runner self-test fixture, not port evidence', source: 'source_repo/oracle.ts', ...FAMILIES[family] };
}
async function fixture(t, family = 'move') {
  await mkdir(path.join(project, 'tmp'), { recursive: true });
  const root = await mkdtemp(path.join(project, 'tmp/acceptance-runner-'));
  t.after(() => rm(root, { recursive: true }));
  for (const directory of ['source_repo', 'target', 'logs', 'scripts/acceptance-adapters']) await mkdir(path.join(root, directory), { recursive: true });
  const source = 'if (!webgl2) test.skip();\n';
  const artifact = Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]);
  const item = contract(family);
  await writeFile(path.join(root, 'source_repo/oracle.ts'), source);
  await writeFile(path.join(root, 'target/self-test.wasm'), artifact);
  await writeFile(path.join(root, 'logs/test.log'), 'self-test fixture command\n');
  await writeFile(path.join(root, `scripts/acceptance-adapters/${item.adapter}`), '// fixture only\n');
  const domain = Object.fromEntries(item.fields.map(field => [field, []]));
  const result = {
    caseId: item.id,
    sourceHashes: [{ path: 'source_repo/oracle.ts', sha256: hash(source) }],
    artifactHashes: [{ path: 'target/self-test.wasm', sha256: hash(artifact) }],
    commands: [{ argv: ['self-test-fixture'], exitCode: 0, log: 'logs/test.log' }],
    captures: item.engines.map(engine => ({ engine, runtimeVersion: 'self-test, not actual engine', implementation: engine === 'native' ? 'rust-native' : 'rust-wasm', assertions: 1, reference: copy(domain), rust: copy(domain) })),
  };
  return { root, item, result };
}

test('strict CLI arguments reject unknown, absent, duplicate and invalid deadlines', () => {
  assert.equal(parseArguments(['--case', 'img-01']).caseId, 'img-01');
  assert.equal(parseArguments(['--list']).list, true);
  for (const args of [[], ['--other'], ['--case'], ['--case', '../x'], ['--list', '--case', 'img-01'], ['--case', 'img-01', '--case', 'img-02'], ['--case', 'img-01', '--timeout-ms', 'NaN'], ['--case', 'img-01', '--timeout-ms', '0'], ['--case', 'img-01', '--timeout-ms', '900001']]) assert.throws(() => parseArguments(args));
});

test('case extraction rejects duplicate, unknown and empty contracts', () => {
  const criterion = { id: 280, revision: 1, title: 'fixture', body: 'edge cases', source: 'source_repo/oracle.ts', verification: 'node scripts/acceptance-oracle.mjs --case img-01' };
  assert.equal(buildCases([criterion])[0].id, 'img-01');
  assert.throws(() => buildCases([]), /no acceptance/);
  assert.throws(() => buildCases([criterion, criterion]), /duplicate/);
  assert.throws(() => buildCases([{ ...criterion, verification: 'node scripts/acceptance-oracle.mjs --case other-01' }]), /unknown/);
  const weakened = contract('img'); weakened.engines = ['native'];
  assert.throws(() => validateCaseContracts([weakened]), /weakened/);
});

test('exact mode case verification requires all three browsers in any stated order', () => {
  const criterion = { id: 356, revision: 2, title: 'browser mode', body: 'live state', source: 'source_repo/oracle.ts',
    verification: 'node scripts/acceptance-oracle.mjs --case vt-28. In WebKit/Chromium/Firefox compare actual Rust-backed WASM.' };
  const cases = buildCases([criterion]);
  assert.deepEqual(cases[0].engines, ['chromium', 'firefox', 'webkit']);
  assert.deepEqual(cases[0].fields.slice(-3), ['render', 'api', 'callbacks']);
  const nativeOnly = copy(cases); nativeOnly[0].engines = ['native'];
  assert.throws(() => validateCaseContracts(nativeOnly), /weakened/);
});

test('case catalog cannot weaken verification against the exported criterion authority', () => {
  const criterion = { id: 356, revision: 2, title: 'browser mode', body: 'live state', source: 'source_repo/oracle.ts',
    verification: 'node scripts/acceptance-oracle.mjs --case vt-28. Chromium/Firefox/WebKit actual WASM.' };
  const cases = buildCases([criterion]);
  verifyCaseSnapshot(cases, [criterion]);
  const weakened = { ...criterion, verification: 'node scripts/acceptance-oracle.mjs --case vt-28. Native only.' };
  const changedCatalog = buildCases([weakened]);
  assert.throws(() => verifyCaseSnapshot(changedCatalog, [criterion]), /authoritative/);
  assert.throws(() => buildCases([{ ...criterion, verification: `${criterion.verification} [full text: missing]` }]), /unresolved/);
});

test('JSON boundary rejects lossy values while retaining lone UTF16 strings', () => {
  for (const value of [-0, NaN, Infinity, undefined, 1n, new Map(), new Error('hidden message'), new Uint8Array([1])]) assert.throws(() => assertJsonSafe({ value }));
  assertJsonSafe({ value: '\ud800', values: [null, true, 0] });
  assert.equal(JSON.parse(JSON.stringify('\ud800')), '\ud800');
  const reused = {};
  assert.throws(() => assertJsonSafe({ reference: reused, rust: reused }), /reuses/);
});

test('signed zero must be tagged before crossing the JSON worker boundary', () => {
  const item = contract();
  const domain = Object.fromEntries(item.fields.map(field => [field, []]));
  const result = { caseId: item.id, captures: [{
    engine: 'native', implementation: 'rust-native', runtimeVersion: 'self-test', assertions: 1,
    reference: { ...copy(domain), position: -0 }, rust: { ...copy(domain), position: 0 },
  }] };
  assert.throws(() => validateCaptures(item, result), /losslessly/);
  assert.equal(JSON.stringify(-0), '0');
  assertJsonSafe({ position: { number: '-0' } });
});

test('serialization hooks, array properties and hidden fields cannot erase mismatches', () => {
  const extra = []; extra.label = 'source state';
  const symbol = { [Symbol('state')]: 1 };
  const hidden = Object.defineProperty({}, 'state', { value: 1 });
  const hook = Object.defineProperty([], 'toJSON', { value: () => [] });
  const sparse = Array(2);
  let getterCalls = 0;
  const accessor = Object.defineProperty({}, 'state', { enumerable: true, get() { getterCalls++; return 1; } });
  const proxy = new Proxy({}, {});
  const inheritedProxy = Object.create(new Proxy(Object.create(null), {
    getOwnPropertyDescriptor() { return undefined; },
    get() { return () => ({}); },
  }));
  for (const value of [extra, symbol, hidden, hook, sparse, accessor, proxy, inheritedProxy]) assert.throws(() => assertJsonSafe(value));
  assert.equal(getterCalls, 0);
  assertJsonSafe(runInNewContext('({ state: [1,2] })'));
  assertJsonSafe(Object.assign(Object.create(null), { state: 1 }));
  const cycle = {}; cycle.self = cycle;
  assert.throws(() => assertJsonSafe(cycle), /reuses/);
});

test('unsafe paths are rejected before file reads', () => {
  for (const value of ['', '../file', 'source_repo/../file', '/absolute', 'C:/file', 'a\\b', 'a//b', 'a\0b', 'target/file:stream']) assert.throws(() => portablePath(value));
  assert.equal(portablePath('target/a.wasm'), 'target/a.wasm');
});

test('zero assertions, mismatches and wrong capture domains fail closed', async t => {
  const { item, result } = await fixture(t);
  for (const mutate of [r => { r.captures = []; }, r => { r.captures[0].assertions = 0; }, r => { r.captures[0].rust.events = ['different']; }, r => { delete r.captures[0].rust.postState; }, r => { r.caseId = 'move-02'; }, r => { r.captures[0].implementation = 'original-js'; }]) {
    const changed = copy(result); mutate(changed); assert.throws(() => validateCaptures(item, changed));
  }
});

test('browser matrix cannot omit or duplicate an engine or use native substitution', async t => {
  const { item, result } = await fixture(t, 'img');
  assert.equal(validateCaptures(item, result).executedRuntimes, 3);
  for (const mutate of [r => r.captures.pop(), r => r.captures.push(copy(r.captures[0])), r => { r.captures[0].engine = 'native'; }, r => { r.captures[0].runtimeVersion = ''; }]) {
    const changed = copy(result); mutate(changed); assert.throws(() => validateCaptures(item, changed));
  }
});

test('source-defined GPU skips remain skips and cannot erase every assertion', async t => {
  const { root, item, result } = await fixture(t, 'gpu');
  result.captures[1] = { engine: 'firefox', skipped: { reason: 'actual source guard fixture', source: 'source_repo/oracle.ts', condition: 'if (!webgl2) test.skip();' } };
  assert.equal((await validateResult(root, item, result)).skippedRuntimes, 1);
  result.captures[1].skipped.condition = 'invented guard';
  await assert.rejects(validateResult(root, item, result), /not present/);
  result.captures = item.engines.map(engine => ({ engine, skipped: { reason: 'fixture', source: 'source_repo/oracle.ts', condition: 'guard' } }));
  assert.throws(() => validateCaptures(item, result), /all runtimes/);
});

test('hash, artifact, command and nonregular reference failures propagate', async t => {
  const { root, item, result } = await fixture(t);
  assert.equal((await validateResult(root, item, result)).assertions, 1);
  for (const mutate of [r => { r.sourceHashes[0].sha256 = '0'.repeat(64); }, r => { r.artifactHashes = []; }, r => { r.commands = []; }, r => { r.commands[0].exitCode = 1; }, r => { r.commands[0].log = 'logs'; }]) {
    const changed = copy(result); mutate(changed); await assert.rejects(validateResult(root, item, changed));
  }
  await assert.rejects(regularFile(root, 'logs'), /nonregular/);
});

test('worktree reference root may be shared without allowing arbitrary nested links', async t => {
  const { root, item, result } = await fixture(t);
  const reference = path.join(root, 'shared-reference');
  await rename(path.join(root, 'source_repo'), reference);
  await symlink(reference, path.join(root, 'source_repo'), 'junction');
  assert.equal((await validateResult(root, item, result)).assertions, 1);
  await assert.rejects(regularFile(root, 'source_repo/oracle.ts'), /symbolic/);
  const other = path.join(root, 'outside');
  await mkdir(other);
  await writeFile(path.join(other, 'secret.ts'), 'not oracle input');
  await symlink(other, path.join(reference, 'nested'), 'junction');
  await assert.rejects(referenceFile(root, 'source_repo/nested/secret.ts'), /symbolic/);
});

test('missing family adapter is retained as unrun, never success', async t => {
  const { root, item } = await fixture(t);
  await rm(path.join(root, `scripts/acceptance-adapters/${item.adapter}`));
  const receipt = await runAttempt(root, item, { observeDisk: enoughDisk });
  assert.equal(receipt.status, 'unrun');
  assert.equal(receipt.failurePhase, 'locate-adapter');
  assert.equal(JSON.parse(await readFile(path.join(root, receipt.outputDirectory, 'receipt.json'), 'utf8')).status, 'unrun');
});

test('receipt schema is validated after successful process exit and attempts are unique', async t => {
  const { root, item, result } = await fixture(t);
  const execute = async (_, args, options) => {
    const configuration = JSON.parse(await readFile(args[1], 'utf8'));
    await writeFile(options.log, 'self-test execution receipt\n');
    await writeFile(path.join(configuration.outputDirectory, 'worker-result.json'), JSON.stringify(result));
    return { status: 'passed', exitCode: 0 };
  };
  const first = await runAttempt(root, item, { execute, observeDisk: enoughDisk });
  const second = await runAttempt(root, item, { execute, observeDisk: enoughDisk });
  assert.equal(first.status, 'passed');
  assert.notEqual(first.outputDirectory, second.outputDirectory);
  const bad = await runAttempt(root, item, { observeDisk: enoughDisk, execute: async () => ({ status: 'passed', exitCode: 0 }) });
  assert.equal(bad.status, 'failed');
  assert.equal(bad.failurePhase, 'validate-result');
});

test('preflight cancellation and low disk never start the adapter', async t => {
  const { root, item } = await fixture(t);
  const controller = new AbortController(); controller.abort(new Error('test cancellation'));
  const execute = async () => { throw new Error('must not execute'); };
  const aborted = await runAttempt(root, item, { execute, observeDisk: enoughDisk, signal: controller.signal });
  assert.equal(aborted.cancelled, true);
  const low = await runAttempt(root, item, { execute, observeDisk: async () => ({ availableBytes: '0', minimumBytes: '1', sufficient: false }) });
  assert.equal(low.failurePhase, 'resource-preflight');
  assert.match(low.error, /Insufficient disk/);
});

test('actual hung and throwing child processes fail with retained output', { timeout: 20_000 }, async t => {
  const { root, item } = await fixture(t);
  const hung = await runAttempt(root, item, { timeoutMs: 100, observeDisk: enoughDisk, execute: (_, __, options) => runCommand(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], options) });
  assert.equal(hung.status, 'failed');
  assert.equal(hung.process.timedOut, true);
  const thrown = await runAttempt(root, item, { observeDisk: enoughDisk, execute: (_, __, options) => runCommand(process.execPath, ['-e', 'throw new Error("callback failure fixture")'], options) });
  assert.equal(thrown.status, 'failed');
  assert.match(await readFile(path.join(root, thrown.outputDirectory, 'process.log'), 'utf8'), /callback failure fixture/);
});

test('active cancellation stops an actual owned adapter process', { timeout: 20_000 }, async t => {
  const { root, item } = await fixture(t);
  const receipt = await runAttempt(root, item, {
    timeoutMs: 10_000, signal: AbortSignal.timeout(100), observeDisk: enoughDisk,
    execute: (_, __, options) => runCommand(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], options),
  });
  assert.equal(receipt.status, 'failed');
  assert.equal(receipt.cancelled, true);
  assert.equal(receipt.process.aborted, true);
});

test('actual worker checks data before JSON can hide signed zero or array properties', { timeout: 20_000 }, async t => {
  const { root, item, result } = await fixture(t);
  for (const file of ['acceptance-worker.mjs', 'acceptance-validation.mjs']) {
    await writeFile(path.join(root, 'scripts', file), await readFile(path.join(project, 'scripts', file)));
  }
  const adapter = path.join(root, `scripts/acceptance-adapters/${item.adapter}`);
  const cases = [
    { mutation: '', status: 'passed' },
    { mutation: 'result.captures[0].reference.position=-0; result.captures[0].rust.position=0;', status: 'failed' },
    { mutation: 'result.captures[0].reference.returns.hidden=1; result.captures[0].rust.returns.hidden=2;', status: 'failed' },
  ];
  for (const scenario of cases) {
    await writeFile(adapter, `export async function runCase(){ const result=${JSON.stringify(result)}; ${scenario.mutation} return result; }`);
    const receipt = await runAttempt(root, item, { observeDisk: enoughDisk });
    assert.equal(receipt.status, scenario.status, scenario.mutation || 'synthetic worker control, not port evidence');
  }
});

test('parallel terminal case owners route exactly and duplicates fail', async t => {
  const { root } = await fixture(t);
  const directory = path.join(root, 'scripts/acceptance-adapters/terminal');
  await mkdir(directory);
  await writeFile(path.join(directory, 'one.cases.mjs'), 'export const caseIds=["vt-22"]; export async function runCase(c){return {fixture:c.caseId}};');
  assert.deepEqual(await routeTerminal({ root, caseId: 'vt-22' }), { fixture: 'vt-22' });
  await assert.rejects(routeTerminal({ root, caseId: 'vt-99' }), /no original/);
  await writeFile(path.join(directory, 'two.cases.mjs'), 'export const caseIds=["vt-22"]; export async function runCase(){return {}};');
  await assert.rejects(routeTerminal({ root, caseId: 'vt-22' }), /duplicate/);
});
