// Copyright (c) 2020-2026 The xterm.js authors. Licensed under MIT.
// Runs unchanged ImageStorage with real upstream headless buffers and a drawing recorder.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const temporary = resolve(root, 'tmp/image-storage');
mkdirSync(temporary, { recursive: true });
const require = createRequire(resolve(root, 'source_repo/package.json'));
const { build } = require('esbuild');
for (const [entry, output, tsconfig] of [
  ['src/headless/public/Terminal.ts', 'terminal.cjs', 'src/headless/tsconfig.json'],
  ['addons/addon-image/src/ImageStorage.ts', 'storage.cjs', 'addons/addon-image/src/tsconfig.json']
]) {
  await build({ entryPoints: [resolve(root, 'source_repo', entry)], outfile: resolve(temporary, output),
    absWorkingDir: resolve(root, 'source_repo'), bundle: true, platform: 'node', format: 'cjs',
    tsconfig: resolve(root, 'source_repo', tsconfig) });
}
const { Terminal } = require(resolve(temporary, 'terminal.cjs'));
const { ImageStorage } = require(resolve(temporary, 'storage.cjs'));
const number = value => Object.is(value, -0) ? '-0' : String(value);
const parse = value => typeof value === 'string' ? Number(value) : value;
const layerName = value => value[0].toUpperCase() + value.slice(1);

function fixture(config) {
  const events = [];
  class Canvas {
    constructor(width, height) { this.width = width; this.height = height; }
    getContext() { return { drawImage: () => events.push('bitmap-to-canvas') }; }
  }
  class Bitmap extends Canvas {
    constructor(width, height, label) { super(width, height); this.label = label; }
    close() { events.push(`close:${this.label}`); this.width = 0; this.height = 0; }
  }
  globalThis.ImageBitmap = Bitmap;
  globalThis.window = { ImageBitmap: Bitmap, document: { createElement: () => new Canvas(0, 0) } };
  const terminal = new Terminal({ cols: config.cols, rows: config.rows, scrollback: 5, allowProposedApi: true });
  let size = { width: config.cell[0], height: config.cell[1] };
  const layers = new Set();
  let storage;
  const idOf = spec => [...storage._images].find(([, value]) => value === spec)?.[0] ?? -1;
  const renderer = {
    get cellSize() { return { ...size }; },
    hasLayer: layer => layers.has(layer),
    insertLayerToDom(layer) { events.push(`insert:${layerName(layer)}`); layers.add(layer); },
    removeLayerFromDom(layer) { events.push(`remove:${layerName(layer)}`); layers.delete(layer); },
    rescaleCanvas() { events.push('rescale'); },
    clearAll(layer) { events.push(`clear:${layer === undefined ? 'None' : `Some(${layerName(layer)})`}`); },
    clearLines(start, end) { events.push(`lines:${start}:${end}`); },
    drawPlaceholder(col, row, count) { events.push(`placeholder:${col}:${row}:${count}`); },
    draw(spec, tile, col, row, count) { events.push(`draw:${idOf(spec)}:${tile}:${col}:${row}:${count}`); },
    extractTile(spec, tile) { events.push(`extract:${idOf(spec)}:${tile}`); return new Canvas(1, 1); }
  };
  const options = { storageLimit: 128, showPlaceholder: true };
  storage = new ImageStorage(terminal, renderer, options);
  storage.onImageAdded = () => events.push('added');
  storage.onImageDeleted = id => events.push(`deleted:${id}`);
  terminal.onLineFeed(() => events.push('linefeed'));
  terminal.onResize(metrics => storage.viewportResize(metrics));
  const surface = (width, height, bitmap, label) => bitmap ? new Bitmap(width, height, label) : new Canvas(width, height);
  function snapshot() {
    const buffer = terminal._core.buffer;
    const cells = [];
    for (let row = 0; row < buffer.lines.length; row++) {
      const line = buffer.lines.get(row);
      const values = [];
      for (let col = 0; col < line.length; col++) {
        const cell = line.getCell(col);
        const payload = cell.extended.payload;
        values.push([cell.content >>> 0, cell.fg >>> 0, cell.bg >>> 0, cell.extended.ext >>> 0,
          cell.extended.urlId, payload ? [payload.imageId, number(payload.tileId)] : null]);
      }
      cells.push(values);
    }
    const image = value => value ? [value.width, value.height, value instanceof Bitmap] : null;
    return { cursor: [buffer.x, buffer.y, buffer.ybase, buffer.ydisp], dimensions: [terminal.cols, terminal.rows],
      buffer: terminal.buffer.active.type, marker_count: buffer.markers.length,
      limit: number(storage.getLimit()), usage: number(storage.getUsage()),
      images: [...storage._images].map(([id, spec]) => ({ id, original: image(spec.orig), actual: image(spec.actual),
        same: spec.actual === spec.orig, cell: [number(spec.origCellSize.width), number(spec.origCellSize.height)],
        actual_cell: [number(spec.actualCellSize.width), number(spec.actualCellSize.height)],
        count: number(spec.tileCount), buffer: spec.bufferType, layer: spec.layer, z: number(spec.zIndex),
        marker: spec.marker ? spec.marker.isDisposed : null })), cells, events: events.splice(0) };
  }
  function execute(step) {
    const buffer = terminal._core.buffer;
    switch (step.op) {
      case 'cursor': buffer.x = step.x; buffer.y = step.y; return null;
      case 'add': return storage.addImage(surface(step.w, step.h, step.bitmap, step.label), {
        scrolling: step.scrolling ?? true, layer: step.layer ?? 'top', zIndex: parse(step.z ?? 0), cursorPos: step.cursor ?? 'vt340' });
      case 'limit': storage.setLimit(parse(step.value)); return null;
      case 'delete': storage.deleteImage(step.id); return null;
      case 'reset': storage.reset(); return null;
      case 'render': storage.render({ start: step.start, end: step.end }); return null;
      case 'resize': terminal.resize(step.cols, step.rows); return null;
      case 'wipe': storage.wipeAlternate(); return null;
      case 'alt': terminal._core._bufferService.buffers.activateAltBuffer(); return null;
      case 'normal': terminal._core._bufferService.buffers.activateNormalBuffer(); return null;
      case 'placeholder': options.showPlaceholder = step.value; return null;
      case 'metrics': size = { width: step.cell[0], height: step.cell[1] }; return null;
      case 'actual': {
        const spec = storage._images.get(step.id);
        spec.actual = step.none ? undefined : surface(step.w, step.h, false);
        return null;
      }
      case 'seed': {
        const line = buffer.lines.get(step.y);
        line.setCellFromCodepoint(step.x, step.code, 1, { fg: step.fg ?? 0, bg: step.bg ?? 0, extended: {} });
        return null;
      }
      case 'get': case 'extract': {
        const result = step.op === 'get' ? storage.getImageAtBufferCell(step.x, step.y) : storage.extractTileAtBufferCell(step.x, step.y);
        return result ? [result.width, result.height, result instanceof Bitmap] : null;
      }
      default: throw Error(`Unknown oracle operation ${step.op}`);
    }
  }
  return { execute, snapshot, dispose: () => terminal.dispose() };
}

