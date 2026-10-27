// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Runs validation methods from the untouched source class, not a rewritten oracle.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

process.chdir(resolve(import.meta.dirname, '..'));
const output = 'results/public-api';
mkdirSync(output, { recursive: true });
const summaryPath = `${output}/numeric-summary.json`;
const attempt = `numeric-${Date.now()}`;
writeFileSync(summaryPath, JSON.stringify({ status: 'running', attempt }, null, 2) + '\n');
const fixture = 'crates/xterm-core/tests/fixtures/public-api-numeric.tsv';
const sources = [
  'source_repo/src/headless/public/Terminal.ts',
  'source_repo/src/browser/public/Terminal.ts'
];
const hashes = {};
function load(path) {
  const source = readFileSync(path, 'utf8');
  hashes[path] = createHash('sha256').update(source).digest('hex');
  const stripped = stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm, '').replace(/\bexport /g, ''), { mode: 'transform' });
  return new Function('Disposable', `${stripped}\nreturn Terminal;`)(class {});
}
const [Headless, Browser] = sources.map(load);
const seed = 0x50413031;
let state = seed;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
const bits = new DataView(new ArrayBuffer(8));
function encode(value) {
  bits.setFloat64(0, value, false);
  return bits.getBigUint64(0, false).toString(16).padStart(16, '0');
}
function outcome(callback) {
  try { callback(); return 'ok'; } catch (error) { return error.message; }
}
const values = [0, -0, 1, -1, 0.5, -0.5, Number.MIN_VALUE, -Number.MIN_VALUE,
  Number.MAX_VALUE, -Number.MAX_VALUE, Number.EPSILON, 2 ** 31, -(2 ** 31),
  2 ** 32, 2 ** 53 - 1, 2 ** 53, Infinity, -Infinity, NaN];
for (let i = 0; i < 4096; i++) {
  bits.setUint32(0, random(), false); bits.setUint32(4, random(), false);
  values.push(bits.getFloat64(0, false));
}
const rows = [];
const sharedCalls = { resize: 2, registerMarker: 1, scrollLines: 1, scrollPages: 1, scrollToLine: 1 };
let dispatchCases = 0;
for (const value of values) {
  const expected = outcome(() => Headless.prototype._verifyIntegers(value));
  assert.equal(outcome(() => Browser.prototype._verifyIntegers(value)), expected);
  const positive = outcome(() => Browser.prototype._verifyPositiveIntegers(value));
  rows.push(`integer\t${encode(value)}\t${expected}`);
  rows.push(`positive\t${encode(value)}\t${positive}`);
  // A fake core checks only wrapper rejection/dispatch. It is not engine evidence.
  for (const Type of [Headless, Browser]) {
    const calls = Type === Browser ? { ...sharedCalls, select: 3, selectLines: 2, refresh: 2 } : sharedCalls;
    for (const [name, arity] of Object.entries(calls)) for (let index = 0; index < arity; index++) {
      let dispatched = false;
      const self = Object.create(Type.prototype);
      const args = Array(arity).fill(1); args[index] = value;
      self._core = { [name]: (...forwarded) => { assert.deepEqual(forwarded, args); dispatched = true; } };
      assert.equal(outcome(() => self[name](...args)), expected, `${name} argument ${index}`);
      assert.equal(dispatched, expected === 'ok');
      dispatchCases++;
    }
  }
  for (const key of ['x', 'width', 'height']) {
    let dispatched = false;
    const options = { [key]: value };
    const self = Object.create(Browser.prototype);
    self._core = { registerDecoration: forwarded => { assert.equal(forwarded, options); dispatched = true; } };
    assert.equal(outcome(() => self.registerDecoration(options)), positive);
    assert.equal(dispatched, positive === 'ok');
    dispatchCases++;
  }
}
for (const value of [undefined, null]) {
  const self = Object.create(Browser.prototype);
  let count = 0;
  self._core = { registerDecoration: () => count++ };
  assert.equal(outcome(() => self.registerDecoration({ x: value, width: value, height: value })), 'ok');
  assert.equal(count, 1);
}
for (const Type of [Headless, Browser]) {
  assert.equal(outcome(() => Type.prototype._verifyIntegers()), 'ok');
  for (const value of values.slice(0, 19)) for (const index of [0, 1, 2]) {
    const args = [1, 2, 3]; args[index] = value;
    const expected = outcome(() => Type.prototype._verifyIntegers(value));
    assert.equal(outcome(() => Type.prototype._verifyIntegers(...args)), expected);
  }
}
let readonlyCases = 0;
for (const key of ['cols', 'rows', 'fontSize', 'Rows', 'allowProposedApi', 'unknown']) {
  const expected = outcome(() => Headless.prototype._checkReadonlyOptions(key));
  assert.equal(outcome(() => Browser.prototype._checkReadonlyOptions(key)), expected);
  rows.push(`option\t${key}\t${expected}`); readonlyCases++;
}
for (const allowed of [false, true]) {
  const self = { _core: { optionsService: { options: { allowProposedApi: allowed }, rawOptions: { allowProposedApi: allowed } } } };
  const expected = outcome(() => Headless.prototype._checkProposedApi.call(self));
  assert.equal(outcome(() => Browser.prototype._checkProposedApi.call(self)), expected);
  rows.push(`proposed\t${allowed}\t${expected}`);
}
const modeRows = [];
const modeInputKeys = ['applicationCursorKeys', 'applicationKeypad', 'bracketedPasteMode',
  'origin', 'reverseWraparound', 'sendFocus', 'synchronizedOutput', 'win32InputMode', 'wraparound'];
