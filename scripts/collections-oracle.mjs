// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Type erasure changes no upstream assertion bodies.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../', import.meta.url));
const resultDir = path.join(root, 'results/collections');
mkdirSync(resultDir, { recursive: true });
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
  if (command === 'I') list.insert(value);
  if (command === 'D') expected.push(list.delete(value));
  if (command === 'C') list.clear();
  if (command === 'V') expected.push(Array.from(list.values()));
  if (command === 'K') expected.push(Array.from(list.getKeyIterator(value)));
}
for (let i = 0; i < 5000; i++) {
  const value = random() % 31 - 15;
  switch (random() % 7) {
    case 0: case 1: case 2: operation('I', value); break;
    case 3:
      operation('D', value);
      // Flush after deletion so the trace does not invoke upstream's duplicate
      // pending-index bug. It remains an explicit gap in the result report.
      operation('V');
      break;
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
  command: 'node scripts/collections-oracle.mjs', platform: process.platform,
  node: process.version, cargo: run(cargo, ['--version']).trim(),
  source: Object.fromEntries(files.map(file => [file, createHash('sha256').update(sources[file]).digest('hex')])),
  upstream: { discovered: tests.length, executed: tests.length, passed: tests.length, failed: 0, skipped: 0, unrun: 0, assertions, names: tests.map(test => test.name) },
  differential: { seed, iterations: 5000, map_cases: numbers.length, trace_operations: trace.length, compared_results: expected.length, failures: 0 },
  gaps: ['Real IdleTaskQueue scheduling is not executed.', 'Repeated deletion before a read can trigger upstream sparse-array or RangeError bugs not reproduced in Rust.', 'JavaScript inherited object keys and __proto__ mutation are intentionally not exposed by Rust maps.', 'getKeyIterator uses a Rust borrow and cannot mutate the list during iteration.']
};
writeFileSync(path.join(resultDir, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
