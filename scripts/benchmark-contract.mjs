import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { parseTrace } from './full-trace-capture.mjs';

// Sorted object keys let independent adapters hash the same JSON contract.
export function canonical(value) {
  if (value === null || typeof value === 'boolean' || typeof value === 'string') return JSON.stringify(value);
  if (typeof value === 'number') {
    assert.ok(Number.isFinite(value) && !Object.is(value, -0), 'Expected finite JSON number, not negative zero');
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  assert.ok(value && Object.getPrototypeOf(value) === Object.prototype, 'Expected plain JSON object');
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
}
export const sha256 = value => createHash('sha256').update(value).digest('hex');
export const jsonHash = value => sha256(canonical(value));
const text = value => assert.ok(typeof value === 'string' && value.trim(), 'Expected nonempty text');
const integer = (value, min) => assert.ok(Number.isSafeInteger(value) && value >= min, 'Invalid integer');
const hash = value => assert.match(value, /^[a-f0-9]{64}$/, 'Invalid SHA256');
const array = value => assert.ok(Array.isArray(value), 'Expected array');
function keys(value, expected) {
  canonical(value);
  assert.deepEqual(Object.keys(value).sort(), expected.split(' ').sort(), 'Unexpected or missing fields');
}
function object(value) {
  assert.ok(value && Object.getPrototypeOf(value) === Object.prototype, 'Expected plain JSON object');
}
function dimensions(value) {
  keys(value, 'cols rows scrollback unicode settings');
  integer(value.cols, 2); integer(value.rows, 1); integer(value.scrollback, 0);
  keys(value.unicode, 'provider version settings');
  text(value.unicode.provider); text(value.unicode.version);
  object(value.unicode.settings); object(value.settings);
  for (const key of ['cols', 'rows', 'scrollback', 'unicode']) assert.ok(!Object.hasOwn(value.settings, key), 'Duplicated terminal option');
}
export function outputHash(output) {
  keys(output, 'terminalTrace addons');
  text(output.terminalTrace);
  // RICH2 includes both buffers, attributes, codepoints/UTF16, cursor, modes,
  // scrollback metadata, replies and events. Reuse its strict existing decoder.
  const terminal = parseTrace(output.terminalTrace);
  object(output.addons);
  return jsonHash({ terminal, addons: output.addons });
}
export function caseIdentity(row) {
  keys(row, 'id options operations inputBytes inputSha256 operationCount expectedOutput expectedOutputSha256');
  text(row.id); dimensions(row.options); array(row.operations);
  let byteCount = 0;
  const input = createHash('sha256');
  for (const operation of row.operations) {
    if (operation.kind === 'write') {
      keys(operation, 'kind encoding data');
      assert.ok(['utf8-bytes', 'utf16le'].includes(operation.encoding), 'Unknown write encoding');
      assert.equal(typeof operation.data, 'string');
      const bytes = Buffer.from(operation.data, 'base64');
      assert.equal(bytes.toString('base64'), operation.data, 'Noncanonical or truncated base64');
      if (operation.encoding === 'utf16le') assert.equal(bytes.length % 2, 0, 'Truncated UTF16 unit');
      byteCount += bytes.length; input.update(bytes);
    } else if (operation.kind === 'resize') {
      keys(operation, 'kind cols rows'); integer(operation.cols, 2); integer(operation.rows, 1);
    } else {
      keys(operation, 'kind addon method arguments');
      assert.equal(operation.kind, 'addon'); text(operation.addon); text(operation.method); array(operation.arguments);
    }
  }
  integer(row.inputBytes, 0); integer(row.operationCount, 0);
  assert.equal(row.inputBytes, byteCount, 'Input byte count mismatch');
  assert.equal(row.operationCount, row.operations.length, 'Operation count mismatch');
  assert.equal(row.inputSha256, input.digest('hex'), 'Input hash mismatch');
  assert.equal(row.expectedOutputSha256, outputHash(row.expectedOutput), 'Expected state hash mismatch');
  const final = parseTrace(row.expectedOutput.terminalTrace).at(-1);
  const size = row.operations.filter(operation => operation.kind === 'resize').at(-1) ?? row.options;
  assert.deepEqual(final.dimensions, [size.cols, size.rows], 'Final dimensions differ from workload');
  const version = Array.from({ length: row.options.unicode.version.length }, (_, index) => row.options.unicode.version.charCodeAt(index));
  assert.deepEqual(final.unicodeVersion, version, 'Final Unicode version differs from workload');
  return jsonHash(row);
}
export function validateManifest(manifest) {
  keys(manifest, 'schemaVersion execution environment warmup repetitions cases');
  assert.equal(manifest.schemaVersion, 1);
  assert.ok(['terminal', 'browser-rendering'].includes(manifest.execution));
  object(manifest.environment);
  // Browser adapters must pin matching viewport/DPR/font/palette and completion.
  if (manifest.execution === 'browser-rendering') {
    for (const field of ['viewport', 'dpr', 'font', 'palette', 'frameCompletion']) assert.ok(Object.hasOwn(manifest.environment, field), `Missing browser ${field}`);
  }
  integer(manifest.warmup, 1); integer(manifest.repetitions, 10); array(manifest.cases);
  assert.ok(manifest.cases.length, 'Missing benchmark cases');
  const ids = new Set();
  for (const row of manifest.cases) {
    caseIdentity(row); assert.ok(!ids.has(row.id), 'Duplicate benchmark case'); ids.add(row.id);
  }
  return jsonHash(manifest);
}
export function statistics(samples, minimum = 10) {
  array(samples); integer(minimum, 1); assert.ok(samples.length >= minimum, 'Insufficient timings');
  assert.ok(samples.every(value => typeof value === 'number' && Number.isFinite(value) && value >= 0), 'Invalid duration');
  const sorted = [...samples].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return { count: sorted.length, medianMs: sorted.length % 2 ? sorted[middle] : sorted[middle - 1] / 2 + sorted[middle] / 2,
    p95Ms: sorted[Math.ceil(0.95 * sorted.length) - 1] };
}
export function validateResult(manifest, result, engine) {
  const manifestSha256 = validateManifest(manifest);
  keys(result, 'schemaVersion engine profile manifestSha256 timing build cases');
  assert.equal(result.schemaVersion, 1); assert.equal(result.engine, engine);
  assert.equal(result.profile, engine === 'javascript' ? 'production' : 'release');
  assert.equal(result.manifestSha256, manifestSha256, 'Manifest identity mismatch');
  keys(result.timing, 'clock unit startupIncluded completion');
  assert.deepEqual(result.timing, { clock: 'monotonic', unit: 'ms', startupIncluded: false, completion: 'awaited' });
  keys(result.build, 'commands sourceSha256 artifactSha256 toolchain flags');
  array(result.build.commands); assert.ok(result.build.commands.length); result.build.commands.forEach(text);
  hash(result.build.sourceSha256); hash(result.build.artifactSha256); text(result.build.toolchain); text(result.build.flags);
  array(result.cases); assert.equal(result.cases.length, manifest.cases.length, 'Missing or extra case');
  const summary = [];
  for (let index = 0; index < manifest.cases.length; index++) {
    const expected = manifest.cases[index], actual = result.cases[index];
    keys(actual, 'id caseSha256 inputBytes operationCount warmup measurements');
    assert.equal(actual.id, expected.id, 'Case order/identity mismatch');
    assert.equal(actual.caseSha256, caseIdentity(expected), 'Options/operations/chunks identity mismatch');
    assert.equal(actual.inputBytes, expected.inputBytes); assert.equal(actual.operationCount, expected.operationCount);
    for (const [field, count] of [['warmup', manifest.warmup], ['measurements', manifest.repetitions]]) {
      array(actual[field]); assert.equal(actual[field].length, count, `Wrong ${field} count`);
      for (const sample of actual[field]) {
        keys(sample, 'durationMs completion pendingOperations output outputSha256');
        statistics([sample.durationMs], 1);
        assert.equal(sample.completion, 'completed', 'Unawaited work');
        assert.equal(sample.pendingOperations, 0, 'Pending asynchronous work');
        assert.equal(sample.outputSha256, outputHash(sample.output), 'Reported state hash mismatch');
        assert.equal(sample.outputSha256, expected.expectedOutputSha256, 'Wrong terminal/addon output');
      }
    }
    const stats = statistics(actual.measurements.map(sample => sample.durationMs));
    summary.push({ id: actual.id, ...stats, bytesPerSecond: stats.medianMs > 0 ? expected.inputBytes * 1000 / stats.medianMs : null,
      operationsPerSecond: stats.medianMs > 0 ? expected.operationCount * 1000 / stats.medianMs : null });
  }
  return summary;
}
