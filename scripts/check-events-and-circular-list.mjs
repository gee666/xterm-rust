// Copyright (c) 2016-2026 The xterm.js authors. Licensed under MIT.
// Runs the original assertions unchanged after TypeScript erasure.
import assert from 'node:assert';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';
import { directedChecks } from './events-circular-directed.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const directory = path.join(root, 'results/events-circular-list');
mkdirSync(directory, { recursive: true });
const files = ['Lifecycle.ts', 'Event.ts', 'CircularList.ts', 'Event.test.ts', 'CircularList.test.ts'];
const source = Object.fromEntries(files.map(file => [file, readFileSync(path.join(root, 'source_repo/src/common', file), 'utf8')]));
const rustPaths = ['src/event.rs', 'src/event/dispatch.rs', 'src/lifecycle.rs', 'src/circular_list.rs', 'src/circular_list/storage.rs', 'tests/event.rs', 'tests/event_edges.rs', 'tests/event_errors.rs', 'tests/event_context.rs', 'tests/lifecycle_explicit.rs', 'tests/circular_list.rs', 'tests/circular_list_edges.rs', 'examples/event_trace.rs', 'examples/circular_list_trace.rs'].map(file => `crates/xterm-core/${file}`);
const hash = file => createHash('sha256').update(readFileSync(path.join(root, file))).digest('hex');
const rustSource = Object.fromEntries(rustPaths.map(file => [file, hash(file)]));
function erase(text) {
  return stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
}
let assertions = 0;
const chai = {
  equal(actual, expected, message) { assertions++; assert.equal(actual, expected, message); },
  strictEqual(actual, expected, message) { assertions++; assert.strictEqual(actual, expected, message); },
  deepEqual(actual, expected, message) { assertions++; assert.deepStrictEqual(structuredClone(actual), structuredClone(expected), message); },
  throws(callback, message) {
    assertions++;
    let caught;
    try { callback(); } catch (error) { caught = error; }
    assert.ok(caught, 'expected a thrown error');
    assert.ok(caught.message.includes(message), `expected ${JSON.stringify(caught.message)} to include ${JSON.stringify(message)}`);
  }
};
const tests = [];
const names = [];
const context = vm.createContext({
  assert: chai,
  describe(name, callback) { names.push(name); callback(); names.pop(); },
  it(name, callback) { tests.push({ name: [...names, name].join(' / '), callback }); }
});
vm.runInContext(['Lifecycle.ts', 'Event.ts', 'CircularList.ts'].map(file => erase(source[file])).join('\n') + '\nglobalThis.oracle = { Emitter, CircularList, EventUtils, DisposableStore };', context);
for (const file of ['Event.test.ts', 'CircularList.test.ts']) {
  vm.runInContext(`{\n${erase(source[file])}\n}`, context, { filename: file });
}
const outcomes = [];
for (const test of tests) {
  try { test.callback(); outcomes.push({ name: test.name, status: 'passed' }); }
  catch (error) { outcomes.push({ name: test.name, status: 'failed', error: String(error) }); }
}
writeFileSync(path.join(directory, 'upstream-tests.json'), JSON.stringify({ assertions, outcomes }, null, 2) + '\n');
assert.ok(outcomes.every(test => test.status === 'passed'), 'original upstream assertion failure');
const directed = directedChecks(context.oracle);
writeFileSync(path.join(directory, 'directed-source-tests.json'), JSON.stringify(directed, null, 2) + '\n');
assert.ok(directed.every(test => test.status === 'passed'), 'directed source assertion failure');

