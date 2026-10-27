// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Observe every original Buffer resize call while all upstream assertions run.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '..');
const dependencies = resolve(root, 'source_repo/node_modules');
const { build } = await import(pathToFileURL(resolve(dependencies, 'esbuild/lib/main.js')));
const { default: Mocha } = await import(pathToFileURL(resolve(dependencies, 'mocha/index.js')));
const temporary = resolve(root, 'tmp/buffer-resize-oracle');
mkdirSync(temporary, { recursive: true });
const rows = [];
const cases = [];
let title = 'suite setup';
let sourceTestOrdinal = -1;
const hex = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',') || '-';
function snapshot(buffer, prefix, identities, markers) {
  rows.push([prefix, buffer._cols, buffer._rows, buffer.hostCols ?? buffer._bufferService.cols,
    buffer._bufferService.rows, +buffer._hasScrollback, buffer.lines.maxLength,
    buffer.x, buffer.y, buffer.ybase, buffer.ydisp, buffer.savedX, buffer.savedY,
    buffer.scrollTop, buffer.scrollBottom, buffer.lines.length].join('\t'));
  for (let y = 0; y < buffer.lines.length; ++y) {
    const line = buffer.lines.get(y);
    const cells = [];
    for (let x = 0; x < line.length; ++x) {
      const cell = line.getCell(x);
      cells.push([cell.content >>> 0, cell.fg >>> 0, cell.bg >>> 0,
        cell.extended.ext >>> 0, cell.extended.urlId, hex(cell.combinedData)].join(','));
    }
    rows.push(['line', identities.get(line) ?? 0, +line.isWrapped, cells.join('|') || '-',
      hex(line.translateToString(false)), hex(line.translateToString(true))].join('\t'));
  }
  rows.push(['markers', markers.map(m => `${m.line},${+m.isDisposed}`).join('|') || '-'].join('\t'));
}
globalThis.__captureBufferResize = Buffer => {
  const original = Buffer.prototype.resize;
  Buffer.prototype.resize = function(cols, height) {
    const id = cases.length;
    const identities = new Map();
    for (let y = 0; y < this.lines.length; ++y) identities.set(this.lines.get(y), y + 1);
    const markers = this.markers.slice();
    const options = this._optionsService.rawOptions;
    const pty = options.windowsPty;
    const events = [];
    const subscriptions = [
      this.lines.onInsert(e => events.push(`insert,${e.index},${e.amount}`)),
      this.lines.onDelete(e => events.push(`delete,${e.index},${e.amount}`)),
      this.lines.onTrim(n => events.push(`trim,${n}`)),
      ...markers.map((m, index) => m.onDispose(() => events.push(`marker,${index}`))),
    ];
    rows.push(['case', id, title.replaceAll('\t', ' ')].join('\t'));
    rows.push(['options', options.scrollback, options.tabStopWidth, +options.reflowCursorLine,
      pty.backend ?? '-', pty.buildNumber ?? '-'].join('\t'));
    snapshot(this, 'before', identities, markers);
    rows.push(['resize', cols, height].join('\t'));
    try {
      const result = original.call(this, cols, height);
      snapshot(this, 'after', identities, markers);
      rows.push(['events', events.join('|') || '-'].join('\t'));
      cases.push({ id, title, sourceTestOrdinal, cols, rows: height });
      return result;
    } finally { subscriptions.forEach(s => s.dispose()); }
  };
};
const observerPlugin = { name: 'observe-resize', setup(build) {
  build.onLoad({ filter: /[/\\]buffer[/\\]Buffer\.ts$/ }, args => ({
    contents: readFileSync(args.path, 'utf8') + '\nglobalThis.__captureBufferResize(Buffer);\n', loader: 'ts',
  }));
} };
const bundle = resolve(temporary, 'Buffer.test.cjs');
const built = await build({
  absWorkingDir: root, entryPoints: ['source_repo/src/common/buffer/Buffer.test.ts'],
  outfile: bundle, bundle: true, platform: 'node', format: 'cjs', metafile: true,
  tsconfig: 'source_repo/src/common/tsconfig.json',
  plugins: [observerPlugin],
});
const mocha = new Mocha({ reporter: 'dot' });
mocha.addFile(bundle);
await mocha.loadFilesAsync();
let runner;
const outcomes = [];
const failures = await new Promise(done => {
  runner = mocha.run(done);
  runner.on('test', test => { title = test.fullTitle(); sourceTestOrdinal++; });
  runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
  runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
});
const originalResizeCalls = cases.length;
const seed = 0xb0ff67;
let randomState = seed;
function random() {
  randomState ^= randomState << 13; randomState ^= randomState >>> 17; randomState ^= randomState << 5;
  return randomState >>> 0;
}
const traceBundle = resolve(temporary, 'trace-entry.cjs');
await build({
  absWorkingDir: root, stdin: {
    contents: `export { Buffer } from './buffer/Buffer';
      export { MockOptionsService, MockBufferService, MockLogService } from './TestUtils.test';
      export { CellData } from './buffer/CellData';
      export { ExtendedAttrs } from './buffer/AttributeData';`,
    resolveDir: resolve(root, 'source_repo/src/common'), loader: 'ts',
  }, outfile: traceBundle, bundle: true, platform: 'node', format: 'cjs',
  tsconfig: 'source_repo/src/common/tsconfig.json', plugins: [observerPlugin],
});
const { Buffer, MockOptionsService, MockBufferService, MockLogService, CellData, ExtendedAttrs } =
  (await import(pathToFileURL(traceBundle))).default;
