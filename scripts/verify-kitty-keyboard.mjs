#!/usr/bin/env node
// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Uses the legacy keyboard verifier's TS-erasure method.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import os from 'node:os';

const root = fileURLToPath(new URL('../', import.meta.url));
const sourcePath = 'source_repo/src/common/input/KittyKeyboard.ts';
const testPath = sourcePath.replace('.ts', '.test.ts');
const outputDir = path.join(root, 'results/kitty-keyboard');
mkdirSync(outputDir, { recursive: true });
const read = name => readFileSync(path.join(root, name), 'utf8');
function erase(source) {
  return stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
}
assert.equal(process.versions.unicode, '17.0', 'physical-key lowercase reference requires Unicode 17.0');
const tests = [];
const groups = [];
const captured = [];
const protocolCases = [];
let activeName;
let assertions = 0;
const context = vm.createContext({
  assert: { strictEqual: (actual, expected) => { assertions++; assert.strictEqual(actual, expected); } },
  describe: (name, run) => {
    groups.push({ name, hooks: [] });
    run();
    groups.pop();
  },
  beforeEach: run => groups.at(-1).hooks.push(run),
  it: (name, run) => tests.push({ name: [...groups.map(g => g.name), name].join(' / '),
    hooks: groups.flatMap(g => g.hooks), run })
});
const resultEnum = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum, 'KeyboardResultType not found');
vm.runInContext(erase(read('source_repo/src/common/data/EscapeSequences.ts')) + '\n' + erase(resultEnum) + '\n' +
  erase(read(sourcePath)) + '\nthis.KittyKeyboard = KittyKeyboard;', context);
const Kitty = context.KittyKeyboard;
const evaluate = Kitty.prototype.evaluate;
const shouldUseProtocol = Kitty.shouldUseProtocol;
Kitty.prototype.evaluate = function(event, flags, eventType = 1, mac = false) {
  const result = evaluate.call(this, event, flags, eventType, mac);
  if (activeName) captured.push({ name: activeName, event, flags, eventType, mac, result });
  return result;
};
Kitty.shouldUseProtocol = flags => {
  const result = shouldUseProtocol(flags);
  if (activeName) protocolCases.push({ name: activeName, flags, result });
  return result;
};
vm.runInContext(erase(read(testPath)), context);
for (const test of tests) {
  activeName = test.name;
  test.hooks.forEach(run => run());
  test.run();
}
activeName = undefined;
assert.equal(assertions, captured.length + protocolCases.length, 'all upstream assertions must map to Rust inputs');
const hex = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',');
function eventRow(item) {
  const e = item.event;
  const modifiers = Number(e.shiftKey) | Number(e.altKey) << 1 | Number(e.ctrlKey) << 2 | Number(e.metaKey) << 3;
  return [e.keyCode, hex(e.key), hex(e.code), modifiers, item.flags, item.eventType, Number(item.mac)].join('\t');
}
function resultRow(result) {
  return [result.type, Number(result.cancel), result.key === undefined ? '-' : hex(result.key)].join('\t');
}
function fixture(name, contents) {
  const target = path.join(root, 'crates/xterm-core/tests/fixtures', name);
  if (process.argv.includes('--generate')) {
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, contents);
  } else {
    assert.equal(readFileSync(target, 'utf8'), contents, `${name} reference drift; review before --generate`);
  }
}
fixture('kitty_upstream.tsv', captured.map(item => `${eventRow(item)}\t${resultRow(item.result)}\t${item.name}`).join('\n') + '\n');
fixture('kitty_protocol_upstream.tsv', protocolCases.map(item => `${item.flags}\t${Number(item.result)}\t${item.name}`).join('\n') + '\n');

const seed = 0x4b495454;
let state = seed;
function random() {
  state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
  return state;
}
const cases = [...captured];
const kitty = new Kitty();
const categories = { upstream: captured.length };
function add(key, code, modifiers, flags, eventType, mac, name) {
  const event = { keyCode: 0, key, code, shiftKey: !!(modifiers & 1), altKey: !!(modifiers & 2),
    ctrlKey: !!(modifiers & 4), metaKey: !!(modifiers & 8), type: eventType === 3 ? 'keyup' : 'keydown' };
  cases.push({ event, flags, eventType, mac, name, result: evaluate.call(kitty, event, flags, eventType, mac) });
  categories[name] = (categories[name] || 0) + 1;
}
const mappedKeys = Object.keys(kitty._functionalKeyCodes).concat(Object.keys(kitty._csiTildeKeys),
  Object.keys(kitty._csiLetterKeys), Object.keys(kitty._ss3FunctionKeys));
const keys = [...new Set([...mappedKeys, 'Shift', 'Alt', 'Control', 'Meta', 'Dead', 'Unidentified',
  '', 'a', 'A', ' ', 'é', '中', '😀', '\ud800', '\udfff', '\0',
  ...Array.from({ length: 10 }, (_, i) => `F${26 + i}`), ...Object.getOwnPropertyNames(Object.prototype)])];