const seed = 0xc1ac01a7;
let randomState = seed;
function random() { randomState = (Math.imul(randomState, 1664525) + 1013904223) >>> 0; return randomState; }
const circleTrace = [];
const circleExpected = [];
let circle;
let events;
function values() { return Array.from({ length: circle.maxLength * 2 + 2 }, (_, index) => circle.get(index) ?? '_').join(','); }
function circleCommand(command) {
  circleTrace.push(command);
  const [operation, ...fields] = command.split('\t');
  const numbers = fields.map(Number);
  let result = 'ok';
  switch (operation) {
    case 'N':
      circle = new context.oracle.CircularList(numbers[0]); events = [];
      circle.onDelete(change => events.push(`D,${change.index},${change.amount},${circle.length},${values()}`));
      circle.onInsert(change => events.push(`I,${change.index},${change.amount},${circle.length},${values()}`));
      circle.onTrim(amount => events.push(`T,${amount},${circle.length},${values()}`));
      break;
    case 'P': circle.push(numbers[0]); break;
    case 'S': case 'Q': circle.set(numbers[0], fields[1] === '_' ? undefined : numbers[1]); break;
    case 'G': case 'Z': result = circle.get(numbers[0]) ?? '_'; break;
    case 'L': circle.length = numbers[0]; break;
    case 'M': circle.maxLength = numbers[0]; break;
    case 'O': result = circle.pop() ?? '_'; break;
    case 'R': try { result = circle.recycle() ?? '_'; } catch (error) { result = error.message; } break;
    case 'D': circle.splice(...numbers); break;
    case 'T': circle.trimStart(numbers[0]); break;
    case 'H': try { circle.shiftElements(...numbers); } catch (error) { result = error.message; } break;
    case 'X': circle.dispose(); break;
    default: throw new Error(`unknown command ${operation}`);
  }
  circleExpected.push(`${circle.length}|${circle.maxLength}|${circle.isFull}|${result}|${values()}|${events.join(';')}`);
  events = [];
}
for (const command of ['N\t5', ...[1,2,3,4,5].map(value => `P\t${value}`), 'Q\t-1\t99', 'G\t-1', 'T\t1', 'G\t-1', 'Q\t-1\t88', 'T\t5', 'G\t-1', ...[6,7,8,9,10,11].map(value => `P\t${value}`), 'G\t-2', 'M\t6', 'G\t-1']) circleCommand(command);
for (const command of ['N\t5', ...[1,2,3,4,5,6].map(value => `P\t${value}`)]) circleCommand(command);
for (const index of [NaN, Infinity, -Infinity, '-0', 0, 0.5, -0.5, Number.MIN_VALUE, Number.EPSILON, 2**53-1, 2**53, 2**53+2, Number.MAX_SAFE_INTEGER, Number.MAX_VALUE, 1e100, 1e300, -1e100, -1e300]) circleCommand(`Z\t${index}`);
for (let index = 0; index < 20000; index++) {
  if (index % 257 === 0) { circleCommand(`N\t${random() % 12 + 1}`); continue; }
  const value = random() % 1000 - 500;
  switch (random() % 15) {
    case 0: case 1: circleCommand(`P\t${value}`); break;
    case 2: circleCommand(`S\t${random() % 40}\t${random() % 3 ? value : '_'}`); break;
    case 3: circleCommand(`L\t${random() % (circle.maxLength + 4)}`); break;
    case 4: circleCommand(`M\t${random() % 12 + 1}`); break;
    case 5: if (circle.length) circleCommand('O'); else circleCommand('R'); break;
    case 6: circleCommand('R'); break;
    case 7: {
      const start = random() % (circle.length + 1);
      const count = random() % (circle.length - start + 1);
      const items = Array.from({ length: random() % 5 }, () => random() % 1000 - 500);
      circleCommand(['D', start, count, ...items].join('\t')); break;
    }
    case 8: circleCommand(`T\t${random() % 20}`); break;
    case 9: case 10:
      circleCommand(`H\t${random() % (circle.length + 2) - 1}\t${random() % 5}\t${random() % 9 - 2}`); break;
    case 11: circleCommand('X'); break;
    case 12: circleCommand(`Q\t${random() % 41 - 20}\t${value}`); break;
    case 13: circleCommand(`G\t${random() % 41 - 20}`); break;
    case 14: circleCommand(`Z\t${(random() % 1000 - 500) / 8}`); break;
  }
}

