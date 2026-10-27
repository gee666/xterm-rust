// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Executes read-only upstream implementations and assertions.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { Disposable, DisposableStore, toDisposable } from '../source_repo/src/common/Lifecycle.ts';

process.chdir(resolve(import.meta.dirname, '..'));
mkdirSync('results/core-services', { recursive: true });
const sourceFiles = [
  'source_repo/src/common/Lifecycle.ts', 'source_repo/src/common/Event.ts',
  'source_repo/src/common/services/OptionsService.ts', 'source_repo/src/common/services/Services.ts',
  'source_repo/src/common/services/CoreService.ts', 'source_repo/src/common/services/CoreService.test.ts',
  'source_repo/src/common/services/LogService.ts',
];
function load(path, names, values, returned) {
  const source = readFileSync(path, 'utf8').replace(/^import .*;\r?$/gm, '')
    .replace(/@I(?:Buffer|Log|Options)Service\s+/g, '').replace(/\bexport /g, '');
  return new Function(...names, stripTypeScriptTypes(source, { mode: 'transform' }) + `\nreturn {${returned}};`)(...values);
}
const { Emitter } = load(sourceFiles[1], ['DisposableStore', 'toDisposable'], [DisposableStore, toDisposable], 'Emitter');
const { OptionsService, DEFAULT_OPTIONS } = load(sourceFiles[2], ['Disposable', 'toDisposable', 'Emitter', 'isMac'],
  [Disposable, toDisposable, Emitter, false], 'OptionsService, DEFAULT_OPTIONS');
const enumSource = readFileSync(sourceFiles[3], 'utf8').match(/export enum LogLevelEnum \{[^}]+\}/)[0].replace('export ', '');
const LogLevelEnum = new Function(stripTypeScriptTypes(enumSource, { mode: 'transform' }) + '\nreturn LogLevelEnum;')();
const { CoreService } = load(sourceFiles[4], ['Disposable', 'Emitter'], [Disposable, Emitter], 'CoreService');
let records = [];
const hex = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',');
const unhex = text => text ? String.fromCharCode(...text.split(',').map(unit => parseInt(unit, 16))) : '';
const levels = ['trace', 'debug', 'info', 'warn', 'error'];
const consoleSink = Object.fromEntries(['log', 'info', 'warn', 'error'].map(route => [route, (text, ...params) => {
  const level = activeLevel;
  records.push(`L:${level}:${route}:${hex(text)}:${params.map(units => units.map(unit => unit.toString(16)).join(',')).join('/')}`);
}]));
let activeLevel = 0;
const { LogService } = load(sourceFiles[6], ['Disposable', 'LogLevelEnum', 'console'], [Disposable, LogLevelEnum, consoleSink], 'LogService');

// The three original tests only read constructor options and cursor state.
// Their mock fixtures are represented by the exact fields those tests consume;
// original test bodies and assertions are not edited.
const tests = [], contexts = [];
const describe = (_name, callback) => { contexts.push([]); callback(); contexts.pop(); };
const it = (name, callback) => tests.push({ name, callback, before: contexts.flat() });
let assertions = 0;
const countedAssert = { equal(...args) { assertions++; assert.equal(...args); } };
class MockOptionsService { constructor(value = {}) { this.rawOptions = { ...DEFAULT_OPTIONS, ...value }; } }
class MockBufferService { constructor() { this.buffer = { ybase: 0, ydisp: 0 }; } }
class MockLogService {}
const testSource = readFileSync(sourceFiles[5], 'utf8').replace(/^import .*;\r?$/gm, '');
new Function('CoreService', 'MockBufferService', 'MockLogService', 'MockOptionsService', 'assert', 'describe', 'it', 'beforeEach',
  stripTypeScriptTypes(testSource, { mode: 'transform' }))(CoreService, MockBufferService, MockLogService, MockOptionsService,
  countedAssert, describe, it, callback => contexts.at(-1).push(callback));
for (const test of tests) { for (const before of test.before) before(); test.callback(); }
assert.equal(tests.length, 3); assert.equal(assertions, 4);

