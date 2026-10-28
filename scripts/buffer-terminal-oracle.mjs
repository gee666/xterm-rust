// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Real parser/buffer/addon cooperation, not hydrated input snapshots.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { hex, snapshot } from './terminal-snapshot.mjs';
const root = resolve(import.meta.dirname, '..');
const require = createRequire(resolve(root, 'source_repo/package.json'));
const { build } = require('esbuild');
const output = resolve(root, 'results/buffer-terminal');
mkdirSync(output, { recursive: true });
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify({ status: 'incomplete', overallEquivalence: 'NOT PROVEN' }) + '\n');
const temporary = resolve(root, 'tmp/buffer-terminal');
mkdirSync(temporary, { recursive: true });
const bundle = resolve(temporary, 'reference.cjs');
const built = await build({
  absWorkingDir: root, stdin: { resolveDir: root, loader: 'ts', contents: `
export { Terminal } from './source_repo/src/headless/public/Terminal';
export { SerializeAddon } from './source_repo/addons/addon-serialize/src/SerializeAddon';
export { SearchEngine } from './source_repo/addons/addon-search/src/SearchEngine';
export { SearchLineCache } from './source_repo/addons/addon-search/src/SearchLineCache';` },
  outfile: bundle, bundle: true, platform: 'node', format: 'cjs', target: 'node24',
  tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true, logLevel: 'silent',
  alias: { common: resolve(root, 'source_repo/src/common'), browser: resolve(root, 'source_repo/src/browser') },
});
const sourceHashes = Object.fromEntries(Object.keys(built.metafile.inputs).filter(path => path.startsWith('source_repo/')).sort().map(path =>
  [path, createHash('sha256').update(readFileSync(resolve(root, path))).digest('hex')]));
const { Terminal, SerializeAddon, SearchEngine, SearchLineCache } = require(bundle);
const queries = ['abc', '中', 'next', 'line', 'e\u0301'];
const traces = [
  { id: 'scroll-boundaries', cols: 6, rows: 3, scrollback: 4, ops: [
    ['S', -100], ['P', -1], ['T', ''], ['B', ''], ['L', -1], ['L', 100],
    ['W', '0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7'], ['M', -100], ['M', 100], ['A', -1], ['D', ''],
    ['L', 9007199254740992], ['L', -9007199254740992], ['T', ''], ['S', 1], ['P', 1], ['L', 2], ['B', ''], ['S', 100],
    ['W', '\x1b[?1049hALT'], ['M', 0], ['P', -100], ['T', ''], ['B', ''],
    ['L', -100], ['L', 100], ['C', ''], ['W', '\x1b[?1049l'], ['Q', ''], ['S', -100], ['C', '']] },
  { id: 'wrapped-styled-clear', cols: 6, rows: 4, scrollback: 3, ops: [
    ['W', 'old0\r\nold1\r\nold2\r\n'], ['W', '\x1b[35mA中e\u0301B中\x1b[0m'],
    ['M', -1], ['S', -100], ['W', '\x1b[2;2H'], ['C', ''], ['W', '\r\nnext']] },
  { id: 'styled-clear', cols: 20, rows: 3, scrollback: 4, ops: [
    ['W', 'old0\r\nold1\r\nold2\r\nold3\r\n'], ['M', -1],
    ['W', '\x1b[32mprompt> \x1b[0minput'], ['M', 0], ['S', -2], ['C', ''],
    ['S', 100], ['W', '\r\nnext']] },
  { id: 'reflow-markers', cols: 8, rows: 4, scrollback: 6, ops: [
    ['W', 'abcdefghijklmnop\r\nnext\r\n'], ['M', -2], ['M', 0], ['R', [4, 4]],
    ['R', [10, 3]], ['S', -2], ['W', '\x1b[6n']] },
  { id: 'overflow', cols: 6, rows: 3, scrollback: 2, ops: [
    ['W', 'line0\r\n'], ['M', -1], ['W', 'line1\r\nline2\r\nline3\r\nline4\r\n'],
    ['M', -1], ['R', [3, 2]], ['W', 'line5\r\nline6\r\n'], ['R', [8, 4]]] },
  { id: 'alternate-lifetime', cols: 8, rows: 3, scrollback: 4, ops: [
    ['W', 'normal'], ['M', 0], ['W', '\x1b[?1049hALT'], ['M', 0],
    ['R', [5, 4]], ['W', '\x1b[?1049l'], ['W', '\x1b[?47hX\x1b[?47l']] },
  { id: 'wide-combined', cols: 6, rows: 4, scrollback: 4, ops: [
    ['W', '12345中e\u0301🙂'], ['M', 0], ['W', '\r\nnext'], ['R', [5, 3]],
    ['W', '\x1b[31;4:3mred\x1b[0m'], ['R', [9, 4]]] },
  { id: 'saved-cursor', cols: 8, rows: 4, scrollback: 4, ops: [
    ['W', '\x1b[2;2H\x1b7abc'], ['M', 0], ['R', [4, 2]], ['W', '\x1b8Z'],
    ['W', '\x1b[?6h\x1b[2;3r\x1b[Hnext\nline\nabc'], ['R', [7, 4]]] },
];
traces.push({ id: 'original-five-row-clear', cols: 80, rows: 5, scrollback: 1000,
  ops: [...Array.from({ length: 10 }, (_, i) => ['W', '\n\rtest' + i]),
    ['M', 1], ['M', 2], ['M', 3], ['M', 4], ['C', '']] });
