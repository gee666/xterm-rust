// Execute read-only xterm.js cell sources and retain deterministic Rust fixtures.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';

const root = resolve(import.meta.dirname, '..');
const temporary = resolve(root, 'tmp/cell-oracle');
mkdirSync(temporary, { recursive: true });
const hashes = {};
function read(relative) {
  const source = readFileSync(resolve(root, relative), 'utf8');
  hashes[relative] = createHash('sha256').update(source).digest('hex');
  return source;
}
function emit(name, source) {
  const path = resolve(temporary, name + '.mjs');
  writeFileSync(path, stripTypeScriptTypes(source, { mode: 'transform' }));
  return pathToFileURL(path).href;
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
const { CellData } = await import(cellUrl);
const { AttributeData, ExtendedAttrs } = await import(attributes);
const { Attributes: C, BgFlags, FgFlags } = await import(constants);
const outcomes = [];
const suites = [];
globalThis.describe = (name, callback) => {
  suites.push(name);
  try { callback(); } finally { suites.pop(); }
};
globalThis.it = (name, callback) => {
  try { callback(); outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' }); }
  catch (error) { outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error) }); }
};
const assertionUrl = 'node:assert/strict';
await import(emit('cell-tests', read(base + 'buffer/CellData.test.ts')
  .replace("'./Constants'", JSON.stringify(constants))
  .replace("'./CellData'", JSON.stringify(cellUrl))
  .replace("import { assert } from 'chai';", `import assert from '${assertionUrl}';`)));
// The remaining BufferLine suites require BufferLine, which this task does not port.
const bufferTests = read(base + 'buffer/BufferLine.test.ts');
const start = bufferTests.indexOf("describe('AttributeData'");
const end = bufferTests.indexOf("describe('BufferLine'");
assert(start >= 0 && end > start);
await import(emit('buffer-attribute-tests', `
import { Attributes, BgFlags, FgFlags, UnderlineStyle, Content } from ${JSON.stringify(constants)};
import { CellData } from ${JSON.stringify(cellUrl)};
import { AttributeData } from ${JSON.stringify(attributes)};
import assert from '${assertionUrl}';
` + bufferTests.slice(start, end)));

