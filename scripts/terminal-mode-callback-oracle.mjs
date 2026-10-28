// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Source-derived callback cases, separate from unchanged upstream assertion counts.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { loadTerminalReference } from './terminal-reference.mjs';
const require = createRequire(import.meta.url);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
await mkdir('results/terminal-modes', { recursive: true });
await writeFile('results/terminal-modes/callback-reference.json', `${JSON.stringify({ status: 'incomplete', command: 'terminal-mode-callback-oracle.mjs', rustStatus: 'unrun by this command' }, null, 2)}\n`);
const cached = process.argv.includes('--cached');
if (process.argv.slice(2).some(arg => arg !== '--cached')) throw new Error('Only --cached is supported');
let Terminal;
let sources;
let artifact;
if (cached) {
  const metadata = JSON.parse(await readFile('results/terminal-modes/oracle.json', 'utf8'));
  sources = Object.entries(metadata.sourceHashes).map(([file, sha256]) => ({ file, sha256 }));
  for (const source of sources) assert.equal(hash(await readFile(source.file)), source.sha256, `reference changed: ${source.file}`);
  const file = 'tmp/terminal-modes/headless.cjs';
  artifact = { path: file, sha256: hash(await readFile(file)),
    provenance: 'Previously generated read-only reference bundle from terminal-modes-oracle.mjs; this is not a fresh bundler run.' };
  ({ Terminal } = require(path.resolve(file)));
} else ({ Terminal, sources } = await loadTerminalReference());
const cases = [];
function run(name, action) {
  const terminal = new Terminal({ cols: 80, rows: 30, scrollback: 1000, allowProposedApi: true });
  const core = terminal._core;
  const handler = core._inputHandler;
  const Params = handler._parser._params.constructor;
  const replies = [];
  terminal.onData(value => replies.push(value));
  try { cases.push({ name, passed: true, capture: action({ terminal, core, handler, params: values => Params.fromArray(values), replies }) }); }
  catch (error) { cases.push({ name, passed: false, error: error.stack ?? String(error) }); }
  finally { terminal.dispose(); }
}
run('ANSI option observer before next parameter', ({ terminal, core, handler, params }) => {
  const seen = [];
  core.optionsService.onSpecificOptionChange('convertEol', () => seen.push(core.coreService.modes.insertMode));
  assert.equal(handler.setMode(params([20, 4])), true);
  assert.deepEqual(seen, [false]); assert.equal(core.coreService.modes.insertMode, true);
  return { seen, insert: core.coreService.modes.insertMode, convertEol: terminal.options.convertEol };
});
run('mouse observer before following cursor mode', ({ core, handler, params }) => {
  const seen = [];
  core.mouseStateService.onProtocolChange(() => seen.push(core.coreService.decPrivateModes.applicationCursorKeys));
  handler.setModePrivate(params([9, 1]));
  assert.deepEqual(seen, [false]); assert.equal(core.coreService.decPrivateModes.applicationCursorKeys, true);
  return { seen, protocol: core.mouseStateService.activeProtocol, cursor: core.coreService.decPrivateModes.applicationCursorKeys };
});
run('query old DEC object after getter reset', ({ terminal, core, handler, params, replies }) => {
  core.coreService.decPrivateModes.win32InputMode = true;
  terminal.options.vtExtensions = { get win32InputMode() { core.coreService.reset(); return true; } };
  handler.requestMode(params([9001]), false);
  assert.deepEqual(replies, ['\x1b[?9001;1$y']); assert.equal(core.coreService.decPrivateModes.win32InputMode, false);
  return { replies, current: core.coreService.decPrivateModes.win32InputMode };
});
run('query in-place DEC mutation by getter', ({ terminal, core, handler, params, replies }) => {
  terminal.options.vtExtensions = { get win32InputMode() { core.coreService.decPrivateModes.win32InputMode = true; return true; } };
  handler.requestMode(params([9001]), false);
  assert.deepEqual(replies, ['\x1b[?9001;1$y']); assert.equal(core.coreService.decPrivateModes.win32InputMode, true);
  return { replies, current: core.coreService.decPrivateModes.win32InputMode };
});
run('query numeric columns captured before resizing getter', ({ terminal, core, handler, params, replies }) => {
  terminal.options.windowOptions = { get setWinLines() { core._bufferService.resize(132, core.rows); return true; } };
  handler.requestMode(params([3]), false);
  assert.deepEqual(replies, ['\x1b[?3;2$y']); assert.equal(core.cols, 132);
  return { replies, columns: core.cols };
});
run('throwing gate retains prior mode and stops later mode', ({ terminal, core, handler, params }) => {
  core.coreService.decPrivateModes.wraparound = false;
  const failure = new Error('denied gate');
  terminal.options.vtExtensions = { get win32InputMode() { throw failure; } };
  assert.throws(() => handler.setModePrivate(params([1, 9001, 7])), error => error === failure);
  const flags = core.coreService.decPrivateModes;
  assert.deepEqual([flags.applicationCursorKeys, flags.win32InputMode, flags.wraparound], [true, false, false]);
  return { originalError: true, modes: [flags.applicationCursorKeys, flags.win32InputMode, flags.wraparound] };
});
run('disabled stdin suppresses replies but not incoming mutations', ({ terminal, core, handler, replies }) => {
  terminal.options.disableStdin = true;
  handler.parse('A\x1b[?1h\x1b[?1$pB\x1b[4h\x1b[4$pC');
  assert.deepEqual(replies, []);
  assert.equal(core.coreService.decPrivateModes.applicationCursorKeys, true);
  assert.equal(core.coreService.modes.insertMode, true);
  assert.equal(core.buffers.active.lines.get(0).translateToString(true), 'ABC');
  terminal.options.disableStdin = false;
  handler.parse('\x1b[?1$p\x1b[4$p');
  assert.deepEqual(replies, ['\x1b[?1;1$y', '\x1b[4;1$y']);
  return { replies, insert: core.coreService.modes.insertMode, cursor: core.coreService.decPrivateModes.applicationCursorKeys, text: core.buffers.active.lines.get(0).translateToString(true) };
});
for (const throws of [false, true]) run(`keypad logger ${throws ? 'failure' : 'observation'} before mutation`, ({ terminal, core, handler, params }) => {
  const seen = []; const failure = new Error('keypad logger rejected');
  terminal.options.logger = { trace() {}, info() {}, warn() {}, error() {}, debug(message) {
    if (message !== 'Serial port requested application keypad.') return;
    const modes = core.coreService.decPrivateModes;
    seen.push([modes.applicationKeypad, modes.applicationCursorKeys]);
    if (throws) throw failure;
  } };
  terminal.options.logLevel = 'debug';
  if (throws) assert.throws(() => handler.setModePrivate(params([66, 1])), error => error === failure);
  else handler.setModePrivate(params([66, 1]));
  const modes = core.coreService.decPrivateModes;
  assert.deepEqual(seen, [[false, false]]);
  assert.deepEqual([modes.applicationKeypad, modes.applicationCursorKeys], [!throws, !throws]);
  return { seen, modes: [modes.applicationKeypad, modes.applicationCursorKeys], originalError: throws };
});
await mkdir('results/terminal-modes', { recursive: true });
const failures = cases.filter(test => !test.passed).length;
const report = { status: failures ? 'failed source-derived cases' : 'passed source-derived cases only', node: process.version, platform: process.platform,
  sourceHashes: sources, referenceArtifact: artifact ?? null, boundary: 'Actual source InputHandler methods and Params, constructed by the real source headless Terminal. These are added assertions, not the original20 tests or Rust execution.',
  cases, executed: cases.length, passed: cases.length - failures, failed: failures, rustStatus: 'unrun by this command' };
await writeFile('results/terminal-modes/callback-reference.json', `${JSON.stringify(report, null, 2)}\n`);
assert.equal(cases.length, 9);
assert.equal(failures, 0, 'Source-derived callback assertions failed; inspect retained report');
console.log(JSON.stringify({ status: report.status, cases: cases.length, cached }));
