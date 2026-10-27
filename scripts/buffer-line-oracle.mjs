// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Execute the read-only reference and save shared deterministic operation traces.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = resolve(import.meta.dirname, '..');
const temporary = resolve(root, 'tmp/buffer-line-oracle');
mkdirSync(temporary, { recursive: true });
const sourceHashes = {};
function read(relative) {
  const value = readFileSync(resolve(root, relative), 'utf8');
  sourceHashes[relative] = createHash('sha256').update(value).digest('hex');
  return value;
}
function emit(name, source) {
  const file = resolve(temporary, name + '.mjs');
  writeFileSync(file, stripTypeScriptTypes(source, { mode: 'transform' }));
  return pathToFileURL(file).href;
}
const base = 'source_repo/src/common/';
const constants = emit('constants', read(base + 'buffer/Constants.ts'));
const decoder = emit('decoder', read(base + 'input/TextDecoder.ts'));
const attributes = emit('attributes', read(base + 'buffer/AttributeData.ts')
  .replace(/^import .* from '\.\.\/Types';\r?\n/m, '')
  .replace(/^import .* from '\.\/Types';\r?\n/m, '')
  .replace("'./Constants'", JSON.stringify(constants)));
const cellUrl = emit('cell', read(base + 'buffer/CellData.ts')
  .replace(/^import .* from '\.\/Types';\r?\n/m, '')
  .replace("'../input/TextDecoder'", JSON.stringify(decoder))
  .replace("'./Constants'", JSON.stringify(constants))
  .replace("'./AttributeData'", JSON.stringify(attributes)));
const lineUrl = emit('line', read(base + 'buffer/BufferLine.ts')
  .replace(/^import .* from '\.\/Types';\r?\n/m, '')
  .replace("'../input/TextDecoder'", JSON.stringify(decoder))
  .replace("'./Constants'", JSON.stringify(constants))
  .replace("'./CellData'", JSON.stringify(cellUrl))
  .replace("'./AttributeData'", JSON.stringify(attributes)));
const helperSource = read(base + 'TestUtils.test.ts');
const helperStart = helperSource.indexOf('export function createCellData');
const helperEnd = helperSource.indexOf('export class MockBufferService');
assert(helperStart >= 0 && helperEnd > helperStart);
const helpers = emit('helpers', `
import { CellData } from ${JSON.stringify(cellUrl)};
import { DEFAULT_ATTR, NULL_CELL_CHAR, NULL_CELL_WIDTH } from ${JSON.stringify(constants)};
` + helperSource.slice(helperStart, helperEnd));
const outcomes = [];
const suites = [];
globalThis.describe = (name, callback) => {
  suites.push(name);
  try { callback(); } finally { suites.pop(); }
};
globalThis.it = (name, callback) => {
  try { callback(); outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' }); }
  catch (error) { outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) }); }
};
await import(emit('upstream-tests', read(base + 'buffer/BufferLine.test.ts')
  .replace("'./Constants'", JSON.stringify(constants))
  .replace("'./CellData'", JSON.stringify(cellUrl))
  .replace("'./BufferLine'", JSON.stringify(lineUrl))
  .replace("'./AttributeData'", JSON.stringify(attributes))
  .replace("'../TestUtils.test'", JSON.stringify(helpers))
  .replace(/^import .* from '\.\/Types';\r?\n/m, '')
  .replace("import { assert } from 'chai';", "import assert from 'node:assert/strict';")));
