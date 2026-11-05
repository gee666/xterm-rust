import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { canonical, sha256, caseIdentity, outputHash, validateManifest, validateResult, statistics } from './benchmark-contract.mjs';
import { runBenchmarks } from './benchmark-runner.mjs';
import { manifest, result } from './test-fixtures/benchmark-data.mjs';
const root = fileURLToPath(new URL('..', import.meta.url));
const adapter = fileURLToPath(new URL('test-fixtures/benchmark-adapter.mjs', import.meta.url));

function temporary(run) {
  mkdirSync(join(root, 'tmp'), { recursive: true });
  const owned = mkdtempSync(join(root, 'tmp/benchmark-test-'));
  return Promise.resolve().then(() => run(owned)).finally(() => rmSync(owned, { recursive: true }));
}
function config(owned, mode = 'valid') {
  const scratchRoot = join(owned, 'scratch'); mkdirSync(scratchRoot);
  return { manifest: manifest(), adapters: Object.fromEntries(['javascript', 'rust'].map(engine => [engine,
    { program: process.execPath, args: [adapter, mode, engine], cwd: root }])), scratchRoot,
    output: join(owned, 'receipt.json'), timeoutMs: 5000 };
}

test('canonical JSON is key-order independent and rejects lossy values', () => {
  assert.equal(canonical({ b: 1, a: [true, null, '\ud800'] }), canonical({ a: [true, null, '\ud800'], b: 1 }));
  for (const value of [undefined, NaN, Infinity, -0, new Date(), () => {}]) assert.throws(() => canonical(value));
});

test('manifest independently recounts bytes/operations and preserves UTF16 and chunks', () => {
  const input = manifest(); validateManifest(input);
  assert.equal(input.cases[0].inputBytes, 4);
  const original = caseIdentity(input.cases[0]);
  const bytes = Buffer.from(input.cases[0].operations[0].data, 'base64');
  input.cases[0].operations = [bytes.subarray(0, 2), bytes.subarray(2)].map(chunk => ({ kind: 'write', encoding: 'utf16le', data: chunk.toString('base64') }));
  input.cases[0].operationCount = 2;
  assert.notEqual(caseIdentity(input.cases[0]), original);
  input.cases[0].operations = [];
  input.cases[0].inputBytes = 0; input.cases[0].inputSha256 = sha256(Buffer.alloc(0)); input.cases[0].operationCount = 0;
  validateManifest(input);
});

const manifestMutations = {
  'changed bytes': m => { m.cases[0].operations[0].data = 'YgAAAA=='; },
  'truncated base64': m => { m.cases[0].operations[0].data = 'YQ'; },
  'truncated UTF16': m => { m.cases[0].operations[0].data = 'YQ=='; },
  'wrong input count': m => { m.cases[0].inputBytes++; },
  'wrong operation count': m => { m.cases[0].operationCount++; },
  'wrong state hash': m => { m.cases[0].expectedOutputSha256 = '0'.repeat(64); },
  'missing cases': m => { m.cases = []; },
  'duplicate cases': m => { m.cases.push(structuredClone(m.cases[0])); },
  'invalid dimensions': m => { m.cases[0].options.cols = 0; },
  'output dimensions mismatch': m => { m.cases[0].options.rows = 2; },
  'output Unicode mismatch': m => { m.cases[0].options.unicode.version = '11'; },
  'nonobject addon output': m => { m.cases[0].expectedOutput.addons = 4; },
  'duplicated option': m => { m.cases[0].options.settings.cols = 99; },
  'missing browser environment': m => { m.execution = 'browser-rendering'; },
  'no warmup': m => { m.warmup = 0; },
  'insufficient repetitions': m => { m.repetitions = 9; }
};
for (const [name, mutate] of Object.entries(manifestMutations)) test(`manifest rejects ${name}`, () => {
  const input = manifest(); mutate(input); assert.throws(() => validateManifest(input));
});

test('median arithmetic mean and p95 nearest-rank retain all raw outliers', () => {
  assert.deepEqual(statistics([10, 1, 9, 2, 8, 3, 7, 4, 6, 5]), { count: 10, medianMs: 5.5, p95Ms: 10 });
  assert.deepEqual(statistics([7, 2, 3], 1), { count: 3, medianMs: 3, p95Ms: 7 });
  assert.deepEqual(statistics(Array(10).fill(0)), { count: 10, medianMs: 0, p95Ms: 0 });
  assert.equal(statistics([...Array(19).fill(1), 1000]).p95Ms, 1);
  for (const values of [[], [1], [...Array(9).fill(1), NaN], [...Array(9).fill(1), Infinity], [...Array(9).fill(1), -1]]) assert.throws(() => statistics(values));
});

test('full snapshots, addon outputs and every sample must match before aggregation', () => {
  const input = manifest(), receipt = result(input, 'rust');
  assert.equal(validateResult(input, receipt, 'rust')[0].count, 10);
  receipt.cases[0].measurements[0].output.addons.serialize = 'changed';
  receipt.cases[0].measurements[0].outputSha256 = outputHash(receipt.cases[0].measurements[0].output);
  assert.throws(() => validateResult(input, receipt, 'rust'), /Wrong terminal\/addon output/);
});