const eventTrace = [];
const eventExpected = [];
let emitter;
let handles;
let log;
function subscribe(id, action, target) {
  handles.set(id, emitter.event(value => {
    log.push(`${id}:${value}`);
    switch (action) {
      case 'normal': break;
      case 'remove': handles.get(target)?.dispose(); break;
      case 'add': subscribe(target, 'normal'); break;
      case 'nested': if (value === 1) emitter.fire(2); break;
      case 'dispose': emitter.dispose(); break;
      default: throw new Error(`unknown action ${action}`);
    }
  }));
}
function eventCommand(command) {
  eventTrace.push(command);
  const [operation, id, action, target] = command.split('\t');
  let result = 'ok';
  switch (operation) {
    case 'N': emitter = new context.oracle.Emitter(); handles = new Map(); log = []; break;
    case 'A': subscribe(Number(id), action, Number(target)); break;
    case 'K': handles.get(Number(id))?.dispose(); break;
    case 'X': emitter.dispose(); break;
    case 'F': try { emitter.fire(Number(id)); } catch (error) {
      assert.equal(error.name, 'TypeError'); result = 'error';
    } break;
    default: throw new Error(`unknown command ${operation}`);
  }
  eventExpected.push(`${result}|${log.join(',')}`); log = [];
}
// Ensure the subtle disposal and copy-on-write paths are not left to chance.
for (const command of ['N', 'A\t1\tdispose', 'A\t2\tnormal', 'F\t1', 'N', 'A\t1\tadd\t3', 'A\t2\tdispose', 'F\t1']) eventCommand(command);
for (let index = 0; index < 10000; index++) {
  if (index % 97 === 0) { eventCommand('N'); continue; }
  const id = random() % 20;
  switch (random() % 8) {
    case 0: case 1: case 2: {
      const action = ['normal', 'remove', 'add', 'nested', 'dispose'][random() % 5];
      eventCommand(`A\t${id}\t${action}\t${random() % 20}`); break;
    }
    case 3: eventCommand(`K\t${id}`); break;
    case 4: case 5: case 6: eventCommand(`F\t${random() % 4}`); break;
    case 7: eventCommand('X'); break;
  }
}
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 64 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}\n${result.stdout}\n${result.stderr}`);
  return result.stdout + (options.includeStderr ? result.stderr : '');
}
const cargo = process.env.CARGO || 'cargo';
const rustc = process.env.RUSTC || 'rustc';
const isolated = process.argv.includes('--isolated');
const suffix = process.platform === 'win32' ? '.exe' : '';
// In-flight workspace edits must not prevent independently checking these exact
// modules. This optional mode creates only small, nonincremental files in tmp/;
// it is recorded as isolated evidence, never a workspace-quality pass.
const temporary = path.join(root, 'tmp');
mkdirSync(temporary, { recursive: true });
const library = path.join(temporary, 'libevents_circular_list.rlib');
const extern = ['--edition=2021', '--extern', `xterm_core=${library}`, '-Dwarnings'];
if (isolated) {
  const entry = path.join(temporary, 'events_circular_lib.rs');
  writeFileSync(entry, ['lifecycle', 'event', 'circular_list'].map(name => `#[path = "../crates/xterm-core/src/${name}.rs"] pub mod ${name};`).join('\n'));
  run(rustc, ['--edition=2021', '--crate-name', 'xterm_core', '--crate-type', 'rlib', entry, '-o', library, '-Dwarnings']);
  for (const name of ['event_trace', 'circular_list_trace']) run(rustc, [...extern, `crates/xterm-core/examples/${name}.rs`, '-o', path.join(temporary, name + suffix)]);
} else {
  run(cargo, ['build', '-p', 'xterm-core', '--example', 'event_trace', '--example', 'circular_list_trace']);
}
for (const [name, trace, expected] of [['event_trace', eventTrace, eventExpected], ['circular_list_trace', circleTrace, circleExpected]]) {
  const binary = isolated ? path.join(temporary, name + suffix) : path.join(process.env.CARGO_TARGET_DIR || path.join(root, 'target'), 'debug/examples', name + suffix);
  const actual = run(binary, [], { input: trace.join('\n') + '\n' }).trimEnd().split(/\r?\n/);
  const mismatch = expected.findIndex((line, index) => line !== actual[index]);
  if (mismatch !== -1 || actual.length !== expected.length) {
    const index = mismatch === -1 ? Math.min(actual.length, expected.length) : mismatch;
    let start = Math.min(index, trace.length - 1);
    while (start > 0 && trace[start] !== 'N' && !trace[start].startsWith('N\t')) start--;
    writeFileSync(path.join(directory, `${name}-counterexample.json`), JSON.stringify({ seed, mismatch: index, traceOffset: start, trace: trace.slice(start, index + 1), expected: expected[index], actual: actual[index] }, null, 2));
    assert.fail(`${name} mismatch at ${index}`);
  }
}
const testNames = ['event', 'event_edges', 'event_errors', 'event_context', 'circular_list', 'circular_list_edges', 'lifecycle_explicit'];
const rustTests = isolated ? testNames.map(name => {
  const binary = path.join(temporary, name + '_tests' + suffix);
  run(rustc, [...extern, '--test', `crates/xterm-core/tests/${name}.rs`, '-o', binary]);
  return run(binary, [], { includeStderr: true });
}).join('\n') : run(cargo, ['test', '-p', 'xterm-core', ...testNames.flatMap(name => ['--test', name])], { includeStderr: true });
const rustCount = [...rustTests.matchAll(/test result: ok\. (\d+) passed/g)].reduce((total, match) => total + Number(match[1]), 0);
assert.ok(rustCount >= 83, 'missing scoped Rust tests');
const rustLog = isolated ? 'rust-tests-isolated.log' : 'rust-tests.log';
writeFileSync(path.join(directory, rustLog), rustTests);
for (const file of rustPaths) assert.equal(hash(file), rustSource[file], `Rust input changed during verification: ${file}`);
const report = {
  rustSource, directed,
  status: 'passed scoped checks; terminal equivalence NOT PROVEN',
  harnessSource: Object.fromEntries(['scripts/check-events-and-circular-list.mjs', 'scripts/events-circular-directed.mjs'].map(file => [file, hash(file)])),
  command: 'node scripts/check-events-and-circular-list.mjs', platform: process.platform,
  node: process.version, rustc: run(rustc, ['--version']).trim(), cargo: run(cargo, ['--version']).trim(),
  buildMode: isolated ? 'isolated exact utility modules, not a workspace check' : 'workspace Cargo build',
  source: Object.fromEntries(files.map(file => [file, createHash('sha256').update(source[file]).digest('hex')])),
  upstream: { discovered: tests.length, executed: tests.length, passed: outcomes.filter(test => test.status === 'passed').length, failed: 0, skipped: 0, unrun: 0, assertions, outcomes },
  rust: { tests: rustCount, upstreamMapped: 26, added: rustCount - 26, log: rustLog },
  differential: { seed, circleIterations: 20000, circleComparedStates: circleExpected.length, eventIterations: 10000, eventComparedTraces: eventExpected.length, failures: 0, comparison: 'all wrapped slots, length, capacity, fullness, operation result, event order/payload and event-time storage snapshots; listener traces, nested dispatch, additions/removals and disposal errors' },
  differences: [
    'CircularList requires a positive capacity. Zero capacity is rejected rather than constructing the degenerate upstream array.',
    'Canonical lengths/indexes are usize; get_signed/set_signed preserve negative property slots and get_number preserves public f64 indexing. Empty pop and invalid splice ranges are rejected instead of negative lengths.',
    'Captured closures replace thisArgs. Rust events borrow payloads. EmitError distinguishes cleared listener arrays from retained concrete callback errors, including browser JsValue wrappers.',
    'Cloning Rust elements copies values. Rc elements preserve JavaScript mutable object identity.',
    'EventUtils.any combines a homogeneous Rust payload type; erase payloads explicitly for the heterogeneous JavaScript void overload.'
  ]
};
writeFileSync(path.join(directory, isolated ? 'verification-isolated.json' : 'verification.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