delete globalThis.describe;
delete globalThis.it;
const { BufferLine } = await import(lineUrl);
const { CellData } = await import(cellUrl);
const { ExtendedAttrs } = await import(attributes);
const { BgFlags, Attributes } = await import(constants);
const seed = 0xb0ff113e;
let state = seed;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
const hex = chars => Array.from({ length: chars.length }, (_, i) => chars.charCodeAt(i).toString(16)).join(',') || '-';
const strings = ['', 'a', ' ', '中', '𝄞', 'e\u0301', '𓂀\u0301', '\ud800', '\udc00', '\t', '\ufeff', '👩‍👩‍👧‍👦'];
function snapshot(line, start, end) {
  const cells = [];
  for (let i = 0; i < line.length; ++i) {
    const cell = line.getCell(i);
    cells.push([cell.content >>> 0, line.getFg(i), line.getBg(i), cell.extended.ext >>> 0,
      cell.extended.urlId, line.getCodePoint(i), line.getWidth(i), line.hasWidth(i),
      line.hasContent(i), line.isCombined(i), line.isProtected(i)].join(',') + ':' + hex(line.getString(i)));
  }
  const columns = [];
  const full = hex(line.translateToString(false, 0, line.length, columns));
  const trimmed = hex(line.translateToString(true, 0, line.length));
  const rangeColumns = [];
  const range = hex(line.translateToString(false, start, end, rangeColumns));
  const trimColumns = [];
  const trimRange = hex(line.translateToString(true, start, end, trimColumns));
  // Canonical full then trimmed deliberately exercises the cache's trimEnd behavior.
  return [cells.join('|') || '-', line.getTrimmedLength(), line.getNoBgTrimmedLength(),
    full, trimmed, columns.join(',') || '-', hex(line.translateToString(false)),
    hex(line.translateToString(true)), +line.isWrapped, range, rangeColumns.join(',') || '-',
    trimRange, trimColumns.join(',') || '-'];
}
const rows = [];
const opCounts = {};
for (let trace = 0; trace < 128; ++trace) {
  let line = new BufferLine(12, undefined, trace % 2 === 1);
  for (let step = 0; step < 64; ++step) {
    const choices = ['cell', 'legacy', 'codepoint', 'add', 'insert', 'delete', 'replace',
      'resize', 'fill', 'within', 'clone', 'copy', 'other', 'cleanup'];
    const op = choices[(trace + step) % choices.length];
    const a = random() % Math.max(1, line.length);
    const b = random() % (op === 'resize' ? 33 : Math.max(1, line.length));
    const c = op === 'resize' ? 0 : random() % (Math.min(line.length - a, line.length - b) + 1);
    const flag = random() % 2;
    const chars = strings[random() % strings.length];
    const width = random() % 3;
    const cell = CellData.fromCharData([random(), chars, width, 0]);
    cell.bg = (random() & (Attributes.CM_MASK | Attributes.RGB_MASK)) >>> 0;
    if (random() % 3 === 0) cell.bg |= BgFlags.PROTECTED;
    if (random() % 2 === 0) cell.bg |= BgFlags.HAS_EXTENDED;
    cell.extended = new ExtendedAttrs(random(), random() % 3 === 0 ? random() : 0);
    const cp = [0, 65, 0x301, 0x4e2d, 0x1f600, 0xd800][random() % 6];
    const input = [trace, step, op, a, b, c, flag, cell.fg, cell.bg >>> 0,
      cell.extended.ext >>> 0, cell.extended.urlId, width, hex(chars), cp];
    let result = '-';
    switch (op) {
      case 'cell': if (line.length) line.setCell(a, cell); break;
      case 'legacy': if (line.length) line.set(a, cell.getAsCharData()); break;
      case 'codepoint': if (line.length) line.setCellFromCodepoint(a, cp, width, cell); break;
      case 'add': if (line.length) line.addCodepointToCell(a, cp, width); break;
      case 'insert': if (line.length) line.insertCells(a, b, cell); break;
      case 'delete': if (line.length) line.deleteCells(a, b, cell); break;
      case 'replace': line.replaceCells(a, b, cell, !!flag); break;
      case 'resize': result = +line.resize(b, cell); break;
      case 'fill': line.fill(cell, !!flag); break;
      case 'within': line.copyCellsFrom(line, a, b, c, !!flag); break;
      case 'clone': line = line.clone(); break;
      case 'copy': { const src = line; line = new BufferLine(step % 5); line.copyFrom(src); break; }
      case 'other': { const src = new BufferLine(line.length, cell, !!flag); line.copyCellsFrom(src, a, b, c, !!flag); break; }
      case 'cleanup': result = line.cleanupMemory(); break;
    }
    opCounts[op] = (opCounts[op] ?? 0) + 1;
    rows.push([...input, ...snapshot(line, a, b), result].join('\t'));
  }
}
const fixture = 'crates/xterm-core/tests/fixtures/buffer-line-traces.tsv';
mkdirSync(resolve(root, 'crates/xterm-core/tests/fixtures'), { recursive: true });
writeFileSync(resolve(root, fixture), rows.join('\n') + '\n');
const report = {
  scope: 'Complete original BufferLine.test.ts baseline and seeded BufferLine operation traces',
  overallEquivalence: 'NOT PROVEN', node: process.version, platform: process.platform, sourceHashes,
  seed: '0xb0ff113e', traces: 128, operations: rows.length, opCounts,
  fixture, fixtureSha256: createHash('sha256').update(readFileSync(resolve(root, fixture))).digest('hex'),
  baseline: { discovered: outcomes.length, executed: outcomes.length,
    passed: outcomes.filter(o => o.status === 'passed').length,
    failed: outcomes.filter(o => o.status === 'failed').length, skipped: 0, unrun: 0, outcomes },
  limitations: ['Native Linux/macOS runtime unrun', 'Extended attribute shared mutation is independently reviewed, not covered by these value traces', 'Traces do not replace all upstream assertion migrations'],
};
mkdirSync(resolve(root, 'results/buffer-line'), { recursive: true });
writeFileSync(resolve(root, 'results/buffer-line/oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
if (report.baseline.failed) process.exitCode = 1;