let state = 0xce11da7a;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
function hexUnits(chars) {
  return Array.from({ length: chars.length }, (_, i) => chars.charCodeAt(i).toString(16)).join(',') || '-';
}
function snapshot(a) {
  return [a.isInverse() >>> 0, a.isBold() >>> 0, a.isUnderline() >>> 0,
    a.isBlink() >>> 0, a.isInvisible() >>> 0, a.isItalic() >>> 0,
    a.isDim() >>> 0, a.isStrikethrough() >>> 0, a.isProtected() >>> 0,
    a.isOverline() >>> 0, a.getFgColorMode(), a.getBgColorMode(),
    +a.isFgRGB(), +a.isBgRGB(), +a.isFgPalette(), +a.isBgPalette(),
    +a.isFgDefault(), +a.isBgDefault(), +a.isAttributeDefault(),
    a.getFgColor(), a.getBgColor(), a.hasExtendedAttrs(), a.getUnderlineColor(),
    a.getUnderlineColorMode(), +a.isUnderlineColorRGB(), +a.isUnderlineColorPalette(),
    +a.isUnderlineColorDefault(), a.getUnderlineStyle(), a.getUnderlineVariantOffset(),
    a.extended.ext >>> 0, +a.extended.isEmpty(), a.getWidth(), a.isCombined(),
    Number.isNaN(a.getCode()) ? -1 : a.getCode()];
}
const rawRows = [];
for (let i = 0; i < 2048; ++i) {
  const a = new CellData();
  a.fg = random(); a.bg = random();
  const ext = random(); const url = i % 3 === 0 ? random() : 0;
  a.extended = new ExtendedAttrs(ext, url);
  a.content = random() & 0xffffff;
  a.combinedData = String.fromCharCode(random() & 0xffff, random() & 0xffff);
  const b = new CellData();
  b.fg = a.fg; b.bg = a.bg; b.extended = a.extended.clone();
  const bit = (1 << (i % 32)) >>> 0;
  if (i % 5 === 0) b.fg = (b.fg ^ bit) >>> 0;
  if (i % 5 === 1) b.bg = (b.bg ^ bit) >>> 0;
  if (i % 5 === 2) b.extended.ext = (b.extended.ext ^ bit) >>> 0;
  if (i % 5 === 3) b.extended.urlId = b.extended.urlId ? 0 : 1;
  rawRows.push([a.fg, a.bg, ext, url, a.content, hexUnits(a.combinedData),
    snapshot(a).join(','), hexUnits(a.getChars()),
    b.fg, b.bg, b.extended.ext >>> 0, b.extended.urlId, +a.attributesEquals(b)].join('\t'));
}
const charRows = [];
const strings = ['', 'a', 'e\u0301', '𝄞', '𓂀\u0301', '１', '\ud800', '\udc00', '\ud800x', 'x\udc00', '👩‍👩‍👧‍👦'];
for (let i = 0; i < 1024; ++i) {
  const chars = i < strings.length ? strings[i] : String.fromCharCode(...Array.from({length: random() % 6}, () => random() & 0xffff));
  const fg = random(); const width = i < strings.length ? 1 : random() | 0;
  const a = new CellData();
  a.bg = 0xffffffff; a.extended = new ExtendedAttrs(0xffffffff, 42);
  a.combinedData = 'old';
  a.setFromCharData([fg, chars, width, 123456]);
  charRows.push([fg, width, hexUnits(chars), a.content >>> 0, a.bg, a.getWidth(),
    Number.isNaN(a.getCode()) ? -1 : a.getCode(), a.isCombined(), hexUnits(a.getChars()),
    hexUnits(a.combinedData), a.extended.ext >>> 0, a.extended.urlId].join('\t'));
}
const extendedRows = [];
const extended = [new ExtendedAttrs(), new ExtendedAttrs(), new ExtendedAttrs()];
for (let i = 0; i < 4096; ++i) {
  const target = random() % 3;
  const operation = random() % 8;
  const value = random();
  const attrs = extended[target];
  switch (operation) {
    case 0: attrs.ext = value; break;
    case 1: attrs.underlineStyle = value; break;
    case 2: attrs.underlineColor = value; break;
    case 3: attrs.underlineVariantOffset = value; break;
    case 4: attrs.urlId = value % 3 === 0 ? 0 : value; break;
    case 5: attrs.payload = value % 2 ? {} : undefined; break;
    case 6: extended[target] = extended[value % 3]; break;
    case 7: extended[target] = extended[value % 3].clone(); break;
  }
  const snapshot = extended.flatMap(a => [a.ext >>> 0, a.underlineStyle, a.underlineColor,
    a.underlineVariantOffset, a.urlId, +a.isEmpty(), +(a.payload !== undefined)]);
  snapshot.push(+(extended[0] === extended[1]), +(extended[0] === extended[2]), +(extended[1] === extended[2]));
  extendedRows.push([target, operation, value, snapshot.join(',')].join('\t'));
}
const fixtures = resolve(root, 'crates/xterm-core/tests/fixtures');
mkdirSync(fixtures, {recursive: true});
writeFileSync(resolve(fixtures, 'cell-raw.tsv'), rawRows.join('\n') + '\n');
writeFileSync(resolve(fixtures, 'cell-char-data.tsv'), charRows.join('\n') + '\n');
writeFileSync(resolve(fixtures, 'cell-extended.tsv'), extendedRows.join('\n') + '\n');
const report = {
  scope: 'CellData.test.ts complete; BufferLine.test.ts AttributeData and CellData describes only',
  overallEquivalence: 'NOT PROVEN', node: process.version, platform: process.platform,
  sourceHashes: hashes,
  fixtureHashes: Object.fromEntries(['cell-raw.tsv', 'cell-char-data.tsv', 'cell-extended.tsv']
    .map(name => [name, createHash('sha256').update(readFileSync(resolve(fixtures, name))).digest('hex')])),
  seed: '0xce11da7a', upstream: {
    discovered: outcomes.length, executed: outcomes.length, skipped: 0, unrun: 0,
    passed: outcomes.filter(t => t.status === 'passed').length,
    failed: outcomes.filter(t => t.status === 'failed').length, outcomes,
  }, fixtures: { rawCells: rawRows.length, charData: charRows.length, extendedOperations: extendedRows.length },
  unrun: ['Remaining BufferLine suites', 'Terminal integration', 'Native Linux/macOS runtime'],
};
const results = resolve(root, 'results/cell');
mkdirSync(results, {recursive:true});
writeFileSync(resolve(results, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ upstream: report.upstream, fixtures: report.fixtures }, null, 2));
if (report.upstream.failed) process.exitCode = 1;
