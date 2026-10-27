// Copyright (c) 2019-2020 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { Disposable, DisposableStore, toDisposable } from '../source_repo/src/common/Lifecycle.ts';

process.chdir(resolve(import.meta.dirname, '..'));
mkdirSync('results/options', { recursive: true });
function load(path, names, values, returned) {
  const source = readFileSync(path, 'utf8').replace(/^import .*;\r?$/gm, '').replace(/\bexport /g, '');
  return new Function(...names, stripTypeScriptTypes(source, { mode: 'transform' }) + `\nreturn {${returned}};`)(...values);
}
const { Emitter } = load('source_repo/src/common/Event.ts', ['DisposableStore', 'toDisposable'], [DisposableStore, toDisposable], 'Emitter');
const { OptionsService, DEFAULT_OPTIONS } = load('source_repo/src/common/services/OptionsService.ts',
  ['Disposable', 'toDisposable', 'Emitter', 'isMac'], [Disposable, toDisposable, Emitter, false], 'OptionsService,DEFAULT_OPTIONS');

// Execute the original assertions, including both repeated specific-option cases.
const cases = [], contexts = [];
const describe = (_name, fn) => { contexts.push({ before: [], after: [] }); fn(); contexts.pop(); };
const it = (name, fn) => cases.push({ name, fn, before: contexts.flatMap(c => c.before), after: contexts.toReversed().flatMap(c => c.after) });
let assertions = 0;
const countedAssert = new Proxy(assert, { get(target, key) {
  const fn = key === 'notOk' ? v => assert.ok(!v) : target[key];
  return typeof fn === 'function' ? (...args) => { assertions++; return fn(...args); } : fn;
} });
const testSource = readFileSync('source_repo/src/common/services/OptionsService.test.ts', 'utf8').replace(/^import .*;\r?$/gm, '');
new Function('assert', 'OptionsService', 'DEFAULT_OPTIONS', 'describe', 'it', 'beforeEach', 'afterEach', stripTypeScriptTypes(testSource))(
  countedAssert, OptionsService, DEFAULT_OPTIONS, describe, it,
  fn => contexts.at(-1).before.push(fn), fn => contexts.at(-1).after.push(fn));
for (const test of cases) {
  for (const fn of test.before) await fn();
  try { await test.fn(); } finally { for (const fn of test.after) await fn(); }
}
assert.equal(cases.length, 10);
assert.equal(Object.keys(DEFAULT_OPTIONS).length, 48);

