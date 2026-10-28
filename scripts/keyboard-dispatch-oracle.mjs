#!/usr/bin/env node
// Licensed under MIT. Executes original CoreBrowserTerminal method bodies.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import vm from 'node:vm';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { focusedBuild } from './keyboard-dispatch-focused.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const sourcePath = 'source_repo/src/browser/CoreBrowserTerminal.ts';
const resultDir = path.join(root, 'results/keyboard-dispatch');
mkdirSync(resultDir, { recursive: true });
const read = name => readFileSync(path.join(root, name), 'utf8');
function erase(source) {
  return stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
}
const source = read(sourcePath);
const start = source.indexOf('  protected _keyDown(');
const end = source.indexOf('  /**\r\n   * Resizes the terminal.', start);
const unixEnd = source.indexOf('  /**\n   * Resizes the terminal.', start);
const methods = source.slice(start, end < 0 ? unixEnd : end);
assert.ok(start >= 0 && (end >= 0 || unixEnd >= 0), 'source method boundaries missing');
const helperStart = source.indexOf('function wasModifierKeyOnlyEvent(');
assert.ok(helperStart >= 0, 'source modifier helper missing');
const context = vm.createContext({});
const resultEnum = read('source_repo/src/common/Types.ts').match(/export const enum KeyboardResultType\s*\{[^}]*\}/)?.[0];
assert.ok(resultEnum);
const sources = ['source_repo/src/common/data/EscapeSequences.ts', 'source_repo/src/common/input/Keyboard.ts',
  'source_repo/src/common/input/KittyKeyboard.ts', 'source_repo/src/common/input/Win32InputMode.ts'];
vm.runInContext(erase(resultEnum) + '\n' + sources.map(name => erase(read(name))).join('\n') + '\n' +
  erase(`class DispatchReference { ${methods} }\n${source.slice(helperStart)}`) + '\n' +
  'this.Reference = DispatchReference; this.Kitty = KittyKeyboard; this.Win32 = Win32InputMode;', context);