const cases = [
  { name: 'tiles-and-iip-cursor', config: { cols: 6, rows: 3, cell: [2, 2] }, steps: [
    { op: 'seed', x: 1, y: 1, code: 65, fg: 66051, bg: 263430 }, { op: 'cursor', x: 1, y: 1 },
    { op: 'add', w: 5, h: 3, cursor: 'iip' }, { op: 'get', x: 1, y: 1 }, { op: 'extract', x: 2, y: 2 }, { op: 'render', start: 0, end: 2 }] },
  { name: 'non-scrolling-and-overwrite', config: { cols: 4, rows: 2, cell: [2, 2] }, steps: [
    { op: 'cursor', x: 3, y: 1 }, { op: 'add', w: 12, h: 8, scrolling: false }, { op: 'cursor', x: 0, y: 0 },
    { op: 'add', w: 2, h: 2 }, { op: 'add', w: 2, h: 2 }, { op: 'render', start: 0, end: 1 }] },
  { name: 'resized-accounting-and-fifo', config: { cols: 8, rows: 2, cell: [1000, 1000] }, steps: [
    { op: 'limit', value: 0.5 }, { op: 'add', w: 200, h: 200 }, { op: 'cursor', x: 1, y: 0 },
    { op: 'add', w: 200, h: 200 }, { op: 'cursor', x: 2, y: 0 }, { op: 'add', w: 200, h: 200 },
    { op: 'cursor', x: 3, y: 0 }, { op: 'add', w: 100, h: 100 }, { op: 'actual', id: 3, w: 400, h: 400 },
    { op: 'limit', value: 0.5 }, { op: 'limit', value: 'NaN' }] },
  { name: 'bitmap-ownership-and-empty-clear', config: { cols: 6, rows: 3, cell: [1, 1] }, steps: [
    { op: 'add', w: 2, h: 1, bitmap: true, label: 'bitmap' }, { op: 'get', x: 0, y: 0 },
    { op: 'render', start: 0, end: 1 }, { op: 'delete', id: 1 }, { op: 'delete', id: 1 },
    { op: 'render', start: 0, end: 1 }, { op: 'render', start: 0, end: 1 }] },
  { name: 'width-growth-quirk', config: { cols: 3, rows: 2, cell: [1, 1] }, steps: [
    { op: 'add', w: 6, h: 1 }, { op: 'resize', cols: 6, rows: 2 }, { op: 'seed', x: 4, y: 0, code: 88 },
    { op: 'resize', cols: 8, rows: 2 }, { op: 'render', start: 0, end: 1 }, { op: 'resize', cols: 4, rows: 2 }] },
  { name: 'alternate-and-normal', config: { cols: 6, rows: 3, cell: [1, 1] }, steps: [
    { op: 'add', w: 1, h: 1 }, { op: 'alt' }, { op: 'add', w: 2, h: 1 }, { op: 'cursor', x: 3, y: 0 },
    { op: 'add', w: 1, h: 1 }, { op: 'wipe' }, { op: 'render', start: 0, end: 1 }, { op: 'normal' },
    { op: 'render', start: 0, end: 1 }, { op: 'reset' }] },
  { name: 'layer-z-order-and-placeholder', config: { cols: 8, rows: 3, cell: [1, 1] }, steps: [
    { op: 'add', w: 2, h: 1, z: 10 }, { op: 'cursor', x: 3, y: 0 },
    { op: 'add', w: 2, h: 1, z: -4, layer: 'bottom' }, { op: 'render', start: 0, end: 1 },
    { op: 'delete', id: 1 }, { op: 'render', start: 0, end: 1 }, { op: 'placeholder', value: false },
    { op: 'render', start: 0, end: 1 }, { op: 'delete', id: 2 }, { op: 'render', start: 0, end: 1 }] },
  { name: 'fallback-empty-and-limits', config: { cols: 6, rows: 3, cell: [-1, 8] }, steps: [
    { op: 'add', w: 8, h: 15 }, { op: 'add', w: 0, h: 0 }, { op: 'add', w: 1, h: 1 },
    { op: 'limit', value: 0.499 }, { op: 'limit', value: 'Infinity' }, { op: 'limit', value: 1000 }, { op: 'reset' }] }
];
let seed = 0x1a6e126;
const random = () => { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; };
const randomSteps = [];
for (let index = 0; index < 500; index++) {
  switch (random() % 8) {
    case 0: randomSteps.push({ op: 'cursor', x: random() % 6, y: random() % 3 }); break;
    case 1: case 2: randomSteps.push({ op: 'add', w: random() % 8, h: random() % 5,
      bitmap: !!(random() % 2), label: 'random', scrolling: !!(random() % 2), cursor: random() % 2 ? 'iip' : 'vt340',
      layer: random() % 2 ? 'top' : 'bottom', z: (random() % 9) - 4 }); break;
    case 3: randomSteps.push({ op: 'delete', id: random() % (index + 1) }); break;
    case 4: randomSteps.push({ op: 'render', start: 0, end: 2 }); break;
    case 5: randomSteps.push({ op: 'reset' }); break;
    case 6: randomSteps.push({ op: 'get', x: random() % 8, y: random() % 8 }); break;
    case 7: randomSteps.push({ op: 'seed', x: random() % 6, y: random() % 3, code: 97 + random() % 26 }); break;
  }
}
cases.push({ name: 'seeded-storage', config: { cols: 6, rows: 3, cell: [2, 2] }, steps: randomSteps });
let count = 0;
for (const test of cases) {
  const state = fixture(test.config);
  for (const step of test.steps) {
    try { step.result = state.execute(step); }
    catch (error) { step.error = `${error.name}:${error.message}`; }
    step.expected = state.snapshot();
    count++;
  }
  state.dispose();
}
const output = JSON.stringify({ seed: '0x1a6e126', scope: 'ImageStorage with actual source/Rust buffers and a drawing recorder, not browser pixels', cases }) + '\n';
const file = resolve(root, 'crates/xterm-addons/tests/fixtures/image-storage.json');
if (process.argv.includes('--generate')) writeFileSync(file, output);
else assert.equal(readFileSync(file, 'utf8'), output, 'ImageStorage fixture drift');
console.log(`ImageStorage source: ${cases.length} cases, ${count} snapshots; fixture ${process.argv.includes('--generate') ? 'generated' : 'verified'}.`);
