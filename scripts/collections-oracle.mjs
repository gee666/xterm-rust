// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Type erasure changes no upstream assertion bodies.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { readFileSync, mkdirSync, writeFileSync, existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../', import.meta.url));
const resultDir = path.join(root, 'results/collections');
mkdirSync(resultDir, { recursive: true });
const reportPath = path.join(resultDir, 'oracle.json');
const previousReport = existsSync(reportPath) ? readFileSync(reportPath, 'utf8') : undefined;
const scriptHash = createHash('sha256').update(readFileSync(fileURLToPath(import.meta.url))).digest('hex');
try {
const files = ['SortedList.ts', 'SortedList.test.ts', 'MultiKeyMap.ts', 'MultiKeyMap.test.ts'];
const sources = Object.fromEntries(files.map(file => [file, readFileSync(path.join(root, 'source_repo/src/common', file), 'utf8')]));
function erase(source) {
  return stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
}
// Upstream unit tests only flush on reads. Keep queued callbacks deferred; no
// real browser idle scheduler is involved and scheduling is not claimed tested.
class IdleTaskQueue {
  tasks = [];
  enqueue(task) { this.tasks.push(task); }
  clear() { this.tasks = []; }
  flush() { for (const task of this.tasks.splice(0)) task(); }
}
let assertions = 0;
const chai = {
  strictEqual(a, b, message) { assertions++; assert.equal(a, b, message); },
  deepStrictEqual(a, b, message) {
    assertions++;
    assert.deepStrictEqual(structuredClone(a), structuredClone(b), message);
  }
};
const tests = [];
const names = [];
const hooks = [];
const context = vm.createContext({
  IdleTaskQueue, MockLogService: class {}, assert: chai,
  describe(name, callback) {
    names.push(name); hooks.push([]); callback(); hooks.pop(); names.pop();
  },
  beforeEach(callback) { hooks.at(-1).push(callback); },
  it(name, callback) { tests.push({ name: [...names, name].join(' / '), callback, hooks: hooks.flat() }); }
});
vm.runInContext(`${erase(sources['SortedList.ts'])}\n${erase(sources['MultiKeyMap.ts'])}\nglobalThis.implementations = { SortedList, TwoKeyMap, FourKeyMap };`, context);
for (const file of ['SortedList.test.ts', 'MultiKeyMap.test.ts']) {
  // Test suites each have their own top-level `strictEqual` binding.
  vm.runInContext(`{\n${erase(sources[file])}\n}`, context, { filename: file });
}
for (const test of tests) {
  for (const hook of test.hooks) hook();
  test.callback();
}

const seed = 0x50a7ed;
let state = seed;
function random() { state = (Math.imul(state, 1664525) + 1013904223) >>> 0; return state; }
const list = new context.implementations.SortedList(value => value, {});
const trace = [];
const expected = [];
function operation(command, value) {
  trace.push(value === undefined ? command : `${command}\t${value}`);
  try {
    if (command === 'I') list.insert(value);
    if (command === 'D') expected.push(list.delete(value));
    if (command === 'C') list.clear();
    if (command === 'V') expected.push(Array.from(list.values()));
    if (command === 'K') expected.push(Array.from(list.getKeyIterator(value)));
  } catch (error) {
    expected.push(error.name);
  }
}
// Retain the independent review counterexample and failed-flush recovery.
for (const [command, value] of [
  ['I', 1], ['I', 2], ['I', 3], ['D', 1], ['D', 1], ['D', 2], ['V'],
  ['C'], ['I', 1], ['D', 1], ['D', 1], ['V'], ['V'], ['I', 2], ['V'], ['C'], ['V'],
]) operation(command, value);
for (let i = 0; i < 5000; i++) {
  const value = random() % 31 - 15;
  switch (random() % 7) {
    case 0: case 1: case 2: operation('I', value); break;
    case 3: operation('D', value); break;
    case 4: operation('K', value); break;
    case 5: operation('V'); break;
    case 6: operation('C'); break;
  }
}
operation('V');
const map = new context.implementations.TwoKeyMap();
const numbers = [-0, NaN, Infinity, -Infinity, 1e-7, 1e-6, 1e20, 1e21, 0.1 + 0.2, 4294967295];
for (let i = 0; i < 1000; i++) {
  const sign = random() & 1 ? 1 : -1;
  numbers.push(sign * random() * 10 ** (random() % 40 - 20));
}
for (const number of numbers) {
  map.set(number, 'key', 1);
  trace.push(`N\t${number}`);
  trace.push(`S\t${String(number)}`);
  expected.push(map.get(String(number), 'key') !== undefined);
  map.clear();
}
// Exercise raw IEEE-754 bit patterns without a decimal round trip. The four
// key positions are checked independently against the original FourKeyMap.
const bits = [0n, 0x8000000000000000n, 1n, 0x0010000000000000n, 0x7fefffffffffffffn,
  0x7ff0000000000000n, 0xfff0000000000000n, 0x7ff0000000000001n, 0xfff8000000000001n];
for (let i = 0; i < 4096; i++) bits.push(BigInt(random()) << 32n | BigInt(random()));
const bytes = new DataView(new ArrayBuffer(8));
const four = new context.implementations.FourKeyMap();
for (const bit of bits) {
  bytes.setBigUint64(0, bit, false);
  const value = bytes.getFloat64(0, false);
  const hex = bit.toString(16);
  const text = String(value);
  map.set(value, 'key', 1);
  trace.push(`B\t${hex}`, `S\t${text}`);
  expected.push(map.get(text, 'key') !== undefined);
  map.clear();
  for (let position = 0; position < 4; position++) {
    const keys = [1,2,3,4]; keys[position] = value;
    const query = ['1','2','3','4']; query[position] = text;
    four.set(...keys, 1);
    trace.push(`Q\t${hex}\t${position}\t${text}`);
    expected.push(four.get(...query) !== undefined);
    four.clear();
  }
}
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}
const cargo = process.env.CARGO || 'cargo';
run(cargo, ['build', '-p', 'xterm-core', '--example', 'collections_trace']);
const binary = path.join(process.env.CARGO_TARGET_DIR || path.join(root, 'target'), 'debug/examples', process.platform === 'win32' ? 'collections_trace.exe' : 'collections_trace');
const actual = run(binary, [], { input: trace.join('\n') + '\n' }).trim().split(/\r?\n/).map(line => JSON.parse(line));
try {
  assert.deepStrictEqual(actual, expected);
} catch (error) {
  const mismatch = expected.findIndex((value, i) => JSON.stringify(value) !== JSON.stringify(actual[i]));
  writeFileSync(path.join(resultDir, 'counterexample.json'), JSON.stringify({ seed, mismatch, trace, expected, actual }, null, 2));
  throw error;
}
const report = {
  status: 'passed scoped collection checks; full equivalence NOT PROVEN',
  command: 'node scripts/collections-oracle.mjs', script_sha256: scriptHash, platform: process.platform,
  node: process.version, cargo: run(cargo, ['--version']).trim(),
  source: Object.fromEntries(files.map(file => [file, createHash('sha256').update(sources[file]).digest('hex')])),
  upstream: { discovered: tests.length, executed: tests.length, passed: tests.length, failed: 0, skipped: 0, unrun: 0, assertions, names: tests.map(test => test.name) },
  differential: { seed, iterations: 5000, map_cases: numbers.length, raw_f64_bit_cases: bits.length, four_key_position_cases: bits.length * 4, trace_operations: trace.length, compared_results: expected.length, failures: 0 },
  gaps: ['Real IdleTaskQueue scheduling is not executed.', 'JavaScript inherited object keys and __proto__ mutation are not yet represented by Rust maps.', 'getKeyIterator uses a Rust borrow and cannot mutate the list during iteration.', 'Task105 still tracks inherited/prototype lookup and mutation semantics; numeric bit and four-position cases now run.']
};
writeFileSync(path.join(resultDir, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
} catch (error) {
  let previous;
  if (previousReport) {
    const digest = createHash('sha256').update(previousReport).digest('hex');
    previous = `archive/oracle-${digest}.json`;
    mkdirSync(path.join(resultDir,'archive'), {recursive:true});
    writeFileSync(path.join(resultDir,previous), previousReport);
  }
  const failure = {status:'failed; prior report is not current evidence', command:'node scripts/collections-oracle.mjs', script_sha256:scriptHash,
    node:process.version,platform:process.platform,error:String(error.stack || error),previous_report:previous};
  writeFileSync(reportPath,JSON.stringify(failure,null,2)+'\n');
  console.error(error);
  process.exitCode=1;
}
