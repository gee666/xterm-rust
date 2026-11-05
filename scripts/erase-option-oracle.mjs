// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// ED2 reads the raw getter before any row scan or reset. Keep source untouched.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mkdirSync, readFileSync, writeFileSync, rmSync, mkdtempSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {snapshot} from './terminal-snapshot.mjs';

process.chdir(resolve(import.meta.dirname, '..'));
const source = resolve(process.env.XTERM_SOURCE_ROOT ?? 'source_repo');
const require = createRequire(join(source, 'package.json'));
const esbuild = require('esbuild');
const manifest = JSON.parse(readFileSync('docs/baseline/reference-manifest.json', 'utf8'));
function verifyReference() {
  for (const entry of manifest) {
    const bytes = readFileSync(join(source, entry.path));
    assert.equal(bytes.length, entry.bytes, entry.path);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, entry.path);
  }
}
verifyReference();
mkdirSync('tmp', {recursive: true});
const scratch = mkdtempSync('tmp/erase-option-');
const output = [];
try {
  await esbuild.build({entryPoints: [join(source, 'src/headless/public/Terminal.ts')],
    outfile: join(scratch, 'source.cjs'), bundle: true, platform: 'node', format: 'cjs',
    tsconfig: join(source, 'src/headless/tsconfig.json'), logLevel: 'silent'});
  const {Terminal} = require(resolve(scratch, 'source.cjs'));
  const values = {false: false, true: true, zero: 0, nan: NaN, empty: '', string: 'false', object: {}, null: null, undefined: undefined};
  const actions = ['inspect', 'reset', 'alt', 'normal', 'resize', 'cursor', 'attributes', 'error', 'mutate-error', 'event-reset', 'event-error'];
  for (const alternate of [false, true]) for (const selective of [false, true]) {
    for (const [name, value] of Object.entries(values)) for (const action of actions) {
      const terminal = new Terminal({cols: 5, rows: 4, scrollback: 2, allowProposedApi: true});
      const core = terminal._core;
      const parse = text => core._inputHandler.parse(text);
      if (alternate) parse('\x1b[?1049h');
      parse('\x1b[31;44mAAAAA\r\nBBBBB\r\n\x1b[1"qCCCCC\x1b[2"q\x1b[3;5H');
      core.buffer.x = 7; // ED2 clamps to cols before invoking the getter.
      const records = [], events = [];
      let armed = true, eventArmed = true, calls = 0;
      const original = new Error('erase option sentinel');
      core.onCursorMove(() => events.push('C'));
      core.onScroll(position => {
        events.push(`S${position}`);
        records.push(snapshot(terminal));
        if (eventArmed) {
          eventArmed = false;
          if (action === 'event-reset') terminal.reset();
          if (action === 'event-error') throw original;
        }
      });
      core.onRender(({start, end}) => events.push(`R${start}:${end}`));
      core.onResize(({cols, rows}) => events.push(`Z${cols}:${rows}`));
      Object.defineProperty(core.optionsService.rawOptions, 'scrollOnEraseInDisplay', {configurable: true,
        get() {
          calls++;
          if (!armed) return value;
          armed = false;
          records.push(snapshot(terminal));
          if (action === 'error') throw original;
          if (action === 'reset') terminal.reset();
          if (action === 'alt') core.buffers.activateAltBuffer();
          if (action === 'normal') core.buffers.activateNormalBuffer();
          if (action === 'resize') terminal.resize(7, 5);
          if (action === 'cursor' || action === 'mutate-error') {
            core.buffer.x = 1;
            core.buffer.y = 0;
          }
          if (action === 'attributes') core._inputHandler.getAttrData().bg = 0x1000002;
          records.push(snapshot(terminal));
          if (action === 'mutate-error') throw original;
          return value;
        }});
      let failure;
      try {parse(`\x1b[${selective ? '?' : ''}2J`);} catch (error) {failure = error;}
      const throws = action === 'error' || action === 'mutate-error' || (action === 'event-error' && Boolean(value));
      assert.equal(failure, throws ? original : undefined);
      assert.equal(calls, 1);
      output.push([+alternate, +selective, name, action, calls, +!!failure,
        records.join('~'), snapshot(terminal), events.join('/') || '-'].join('\t'));
      terminal.dispose();
    }
  }
  const fixture = 'crates/xterm-core/tests/fixtures/erase-option.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'ED2 option fixture drift');
  verifyReference();
  if (!process.argv.includes('--no-receipt')) {
    mkdirSync('results/erase-option', {recursive: true});
    writeFileSync('results/erase-option/source.json', JSON.stringify({
      command: process.argv.slice(1), node: process.version, platform: process.platform,
      source, referenceFilesVerifiedBeforeAndAfter: manifest.length, cases: output.length,
      fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
      scope: 'ED2 raw scrollOnEraseInDisplay getter, two buffers, selective erase, truthiness, callback/error phases. Not full equivalence.'
    }, null, 2) + '\n');
  }
  console.log(`erase option source cases=${output.length} passed=${output.length}; reference=${manifest.length} pre/post PASS`);
} finally {
  esbuild.stop();
  rmSync(scratch, {recursive: true, force: true});
}