const resultMutations = {
  'changed options': (m, r) => { m.cases[0].options.settings.convertEol = true; r.manifestSha256 = validateManifest(m); },
  'changed Unicode version': (m, r) => {
    m.cases[0].options.unicode.version = '11';
    m.cases[0].expectedOutput.terminalTrace = m.cases[0].expectedOutput.terminalTrace.replace('~36|0', '~31,31|0');
    m.cases[0].expectedOutputSha256 = outputHash(m.cases[0].expectedOutput);
    r.manifestSha256 = validateManifest(m);
  },
  'missing case': (m, r) => { r.cases = []; },
  'missing warmup': (m, r) => { r.cases[0].warmup = []; },
  'missing iteration': (m, r) => { r.cases[0].measurements.pop(); },
  'pending async': (m, r) => { r.cases[0].measurements[0].pendingOperations = 1; },
  'unawaited sample': (m, r) => { r.cases[0].warmup[0].completion = 'started'; },
  'wrong state': (m, r) => { r.cases[0].measurements[0].output.terminalTrace = r.cases[0].measurements[0].output.terminalTrace.replace('4194304,0,0,,0,0', '4194304,1,0,,0,0'); },
  'nonfinite timing': (m, r) => { r.cases[0].measurements[0].durationMs = NaN; },
  'debug build': (m, r) => { r.profile = 'debug'; },
  'wrong clock': (m, r) => { r.timing.clock = 'wall'; },
  'missing provenance': (m, r) => { r.build.artifactSha256 = ''; }
};
for (const [name, mutate] of Object.entries(resultMutations)) test(`result rejects ${name}`, () => {
  const input = manifest(), receipt = result(input, 'rust'); mutate(input, receipt);
  assert.throws(() => validateResult(input, receipt, 'rust'));
});

test('zero measured durations remain zero and yield no throughput', () => {
  const input = manifest(), receipt = result(input, 'rust');
  receipt.cases[0].measurements.forEach(sample => { sample.durationMs = 0; });
  const row = validateResult(input, receipt, 'rust')[0];
  assert.equal(row.medianMs, 0); assert.equal(row.bytesPerSecond, null); assert.equal(row.operationsPerSecond, null);
});

test('runner passes shared input to both engines and removes only owned scratch', () => temporary(async owned => {
  const settings = config(owned);
  const sentinel = join(settings.scratchRoot, 'keep'); writeFileSync(sentinel, 'unrelated');
  const receipt = await runBenchmarks(settings);
  assert.equal(receipt.status, 'passed', receipt.error);
  assert.equal(receipt.summary[0].javascriptOverRustMedian, 1);
  assert.equal(receipt.commands.length, 2);
  assert.deepEqual(readdirSync(settings.scratchRoot), ['keep']);
  assert.equal(JSON.parse(readFileSync(settings.output, 'utf8')).status, 'passed');
}));

for (const mode of ['nonzero', 'signal', 'malformed', 'missing', 'pending', 'timeout', 'partial-nonzero', 'partial-timeout', 'oversized', 'oversized-monitor']) test(`runner retains ${mode} failure without ratios or leaked scratch`, () => temporary(async owned => {
  const settings = config(owned, mode);
  if (mode === 'timeout' || mode === 'partial-timeout') settings.timeoutMs = 300;
  writeFileSync(settings.output, '{"status":"passed","summary":"stale"}');
  const receipt = await runBenchmarks(settings);
  assert.equal(receipt.status, 'failed'); assert.equal(receipt.summary, null); assert.ok(receipt.error);
  assert.deepEqual(readdirSync(settings.scratchRoot), []);
  assert.equal(JSON.parse(readFileSync(settings.output, 'utf8')).status, 'failed');
  if (mode === 'timeout') assert.equal(receipt.commands[0].timedOut, true);
  if (mode === 'nonzero') assert.equal(receipt.commands[0].exitCode, 7);
  if (mode === 'malformed') assert.equal(receipt.results.javascript.raw, '{broken');
  if (mode.startsWith('partial-')) assert.equal(receipt.results.javascript.raw, '{partial-result}');
  if (mode === 'partial-nonzero' || mode === 'oversized') assert.equal(receipt.commands[0].exitCode, 7);
  if (mode.startsWith('oversized')) {
    assert.equal(receipt.commands[0].output.truncated, true);
    assert.ok(receipt.commands[0].output.bytes > 8 * 1024 * 1024);
    assert.ok(receipt.commands[0].output.raw.length <= 64 * 1024);
  }
  if (mode === 'oversized-monitor') {
    assert.match(receipt.commands[0].resourceError, /size limit exceeded/);
    assert.equal(receipt.commands[0].timedOut, false);
  }
}));

test('CLI exits nonzero and retains invalid-manifest failure', () => temporary(owned => {
  const settings = config(owned); settings.manifest.repetitions = 0;
  const file = join(owned, 'config.json'); writeFileSync(file, JSON.stringify(settings));
  const child = spawnSync(process.execPath, [resolve(root, 'scripts/benchmark-runner.mjs'), '--config', file], { timeout: 5000, encoding: 'utf8' });
  assert.equal(child.status, 1, child.stderr);
  assert.equal(JSON.parse(readFileSync(settings.output, 'utf8')).status, 'failed');
}));
