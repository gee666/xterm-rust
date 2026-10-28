// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Direct unchanged InputHandler producer/error behavior, not a JS port engine.
import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { loadTerminalReference } from './terminal-reference.mjs';

const output = 'results/a11y-events';
await mkdir(output, { recursive: true });
await writeFile(`${output}/source-attempt.json`, JSON.stringify({ status: 'running', command: 'node scripts/a11y-event-oracle.mjs' }) + '\n');
const { Terminal, sources } = await loadTerminalReference();
const hex = text => Array.from({ length: text.length }, (_, index) => text.charCodeAt(index).toString(16)).join(',');
const units = values => String.fromCharCode(...values);
const custom = { '!': '', '[': '\ud800', ']': '\udc00', '@': '\ud83d\ude00', '?': 'ij', '#': '\0x' };
const cases = [];
function add(name, chunks, expected, options = {}) { cases.push({ name, chunks, expected, ...options }); }
const mixed = [0xdc00, 0xd83d, 0xde00, 0xd800, 65];
const mixedExpected = ['dc00', 'd83d,de00', 'd800', '41'];
add('whole-utf16', [mixed], mixedExpected);
add('split-utf16', mixed.map(unit => [unit]), mixedExpected);
add('pending-high-and-empty', [[0xd800], [], [65]], ['d800', '41']);
add('custom-first-unit', [[33, 91, 93, 64, 63, 35]], ['21', 'd800', 'dc00', 'd83d', '69', '0'], { custom: true });
add('reader-disabled', [mixed], [], { enabled: false });
add('source-bom-soft-hyphen', [[0xfeff, 0xad, 0xdc00, 65]], ['dc00', '41']);
add('malformed-utf8-surrogate', [[0xed, 0xa0, 0x80, 0xf0, 0x9f, 0x98, 0x80, 0xef, 0xbb, 0xbf, 66]], ['d83d,de00', '42'], { bytes: true });
const outcomes = [];
function terminal(enabled = true) {
  return new Terminal({ cols: 40, rows: 3, allowProposedApi: true, screenReaderMode: enabled });
}
for (const test of cases) {
  const t = terminal(test.enabled !== false);
  if (test.custom) t._core._charsetService.setgCharset(0, custom);
  const actual = [];
  t._core._inputHandler.onA11yChar(value => actual.push(hex(value)));
  for (const chunk of test.chunks) {
    assert.equal(t._core._inputHandler.parse(test.bytes ? Uint8Array.from(chunk) : units(chunk)), undefined);
  }
  const record = { name: test.name, bytes: !!test.bytes, chunks: test.chunks.map(chunk => chunk.map(value => value.toString(16)).join(',')), actual, expected: test.expected };
  if (JSON.stringify(actual) !== JSON.stringify(test.expected)) await writeFile(`${output}/counterexample.json`, JSON.stringify(record, null, 2) + '\n');
  assert.deepEqual(actual, test.expected, test.name);
  outcomes.push({ ...record, status: 'passed' });
  t.dispose();
}
const opaque = { toString() { throw new Error('a11y failure must not be stringified'); } };
const errors = [['opaque-object', opaque], ['symbol', Symbol('a11y sentinel')], ['number', 17], ['string', 'a11y failure'], ['null', null], ['undefined', undefined]];
const errorOutcomes = [];
for (const [kind, sentinel] of errors) {
  const t = terminal();
  const payloads = [];
  const subscription = t._core._inputHandler.onA11yChar(value => { payloads.push(hex(value)); throw sentinel; });
  let caught = false;
  let actual;
  try { t._core._inputHandler.parse(units([0xdc00, 66])); }
  catch (error) { caught = true; actual = error; }
  const buffer = t._core.buffer;
  const cell = buffer.lines.get(buffer.ybase + buffer.y).getCell(0).getChars();
  assert(caught, kind);
  assert(Object.is(actual, sentinel), `${kind} lost thrown identity`);
  assert.deepEqual(payloads, ['dc00'], kind);
  assert.equal(buffer.x, 0, kind);
  assert.equal(cell, '', kind);
  subscription.dispose();
  assert.equal(t._core._inputHandler.parse('C'), undefined);
  assert.equal(buffer.x, 1, kind);
  assert.equal(buffer.lines.get(buffer.ybase + buffer.y).getCell(0).getChars(), 'C', kind);
  errorOutcomes.push({ kind, sameThrownValue: true, payloads, beforeCursor: 0, beforeCell: '', recoveryCursor: 1, recoveryCell: '43', status: 'passed' });
  t.dispose();
}
const report = { command: 'node scripts/a11y-event-oracle.mjs', node: process.version, platform: process.platform,
  sources, newProducerCases: outcomes.length, newFailureCases: errorOutcomes.length,
  originalDedicatedA11ySurrogateCasesClaimed: 0,
  scope: 'Actual unchanged InputHandler synchronous parse, native custom charset and callback semantics. No JS write-queue or browser DOM pass is claimed.',
  outcomes, errorOutcomes, status: 'passed' };
await writeFile(`${output}/source.json`, JSON.stringify(report, null, 2) + '\n');
await writeFile(`${output}/source-attempt.json`, JSON.stringify({ status: 'passed', producerCases: outcomes.length, failureCases: errorOutcomes.length }) + '\n');
console.log(JSON.stringify({ producerCases: outcomes.length, failureCases: errorOutcomes.length, status: 'passed' }));