const strings = ['a', '中', '😁', 'e\u0301', '\ud800', '\udc00', ' ', '', '👩‍👧'];
sourceTestOrdinal = -1;
for (let trace = 0; trace < 128; trace++) {
  const cols = 2 + random() % 15;
  const height = 1 + random() % 6;
  const backend = ['conpty', 'winpty', undefined][random() % 3];
  const buildNumber = [0, 21375, 21376, undefined][random() % 4];
  const options = new MockOptionsService({ scrollback: random() % 5,
    reflowCursorLine: random() % 2 === 1, windowsPty: { backend, buildNumber } });
  const host = new MockBufferService(cols, height);
  const buffer = new Buffer(random() % 4 !== 0, options, host, new MockLogService());
  buffer.fillViewportRows();
  for (let step = 0; step < 8; step++) {
    title = `seed 0xb0ff67 trace ${trace} step ${step}`;
    for (let i = 0; i < 6; i++) {
      const y = random() % buffer.lines.length;
      const line = buffer.lines.get(y);
      const x = random() % line.length;
      const chars = strings[random() % strings.length];
      const cell = CellData.fromCharData([random(), chars, 1, 0]);
      cell.bg = random() & 0x07ffffff;
      cell.extended = new ExtendedAttrs(random(), random() % 5);
      line.setCell(x, cell);
      buffer.setWrapped(y, y > 0 && random() % 3 === 0);
    }
    buffer.ybase = Math.max(0, buffer.lines.length - buffer._rows);
    buffer.ydisp = random() % (buffer.ybase + 1);
    buffer.y = random() % Math.min(buffer._rows, buffer.lines.length - buffer.ybase);
    buffer.x = random() % buffer._cols;
    buffer.savedX = random() % 20;
    buffer.savedY = random() % (buffer.lines.length + 1);
    buffer.scrollTop = random() % buffer._rows;
    if (random() % 3 === 0) buffer.addMarker(random() % buffer.lines.length);
    const nextCols = 2 + random() % 15;
    const nextRows = 1 + random() % 6;
    if (random() % 2) { host.cols = nextCols; host.rows = nextRows; }
    buffer.resize(nextCols, nextRows);
  }
  buffer.dispose();
}
delete globalThis.__captureBufferResize;
const fixture = 'crates/xterm-core/tests/fixtures/buffer-resize.tsv';
mkdirSync(resolve(root, 'crates/xterm-core/tests/fixtures'), { recursive: true });
writeFileSync(resolve(root, fixture), rows.join('\n') + '\n');
const sourceHashes = {};
for (const file of Object.keys(built.metafile.inputs).filter(p => p.startsWith('source_repo/src/'))) {
  sourceHashes[file] = createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex');
}
const report = { command: 'node scripts/buffer-resize-oracle.mjs', node: process.version,
  platform: process.platform, overallEquivalence: 'NOT PROVEN', sourceHashes,
  fixture, fixtureSha256: createHash('sha256').update(readFileSync(resolve(root, fixture))).digest('hex'),
  baseline: { discovered: runner.total, passed: runner.stats.passes, failed: failures,
    skipped: runner.stats.pending, executed: runner.stats.tests - runner.stats.pending,
    unrun: runner.total - runner.stats.tests, outcomes },
  resizeCalls: cases.length, originalResizeCalls, seed: '0xb0ff67',
  randomizedTraces: 128, randomizedResizeCalls: cases.length - originalResizeCalls, cases,
  limitations: ['Resize snapshots do not port non-resize assertions; see Rust upstream tests.',
    'Cleanup backing-memory assertions require separate Rust observation.',
    'Snapshot input hydration does not prove preceding operation ordering.',
    'Native Linux/macOS and whole-terminal integration are separate unrun evidence.'],
};
mkdirSync(resolve(root, 'results/buffer'), { recursive: true });
writeFileSync(resolve(root, 'results/buffer/resize-oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(`${cases.length} resize input/output pairs, ${runner.stats.passes}/${runner.total} original tests passing`);
if (failures || runner.stats.pending) process.exitCode = 1;
