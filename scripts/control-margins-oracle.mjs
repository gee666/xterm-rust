// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Reuse the whole-terminal projection, with raw cursor boundary cases kept explicit.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { resolve } from 'node:path';
import { hex, snapshot } from './terminal-snapshot.mjs';
import { canonicalTsvRecords } from './tsv-records.mjs';
const root = resolve(import.meta.dirname, '..');
const require = createRequire(resolve(process.env.XTERM_SOURCE_DEPENDENCIES || resolve(root, 'source_repo'), 'package.json'));
const { build } = require('esbuild');
const temporary = resolve(root, 'tmp/control-margins-oracle');
const output = resolve(root, 'results/control-margins');
const fixture = resolve(root, 'crates/xterm-core/tests/fixtures/control-margins.tsv');
const manifest = JSON.parse(readFileSync(resolve(root, 'docs/baseline/reference-manifest.json')));
function integrity() {
  for (const entry of manifest) assert.equal(createHash('sha256').update(readFileSync(resolve(root, 'source_repo', entry.path))).digest('hex'), entry.sha256, entry.path);
  return manifest.length;
}
mkdirSync(temporary, { recursive: true });
mkdirSync(output, { recursive: true });
try {
  const before = integrity();
  const bundle = resolve(temporary, 'Terminal.cjs');
  const built = await build({ absWorkingDir: root, entryPoints: ['source_repo/src/headless/public/Terminal.ts'],
    outfile: bundle, bundle: true, platform: 'node', format: 'cjs', tsconfig: 'source_repo/src/headless/tsconfig.json', metafile: true });
  const { Terminal } = require(bundle);
  const records = [];
  for (const alt of [false, true]) for (const origin of [false, true]) for (const reverse of [false, true]) {
    for (const x of [0, 5, 9]) for (const y of [0, 1, 2, 3, 4, 9]) for (const command of ['\x08', '\x1bM']) {
      const terminal = new Terminal({ cols: 5, rows: 5, scrollback: 3, allowProposedApi: true });
      const setup = (alt ? '\x1b[?1049h' : '') + 'AAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD\r\nEEEEE' + '\x1b[2;4r' + (origin ? '\x1b[?6h' : '') + (reverse ? '\x1b[?45h' : '');
      await new Promise(done => terminal.write(setup, done));
      const buffer = terminal._core.buffers.active;
      for (let row = 1; row < 5; row++) buffer.setWrapped(row, true);
      buffer.x = x;
      buffer.y = y;
      const events = [];
      terminal.onBell(() => events.push('B')); terminal.onCursorMove(() => events.push('C'));
      terminal.onLineFeed(() => events.push('L')); terminal.onScroll(y => events.push(`S${y}`));
      terminal.onData(text => events.push('D' + hex(text))); terminal.onTitleChange(text => events.push('T' + hex(text)));
      terminal._core.onRender(event => events.push(`R${event.start}:${event.end}`));
      await new Promise(done => terminal.write(command, done));
      records.push([+alt, +origin, +reverse, x, y, hex(command), snapshot(terminal), events.join('/')].join('\t'));
      terminal.dispose();
    }
  }
  const contents = records.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, contents);
  else assert.equal(canonicalTsvRecords(readFileSync(fixture, 'utf8')), contents, 'control margin source fixture drift');
  const after = integrity();
  const report = { status: 'passed', command: 'node scripts/control-margins-oracle.mjs' + (process.argv.includes('--generate') ? ' --generate' : ''),
    node: process.version, platform: process.platform, cases: records.length, sourceIntegrity: { before, after },
    fixture: 'crates/xterm-core/tests/fixtures/control-margins.tsv', fixtureSha256: createHash('sha256').update(contents).digest('hex'),
    sourceHashes: Object.fromEntries(Object.keys(built.metafile.inputs).filter(p => p.startsWith('source_repo/')).map(p => [p, createHash('sha256').update(readFileSync(resolve(root, p))).digest('hex')])),
    compared: 'Both complete buffers, cursor/margins, modes, attributes, wrapped flags, cells, ordered cursor/render events.',
    limits: ['Direct cursor mutation models InputHandler boundary inputs, not all public API reachability.', 'Windows source execution only. No browser or Linux/macOS runtime.', 'Full equivalence NOT PROVEN.'] };
  writeFileSync(resolve(output, 'source.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ cases: records.length, sourceIntegrity: report.sourceIntegrity }));
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
