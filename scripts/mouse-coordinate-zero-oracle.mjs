#!/usr/bin/env node
// Copyright (c) 2017-2026 The xterm.js authors. MIT license.
// Execute the original coordinate methods, not a rewritten clamp oracle.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../', import.meta.url));
const reference = process.env.XT_REFERENCE_ROOT ?? path.join(root, 'source_repo');
const files = ['src/browser/input/Mouse.ts', 'src/browser/services/MouseCoordsService.ts', 'src/browser/Dom.ts'];
const sources = files.map(file => readFileSync(path.join(reference, file)));
const erase = bytes => stripTypeScriptTypes(bytes.toString().replace(/^import .*;\r?$/gm, '')
  .replace(/^export /gm, '').replace(/@(IRenderService|ICharSizeService)\s+/g, ''), { mode: 'transform' });
const context = vm.createContext({});
const dom = erase(sources[2]);
const start = dom.indexOf('function getWindow(');
const end = dom.indexOf('\nclass DomListener', start);
assert.ok(start >= 0 && end > start, 'unchanged getWindow source boundaries');
vm.runInContext(dom.slice(start, end), context);
vm.runInContext(erase(sources[0]), context);
vm.runInContext(erase(sources[1]), context);
const getCoords = vm.runInContext('getCoords', context);
const MouseCoordsService = vm.runInContext('MouseCoordsService', context);
const token = number => Object.is(number, -0) ? '-0' : String(number);
const values = [-Infinity, -1, -Number.MIN_VALUE, -0, 0, Number.MIN_VALUE, 1, Infinity, NaN];
const rows = [];
function capture(client, origin, padding, cell, canvas, count, selection, valid) {
  const window = { getComputedStyle: () => ({ getPropertyValue: () => token(padding) + 'px' }) };
  const element = { ownerDocument: { defaultView: window }, getBoundingClientRect: () => ({ left: origin, top: origin }) };
  const event = { clientX: client, clientY: client };
  const coords = getCoords(window, event, element, count, count, valid, cell, cell, selection);
  const service = new MouseCoordsService({ hasValidSize: valid }, {
    dimensions: { css: { cell: { width: cell, height: cell }, canvas: { width: canvas, height: canvas } } }
  });
  const report = service.getMouseReportCoords(event, element);
  rows.push([+valid, origin, origin, padding, padding, cell, cell, canvas, canvas,
    client, client, count, count, +selection].map(token)
    .concat(coords ? Array.from(coords, token) : ['u', 'u'])
    .concat(report ? ['col', 'row', 'x', 'y'].map(key => token(report[key])) : ['u', 'u', 'u', 'u']).join('\t'));
}
// Cross all zero signs at the max lower-bound tie and min upper-bound tie.
for (const client of values) for (const origin of [-0, 0]) for (const padding of [-0, 0]) {
  for (const cell of [-1, -0, 0, 1, Infinity]) for (const canvas of [0, 1, 2]) {
    for (const count of [-0, 0, 1]) for (const selection of [false, true]) {
      capture(client, origin, padding, cell, canvas, count, selection, true);
    }
  }
}
for (const value of values) capture(value, 0, 0, 1, 2, 1, false, false);
const fixture = rows.join('\n') + '\n';
const destination = path.join(root, 'crates/xterm-web/tests/fixtures/mouse-coordinate-zero.tsv');
if (process.argv.includes('--generate')) writeFileSync(destination, fixture);
else assert.equal(readFileSync(destination, 'utf8'), fixture, 'source fixture drift requires reviewed --generate');
const directory = path.join(root, 'results/mouse-coordinate-zero');
mkdirSync(directory, { recursive: true });
const report = {
  command: 'node scripts/mouse-coordinate-zero-oracle.mjs', node: process.version, platform: process.platform,
  sourceHashes: Object.fromEntries(files.map((file, index) => [file, createHash('sha256').update(sources[index]).digest('hex')])),
  cases: rows.length, fixtureSha256: createHash('sha256').update(fixture).digest('hex'),
  originalSuitesExecuted: 0,
  scope: 'Original getCoords and MouseCoordsService, zero signs, subnormals, infinity, NaN, invalid size. Rust/browser execution is separate.'
};
writeFileSync(path.join(directory, 'source.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
