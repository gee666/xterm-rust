// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Uses generated reference modules and runs the unchanged original assertions.
import assert from 'node:assert/strict';
import { createRequire, Module } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { runCommand } from './quality-process.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const source = path.join(root, 'source_repo');
const directory = path.join(root, 'results/browser-mouse');
const fixtureDirectory = path.join(root, 'crates/xterm-web/tests/fixtures');
mkdirSync(directory, { recursive: true });
mkdirSync(fixtureDirectory, { recursive: true });
const build = await runCommand(process.execPath, ['bin/esbuild_all.mjs'], {
  cwd: source, log: path.join(directory, 'source-build.log'), timeoutMs: 120000,
});
if (build.status !== 'passed') throw new Error('Reference build failed; inspect source-build.log');
process.env.NODE_PATH = path.join(source, 'out-esbuild');
Module._initPaths();
const require = createRequire(path.join(source, 'package.json'));
const { MouseService } = require('./out-esbuild/browser/services/MouseService.js');
const { MouseStateService } = require('./out-esbuild/common/services/MouseStateService.js');
const { getCoords } = require('./out-esbuild/browser/input/Mouse.js');
const { MouseCoordsService } = require('./out-esbuild/browser/services/MouseCoordsService.js');
const Mocha = require('mocha');
const hex = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',') || '-';
const number = value => Object.is(value, -0) ? '-0' : String(value);
const flag = value => value === undefined ? 'u' : value ? '1' : '0';
const eventFields = event => ['col', 'row', 'x', 'y', 'button', 'action'].map(key => number(event[key]))
  .concat(['ctrl', 'alt', 'shift'].map(key => flag(event[key])));
const triggers = [];
let activeName = '';
const originalTrigger = MouseService.prototype._triggerMouseEvent;
MouseService.prototype._triggerMouseEvent = function(event) {
  const before = eventFields(event);
  const reports = [];
  const core = this._coreService;
  const data = core.triggerDataEvent;
  const binary = core.triggerBinaryEvent;
  core.triggerDataEvent = function(payload, user) {
    assert.equal(user, true, 'mouse reports must retain wasUserInput');
    reports.push(['data', payload, user]); return data.call(this, payload, user);
  };
  core.triggerBinaryEvent = function(payload) { reports.push(['binary', payload]); return binary.call(this, payload); };
  try {
    const accepted = originalTrigger.call(this, event);
    if (reports.length > 1) throw new Error('one trigger unexpectedly emitted multiple reports');
    triggers.push([this._mouseStateService.activeProtocol, this._mouseStateService.activeEncoding,
      number(this._bufferService.cols), number(this._bufferService.rows), ...before, +accepted,
      ...eventFields(event), reports[0]?.[0] || '-', reports.length ? hex(reports[0][1]) : '-', activeName].join('\t'));
    return accepted;
  } finally {
    core.triggerDataEvent = data;
    core.triggerBinaryEvent = binary;
  }
};
const mocha = new Mocha({ reporter: 'json', reporterOptions: { output: path.join(directory, 'upstream.json') } });
mocha.addFile(path.join(source, 'out-esbuild/browser/input/Mouse.test.js'));
mocha.addFile(path.join(source, 'out-esbuild/browser/services/MouseService.test.js'));
let failures;
try {
  failures = await new Promise(resolve => {
    const runner = mocha.run(resolve);
    runner.on('test', test => { activeName = test.fullTitle(); });
  });
} finally { MouseService.prototype._triggerMouseEvent = originalTrigger; }
if (failures) throw new Error(`Original mouse assertions failed: ${failures}`);