for (let mask = 0; mask < 2048; mask++) for (const protocol of ['NONE', 'X10', 'VT200', 'DRAG', 'ANY', 'CUSTOM']) {
  const privateModes = Object.fromEntries(modeInputKeys.map((key, index) => [key, Boolean(mask & (1 << index))]));
  const self = { _core: {
    coreService: { decPrivateModes: privateModes, modes: { insertMode: Boolean(mask & 512) }, isCursorHidden: Boolean(mask & 1024) },
    mouseStateService: { activeProtocol: protocol }
  } };
  const headless = Object.getOwnPropertyDescriptor(Headless.prototype, 'modes').get.call(self);
  const browser = Object.getOwnPropertyDescriptor(Browser.prototype, 'modes').get.call(self);
  assert.deepEqual(browser, headless);
  // Object key order follows the public source getter; include every field.
  modeRows.push(`${mask}\t${protocol}\t${Object.values(headless).map(value => typeof value === 'boolean' ? Number(value) : value).join(',')}`);
}
const modeFixture = 'crates/xterm-core/tests/fixtures/public-api-modes.tsv';
const modeText = `${modeRows.join('\n')}\n`;
if (process.argv.includes('--generate')) writeFileSync(modeFixture, modeText);
else assert.equal(readFileSync(modeFixture, 'utf8'), modeText, 'modes fixture drift; review before --generate');
const fixtureText = `${rows.join('\n')}\n`;
if (process.argv.includes('--generate')) writeFileSync(fixture, fixtureText);
else assert.equal(readFileSync(fixture, 'utf8'), fixtureText, 'source/fixture drift; review before --generate');
const summary = {
  status: 'running', attempt, seed, randomValues: 4096, numericValues: values.length,
  fixtureRows: rows.length, modeSnapshotRows: modeRows.length, wrapperDispatchCases: dispatchCases, readonlyCases,
  sources: hashes, fixtureSha256: createHash('sha256').update(readFileSync(fixture)).digest('hex'),
  modeFixtureSha256: createHash('sha256').update(readFileSync(modeFixture)).digest('hex'),
  rustInputs: Object.fromEntries(['crates/xterm-core/src/public_api/validation.rs',
    'crates/xterm-core/src/headless/modes.rs', 'crates/xterm-core/tests/public_api_differential.rs']
    .map(path => [path, createHash('sha256').update(readFileSync(path)).digest('hex')])),
  node: process.version, platform: `${process.platform}-${process.arch}`,
  command: 'cargo test -p xterm-core --test public_api_differential',
  scope: 'PA01 validators/fake-core dispatch; PA02 readonly/proposed errors; PA05 service-backed snapshots, not escape dispatch',
  unrun: ['real engine clamp/state/events', 'WASM ABI coercion', 'browser DOM', 'PA02 live options through terminal', 'PA03/PA04/PA06', 'PA05 escape dispatch/negotiation']
};
writeFileSync(summaryPath, JSON.stringify(summary, null, 2) + '\n');
const cargo = process.env.CARGO ?? (process.platform === 'win32' ? `${process.env.USERPROFILE}/.cargo/bin/cargo.exe` : 'cargo');
const run = spawnSync(cargo, ['test', '-p', 'xterm-core', '--test', 'public_api_differential'], {
  encoding: 'utf8', timeout: 180000, maxBuffer: 8 * 1024 * 1024
});
for (const name of ['numeric-cargo', attempt]) {
  writeFileSync(`${output}/${name}.stdout.log`, run.stdout ?? '');
  writeFileSync(`${output}/${name}.stderr.log`, run.stderr ?? '');
}
summary.status = run.status === 0 ? 'passed' : 'failed';
summary.exitCode = run.status;
summary.error = run.error?.message ?? null;
summary.changedDuringRun = Object.entries(summary.rustInputs).filter(([path, hash]) =>
  createHash('sha256').update(readFileSync(path)).digest('hex') !== hash).map(([path]) => path);
if (summary.changedDuringRun.length) summary.status = 'changed-during-run';
for (const path of sources) assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'), hashes[path]);
writeFileSync(summaryPath, JSON.stringify(summary, null, 2) + '\n');
writeFileSync(`${output}/${attempt}.json`, JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
process.exitCode = summary.status === 'passed' ? 0 : 1;
