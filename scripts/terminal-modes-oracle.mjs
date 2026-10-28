// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Unchanged InputHandler assertions and live headless mode operation traces.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
const require = createRequire(path.resolve('source_repo/package.json'));
const { build } = require('esbuild');
const Mocha = require('mocha');
await mkdir('tmp/terminal-modes', { recursive: true });
await mkdir('results/terminal-modes', { recursive: true });
const source = 'source_repo/src/common/InputHandler.test.ts';
const testsFile = path.resolve('tmp/terminal-modes/source-tests.cjs');
const bundled = await build({ entryPoints: [source], outfile: testsFile, bundle: true,
  platform: 'node', format: 'cjs', target: 'node24', metafile: true, logLevel: 'silent',
  tsconfig: 'source_repo/src/common/tsconfig.json' });
const grep = /^InputHandler (?:setMode |DECRQM |alt screen |InputHandler - kitty keyboard )/;
const outcomes = [];
const mocha = new Mocha({ timeout: 10000, grep, reporter: 'spec' });
mocha.addFile(testsFile);
let failures = 0;
await new Promise(resolve => {
  const runner = mocha.run(count => { failures = count; resolve(); });
  runner.on('pass', test => outcomes.push({ name: test.fullTitle(), status: 'passed' }));
  runner.on('fail', (test, error) => outcomes.push({ name: test.fullTitle(), status: 'failed', error: String(error.stack) }));
  runner.on('pending', test => outcomes.push({ name: test.fullTitle(), status: 'pending' }));
});
const productionFile = path.resolve('tmp/terminal-modes/headless.cjs');
const production = await build({ entryPoints: ['source_repo/src/headless/public/Terminal.ts'], outfile: productionFile,
  bundle: true, platform: 'node', format: 'cjs', target: 'node24', metafile: true,
  tsconfig: 'source_repo/src/headless/tsconfig.json', logLevel: 'silent' });
const { Terminal } = require(productionFile);
const terminal = new Terminal({ cols: 80, rows: 30, scrollback: 1000 });
let replies = [];
terminal.onData(data => replies.push(data));
const units = value => Array.from({ length: value.length }, (_, index) => value.charCodeAt(index).toString(16)).join(',') || '-';
const stack = values => values.join(',') || '-';
const fields = ['insertMode', 'applicationCursorKeys', 'applicationKeypad', 'bracketedPasteMode',
  'colorSchemeUpdates', 'origin', 'reverseWraparound', 'sendFocus', 'synchronizedOutput',
  'win32InputMode', 'wraparound', 'isCursorHidden'];
function snapshot() {
  const core = terminal._core;
  const service = core.coreService;
  const dm = service.decPrivateModes;
  const kitty = service.kittyKeyboard;
  const buffer = core.buffers.active;
  const booleans = fields.map(field => +(field === 'insertMode' ? service.modes.insertMode :
    field === 'isCursorHidden' ? service.isCursorHidden : dm[field])).join(',');
  return [booleans, core.mouseStateService.activeProtocol, core.mouseStateService.activeEncoding,
    +terminal.options.cursorBlink, +terminal.options.convertEol,
    +Object.is(core.buffers.active, core.buffers.alt), terminal.cols, terminal.rows,
    buffer.x, buffer.y, buffer.ybase, buffer.ydisp,
    kitty.flags, kitty.mainFlags, kitty.altFlags, stack(kitty.mainStack), stack(kitty.altStack),
    replies.map(units).join(';') || '-'].join('|');
}
const rows = [];
async function step(operation) {
  replies = [];
  const [kind, ...args] = operation.split(':');
  if (kind === 'w') {
    const data = String.fromCharCode(...args[0].split(',').map(value => parseInt(value, 16)));
    await new Promise(resolve => terminal.write(data, resolve));
  } else if (kind === 'o') {
    terminal.options[args[0]] = { ...terminal.options[args[0]], [args[1]]: args[2] === '1' };
  } else throw new Error(`unknown operation ${operation}`);
  rows.push(`${operation}\t${snapshot()}`);
}
const write = data => step(`w:${units(data)}`);
for (const mode of [1, 6, 7, 8, 9, 12, 25, 45, 66, 67, 1000, 1002, 1003, 1004, 1005, 1006, 1015, 1016, 1048, 47, 1047, 1049, 2004, 2026, 2031, 9001, 9999]) {
  for (const action of ['h', 'l']) await write(`\x1b[?${mode}${action}\x1b[?${mode}$p`);
}
await step('o:vtExtensions:kittyKeyboard:1');
await step('o:vtExtensions:win32InputMode:1');
await step('o:quirks:allowSetCursorBlink:1');
for (let flags = 1; flags <= 20; flags++) await write(`\x1b[>${flags}u`);
for (const input of ['\x1b[<u', '\x1b[<0u', '\x1b[<2147483647u', '\x1b[=5u', '\x1b[=2;2u',
  '\x1b[=1;3u', '\x1b[=9;99u', '\x1b[?1049h', '\x1b[>7u', '\x1b[?1049l', '\x1b[?u']) await write(input);
