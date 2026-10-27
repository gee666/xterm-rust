// Copyright (c) 2018-2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

process.chdir(resolve(import.meta.dirname, '..'));
const directory = resolve('tmp/buffer-range-marker');
mkdirSync(directory, { recursive: true });
mkdirSync('results/buffer-range-marker', { recursive: true });
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

if (process.argv.includes('--write-fixtures')) {
  writeFileSync(fixturePath, fixture);
  writeFileSync(markerPath, trace);
} else {
  assert.equal(readFileSync(fixturePath, 'utf8').replaceAll('\r\n', '\n'), fixture);
  assert.equal(readFileSync(markerPath, 'utf8').replaceAll('\r\n', '\n'), trace);
}
const summary = {
  scope: 'BufferRange and Marker utilities only; real Buffer integration remains separate criterion122',
  command: 'node scripts/check-buffer-range-marker.mjs',
  node: process.version, platform: process.platform, seed,
  baselineCases, baselineAssertions, rangeComparisons: ranges.length,
  markerTraceCallbacks: log.length, markerAdditionalBranches: ['live resources', 'event failure', 'state before event', 'id sequencing', 'post-dispose line mutation'],
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'),
  markerTraceSha256: createHash('sha256').update(trace).digest('hex'),
  hashes, status: 'passed'
};
writeFileSync('results/buffer-range-marker/oracle.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