const seed = 0xb0a5e;
let randomState = seed;
function random() {
  randomState ^= randomState << 13; randomState ^= randomState >>> 17; randomState ^= randomState << 5;
  return randomState >>> 0;
}
const values = [-Infinity, -500, -1, -0, 0, 0.5, 1, 10, 20, 223, 500, Infinity, NaN];
const pads = ['0px', '12.5px', '-3.8px', '+2px', '', 'auto', '\ufeff 8px', '\u00858px', '-0px'];
const coords = [];
for (let i = 0; i < 1024; i++) {
  const input = {
    left: values[random() % values.length], top: values[random() % values.length],
    padLeft: pads[random() % pads.length], padTop: pads[random() % pads.length],
    width: values[random() % values.length], height: values[random() % values.length],
    canvasWidth: values[random() % values.length], canvasHeight: values[random() % values.length],
    clientX: values[random() % values.length], clientY: values[random() % values.length],
    cols: values[random() % values.length], rows: values[random() % values.length],
    valid: !!(random() & 1), selection: !!(random() & 1),
  };
  const window = { getComputedStyle: () => ({ getPropertyValue: key => key === 'padding-left' ? input.padLeft : input.padTop }) };
  const element = { getBoundingClientRect: () => ({ left: input.left, top: input.top }), ownerDocument: { defaultView: window } };
  const event = { clientX: input.clientX, clientY: input.clientY };
  const cell = getCoords(window, event, element, input.cols, input.rows, input.valid, input.width, input.height, input.selection);
  const service = new MouseCoordsService({ hasValidSize: input.valid }, {
    dimensions: { css: { cell: { width: input.width, height: input.height }, canvas: { width: input.canvasWidth, height: input.canvasHeight } } }
  });
  const report = service.getMouseReportCoords(event, element);
  coords.push([+input.valid, number(input.left), number(input.top), hex(input.padLeft), hex(input.padTop),
    ...['width', 'height', 'canvasWidth', 'canvasHeight', 'clientX', 'clientY', 'cols', 'rows'].map(key => number(input[key])),
    +input.selection, ...(cell ? cell.map(number) : ['u', 'u']),
    ...['col', 'row', 'x', 'y'].map(key => report ? number(report[key]) : 'u')].join('\t'));
}