for (const key of keys) {
  for (let modifiers = 0; modifiers < 16; modifiers++) {
    for (let flags = 0; flags < 32; flags++) {
      for (let type = 1; type <= 3; type++) add(key, '', modifiers, flags, type, false, 'key/modifier/flag/event matrix');
    }
  }
}
const codes = ['KeyA', 'KeyQ', 'Keyİ', 'KeyΣ', 'Key\ud800', 'Key😀', 'Digit5', 'Digitx',
  'Digit00', 'Numpad0', 'Numpad9', 'NumpadDecimal', 'NumpadDivide', 'NumpadMultiply',
  'NumpadSubtract', 'NumpadAdd', 'NumpadEnter', 'NumpadEqual', 'Numpad8suffix', 'Numpad09',
  'Numpad123', 'Numpad0x10',
  'Numpad', 'Numpad900', 'ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight',
  'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight', 'Semicolon', ''];
for (const code of codes) {
  for (let modifiers = 0; modifiers < 16; modifiers++) {
    for (let flags = 0; flags < 32; flags++) {
      for (let type = 1; type <= 3; type++) {
        for (const mac of [false, true]) add('ƒ', code, modifiers, flags, type, mac, 'physical-code/mac matrix');
      }
    }
  }
}
// Every UTF-16 unit, including lone surrogates, exercises direct text, shifted
// physical-code lowercase, and Option-composed base-key recovery.
for (let unit = 0; unit < 65536; unit++) {
  const key = String.fromCharCode(unit);
  add(key, '', 0, 31, 1, false, 'BMP associated text');
  add(key, `Key${key}`, 1, 31, 2, false, 'BMP shifted lowercase');
  add('Dead', `Key${key}`, 2, 3, 3, true, 'BMP Option base key');
}
for (let i = 0; i < 25000; i++) {
  const modifiers = random() >>> 28;
  const flags = random();
  const type = 1 + (random() >>> 16) % 3;
  const mac = !!(random() & 0x100);
  const key = i % 3 ? keys[random() % keys.length] : String.fromCharCode(random() & 0xffff, random() & 0xffff);
  add(key, codes[random() % codes.length], modifiers, flags, type, mac, 'seeded synthetic events');
}
// Upstream's lexical numpad suffix check accepts more than a single digit.
// Decimal rounding and JavaScript number formatting must survive long suffixes.
for (const suffix of ['00', '08tail', '1e8', '1.5', '1\udfff', '8'.repeat(20), '8'.repeat(309),
  '1234567890123456789012345678901234567890', '900', '9tail', '-1', '']) {
  for (let flags = 0; flags < 32; flags++) add('5', `Numpad${suffix}`, 0, flags, 1, false, 'synthetic numpad suffix');
}

for (let i = 0; i < 5000; i++) {
  const length = 16 + random() % 12;
  let suffix = String(1 + random() % 8);
  for (let digit = 1; digit < length; digit++) suffix += String(random() >>> 16 & 7);
  add('5', `Numpad${suffix}`, 0, 31, 1, false, 'seeded numpad decimal rounding');
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000,
    maxBuffer: 128 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}\n${result.stderr}\n${result.stdout}`);
  return result;
}
const cargo = process.env.CARGO || 'cargo';
run(cargo, ['build', '-p', 'xterm-core', '--example', 'kitty_oracle']);
const targetDir = process.env.CARGO_TARGET_DIR || path.join(root, 'target');
const binary = path.join(targetDir, 'debug/examples', process.platform === 'win32' ? 'kitty_oracle.exe' : 'kitty_oracle');
const actual = run(binary, [], { input: cases.map(eventRow).join('\n') + '\n' }).stdout.trimEnd().split(/\r?\n/);
assert.equal(actual.length, cases.length);
const mismatches = [];
let failures = 0;
for (let i = 0; i < cases.length; i++) {
  const expected = resultRow(cases[i].result);
  if (actual[i] !== expected) {
    failures++;
    if (mismatches.length < 20) mismatches.push({ index: i, case: cases[i], actual: actual[i], expected });
  }
}
const sha256 = name => createHash('sha256').update(readFileSync(path.join(root, name))).digest('hex');
const report = { command: 'node scripts/verify-kitty-keyboard.mjs', platform: process.platform, arch: process.arch,
  osRelease: os.release(), cpu: os.cpus()[0]?.model, node: process.version, unicode: process.versions.unicode,
  rustc: run(process.env.RUSTC || 'rustc', ['--version']).stdout.trim(), seed,
  sourceHashes: { [sourcePath]: sha256(sourcePath), [testPath]: sha256(testPath) },
  upstream: { discovered: tests.length, executed: tests.length, passed: tests.length, failed: 0, skipped: 0, unrun: 0,
    assertions, migratedEvaluationAssertions: captured.length, migratedProtocolAssertions: protocolCases.length,
    names: tests.map(test => test.name) },
  differential: { executed: cases.length, passed: cases.length - failures, failed: failures,
    retainedMismatches: mismatches.length, categories },
  scope: 'KittyKeyboard.ts portable policy only. Browser event wiring, protocol state negotiation and terminal equivalence NOT PROVEN.' };
writeFileSync(path.join(outputDir, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
if (mismatches.length) writeFileSync(path.join(outputDir, 'counterexamples.json'), JSON.stringify({ seed, mismatches }, null, 2) + '\n');
assert.equal(failures, 0, `Kitty differential failed; see ${outputDir}/counterexamples.json`);
console.log(JSON.stringify({ upstreamTests: tests.length, assertions, differential: cases.length, seed }));
