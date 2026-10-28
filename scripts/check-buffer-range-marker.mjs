// Copyright (c) 2018-2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { runScopedOracle } from './oracle-evidence.mjs';

const root = resolve(import.meta.dirname, '..'); process.chdir(root);
const attemptReport = await runScopedOracle(resolve('results/buffer-range-marker'), {
  kind: 'buffer-range-marker-source', command: 'node scripts/check-buffer-range-marker.mjs', scope: 'Range/marker source oracle only; native replay is separate'
}, async context => {
const directory = resolve('tmp/buffer-range-marker', context.runId);
mkdirSync(directory, { recursive: true });
const sourceFiles = ['Lifecycle.ts', 'Event.ts', 'buffer/Marker.ts', 'buffer/BufferRange.ts'];
const hashes = {};
for (const file of sourceFiles) {
  const original = readFileSync(`source_repo/src/common/${file}`, 'utf8');
  hashes[file] = createHash('sha256').update(original).digest('hex');
  const modified = original
    .replace("import { IDisposable, DisposableStore, toDisposable } from './Lifecycle';", "import { DisposableStore, toDisposable } from './Lifecycle.mjs';")
    .replace("import { dispose, IDisposable } from '../Lifecycle';", "import { dispose } from './Lifecycle.mjs';")
    .replace("import { IMarker } from './Types';", '')
    .replace("import { Emitter } from '../Event';", "import { Emitter } from './Event.mjs';")
    .replace("import { IBufferRange } from '@xterm/xterm';", '');
  writeFileSync(resolve(directory, file.split('/').at(-1).replace('.ts', '.mjs')),
    stripTypeScriptTypes(modified, { mode: 'transform' }));
}
const { Marker } = await import(pathToFileURL(resolve(directory, 'Marker.mjs')));
const { getRangeLength } = await import(pathToFileURL(resolve(directory, 'BufferRange.mjs')));
const originalTests = readFileSync('source_repo/src/common/buffer/BufferRange.test.ts', 'utf8');
let baselineCases = 0;
let baselineAssertions = 0;
const originalBody = originalTests
  .replace("import { assert } from 'chai';", '')
  .replace("import { getRangeLength } from './BufferRange';", '')
  .replace("import { IBufferRange } from '@xterm/xterm';", '');
const countedAssert = {
  equal: (...args) => { baselineAssertions++; assert.equal(...args); },
  throws: (...args) => { baselineAssertions++; assert.throws(...args); }
};
new Function('assert', 'getRangeLength', 'describe', 'it', stripTypeScriptTypes(originalBody))(
  countedAssert, getRangeLength, (_name, fn) => fn(), (_name, fn) => { fn(); baselineCases++; });

const seed = 0x42524e47;
let state = seed;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state | 0;
}
const ranges = [];
function addRange(x1, y1, x2, y2, cols) {
  let result;
  try { result = getRangeLength({ start: { x: x1, y: y1 }, end: { x: x2, y: y2 } }, cols); }
  catch (error) { result = `error:${error.message}`; }
  ranges.push([x1, y1, x2, y2, cols, result].join('\t'));
}
for (const min of [-2147483648, -1, 0, 1, 2147483647]) {
  for (const max of [-2147483648, -1, 0, 1, 2147483647]) {
    for (const cols of [-2147483648, -1, 0, 1, 2147483647]) addRange(min, min, max, max, cols);
  }
}
for (let i = 0; i < 1024; i++) addRange(random(), random(), random(), random(), random());
const fixture = ranges.join('\n') + '\n';
const fixturePath = 'crates/xterm-core/tests/fixtures/buffer-range-oracle.tsv';
const numberFixturePath = 'crates/xterm-core/tests/fixtures/buffer-range-number-oracle.tsv';
const numberSeed = 0x424e554d;
let numberState = numberSeed;
const numberBytes = new DataView(new ArrayBuffer(8));
function randomNumber() {
  function bits() { numberState ^= numberState << 13; numberState ^= numberState >>> 17; numberState ^= numberState << 5; return numberState >>> 0; }
  numberBytes.setUint32(0, bits(), true); numberBytes.setUint32(4, bits(), true);
  return numberBytes.getFloat64(0, true);
}
const numberText = value => Object.is(value, -0) ? '-0' : String(value);
const numberRanges = [];
function addNumberRange(x1, y1, x2, y2, cols) {
  let result;
  try { result = numberText(getRangeLength({ start: { x: x1, y: y1 }, end: { x: x2, y: y2 } }, cols)); }
  catch (error) { result = `error:${error.message}`; }
  numberRanges.push([...([x1, y1, x2, y2, cols].map(numberText)), result].join('\t'));
}
const numberEdges = [0, -0, 1, -1, .5, -.5, 1e-7, 1e21, Number.MIN_VALUE, Number.MAX_VALUE, Number.MAX_SAFE_INTEGER, 2 ** 53, 2 ** 32 - 1, Infinity, -Infinity, NaN];
for (const value of numberEdges) {
  for (let field = 0; field < 5; field++) {
    const values = [1, 1, 4, 5, 5]; values[field] = value; addNumberRange(...values);
  }
  addNumberRange(value, 2, -value, 1, 0);
}
for (const start of numberEdges) for (const end of numberEdges) for (const cols of [0, -0, 1, -1, Infinity, NaN]) addNumberRange(0, start, 1, end, cols);
for (let i = 0; i < 2048; i++) addNumberRange(randomNumber(), randomNumber(), randomNumber(), randomNumber(), randomNumber());
const numberFixture = numberRanges.join('\n') + '\n';
const markerPath = 'crates/xterm-core/tests/fixtures/marker-trace.txt';
const log = [];
const marker = new Marker(7);
marker.onDispose(() => {
  log.push(`event:${marker.isDisposed}:${marker.line}`);
  marker.dispose();
  marker.register({ dispose: () => log.push('during-event') });
});
const resource = { dispose: () => log.push('resource') };
assert.equal(marker.register(resource), resource);
marker.register(resource);
marker.dispose(); marker.dispose();
marker.register({ dispose: () => log.push('late') });
marker.onDispose(() => assert.fail('disposed emitter fired'));
marker.dispose();
assert.deepEqual(log, ['event:true:-1', 'resource', 'resource', 'during-event']);
const trace = log.join('\n') + '\n';

// Independent branch checks mirrored by the Rust tests, beyond the shared trace.
const dynamic = new Marker(1), live = [];
dynamic.register({ dispose() {
  live.push('first'); dynamic.dispose();
  dynamic.register({ dispose: () => live.push('appended') });
} });
dynamic.register({ dispose: () => live.push('second') });
dynamic.dispose();
assert.deepEqual(live, ['first', 'second', 'appended']);
const failed = new Marker(2), failure = new Error('listener failure');
let reached = false;
failed.onDispose(() => { throw failure; });
failed.onDispose(() => { reached = true; });
failed.register({ dispose: () => { reached = true; } });
assert.throws(() => failed.dispose(), error => error === failure);
assert.equal(failed.isDisposed, true);
assert.equal(failed.line, -1);
failed.dispose(); assert.equal(reached, false);
failed.line = 19; failed.dispose(); assert.equal(failed.line, 19);
assert.equal(dynamic.id, marker.id + 1);
assert.equal(failed.id, dynamic.id + 1);
Marker._nextId = 2 ** 53;
const limitMarker = new Marker(0), nextLimitMarker = new Marker(0);
assert.equal(limitMarker.id, 2 ** 53);
assert.equal(nextLimitMarker.id, 2 ** 53);
assert.notEqual(limitMarker, nextLimitMarker);

if (process.argv.includes('--write-fixtures')) {
  writeFileSync(fixturePath, fixture);
  writeFileSync(markerPath, trace);
  writeFileSync(numberFixturePath, numberFixture);
} else {
  assert.equal(readFileSync(fixturePath, 'utf8').replaceAll('\r\n', '\n'), fixture);
  assert.equal(readFileSync(markerPath, 'utf8').replaceAll('\r\n', '\n'), trace);
  assert.equal(readFileSync(numberFixturePath, 'utf8').replaceAll('\r\n', '\n'), numberFixture);
}
const summary = {
  scope: 'BufferRange and Marker utilities only; real Buffer integration remains separate criterion122',
  command: 'node scripts/check-buffer-range-marker.mjs',
  node: process.version, platform: process.platform, seed,
  baselineCases, baselineAssertions, rangeComparisons: ranges.length,
  numberSeed, numberRangeComparisons: numberRanges.length,
  numberFixtureSha256: createHash('sha256').update(numberFixture).digest('hex'),
  markerTraceCallbacks: log.length, markerCounterAssertions: 3, markerAdditionalBranches: ['live resources', 'event failure', 'state before event', 'id sequencing', 'post-dispose line mutation'],
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'),
  markerTraceSha256: createHash('sha256').update(trace).digest('hex'),
  hashes, status: 'passed'
};
return summary;
});
console.log(JSON.stringify(attemptReport, null, 2));