const seed = 0xbaff0099;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const chunks = ['abc', '中', 'e\u0301', '🙂', '\r\nnext', '\r\nline', '\x1b[31mred\x1b[0m',
  '\x1b[2;2H@', '\x1b[K', '\x1b[?1049hALT', '\x1b[?1049l', '\x1b7save\x1b8', '\x1b[2J'];
for (let id = 0; id < 64; id++) {
  const ops = [];
  for (let step = 0; step < 12; step++) {
    const kind = random() % 10;
    if (kind === 0) ops.push(['R', [2 + random() % 10, 1 + random() % 5]]);
    else if (kind === 1) ops.push(['S', random() % 7 - 3]);
    else if (kind === 2) ops.push(['M', 0]);
    else ops.push(['W', chunks[random() % chunks.length]]);
  }
  traces.push({ id: `seed-${id}`, cols: 2 + random() % 10, rows: 1 + random() % 5, scrollback: random() % 8, ops });
}
function observe(terminal, events) {
  terminal.onBell(() => events.push('B'));
  terminal.onCursorMove(() => events.push('C'));
  terminal.onLineFeed(() => events.push('L'));
  terminal.onScroll(y => events.push(`S${y}`));
  terminal.onData(text => events.push('D' + hex(text)));
  terminal.onTitleChange(text => events.push('T' + hex(text)));
  terminal._core.onRender(e => events.push(`R${e.start}:${e.end}`));
}
const records = [];
let markersCreated = 0;
const searchErrors = [];
for (const trace of traces) {
  const terminal = new Terminal({ cols: trace.cols, rows: trace.rows, scrollback: trace.scrollback, allowProposedApi: true });
  const addon = new SerializeAddon(); terminal.loadAddon(addon);
  const markers = [], events = [], markerEvents = [];
  observe(terminal, events);
  for (let step = 0; step < trace.ops.length; step++) {
    const [kind, arg] = trace.ops[step];
    events.length = 0; markerEvents.length = 0;
    let markerResult = '-';
    const activeBefore = terminal._core.buffer;
    const clearLine = kind === 'C' ? activeBefore.lines.get(activeBefore.ybase + activeBefore.y) : undefined;
    if (kind === 'W') await new Promise(done => terminal.write(arg, done));
    else if (kind === 'R') terminal.resize(...arg);
    else if (kind === 'S') terminal.scrollLines(arg);
    else if (kind === 'P') terminal.scrollPages(arg);
    else if (kind === 'T') terminal.scrollToTop();
    else if (kind === 'B') terminal.scrollToBottom();
    else if (kind === 'L') terminal.scrollToLine(arg);
    else if (kind === 'C') terminal.clear();
    else if (kind === 'Q') terminal.reset();
    else if (kind === 'M' || kind === 'A' || kind === 'D') {
      const marker = kind === 'A' ? terminal.addMarker(arg) : kind === 'D' ? terminal.registerMarker() : terminal.registerMarker(arg);
      if (marker) {
        const ordinal = markers.length;
        assert(!markers.some(previous => previous.id === marker.id), 'marker ID reused');
        markers.push(marker); markersCreated++;
        marker.onDispose(() => markerEvents.push(ordinal));
        markerResult = String(ordinal);
      }
    }
    const cache = new SearchLineCache(terminal);
    const engine = new SearchEngine(terminal, cache);
    const found = queries.map(query => {
      try {
        const result = engine.find(query, 0, 0);
        return result ? [result.row, result.col, result.size, hex(result.term)].join(':') : '-';
      } catch (error) {
        searchErrors.push({ trace: trace.id, step, query, name: error.name, message: error.message });
        return '!' + hex(error.message);
      }
    }).join('/');
    cache.dispose();
    if (trace.id === 'original-five-row-clear' && kind === 'C') {
      // Preserve original headless clear and marker-disposal assertions.
      assert.equal(terminal.buffer.active.length, 5);
      assert.equal(terminal.buffer.active.getLine(0).translateToString(true), 'test9');
      for (let row = 1; row < 5; row++) assert.equal(terminal.buffer.active.getLine(row).translateToString(true), '');
      assert.equal(markerEvents.length, markers.length);
      assert(markers.every(marker => marker.isDisposed));
      assert.equal(terminal.markers.length, 0);
    }
    const serialized = addon.serialize();
    let restored = '-', restoredEvents = '-';
    if (step === trace.ops.length - 1) {
      const copy = new Terminal({ cols: terminal.cols, rows: terminal.rows, scrollback: trace.scrollback, allowProposedApi: true });
      const copyEvents = [];
      observe(copy, copyEvents);
      await new Promise(done => copy.write(serialized, done));
      restored = snapshot(copy);
      restoredEvents = copyEvents.join('/');
      copy.dispose();
    }
    const normal = terminal._core.buffers.normal;
    const alternate = terminal._core.buffers.alt;
    const saved = [normal.savedX, normal.savedY, alternate.savedX, alternate.savedY].join(',');
    const markerState = markers.map(marker => `${marker.line},${+marker.isDisposed}`).join('|') || '-';
    const publicMarkers = terminal.markers.map(marker => markers.indexOf(marker)).join(',') || '-';
    const clearIdentity = kind === 'C' ? +(clearLine === terminal._core.buffer.lines.get(0)) : '-';
    records.push([trace.id, trace.cols, trace.rows, trace.scrollback, step, kind,
      kind === 'W' ? hex(arg) : Array.isArray(arg) ? arg.join(',') : arg,
      snapshot(terminal), saved, markerResult, markerState, markerEvents.join(',') || '-',
      events.join('/'), hex(serialized), found, restored, publicMarkers, restoredEvents, clearIdentity].join('\t'));
  }
  terminal.dispose();
  assert(markers.every(marker => marker.isDisposed), 'terminal disposal leaves live marker');
}
const fixture = 'crates/xterm-core/tests/fixtures/buffer-terminal.tsv';
const content = records.join('\n') + '\n';
if (process.argv.includes('--generate')) writeFileSync(resolve(root, fixture), content);
else assert.equal(readFileSync(resolve(root, fixture), 'utf8'), content, 'buffer terminal fixture drift');
for (const [file, expected] of Object.entries(sourceHashes)) {
  assert.equal(createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex'), expected);
}
const report = { status: 'passed', overallEquivalence: 'NOT PROVEN',
  command: `node scripts/buffer-terminal-oracle.mjs${process.argv.includes('--generate') ? ' --generate' : ''}`,
  node: process.version, platform: process.platform, sourceHashes, seed: `0x${seed.toString(16)}`,
  traces: traces.length, snapshots: records.length, markersCreated, searches: records.length * queries.length, searchErrors,
  restoreComparisons: traces.length, fixture, fixtureSha256: createHash('sha256').update(content).digest('hex'),
  scope: 'Actual headless parser writes, both buffers/all cells/modes/events, saved cursors, marker reflow/disposal, live SerializeAddon and SearchEngine queries; source restore state is the oracle, not assumed losslessness.',
  limitations: ['SearchAddon DOM/selection lifecycle is separate task59.', 'Native Linux/macOS and browser runtime are not supplied by this JS oracle.'],
};
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ traces: traces.length, snapshots: records.length, markersCreated }));
