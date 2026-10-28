// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
// Transpile read-only source modules in memory. Runtime source and tests stay intact.
import fs from 'node:fs';
import path from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';

const { source, load, hashes, typescript } = sourceLoader();
const suites = [];
const tests = [];
globalThis.describe = (name, callback) => { suites.push({ name, before: [] }); callback(); suites.pop(); };
globalThis.beforeEach = callback => suites.at(-1).before.push(callback);
globalThis.it = (name, callback) => tests.push({ name: [...suites.map(s => s.name), name].join(' / '), callback, before: suites.flatMap(s => s.before) });
const outcomes = [];
try {
  load(path.join(source, 'src/browser/OscLinkProvider.test.ts'));
  load(path.join(source, 'src/browser/Linkifier.test.ts'));
  for (const test of tests) {
    try {
      for (const before of test.before) await before();
      if (test.callback.length) await new Promise((resolve, reject) => test.callback(error => error ? reject(error) : resolve()));
      else await test.callback();
      outcomes.push({ name: test.name, status: 'passed' });
    } catch (error) { outcomes.push({ name: test.name, status: 'failed', error: error.stack }); }
  }
} finally {
  delete globalThis.describe; delete globalThis.beforeEach; delete globalThis.it;
}
const { OscLinkProvider } = load(path.join(source, 'src/browser/OscLinkProvider.ts'));
const { BufferLine } = load(path.join(source, 'src/common/buffer/BufferLine.ts'));
const { CellData } = load(path.join(source, 'src/common/buffer/CellData.ts'));
let seed = 0x1a8c055;
const initialSeed = seed;
const next = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
const cases = [];
function add(rows, y, allow = false) {
  const lines = rows.map(row => {
    const line = new BufferLine(row.cells.length, undefined, row.wrapped);
    row.cells.forEach(([cp, id, width], index) => {
      const cell = CellData.fromCharData([0, cp ? String.fromCodePoint(cp) : '', width, cp]);
      cell.extended.urlId = id; cell.updateExtended(); line.setCell(index, cell);
    });
    return line;
  });
  const uris = [undefined, 'https://example.com/1', 'http://example.com/2', 'javascript:alert(1)', 'not a url', '', 'mailto:x@example.com'];
  const provider = new OscLinkProvider({ buffer: { lines: { get: index => lines[index] } } },
    { rawOptions: { linkHandler: { allowNonHttpProtocols: allow } } },
    { getLinkData: id => uris[id] === undefined ? undefined : { uri: uris[id] } });
  let links;
  provider.provideLinks(y, value => { links = value; });
  const encodedRows = rows.map(row => `${+row.wrapped};${row.cells.map(cell => cell.join(':')).join(',')}`).join('|');
  const expected = links === undefined ? '-' : links.map(link => {
    const r = link.range;
    return [Buffer.from(link.text).toString('hex'), r.start.x, r.start.y, r.end.x, r.end.y].join(',');
  }).join(';');
  cases.push([y, +allow, encodedRows, expected].join('\t'));
}
add([{ wrapped: false, cells: [[97,0,1],[97,0,1],[98,1,1],[98,1,1],[98,1,1]] },
     { wrapped: true, cells: [[99,1,1],[99,1,1],[99,1,1],[99,1,1],[120,0,1]] }], 2);
for (let i = 0; i < 4096; i++) {
  const cols = 1 + next() % 12;
  const rows = Array.from({ length: 1 + next() % 6 }, () => ({ wrapped: next() % 3 !== 0,
    cells: Array.from({ length: cols }, () => [next() % 5 === 0 ? 0 : 97 + next() % 26, next() % 7, next() % 5 === 0 ? 2 : 1]) }));
  add(rows, next() % (rows.length + 2), next() % 2 === 0);
}
fs.mkdirSync('crates/xterm-web/tests/fixtures', { recursive: true });
fs.mkdirSync('results/links', { recursive: true });
fs.writeFileSync('crates/xterm-web/tests/fixtures/osc-links.tsv', cases.join('\n') + '\n');
const result = { command: 'node scripts/link-oracle.mjs', node: process.version, typescript, seed: initialSeed,
  baseline: { discovered: outcomes.length, passed: outcomes.filter(t => t.status === 'passed').length,
    failed: outcomes.filter(t => t.status === 'failed').length, skipped: 0, outcomes },
  differential: { cases: cases.length, fixture: 'crates/xterm-web/tests/fixtures/osc-links.tsv' },
  sources: Object.fromEntries([...hashes].sort()) };
fs.writeFileSync('results/links/oracle.json', JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ baseline: result.baseline, differential: result.differential }, null, 2));
if (result.baseline.failed) process.exitCode = 1;
