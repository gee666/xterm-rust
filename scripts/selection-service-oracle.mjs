// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Runs actual upstream classes and unchanged test assertions, then records traces.
import assert from 'node:assert/strict';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { sourceLoader } from './source-loader.mjs';
const { load, hashes, typescript } = sourceLoader();
const { SelectionService } = load('source_repo/src/browser/services/SelectionService.ts');
const prototype = SelectionService.prototype;
const originalText = Object.getOwnPropertyDescriptor(prototype, 'selectionText').get;
const originalHas = Object.getOwnPropertyDescriptor(prototype, 'hasSelection').get;
const traces = [];
let recording = false;
const units = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join('.') || '-';
const point = position => position ? position.join(',') : '-,-';
const model = value => [+value.isSelectAllActive, value.selectionStartLength, point(value.selectionStart), point(value.selectionEnd)].join(',');
function input(service, op, args) {
  const buffers = service._bufferService;
  const buffer = buffers.buffer;
  const rows = [];
  for (let row = 0; row < buffer.lines.length; row++) {
    const line = buffer.lines.get(row);
    const cells = [];
    for (let column = 0; column < line.length; column++) cells.push(`${line.getWidth(column)},${units(line.getString(column))}`);
    rows.push(`${+line.isWrapped}:${cells.join(';')}`);
  }
  return [op, buffers.cols, buffers.rows, buffer.ybase, service._activeSelectionMode,
    args, units(service._optionsService.rawOptions.wordSeparator), model(service._model), rows.join('|')];
}
function result(service, value = '-') {
  return [model(service._model), point(service._model.finalSelectionStart), point(service._model.finalSelectionEnd),
    +originalHas.call(service), units(originalText.call(service)), value];
}
for (const [name, op] of [['_selectWordAt', 'word'], ['_selectToWordAt', 'toword'], ['_selectLineAt', 'line'], ['selectAll', 'all'], ['selectLines', 'lines']]) {
  const method = prototype[name];
  prototype[name] = function (...args) {
    if (recording) return method.apply(this, args);
    recording = true;
    try {
      const encodedArgs = args.flat().map(value => typeof value === 'boolean' ? +value : value).join(',') || '-';
      const before = input(this, op, encodedArgs);
      const value = method.apply(this, args);
      traces.push([...before, ...result(this)].join('\t'));
      return value;
    } finally { recording = false; }
  };
}
for (const [name, op, getter] of [['selectionText', 'text', originalText], ['hasSelection', 'has', originalHas]]) {
  Object.defineProperty(prototype, name, { configurable: true, get() {
    if (recording) return getter.call(this);
    recording = true;
    try {
      const before = input(this, op, '-');
      const value = getter.call(this);
      traces.push([...before, ...result(this)].join('\t'));
      return value;
    } finally { recording = false; }
  }});
}
const suites = [];
const tests = [];
globalThis.describe = (name, callback) => { suites.push({ name, before: [] }); try { callback(); } finally { suites.pop(); } };
globalThis.beforeEach = callback => suites.at(-1).before.push(callback);
globalThis.it = (name, callback) => tests.push({ name: [...suites.map(suite => suite.name), name].join(' / '), callback, before: suites.flatMap(suite => suite.before) });
const outcomes = [];
try {
  load('source_repo/src/browser/services/SelectionService.test.ts');
  for (const test of tests) {
    try {
      for (const before of test.before) await before();
      await test.callback();
      outcomes.push({ name: test.name, status: 'passed' });
    } catch (error) { outcomes.push({ name: test.name, status: 'failed', error: String(error.stack) }); }
  }
} finally { delete globalThis.describe; delete globalThis.beforeEach; delete globalThis.it; }
assert.equal(outcomes.length, 26, 'source selection service test count changed');
const upstreamTraceCount = traces.length;
const legacyCacheCase = process.argv.includes('--legacy-cache-case');
const commonMocks = load('source_repo/src/common/TestUtils.test.ts');
const browserMocks = load('source_repo/src/browser/TestUtils.test.ts');
const { BufferLine } = load('source_repo/src/common/buffer/BufferLine.ts');
const { CellData } = load('source_repo/src/common/buffer/CellData.ts');
class NativeFixtureSelection extends SelectionService {
  enable() {}
  disable() {}
  refresh() {}
}
const seed = 0x5e1ec710;
let state = seed;
const next = () => { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; };
const tokens = [['a', 1], ['b', 1], [' ', 1], ['中', 2], ['😁', 1], ['é', 1], ['\u00a0', 1], ['\ud800', 1], [',', 1], ['', 1]];
for (let iteration = 0; iteration < 4096; iteration++) {
  const columns = next() % 20 + 2;
  const totalRows = next() % 6 + 1;
  const visibleRows = next() % totalRows + 1;
  const options = new commonMocks.MockOptionsService();
  options.options.wordSeparator = iteration % 2 ? ' ()[]{},' : ' ,';
  const buffers = new commonMocks.MockBufferService(columns, visibleRows, options);
  buffers.buffer.ybase = totalRows - visibleRows;
  buffers.buffer.lines.length = totalRows;
  for (let row = 0; row < totalRows; row++) {
    const line = new BufferLine(columns, undefined, !!(next() & 1));
    for (let column = 0; column < columns; column++) {
      let [chars, width] = tokens[next() % tokens.length];
      if (width === 2 && column === columns - 1) { chars = ' '; width = 1; }
      line.setCell(column, CellData.fromCharData([0, chars, width, chars.codePointAt(0) ?? 0]));
      if (width === 2) line.setCell(++column, CellData.fromCharData([0, '', 0, 0]));
    }
    buffers.buffer.lines.set(row, line);
  }
  const service = new NativeFixtureSelection(null, null, null, buffers, new commonMocks.MockCoreService(),
    new browserMocks.MockMouseService(), options, new commonMocks.MockMouseStateService(), new browserMocks.MockRenderService(), new browserMocks.MockCoreBrowserService());
  service._activeSelectionMode = next() % 4;
  service._model.selectionStart = [next() % (columns + 3) - 2, next() % totalRows];
  service._model.selectionEnd = next() % 4 ? [next() % (columns + 1), next() % totalRows] : undefined;
  service._model.selectionStartLength = next() % (columns * 3) - columns;
  switch (next() % (legacyCacheCase ? 5 : 6)) {
    case 0: service._selectWordAt([next() % columns, next() % totalRows], !!(next() & 1)); break;
    case 1: service._selectLineAt(next() % totalRows); break;
    case 2: service.selectAll(); break;
    case 3: service.selectLines(next() % (totalRows + 2) - 1, next() % (totalRows + 2)); break;
    case 4: void service.selectionText; break;
    case 5: service._selectToWordAt([next() % columns, next() % totalRows]); break;
  }
}
await mkdir('results/selection-service', { recursive: true });
if (legacyCacheCase) {
  assert(outcomes.every(test => test.status === 'passed'));
  const value = traces[269];
  assert.equal(value.split('\t').length, 15);
  assert(value.startsWith('word\t11\t2\t3\t0\t2,1,0\t'), 'historical cache counterexample input changed');
  assert.equal(value.split('\t')[9], '0,6,9,0,1,4');
  await writeFile('results/selection-service/counterexamples/signed-cache-269.tsv', value + '\n');
  await writeFile('crates/xterm-web/tests/fixtures/selection-cache-regression.tsv', value + '\n');
  await writeFile('results/selection-service/counterexamples/signed-cache-269.json', JSON.stringify({
    seed, originalOperationKinds: 5, originalTraceIndex: 269, sourceHashes: Object.fromEntries([...hashes].sort()),
    sha256: createHash('sha256').update(value + '\n').digest('hex'),
    note: 'Recreated directly from the original seeded five-operation source corpus. The first attempted fixture copy accidentally retained the word undefined because a truncated read was indexed without validation. This regeneration validates schema and known input/model before writing.'
  }, null, 2) + '\n');
  console.log('Historical source cache counterexample regenerated and schema checked');
  process.exit(0);
}
const fixture = 'crates/xterm-web/tests/fixtures/selection-service.tsv';
const corpus = traces.join('\n') + '\n';
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal(await readFile(fixture, 'utf8'), corpus, 'source corpus changed; inspect before regenerating');
const report = {
  command: 'node scripts/selection-service-oracle.mjs', node: process.version, platform: process.platform, typescript,
  sources: Object.fromEntries([...hashes].sort()),
  upstream: { discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(test => test.status === 'passed').length,
    failed: outcomes.filter(test => test.status === 'failed').length, skipped: 0, unrun: 0, outcomes },
  traces: { cases: traces.length, upstreamTraceCount, randomCases: 4096, seed, sha256: createHash('sha256').update(corpus).digest('hex'), fixture,
    description: 'Replay source operations and observations from the complete 26-case unit suite. Force-selection and membership assertions are ported separately in Rust.' },
  limits: ['The unchanged source test subclass disables DOM/refresh. This unit baseline is not browser or terminal integration evidence.',
    'Source prototype instrumentation records inputs/results but delegates every original method and assertion unchanged.']
};
await writeFile('results/selection-service/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ upstream: report.upstream, traces: report.traces }, null, 2));
assert.equal(report.upstream.failed, 0, 'unchanged upstream assertions failed');