const seed = 0x6d0de5;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const modes = [1, 2, 6, 7, 9, 12, 25, 45, 66, 1000, 1002, 1003, 1004, 1005, 1006, 1015, 1016, 1048, 47, 1047, 1049, 2004, 2026, 2031, 9001, 9999];
for (let iteration = 0; iteration < 2048; iteration++) {
  switch (random() % 8) {
    case 0: case 1: case 2: {
      const mode = modes[random() % modes.length];
      await write(`\x1b[?${mode}${random() & 1 ? 'h' : 'l'}\x1b[?${mode}$p`); break;
    }
    case 3: {
      const mode = [2, 4, 12, 20, 9999][random() % 5];
      await write(`\x1b[${mode}${random() & 1 ? 'h' : 'l'}\x1b[${mode}$p`); break;
    }
    case 4: await write(`\x1b[=${random() % 64};${random() % 5}u\x1b[?u`); break;
    case 5: await write(`\x1b[>${random() % 64}u\x1b[?u`); break;
    case 6: await write(`\x1b[<${random() % 24}u\x1b[?u`); break;
    case 7: {
      const key = ['kittyKeyboard', 'win32InputMode', 'colorSchemeQuery'][random() % 3];
      await step(`o:vtExtensions:${key}:${random() & 1}`); break;
    }
  }
}
terminal.dispose();
const corpus = `${rows.join('\n')}\n`;
const fixture = 'crates/xterm-core/tests/fixtures/terminal-modes.tsv';
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal((await readFile(fixture, 'utf8')).replace(/\r\n/g, '\n'), corpus, 'mode fixture differs from unchanged source');
const hashes = {};
for (const file of [...new Set([...Object.keys(bundled.metafile.inputs), ...Object.keys(production.metafile.inputs)])].sort()) {
  hashes[file] = createHash('sha256').update(await readFile(file)).digest('hex');
}
const report = { command: 'node scripts/terminal-modes-oracle.mjs', platform: process.platform,
  node: process.version, versions: { esbuild: require('esbuild/package.json').version, mocha: require('mocha/package.json').version },
  sourceHashes: hashes, upstream: { discovered: outcomes.length, executed: outcomes.filter(test => test.status !== 'pending').length,
    passed: outcomes.filter(test => test.status === 'passed').length, failed: failures, pending: outcomes.filter(test => test.status === 'pending').length,
    skipped: 0, unrun: 0, selected: 'InputHandler setMode, DECRQM, all alt screen cases, Kitty negotiation; remaining InputHandler suites not selected' },
  differential: { seed, traces: rows.length, corpusSha256: createHash('sha256').update(corpus).digest('hex'), fields }, outcomes };
await writeFile('results/terminal-modes/oracle.json', `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ upstream: report.upstream, differential: report.differential }));
assert.equal(outcomes.length, 20, 'mode upstream inventory changed');
assert.equal(failures, 0);
