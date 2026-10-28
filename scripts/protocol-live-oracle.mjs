// Licensed under MIT. Source-only live-getter cases for Kitty and Win32.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
import vm from 'node:vm';
const root = resolve(import.meta.dirname, '..');
const hashes = {};
function read(name) { const text = readFileSync(resolve(root, name), 'utf8'); hashes[name] = createHash('sha256').update(text).digest('hex'); return text; }
function erase(text) { return stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' }); }
const context = vm.createContext({});
const resultEnum = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum);
vm.runInContext(erase(resultEnum) + '\n' + erase(read('source_repo/src/common/data/EscapeSequences.ts')) + '\n' +
  erase(read('source_repo/src/common/input/KittyKeyboard.ts')) + '\n' + erase(read('source_repo/src/common/input/Win32InputMode.ts')) +
  '\nthis.kitty=new KittyKeyboard(); this.win32=new Win32InputMode();', context);
const fields = ['keyCode', 'key', 'code', 'shiftKey', 'altKey', 'ctrlKey', 'metaKey'];
const letters = ['n', 'k', 'd', 's', 'a', 'c', 'm'];
const hex = text => Array.from({ length: text.length }, (_, index) => text.charCodeAt(index).toString(16)).join(',') || '-';
const num = value => Number.isNaN(value) ? 'NaN' : value === Infinity ? 'Infinity' : value === -Infinity ? '-Infinity' : Object.is(value, -0) ? '-0' : String(value);
function input(name, engine, flags, type, mac, changes = {}) {
  return { name, engine, flags, type, mac, fields: { keyCode: [65], key: ['a'], code: ['KeyA'],
    shiftKey: [false], altKey: [false], ctrlKey: [false], metaKey: [false], ...changes } };
}
const cases = [];
const names = ['a', 'A', 'Enter', 'Tab', 'Backspace', 'Escape', 'CapsLock', 'Shift', 'ArrowUp', 'F1', 'F5', 'F13', 'Dead', '😀', '\ud800', ''];
for (let mask = 0; mask < 16; mask++) for (const key of names) {
  const changes = { key: [key], code: [key === 'Shift' ? 'ShiftLeft' : 'KeyA'],
    shiftKey: [!!(mask & 1)], altKey: [!!(mask & 2)], ctrlKey: [!!(mask & 4)], metaKey: [!!(mask & 8)] };
  for (const flags of [0, 1, 2, 8, 31]) for (let type = 1; type <= 3; type++) {
    cases.push(input(`kitty-${key}-${mask}-${flags}-${type}`, 'K', flags, type, !!(mask & 2), changes));
  }
  for (const type of [0, 1]) cases.push(input(`win32-${key}-${mask}-${type}`, 'W', 0, type, false, changes));
}
const mutations = [
  ['keychanges', { key: ['a', 'b', 'A', '😀', 'x'] }],
  ['keyempties', { key: ['a', '', 'a', '', ''] }],
  ['physicalchanges', { code: ['Numpad0', 'abc123', 'Keyİ', 'Digit5', 'KeyA'], shiftKey: [true] }],
  ['modifierchanges', { shiftKey: [true, false], altKey: [false, true], ctrlKey: [false, true, false], metaKey: [true, false] }],
  ['keylengthchanges', { key: ['a', 'a', '😀', '', 'Enter', 'a'] }],
  ['numpadlexical', { code: ['Numpad123', 'Numpad8suffix', 'Numpad0x10', 'Numpad9suffix'] }],
  ['controlchanges', { key: ['a', 'Enter', 'Backspace', 'Tab', '😀'], ctrlKey: [true, false, true], altKey: [false], metaKey: [false] }],
  ['fractionalnegative', { code: [''], keyCode: [-3.5] }],
  ['nonfinite', { code: [''], keyCode: [NaN, Infinity, -Infinity] }],
];
for (const [name, changes] of mutations) {
  for (const flags of [0, 1, 2, 8, 31]) for (let type = 1; type <= 3; type++) cases.push(input(name, 'K', flags, type, true, changes));
  for (const type of [0, 1]) cases.push(input(name, 'W', 0, type, false, changes));
}
for (const value of [Infinity, -Infinity, NaN, -0, 13.5, 1e-8, 1e21, 9007199254740992]) {
  for (const type of [0, 1]) cases.push(input(`win32-number-${num(value)}-${type}`, 'W', 0, type, false,
    { code: [''], keyCode: [value] }));
}
function execute(test, throwAt) {
  const reads = []; const counts = {}; const event = {}; const marker = {};
  fields.forEach((field, index) => {
    counts[field] = 0;
    Object.defineProperty(event, field, { get() {
      reads.push(letters[index]);
      if (reads.length === throwAt) throw marker;
      const values = test.fields[field]; return values[Math.min(counts[field]++, values.length - 1)];
    } });
  });
  let result;
  try { result = test.engine === 'K' ? context.kitty.evaluate(event, test.flags, test.type, test.mac) : context.win32.evaluateKeyboardEvent(event, !!test.type); }
  catch (error) { assert.equal(error, marker, `unexpected source error ${test.name}`); return [reads.join(''), 'throw', 0, '!']; }
  assert.equal(throwAt, 0, 'requested getter failure was not reached');
  return [reads.join(''), result.type, Number(result.cancel), result.key === undefined ? '!' : hex(result.key)];
}
function row(test, at, output) {
  const f = test.fields;
  return [test.name, test.engine, test.flags, test.type, Number(test.mac), at,
    f.keyCode.map(num).join(';'), f.key.map(hex).join(';'), f.code.map(hex).join(';'),
    ...['shiftKey', 'altKey', 'ctrlKey', 'metaKey'].map(name => f[name].map(Number).join('')), ...output].join('\t');
}
const rows = []; let throwCases = 0;
for (const test of cases) {
  const result = execute(test, 0); rows.push(row(test, 0, result));
  for (let index = 1; index <= result[0].length; index++) {
    rows.push(row(test, index, execute(test, index))); throwCases++;
  }
}
const fixture = rows.join('\n') + '\n';
assert.ok(Buffer.byteLength(fixture) <= 20 * 1024 * 1024, 'source fixture exceeds the approved 20 MiB write budget');
const fixturePath = resolve(root, 'crates/xterm-core/tests/fixtures/protocol-live.tsv');
if (process.argv.includes('--generate')) writeFileSync(fixturePath, fixture);
else assert.equal(readFileSync(fixturePath, 'utf8'), fixture, 'protocol getter fixture drift');
const directory = resolve(root, 'results/protocol-live'); mkdirSync(directory, { recursive: true });
const report = { command: 'node scripts/protocol-live-oracle.mjs', platform: process.platform, node: process.version,
  sourceHashes: hashes, successfulCases: cases.length, throwCases, totalCases: rows.length,
  fixtureSha256: createHash('sha256').update(fixture).digest('hex'),
  scope: 'Original JS getter order and exact thrown marker identity. Rust replay and actual DOM are separate gates; Win32 inherited maps task184/185 not included.' };
writeFileSync(resolve(directory, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