// The same source object retains wheel/touch fractions between all operations.
const render = { dimensions: { css: { cell: { height: 20 } } } };
const core = { triggerDataEvent() {}, triggerBinaryEvent() {}, decPrivateModes: { applicationCursorKeys: false } };
const buffer = { rows: 24, cols: 80, buffer: { hasScrollback: true } };
const options = { rawOptions: { scrollSensitivity: 1, fastScrollSensitivity: 5 } };
const service = new MouseService(render, {}, new MouseStateService(), core, buffer, options, {}, {}, {});
const wheels = [];
function wheel({deltaY, deltaMode, flags, cellHeight, dpr, scroll, fast, rows}) {
  options.rawOptions.scrollSensitivity = scroll;
  options.rawOptions.fastScrollSensitivity = fast;
  buffer.rows = rows;
  const event = { deltaY, deltaMode, ctrlKey: !!(flags & 1), altKey: !!(flags & 2), shiftKey: !!(flags & 4) };
  globalThis.WheelEvent = { DOM_DELTA_PIXEL: 0, DOM_DELTA_PAGE: 2 };
  const output = service._consumeWheelEvent(event, cellHeight, dpr);
  wheels.push([number(deltaY), deltaMode, flags, cellHeight === undefined ? 'u' : number(cellHeight),
    dpr === undefined ? 'u' : number(dpr), number(scroll), number(fast), number(rows),
    number(output), number(service._wheelPartialScroll)].join('\t'));
}
for (let i = 0; i < 2048; i++) {
  if (i % 137 === 0) { service.reset(); wheels.push('reset'); }
  wheel({ deltaY: values[random() % values.length], deltaMode: random() % 4, flags: random() & 7,
    cellHeight: random() % 7 ? values[random() % values.length] : undefined,
    dpr: random() % 7 ? values[random() % values.length] : undefined,
    scroll: values[random() % values.length], fast: values[random() % values.length], rows: values[random() % values.length] });
}
for (const deltaY of [-50, -49.9, -1, 0, 1, 49.9, 50]) for (let deltaMode = 0; deltaMode < 4; deltaMode++) {
  for (let flags = 0; flags < 8; flags++) {
    service.reset(); wheels.push('reset');
    wheel({ deltaY, deltaMode, flags, cellHeight: 20, dpr: 2, scroll: 1, fast: 5, rows: 24 });
  }
}
service.reset(); wheels.push('reset');
for (const deltaY of [1, 1, 1, 1, 1, -1, -1, -49.9, 49.9, 50]) {
  wheel({ deltaY, deltaMode: 0, flags: 0, cellHeight: 20, dpr: 2, scroll: 1, fast: 5, rows: 24 });
}
for (const cellHeight of [-0, 0, 1, -1, Infinity, -Infinity, NaN, undefined]) {
  for (const dpr of [-0, 0, 1, -1, Infinity, -Infinity, NaN, undefined]) {
    for (const deltaY of [1, -1, 50, -50]) {
      service.reset(); wheels.push('reset');
      wheel({ deltaY, deltaMode: 0, flags: 0, cellHeight, dpr, scroll: 1, fast: 5, rows: 24 });
    }
  }
}
const touches = [];
let outputs = [];
core.triggerDataEvent = (payload, user) => {
  if (user !== true) throw new Error('touch output lost wasUserInput');
  outputs.push(hex(payload));
};
const heights = [undefined, 0, NaN, -20, 1, 20, Infinity];
for (let i = 0; i < 512; i++) {
  if (i % 137 === 0) { service._handleTouchStart(); touches.push('reset'); }
  const height = heights[random() % heights.length];
  const translationY = (random() % 801 - 400) / 10;
  const application = !!(random() & 1);
  render.dimensions.css.cell.height = height;
  core.decPrivateModes.applicationCursorKeys = application;
  outputs = [];
  service._handleTouchScrollAsKeys({ translationY });
  touches.push([height === undefined ? 'u' : number(height), number(translationY), +application,
    number(service._touchScrollAccumulator), outputs.join(';') || '-'].join('\t'));
}
// A mutable custom encoder must affect the cached event before the next move.
const customState = new MouseStateService();
customState.activeProtocol = 'ANY';
customState.addEncoding('CUSTOM', event => { event.col = 9; event.shift = true; return 'x'; });
customState.activeEncoding = 'CUSTOM';
const customReports = [];
const custom = new MouseService({}, {}, customState, {
  triggerDataEvent: (text, user) => customReports.push([text, user])
}, { cols: 500, rows: 500 }, { rawOptions: {} }, {}, {}, {});
const customEvent = { col: 0, row: 0, x: 0, y: 0, button: 3, action: 32 };
assert.equal(custom._triggerMouseEvent(customEvent), true);
assert.equal(customEvent.col, 9);
assert.equal(customEvent.shift, true);
assert.equal(custom._triggerMouseEvent({ ...customEvent, col: 8, row: 0 }), false);
assert.deepEqual(customReports, [['x', true]]);
// Compound assignment captures the accumulator before reading translationY.
render.dimensions.css.cell.height = 20;
service._touchScrollAccumulator = 5;
const resetDuringRead = { get translationY() { service._handleTouchStart(); return 2; } };
service._handleTouchScrollAsKeys(resetDuringRead);
assert.equal(service._touchScrollAccumulator, 3);
const translationError = new Error('original translation getter failure');
service._touchScrollAccumulator = 5;
assert.throws(() => service._handleTouchScrollAsKeys({ get translationY() {
  service._handleTouchStart(); throw translationError;
} }), error => error === translationError);
assert.equal(service._touchScrollAccumulator, 0);
const raw = JSON.parse(readFileSync(path.join(directory, 'upstream.json'), 'utf8'));
if (raw.stats.tests !== 15 || raw.stats.passes !== 15) throw new Error('Upstream mouse inventory changed');
const files = { 'browser-mouse-trigger.tsv': triggers, 'browser-mouse-coords.tsv': coords,
  'browser-mouse-wheel.tsv': wheels, 'browser-mouse-touch.tsv': touches };
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const fixtures = {};
for (const [name, rows] of Object.entries(files)) {
  const content = rows.join('\n') + '\n';
  const file = path.join(fixtureDirectory, name);
  if (process.argv.includes('--generate')) writeFileSync(file, content);
  else if (readFileSync(file, 'utf8').replaceAll('\r\n', '\n') !== content) throw new Error(`Source fixture drift: ${name}`);
  fixtures[name] = { rows: rows.length, canonicalSha256: hash(content), fileSha256: hash(readFileSync(file)) };
}
const sourceFiles = ['browser/input/Mouse.ts', 'browser/input/Mouse.test.ts', 'browser/services/MouseCoordsService.ts',
  'browser/services/MouseService.ts', 'browser/services/MouseService.test.ts', 'common/services/MouseStateService.ts'];
writeFileSync(path.join(directory, 'oracle.json'), JSON.stringify({
  node: process.version, platform: process.platform, seed, fixtures, directedCustomEncoderCases: 1, directedTouchReentryCases: 2,
  sourceHashes: Object.fromEntries(sourceFiles.map(file => [file, hash(readFileSync(path.join(source, 'src', file)))])),
  upstream: { discovered: raw.stats.tests, executed: raw.stats.tests, passed: raw.stats.passes, failed: raw.stats.failures, skipped: 0, unrun: 0 },
  names: raw.tests.map(test => test.fullTitle),
  scope: 'Original 15 tests and pure policy corpus; Rust DOM/three-browser proof remains separate.'
}, null, 2) + '\n');
console.log(JSON.stringify({ originalTests: raw.stats.tests, fixtures, seed }));
