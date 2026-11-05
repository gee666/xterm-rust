// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Capture HT's raw getter and synchronous a11y callbacks in the untouched source.
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
const scratch = mkdtempSync('tmp/tab-option-');
const output = [];
try {
  await esbuild.build({entryPoints: [join(source, 'src/headless/public/Terminal.ts')],
    outfile: join(scratch, 'source.cjs'), bundle: true, platform: 'node', format: 'cjs',
    tsconfig: join(source, 'src/headless/tsconfig.json'), logLevel: 'silent'});
  const {Terminal} = require(resolve(scratch, 'source.cjs'));
  for (const alternate of [false, true]) {
    for (const enabled of [false, true]) {
      for (const column of [0, 3, 11, 12]) {
        for (const action of ['inspect', 'reset', 'alt', 'normal', 'resize', 'cursor', 'error', 'mutate-error', 'event-cursor', 'event-error']) {
          const terminal = new Terminal({cols: 12, rows: 3, scrollback: 2, allowProposedApi: true});
          const core = terminal._core;
          const parse = text => core._inputHandler.parse(text);
          if (alternate) parse('\x1b[?1049h');
          parse('AAAA\r\nBBBB\r\nCCCC');
          core.buffer.x = column;
          const records = [], events = [];
          let armed = true, calls = 0;
          const original = new Error('tab option sentinel');
          core.onCursorMove(() => events.push('C'));
          core.onScroll(position => events.push(`S${position}`));
          core.onRender(({start, end}) => events.push(`R${start}:${end}`));
          core.onResize(({cols, rows}) => events.push(`Z${cols}:${rows}`));
          core._inputHandler.onA11yTab(count => {
            events.push(`T${count}`);
            records.push(snapshot(terminal));
            if (action === 'event-cursor') {
              core.buffer.x = 1;
              core.buffer.y = 0;
            }
            if (action === 'event-error') throw original;
          });
          Object.defineProperty(core.optionsService.rawOptions, 'screenReaderMode', {configurable: true,
            get() {
              calls++;
              if (!armed) return enabled;
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
              records.push(snapshot(terminal));
              if (action === 'mutate-error') throw original;
              return enabled;
            }});
          let failure;
          try {parse('\t');} catch (error) {failure = error;}
          const throws = column < 12 && (action === 'error' || action === 'mutate-error' || (enabled && action === 'event-error'));
          assert.equal(failure, throws ? original : undefined);
          output.push([+alternate, +enabled, column, action, calls,
            records.join('~'), snapshot(terminal), events.join('/')].join('\t'));
          terminal.dispose();
        }
      }
    }
  }
  const fixture = 'crates/xterm-core/tests/fixtures/tab-option.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'HT option fixture drift');
  verifyReference();
  mkdirSync('results/tab-option', {recursive: true});
  writeFileSync('results/tab-option/source.json', JSON.stringify({
    command: process.argv.slice(1), node: process.version, platform: process.platform,
    source, referenceFilesVerifiedBeforeAndAfter: manifest.length, cases: output.length,
    fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
    scope: 'HT raw screenReaderMode getter and a11y callback phases, two buffers, truth values and cursor boundaries. Not full equivalence.'
  }, null, 2) + '\n');
  console.log(`tab option source cases=${output.length} passed=${output.length}; reference=${manifest.length} pre/post PASS`);
} finally {
  esbuild.stop();
  rmSync(scratch, {recursive: true, force: true});
}
