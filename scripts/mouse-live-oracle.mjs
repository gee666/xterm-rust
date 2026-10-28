#!/usr/bin/env node
// Copyright (c) 2017-2026 The xterm.js authors. MIT license.
// Executes unchanged source methods with read/effect observers, without a build.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root = fileURLToPath(new URL('../', import.meta.url));
const read = name => readFileSync(path.join(root, name), 'utf8');
const files = ['source_repo/src/browser/services/MouseService.ts', 'source_repo/src/browser/services/MouseCoordsService.ts',
  'source_repo/src/browser/input/Mouse.ts', 'source_repo/src/browser/Dom.ts', 'source_repo/src/common/Types.ts',
  'source_repo/src/common/data/EscapeSequences.ts'];
function erase(text) {
  return stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, '')
    .replace(/@(IRenderService|ICharSizeService|IMouseCoordsService|IMouseStateService|ICoreService|IBufferService|IOptionsService|ISelectionService|ILogService|ICoreBrowserService)\s+/g, ''), { mode: 'transform' });
}
const context = vm.createContext({ WheelEvent: { DOM_DELTA_PIXEL: 0, DOM_DELTA_PAGE: 2 } });
const types = read(files[4]);
for (const name of ['CoreMouseAction', 'CoreMouseButton', 'CoreMouseEventType']) {
  const declaration = types.match(new RegExp('export const enum ' + name + '\\s*\\{[^}]*\\}'))?.[0];
  assert.ok(declaration, `${name} definition missing`);
  vm.runInContext(erase(declaration), context);
}
vm.runInContext(erase(read('source_repo/src/common/data/EscapeSequences.ts')), context);
const dom = erase(read(files[3]));
const windowStart = dom.indexOf('function getWindow(');
const windowEnd = dom.indexOf('\nclass DomListener', windowStart);
assert.ok(windowStart >= 0 && windowEnd > windowStart, 'getWindow source boundaries changed');
vm.runInContext(dom.slice(windowStart, windowEnd), context);
for (const name of [files[2], files[1], files[0]]) vm.runInContext(erase(read(name)), context);
const MouseService = vm.runInContext('MouseService', context);
const MouseCoordsService = vm.runInContext('MouseCoordsService', context);
const specifications = [
  { name: 'wheel', type: 'wheel' },
  { name: 'zero', type: 'wheel', delta: 0 },
  { name: 'veto', type: 'wheel', veto: true },
  { name: 'override', type: 'wheel', override: 'unknown' },
  { name: 'override-number', type: 'wheel', override: 1 },
  { name: 'mutate', type: 'wheel', mutate: true },
  { name: 'shift', type: 'wheel', shift: true },
  { name: 'pixel', type: 'wheel', delta: 30, mode: 0 },
  { name: 'fallback', type: 'mousemove', missingButtons: true },
  { name: 'invalid-size', type: 'wheel', invalid: true },
];
const rows = [];
for (const spec of specifications) {
  const trace = [], data = [];
  const observed = (object, property, key, value) => Object.defineProperty(object, property, {
    get() { trace.push('host:' + key); return value; }
  });
  const canvas = { height: 432 }; observed(canvas, 'width', 'canvas', 720);
  const cell = { height: 18 }; observed(cell, 'width', 'cell', 9);
  const deviceCell = {}; observed(deviceCell, 'height', 'device-height', 18);
  const render = { dimensions: { css: { canvas, cell }, device: { cell: deviceCell } } };
  const character = {}; observed(character, 'hasValidSize', 'valid', !spec.invalid);
  const window = { getComputedStyle: () => ({ getPropertyValue: () => '0px' }) };
  const screen = { ownerDocument: { defaultView: window }, getBoundingClientRect: () => ({ left: 0, top: 0 }) };
  const options = { rawOptions: {} };
  observed(options.rawOptions, 'mouseEventsRequireAlt', 'require-alt', false);
  observed(options.rawOptions, 'scrollSensitivity', 'slow', 1);
  observed(options.rawOptions, 'fastScrollSensitivity', 'fast', 5);
  const buffer = {}; observed(buffer, 'cols', 'cols', 80); observed(buffer, 'rows', 'rows', 24);
  const browser = {}; observed(browser, 'dpr', 'dpr', 1);
  let delta = spec.delta ?? 1;
  const protocol = { areMouseEventsActive: true, isPixelEncoding: false, isDefaultEncoding: false,
    allowCustomWheelEvent() { trace.push('veto'); if (spec.mutate) delta = -1; return !spec.veto; },
    restrictMouseEvent() { trace.push('restrict'); return true; },
    encodeMouseEvent() { trace.push('encode'); return '!'; } };
  const core = { triggerDataEvent(value, user) { trace.push('data'); data.push([value, user]); }, triggerBinaryEvent() { throw new Error('TRACE encoding must not use binary'); } };
  const coords = new MouseCoordsService(character, render);
  const service = new MouseService(render, coords, protocol, core, buffer, options, {}, {}, browser);
  const event = { preventDefault() { trace.push('prevent'); }, stopPropagation() { trace.push('stop'); } };
  const fields = { clientX: 4, clientY: 4, type: spec.type, overrideType: spec.override, buttons: spec.missingButtons ? undefined : 0,
    button: 0, ctrlKey: false, altKey: false, shiftKey: !!spec.shift, deltaMode: spec.mode ?? 1 };
  for (const [key, value] of Object.entries(fields)) Object.defineProperty(event, key, {
    get() { trace.push('event:' + key); return value; }
  });
  Object.defineProperty(event, 'deltaY', { get() { trace.push('event:deltaY'); return delta; } });
  const ctx = { target: { screenElement: screen } };
  if (spec.type === 'mousemove') service._handleMouseMove(ctx, event);
  else service._handleWheel(ctx, event);
  rows.push({ name: spec.name, trace, data });
}
const fixture = rows.map(row => [row.name, row.trace.join(','), row.data.length].join('\t')).join('\n') + '\n';
const fixturePath = path.join(root, 'crates/xterm-web/tests/fixtures/mouse-live.tsv');
if (process.argv.includes('--generate')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8'), fixture, 'source live-read fixture drift requires reviewed --generate');
const buttonValues = [-Infinity, -(2 ** 53), -4294967297.5, -4294967296, -2147483648.5,
  -1.5, -0.5, -0, 0, 0.5, 1, 1.5, 2.9, 3, 3.5, 4, 4.5, 2147483647.9,
  2147483648, 4294967295, 4294967296, 4294967297.5, 2 ** 53, Number.MAX_VALUE, Infinity, NaN];
const buttonRows = buttonValues.map(value => [Object.is(value, -0) ? '-0' : String(value), value | 0,
  Number(value === value), Number(Object.is(value, -0)), Number(value === 3), Number(value === 4)].join('\t'));
const buttonPath = path.join(root, 'crates/xterm-core/tests/fixtures/mouse-button-numeric.tsv');
const buttonFixture = buttonRows.join('\n') + '\n';
if (process.argv.includes('--generate')) writeFileSync(buttonPath, buttonFixture);
else assert.equal(readFileSync(buttonPath, 'utf8'), buttonFixture, 'numeric button source drift');
const directory = path.join(root, 'results/mouse-live'); mkdirSync(directory, { recursive: true });
const report = { command: 'node scripts/mouse-live-oracle.mjs', node: process.version, platform: process.platform,
  sourceHashes: Object.fromEntries(files.map(file => [file, createHash('sha256').update(readFileSync(path.join(root, file))).digest('hex')])),
  originalSuitesExecuted: 0, directedSourceCases: rows.length, rows, numericButtonCases: buttonRows.length,
  scope: 'Unchanged MouseService/MouseCoords methods with event/host read observers. Internal plain render values grouped to host read methods; complete original suites are task85. Rust/live browser comparison is separate.' };
writeFileSync(path.join(directory, 'source.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ sourceCases: rows.length, fixture: path.relative(root, fixturePath) }));