const options = new OptionsService({});
const logger = new LogService(options);
const buffer = { buffer: { ybase: 0, ydisp: 0 } };
const core = new CoreService(buffer, logger, options);
core.onData(text => records.push(`D:${hex(text)}`));
core.onBinary(text => records.push(`B:${hex(text)}`));
core.onUserInput(() => records.push('user'));
core.onRequestScrollToBottom(() => records.push('scroll'));
const customSink = Object.fromEntries(levels.map((name, level) => [name, (text, ...params) => {
  records.push(`L:${level}:custom:${hex(text)}:${params.map(units => units.map(unit => unit.toString(16)).join(',')).join('/')}`);
}]));
// Default console.log is shared by debug/trace; a proxy records the caller level
// without modifying the source log implementations.
for (const [level, name] of levels.entries()) {
  const original = logger[name].bind(logger);
  logger[name] = (...args) => { activeLevel = level; return original(...args); };
}
function snapshot() {
  const modes = core.decPrivateModes, kitty = core.kittyKeyboard;
  const bits = ['applicationCursorKeys', 'applicationKeypad', 'bracketedPasteMode', 'colorSchemeUpdates', 'origin',
    'reverseWraparound', 'sendFocus', 'synchronizedOutput', 'win32InputMode', 'wraparound'].map(key => +modes[key]).join('');
  const stack = array => `[${array.join(', ')}]`;
  return `${+core.isCursorInitialized},${+core.isCursorHidden},${+core.modes.insertMode}|${bits}:${modes.cursorBlink === undefined ? 'u' : modes.cursorBlink ? 't' : 'f'}:${modes.cursorStyle === undefined ? 'u' : ['block', 'underline', 'bar'].indexOf(modes.cursorStyle)}|${kitty.flags},${kitty.mainFlags},${kitty.altFlags}:${stack(kitty.mainStack)}:${stack(kitty.altStack)}`;
}
const operations = [], expected = [];
function operation(...fields) {
  operations.push(fields.join('\t'));
  switch (fields[0]) {
    case 'O': {
      const [, key, kind, value] = fields;
      options.options[key] = ({ u: () => undefined, z: () => null, b: () => value === '1', n: () => Number(value), s: () => unhex(value) })[kind]();
      break;
    }
    case 'P': buffer.buffer = { ybase: +fields[1], ydisp: +fields[2] }; break;
    case 'D': core.triggerDataEvent(unhex(fields[2]), fields[1] === '1'); break;
    case 'B': core.triggerBinaryEvent(unhex(fields[1])); break;
    case 'L': logger[levels[+fields[1]]](unhex(fields[2])); break;
    case 'C': options.options.logger = fields[1] === '1' ? customSink : null; break;
    case 'E': {
      const failed = { [Symbol.toPrimitive](hint) { assert.equal(hint, 'string'); throw new Error('\ud800\0'); } };
      assert.throws(() => { options.options.logLevel = failed; }, error => {
        records.push(`error:${hex(error.message)}`); return true;
      });
      break;
    }
    case 'R': core.reset(); break;
    case 'M': {
      core.isCursorInitialized = true; core.isCursorHidden = true; core.modes.insertMode = true;
      for (const key of Object.keys(core.decPrivateModes)) core.decPrivateModes[key] = true;
      core.decPrivateModes.wraparound = false; core.decPrivateModes.cursorStyle = 'underline';
      core.kittyKeyboard = { flags: 0xffffffff, mainFlags: 7, altFlags: 8, mainStack: [1, 2], altStack: [3, 4] };
      break;
    }
    case 'X': core.dispose(); break;
    case 'Y': logger.dispose(); break;
    default: assert.fail('unknown command');
  }
  expected.push(`${snapshot()}\t${records.join(';')}`); records = [];
}
const option = (key, kind, value = '') => operation('O', key, kind, value);
operation('R'); operation('M'); operation('R');
option('logLevel', 's', hex('trace'));
for (const disabled of [0, 1]) for (const user of [0, 1]) for (const scroll of [0, 1]) for (const [base, display] of [[0, 0], [5, 5], [5, 2], [2, 5]]) {
  option('disableStdin', 'b', String(disabled)); option('scrollOnUserInput', 'b', String(scroll));
  operation('P', String(base), String(display));
  for (const text of ['', '\0', '\ud800', '\udfff', '😀', '\u2028\ufffe']) {
    operation('D', String(user), hex(text)); operation('B', hex(text));
  }
}
for (const custom of [0, 1]) {
  operation('C', String(custom));
  for (const level of [...levels, 'off', 'bad', 'toString', 'constructor', '\ud800']) {
    option('logLevel', 's', hex(level));
    for (let sent = 0; sent < 5; sent++) operation('L', String(sent), hex('😀\0\ud800'));
  }
}
const seed = 0x434f5245;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
for (let i = 0; i < 20000; i++) {
  const command = random() % 10;
  if (command < 3) {
    const text = String.fromCharCode(...Array.from({ length: random() % 20 }, () => random() & 0xffff));
    operation(command === 0 ? 'B' : 'D', ...(command === 0 ? [hex(text)] : [String(random() & 1), hex(text)]));
  } else if (command === 3) option('disableStdin', 'b', String(random() & 1));
  else if (command === 4) option('scrollOnUserInput', 'b', String(random() & 1));
  else if (command === 5) operation('P', String(random() % 1000), String(random() % 1000));
  else if (command === 6) option('logLevel', 's', hex([...levels, 'off', 'bad'][random() % 7]));
  else if (command === 7) operation('C', String(random() & 1));
  else operation(command === 8 ? 'R' : 'M');
}
option('logLevel', 's', hex('info')); operation('E'); operation('L', '2', hex('after conversion failure'));
option('logLevel', 's', hex('trace'));
operation('Y'); option('logLevel', 's', hex('error')); operation('L', '4', hex('after log disposal'));
operation('X'); operation('D', '1', hex('after core disposal')); operation('B', '');
const build = spawnSync(process.env.CARGO ?? 'cargo', ['build', '-p', 'xterm-core', '--example', 'core_service_oracle'], { encoding: 'utf8', timeout: 120000 });
writeFileSync('results/core-services/build.log', `${build.stdout ?? ''}${build.stderr ?? ''}`);
if (build.error) throw build.error; assert.equal(build.status, 0, build.stderr);
const binary = resolve('target/debug/examples/core_service_oracle' + (process.platform === 'win32' ? '.exe' : ''));
const run = spawnSync(binary, [], { input: operations.join('\n') + '\n', encoding: 'utf8', timeout: 120000, maxBuffer: 32 * 1024 * 1024 });
if (run.error) throw run.error; assert.equal(run.status, 0, run.stderr);
const actual = run.stdout.replace(/\r?\n$/, '').split(/\r?\n/);
assert.equal(actual.length, expected.length);
for (let index = 0; index < actual.length; index++) if (actual[index] !== expected[index]) {
  writeFileSync('results/core-services/counterexample.json', JSON.stringify({ seed, index, operation: operations[index], expected: expected[index], actual: actual[index] }, null, 2));
  writeFileSync('results/core-services/counterexample-input.tsv', operations.slice(0, index + 1).join('\n') + '\n');
  assert.fail(`mismatch at ${index}; see results/core-services/counterexample.json`);
}
const sha256 = data => createHash('sha256').update(data).digest('hex');
const rustc = spawnSync(process.env.RUSTC ?? 'rustc', ['--version'], { encoding: 'utf8', timeout: 10000 });
if (rustc.error) throw rustc.error; assert.equal(rustc.status, 0);
const summary = { scope: 'CoreService/LogService only; terminal integration and full equivalence NOT PROVEN',
  command: 'node scripts/check-core-services.mjs', platform: process.platform, arch: process.arch,
  node: process.version, rustc: rustc.stdout.trim(), originalCases: tests.length, originalAssertions: assertions,
  originalMockAdaptation: 'Three constructor-only tests use only rawOptions defaults and supplied showCursorImmediately; mock buffer/log fields not consulted by these assertions.',
  seed, differentialOperations: actual.length, randomizedOperations: 20000, mismatches: 0,
  sources: sourceFiles.map(path => ({ path, sha256: sha256(readFileSync(path)) })),
  implementation: ['crates/xterm-core/src/core_service.rs', 'crates/xterm-core/src/log_service.rs',
    'crates/xterm-core/examples/core_service_oracle.rs', 'scripts/check-core-services.mjs'].map(path => ({ path, sha256: sha256(readFileSync(path)) })),
  operationsSha256: sha256(operations.join('\n')), outputSha256: sha256(actual.join('\n')) };
writeFileSync('results/core-services/summary.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
