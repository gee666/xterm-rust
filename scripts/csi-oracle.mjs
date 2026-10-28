// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// The actual source terminal parses every operation; projections are shared with task28.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { hex, snapshot } from './terminal-snapshot.mjs';
const root = resolve(import.meta.dirname, '..');
const require = createRequire(resolve(root, 'source_repo/package.json'));
const { build } = require('esbuild');
const output = resolve(root, 'results/csi');
mkdirSync(output, { recursive: true });
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify({ status: 'incomplete', started: new Date().toISOString() }) + '\n');
const temporary = resolve(root, 'tmp/csi-oracle');
mkdirSync(temporary, { recursive: true });
const bundle = resolve(temporary, 'Terminal.cjs');
const built = await build({ absWorkingDir: root, entryPoints: ['source_repo/src/headless/public/Terminal.ts'],
  outfile: bundle, bundle: true, platform: 'node', format: 'cjs',
  tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true });
const { Terminal } = require(bundle);
const commands = ['@','A','B','C','D','E','F','G','H','I','J','K','L','M','P','S','T','X','Z','^','`','a','b','d','e','f','g','r','s','u',' @',' A',"'}","'~",'"q','?J','?K'];
const sequence = (command, parameter) => '\x1b[' + (command[0] === '?' ? '?' + parameter + command.slice(1) : parameter + command);
const traces = [];
for (const [index, command] of commands.entries()) {
  for (const parameter of ['', '0', '1', '3', '100']) {
    const text = ['H','f','r'].includes(command) && parameter ? parameter + ';' + parameter : parameter;
    const ops = ['AB界e\u0301🙂XY\r\n01234567\r\nabcdefgh', '\x1b[44m', '\x1b[2;3r',
      index % 2 ? '\x1b[?6h' : '\x1b[?6l', '\x1b[2;4H', 'e\u0301', sequence(command, text)];
    traces.push({ id: `command-${index}-${parameter || 'default'}`, cols: 8, rows: 4, scrollback: 6, ops });
  }
}
const seed = 0xc5107;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const text = ['a','界','🙂','e\u0301','\r\n','\t','\x1b[31;44m','\x1b[0m','\x1b[?6h','\x1b[?6l','\x1b[?7h','\x1b[?7l'];
for (let trace = 0; trace < 64; trace++) {
  const ops = [];
  for (let step = 0; step < 16; step++) {
    if (random() % 4 === 0) ops.push(text[random() % text.length]);
    else {
      const command = commands[random() % commands.length];
      const p = ['', '0', '1', '2', '5', '100'][random() % 6];
      const value = sequence(command, p);
      if (random() % 4 === 0) {
        const split = 1 + random() % (value.length - 1);
        ops.push(value.slice(0, split), value.slice(split));
      } else ops.push(value);
    }
  }
  traces.push({ id: `seed-${trace}`, cols: 2 + random() % 9, rows: 1 + random() % 5, scrollback: 6, ops });
}
const rows = [];
for (const trace of traces) {
  const terminal = new Terminal({ cols: trace.cols, rows: trace.rows, scrollback: trace.scrollback, allowProposedApi: true });
  const events = [];
  terminal.onBell(() => events.push('B')); terminal.onCursorMove(() => events.push('C'));
  terminal.onLineFeed(() => events.push('L')); terminal.onScroll(y => events.push(`S${y}`));
  terminal.onData(text => events.push('D' + hex(text))); terminal.onTitleChange(text => events.push('T' + hex(text)));
  terminal._core.onRender(event => events.push(`R${event.start}:${event.end}`));
  for (const input of trace.ops) {
    events.length = 0;
    await new Promise(done => terminal.write(input, done));
    rows.push([trace.id, trace.cols, trace.rows, trace.scrollback, hex(input), snapshot(terminal), events.join('/')].join('\t'));
  }
  terminal.dispose();
}
const fixture = 'crates/xterm-core/tests/fixtures/csi-terminal.tsv';
const contents = rows.join('\n') + '\n';
if (process.argv.includes('--generate')) writeFileSync(resolve(root, fixture), contents);
else assert.equal(readFileSync(resolve(root, fixture), 'utf8').replace(/\r\n/g, '\n'), contents, 'CSI source fixture drift');
const report = { command: 'node scripts/csi-oracle.mjs' + (process.argv.includes('--generate') ? ' --generate' : ''),
  status: 'passed', node: process.version, platform: process.platform, seed: `0x${seed.toString(16)}`,
  commandFamilies: commands, directedTraces: commands.length * 5, seededTraces: 64,
  traces: traces.length, snapshots: rows.length, fixture,
  fixtureSha256: createHash('sha256').update(readFileSync(resolve(root, fixture))).digest('hex'),
  canonicalFixtureSha256: createHash('sha256').update(contents).digest('hex'),
  sourceHashes: Object.fromEntries(Object.keys(built.metafile.inputs).filter(path => path.startsWith('source_repo/'))
    .map(path => [path, createHash('sha256').update(readFileSync(resolve(root, path))).digest('hex')])),
  compared: 'Both buffers, every cell/codepoint/width/fg/bg/extended attrs/URL/UTF16 string/wrap, cursor/base/display/margins, modes/current attrs and ordered move/render/scroll/data/bell/linefeed/title events.',
  overallEquivalence: 'NOT PROVEN', limits: ['Finite handler corpus, not all terminal/addon/browser contracts.',
    'Negative direct internal cursor mutation is not generated here; explicit remaining source assertions tracked in task152.',
    'No native Linux/macOS execution or browser evidence from this Windows run.'] };
writeFileSync(resolve(output, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ traces: traces.length, snapshots: rows.length, seed: report.seed }));