const hex = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join(',');
const unhex = text => text === '' ? '' : text.split(',').map(unit => String.fromCharCode(parseInt(unit, 16))).join('');
const reference = new context.Reference();
reference._keyDownHandled = false;
reference._keyDownSeen = false;
reference._keyPressHandled = false;
reference._unprocessedDeadKey = false;
const kitty = new context.Kitty();
const win32 = new context.Win32();
let actions;
let config;
let flags;
const action = name => actions.push(name);
const customKeyHandler = () => { action('custom'); return !!(config & 512); };
reference._compositionHelper = { keydown: () => { action('composition'); return !!(config & 1024); } };
reference.updateCursorStyle = () => action('cursor');
reference.scrollLines = count => action(`scroll:${count}`);
reference.scrollToBottom = () => action('bottom');
reference.selectAll = () => action('select');
reference._onKey = { fire: event => action(`key:${hex(event.key)}`) };
reference._showCursor = () => action('show');
reference.focus = () => action('focus');
reference.coreService = { triggerDataEvent: (key, user) => action(`data:${Number(user)}:${hex(key)}`) };
reference.textarea = { set value(value) { assert.equal(value, ''); action('clear'); } };
reference._keyboardService = {
  get useWin32InputMode() { return !!(config & 1) && !!(config & 2); },
  get useKitty() { return !!(config & 4) && context.Kitty.shouldUseProtocol(flags); },
  evaluateKeyDown(event) {
    if (this.useWin32InputMode) return win32.evaluateKeyboardEvent(event, true);
    if (this.useKitty) return kitty.evaluate(event, flags, event.repeat ? 2 : 1, !!(config & 4096) && !!(config & 32));
    return context.evaluateKeyboardEvent(event, !!(config & 8), !!(config & 4096), !!(config & 32));
  },
  evaluateKeyUp(event) {
    if (this.useWin32InputMode) return win32.evaluateKeyboardEvent(event, false);
    if (this.useKitty && flags & 2) return kitty.evaluate(event, flags, 3, !!(config & 4096) && !!(config & 32));
  }
};
function expected(row) {
  const p = row.split('\t');
  config = Number(p[8]); flags = Number(p[9]); actions = [];
  reference._customKeyEventHandler = config & 8192 ? undefined : customKeyHandler;
  reference.browser = { isMac: !!(config & 16), isWindows: !!(config & 64) };
  reference.options = { macOptionIsMeta: !!(config & 32), screenReaderMode: !!(config & 128), scrollOnUserInput: !!(config & 256) };
  reference.optionsService = { rawOptions: reference.options };
  reference.rows = Number(p[10]); reference.buffer = { ybase: Number(p[11]), ydisp: Number(p[12]) };
  const modifiers = Number(p[4]);
  const event = { type: p[0] === 'P' ? 'keypress' : p[0] === 'U' ? 'keyup' : 'keydown',
    keyCode: Number(p[1]), key: unhex(p[2]), code: unhex(p[3]),
    shiftKey: !!(modifiers & 1), altKey: !!(modifiers & 2), ctrlKey: !!(modifiers & 4), metaKey: !!(modifiers & 8),
    repeat: !!(Number(p[5]) & 1), charCode: p[6] === '-' ? undefined : Number(p[6]),
    which: p[7] === '-' ? undefined : Number(p[7]), getModifierState: () => !!(Number(p[5]) & 2),
    preventDefault: () => action('cancel'), stopPropagation: () => {} };
  let returned;
  if (p[0] === 'D') returned = reference._keyDown(event);
  else if (p[0] === 'P') returned = reference._keyPress(event);
  else if (p[0] === 'U') returned = reference._keyUp(event);
  else returned = reference._inputEvent({ data: p[13] === '-' ? undefined : unhex(p[13]),
    inputType: Number(p[14]) & 1 ? 'insertText' : 'insertCompositionText', composed: !!(Number(p[14]) & 2) });
  const state = Number(reference._keyDownHandled) | Number(reference._keyDownSeen) << 1 |
    Number(reference._keyPressHandled) << 2 | Number(reference._unprocessedDeadKey) << 3;
  return [returned === undefined ? 'u' : Number(returned), state, actions.join('|')].join('\t');
}
const rows = [];
const categories = {};
function add(kind, keyCode, key, code, modifiers, bits, charCode, which, options, flags, data, inputBits, name) {
  rows.push([kind, keyCode, hex(key), hex(code), modifiers, bits, charCode ?? '-', which ?? '-', options,
    flags, 24, 40, 0, data === undefined ? '-' : hex(data), inputBits].join('\t'));
  categories[name] = (categories[name] || 0) + 1;
}
const accepted = 512 | 1024;
// Deterministic stateful traces are not reset between operations.
for (const key of ['Dead', 'AltGraph', 'A', 'a', 'Enter', 'ArrowLeft', 'Control', 'Meta', 'PageUp', 'PageDown']) {
  for (let modifiers = 0; modifiers < 16; modifiers++) {
    for (const options of [accepted, accepted | 16, accepted | 64, accepted | 48,
      accepted | 128, accepted | 3, accepted | 4, accepted | 7, accepted | 4096 | 16, accepted | 4096 | 48, accepted | 8192, 0, 512 | 256]) {
      for (const flags of [0, 1, 2, 8, 31]) {
        const code = key === 'Control' ? 'ControlLeft' : key === 'Meta' ? 'MetaLeft' : key === 'A' || key === 'a' ? 'KeyA' : key;
        const keyCode = { Enter: 13, ArrowLeft: 37, Control: 17, Meta: 91, PageUp: 33, PageDown: 34 }[key] ?? 65;
        add('D', keyCode, key, code, modifiers, 0, undefined, undefined, options, flags, undefined, 0, 'named keydown');
        add('P', keyCode, key, code, modifiers, 0, 97, 97, options, flags, undefined, 0, 'following keypress');
        add('I', 0, '', '', 0, 0, undefined, undefined, options, flags, '😀', 3, 'following input');
        add('U', keyCode, key, code, modifiers, 0, undefined, undefined, options, flags, undefined, 0, 'following keyup');
      }
    }
  }
}
for (const code of [0, 16, 37, 47, 48, 81, 224, 65536, 0xffffffff]) {
  for (let options = 0; options < 256; options++) {
    for (let modifiers = 0; modifiers < 16; modifiers++) {
      add('P', code, 'a', '', modifiers, 2, undefined, undefined, accepted | options, 31, undefined, 0, 'third-level keypress boundary');
      add('D', code, 'a', '', modifiers, 2, undefined, undefined, accepted | options, 31, undefined, 0, 'third-level keydown boundary');
    }
  }
}
const seed = 0x44495350;
let state = seed;
function random() { state = (Math.imul(state, 1664525) + 1013904223) >>> 0; return state; }
const keys = ['', 'a', 'A', 'é', '中', '😀', '\ud800', 'Dead', 'AltGraph', 'ArrowUp', 'Enter', 'Control', 'F1', 'Tab'];
for (let i = 0; i < 50000; i++) {
  const kind = ['D', 'P', 'U', 'I'][random() >>> 30];
  const key = keys[(random() >>> 16) % keys.length];
  const charCode = [undefined, 0, 65, 0xd800, 0xffffffff][(random() >>> 16) % 5];
  const which = [undefined, 0, 97, 0xffff, 65536][(random() >>> 16) % 5];
  add(kind, random() >>> 24, key, key === 'Control' ? 'ControlLeft' : 'KeyA', random() >>> 28,
    random() >>> 30, charCode, which, random() >>> 18, random(), i % 5 ? key : undefined,
    random() >>> 30, 'seeded mixed event');
}
const expectedRows = rows.map(expected);
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, encoding: 'utf8', timeout: 120000,
    maxBuffer: 128 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} ${args.join(' ')}\n${result.stderr}`);
  return result;
}
let focused;
let binary;
try {
  if (process.argv.includes('--focused')) {
    focused = focusedBuild(root, resultDir);
    binary = focused.binary;
  } else {
    const build = run(process.env.CARGO || 'cargo', ['build', '-p', 'xterm-web', '--example', 'keyboard_dispatch_oracle']);
    writeFileSync(path.join(resultDir, 'build.log'), build.stdout + build.stderr);
    const target = process.env.CARGO_TARGET_DIR || path.join(root, 'target');
    binary = path.join(target, 'debug/examples', process.platform === 'win32' ? 'keyboard_dispatch_oracle.exe' : 'keyboard_dispatch_oracle');
  }
} catch (error) {
  writeFileSync(path.join(resultDir, 'build-failure.log'), String(error) + '\n');
  throw error;
}
let actual;
try {
  actual = run(binary, [], { input: rows.join('\n') + '\n' }).stdout.trimEnd().split(/\r?\n/);
} finally {
  focused?.cleanup();
}
assert.equal(actual.length, rows.length);
let failed = 0;
const mismatches = [];
for (let i = 0; i < rows.length; i++) {
  if (actual[i] !== expectedRows[i]) {
    failed++;
    if (mismatches.length < 20) mismatches.push({ index: i, input: rows[i], previous: rows.slice(Math.max(0, i - 5), i), expected: expectedRows[i], actual: actual[i] });
  }
}
if (failed) writeFileSync(path.join(resultDir, 'counterexamples.json'), JSON.stringify({ seed, mismatches }, null, 2) + '\n');
const hash = name => createHash('sha256').update(read(name)).digest('hex');
const hashes = Object.fromEntries([sourcePath, ...sources].map(name => [name, hash(name)]));
const productionPaths = [
  'crates/xterm-core/src/input/keyboard.rs', 'crates/xterm-core/src/input/keyboard_lowercase.rs',
  'crates/xterm-core/src/input/kitty_keyboard.rs', 'crates/xterm-core/src/input/kitty_keyboard/keys.rs',
  'crates/xterm-core/src/input/kitty_keyboard/inherited.rs', 'crates/xterm-core/src/input/win32_input_mode.rs',
  'crates/xterm-web/src/input/keyboard.rs', 'crates/xterm-web/src/input/keyboard_dispatch.rs',
  'crates/xterm-web/src/input/keyboard_dispatch_types.rs'
];
const productionHashes = Object.fromEntries(productionPaths.map(name => [name, hash(name)]));
writeFileSync(path.join(resultDir, focused ? 'focused-oracle.json' : 'oracle.json'), JSON.stringify({ command: `node scripts/keyboard-dispatch-oracle.mjs${focused ? ' --focused' : ''}`, buildScope: focused?.scope || 'Full xterm-web Cargo example build',
  platform: process.platform, node: process.version, rustc: run(process.env.RUSTC || 'rustc', ['--version']).stdout.trim(),
  seed, sourceHashes: hashes, productionHashes, executed: rows.length, passed: rows.length - failed, failed, categories,
  originalBrowserSuite: { executed: 0, status: 'Not executed by this source-method runner; tracked separately by browser baseline/e2e tasks.' },
  scope: 'Original method bodies with ordered host effects. Does not execute original Terminal browser suite or establish DOM/terminal integration.' }, null, 2) + '\n');
assert.equal(failed, 0, `dispatch mismatches; see ${resultDir}/counterexamples.json`);
console.log(JSON.stringify({ executed: rows.length, passed: rows.length, seed }));