const service = new OptionsService({});
let events = 0;
service.onOptionChange(() => events++);
const objects = new Map();
const operations = ['@defaults'], expected = Object.entries(DEFAULT_OPTIONS).map(([key, value]) => {
  const encoded = value && typeof value === 'object'
    ? `o:{${Object.entries(value).map(([k,v]) => `${k}=${encode(v)}`).join(',')}}` : encode(value);
  return `${key}\t${encoded}`;
});
let constructorOperations = 0;
function encode(v) {
  if (v === undefined) return 'u';
  if (v === null) return 'z';
  if (typeof v === 'number') return `n:${String(v)}`;
  if (typeof v === 'boolean') return `b:${v}`;
  if (typeof v === 'string') return `s:${Array.from({length: v.length}, (_, i) => v.charCodeAt(i).toString(16)).join(',')}`;
  return 'o';
}
function hintObject(data) {
  if (data === 'split') return { valueOf: () => 0, toString: () => 'zero' };
  if (data === 'bad') return { [Symbol.toPrimitive]: () => ({}) };
  if (data === 'throw') return { [Symbol.toPrimitive]: hint => { throw new TypeError(`coercion ${hint}`); } };
  if (data === 'numeric') return { [Symbol.toPrimitive]: hint => hint === 'number' ? 3.9 : 'three' };
  assert.fail(`unknown hint object ${data}`);
}
function operation(key, kind, data = '') {
  let value;
  if (kind === 'n') value = Number(data);
  if (kind === 'z') value = null;
  if (kind === 'b') value = data === 'true';
  if (kind === 's') value = data === '' ? '' : String.fromCharCode(...data.split(',').map(n => parseInt(n, 16)));
  if (kind === 'o' || kind === 'h') {
    const id = kind === 'h' ? `hint:${data}` : data;
    if (!objects.has(id)) objects.set(id, kind === 'h' ? hintObject(data) : {});
    value = objects.get(id);
  }
  const line = `${key}\t${kind}\t${data}`;
  operations.push(line);
  let prefix = 'ok';
  try { service.options[key] = value; } catch (error) { prefix = `error:${encode(error.message).slice(2)}`; }
  expected.push(`${prefix}\t${encode(service.options[key])}\t${events}`);
}
const keys = Object.keys(DEFAULT_OPTIONS);
const values = [
  ...Array.from({length:9}, (_,i) => ['s', [...String((i+1)*100)].map(c => c.charCodeAt(0).toString(16)).join(',')]),
  ...['\u0009','\u000b','\u000c','\u0020','\u00a0','\u1680','\u2000','\u200a','\u2028','\u2029','\u202f','\u205f','\u3000','\ufeff'].map(ws => ['s', [...`${ws}2${ws}`].map(c => c.charCodeAt(0).toString(16)).join(',')]),
  ['u', ''], ['z', ''], ['b', 'true'], ['b', 'false'],
  ...[-Infinity, -100, -1, -0, 0.01, 0.9, 1, 1.15, 1.25, 1.95, 2.25, 999, 1000, 1001, 4294967296, Infinity, NaN, 1e-7, 1e21].map(n => ['n', String(n)]),
  ...['', 'block', 'bar', 'underline', 'outline', 'normal', 'bold', '600', 'bold700', '2.9', '0x10', '0o17', '0b11', 'inf', '+Infinity', '\ufeff 2\u00a0', '\u00852', '\ud800', '😀'].map(s => ['s', Array.from({length: s.length}, (_, i) => s.charCodeAt(i).toString(16)).join(',')]),
  ['o', 'first'], ['o', 'first'], ['o', 'second'], ...['split','bad','throw','numeric'].map(data => ['h', data]) ];
for (const key of keys) for (const [kind, data] of values) operation(key, kind, data);
// Each constructor case is independently initialized, preserving fallback diagnostics.
const originalError = console.error;
try {
  for (const key of keys) for (const [kind, data] of values) {
    let value;
    if (kind === 'n') value = Number(data);
    if (kind === 'z') value = null;
    if (kind === 'b') value = data === 'true';
    if (kind === 's') value = data === '' ? '' : String.fromCharCode(...data.split(',').map(n => parseInt(n, 16)));
    if (kind === 'o' || kind === 'h') value = objects.get(kind === 'h' ? `hint:${data}` : data);
    const diagnostics = [];
    console.error = error => diagnostics.push(encode(error.message).slice(2));
    const instance = new OptionsService({[key]: value});
    operations.push(`@new\t${key}\t${kind}\t${data}`);
    expected.push(`new:${diagnostics.join(';')}\t${encode(instance.options[key])}`);
    constructorOperations++;
    instance.dispose();
  }
} finally { console.error = originalError; }
const scenario = new OptionsService({}), trace = [];
scenario.onOptionChange(key => {
  trace.push(`general:${key}`);
  if (key === 'cols' && scenario.options.cols === 100) scenario.options.cols = 101;
});
scenario.onSpecificOptionChange('cols', value => trace.push(`specific:${value}`));
scenario.onMultipleOptionChange(['cols','cols','rows'], () => trace.push('multiple'));
scenario.onOptionChange(() => assert.fail('unsubscribed listener')).dispose();
scenario.options.cols = 100;
scenario.rawOptions.rows = -1;
scenario.options.rows = 24;
const first = scenario.options.windowsPty;
scenario.options.windowsPty = null;
const second = scenario.options.windowsPty;
scenario.options.windowsPty = undefined;
trace.push(`fresh:${first !== second}:${second !== scenario.options.windowsPty}`);
scenario.options.documentOverride = {};
scenario.options.linkHandler = {};
scenario.dispose(); scenario.dispose();
trace.push(`clean:${scenario.options.documentOverride === null}:${scenario.options.linkHandler === null}`);
scenario.onOptionChange(() => assert.fail('late disposed listener'));
scenario.options.rows = 30;
trace.push(`after:${scenario.options.rows === 30}`);
const disposing = new OptionsService({});
let called = false, failed = false;
disposing.onOptionChange(() => disposing.dispose());
disposing.onOptionChange(() => called = true);
try { disposing.options.rows = 40; } catch(error) { assert.ok(error instanceof TypeError); failed = true; }
trace.push(`during:${failed}:${called}`);
operations.push('@scenario'); expected.push(trace.join(','));
const seed = 0x4f505453;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
for (let i = 0; i < 20000; i++) {
  const key = keys[random() % keys.length];
  const [kind, data] = values[random() % values.length];
  operation(key, kind, data);
}
const numericKeys = ['cursorWidth','blinkIntervalDuration','minimumContrastRatio','scrollback','lineHeight','tabStopWidth','scrollSensitivity','fastScrollSensitivity'];
for (let i = 0; i < 5000; i++) {
  const radix = [2,8,16][random()%3];
  const prefix = {2:'0b',8:'0o',16:'0x'}[radix];
  let text = prefix;
  const length = 1 + random()%180;
  for (let j = 0; j < length; j++) text += (random()%radix).toString(radix);
  operation(numericKeys[random()%numericKeys.length], 's', [...text].map(c => c.charCodeAt(0).toString(16)).join(','));
}
const cargo = process.env.CARGO ?? 'cargo';
const build = spawnSync(cargo, ['build', '-p', 'xterm-core', '--example', 'options_oracle'], { encoding: 'utf8', timeout: 120000 });
writeFileSync('results/options/build.log', `${build.stdout ?? ''}${build.stderr ?? ''}`);
if (build.error) throw build.error;
assert.equal(build.status, 0, build.stderr);
const binary = resolve('target/debug/examples/options_oracle' + (process.platform === 'win32' ? '.exe' : ''));
const run = spawnSync(binary, [], { input: operations.join('\n') + '\n', encoding: 'utf8', timeout: 120000, maxBuffer: 32 * 1024 * 1024 });
if (run.error) throw run.error;
assert.equal(run.status, 0, run.stderr);
const actual = run.stdout.trimEnd().split(/\r?\n/);
assert.equal(actual.length, expected.length);
for (let i = 0; i < actual.length; i++) {
  if (actual[i] !== expected[i]) {
    writeFileSync('results/options/counterexample.json', JSON.stringify({seed, index: i, operation: i < keys.length ? '@defaults' : operations[i - keys.length + 1], expected: expected[i], actual: actual[i]}, null, 2));
    assert.fail(`Mismatch at ${i}; see results/options/counterexample.json`);
  }
}
const rustc = spawnSync(process.env.RUSTC ?? 'rustc', ['--version'], {encoding:'utf8', timeout:10000});
if (rustc.error) throw rustc.error;
assert.equal(rustc.status, 0);
const summary = { scope: 'OptionsService only; full terminal equivalence NOT PROVEN', command: 'node scripts/check-options.mjs',
  platform: process.platform, arch: process.arch, node: process.version, rustc: rustc.stdout.trim(),
  originalCases: cases.length, originalAssertions: assertions, optionKeys: keys.length, seed, comparedOutputLines: actual.length,
  setterOperations: operations.length - constructorOperations - 2, constructorOperations, defaultComparisons: keys.length, lifecycleScenarios: 1, mismatches: 0,
  sourceSha256: createHash('sha256').update(readFileSync('source_repo/src/common/services/OptionsService.ts')).digest('hex'),
  operationsSha256: createHash('sha256').update(operations.join('\n')).digest('hex'),
  outputSha256: createHash('sha256').update(actual.join('\n')).digest('hex') };
writeFileSync('results/options/summary.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
