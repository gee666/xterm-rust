// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Run the untouched InputHandler so raw option getter exceptions stay synchronous.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mkdirSync, readFileSync, writeFileSync, rmSync, mkdtempSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {hex, snapshot} from './terminal-snapshot.mjs';

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
const scratch = mkdtempSync('tmp/line-feed-option-');
const output = [];
const prelude = '\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\x1b[3;5H';
try {
  await esbuild.build({entryPoints: [join(source, 'src/headless/public/Terminal.ts')],
    outfile: join(scratch, 'source.cjs'), bundle: true, platform: 'node', format: 'cjs',
    tsconfig: join(source, 'src/headless/tsconfig.json'), logLevel: 'silent'});
  const {Terminal} = require(resolve(scratch, 'source.cjs'));
  for (const command of ['\n', '\v', '\f']) {
    for (const alternate of [false, true]) {
      for (const convertEol of [false, true]) {
        for (const action of ['inspect', 'reset', 'alt', 'normal', 'resize', 'cursor', 'nested', 'error', 'mutate-error', 'scroll-cursor', 'scroll-error', 'scroll-later']) {
          const terminal = new Terminal({cols: 5, rows: 4, scrollback: 2, allowProposedApi: true});
          const core = terminal._core;
          const parse = text => core._inputHandler.parse(text);
          if (alternate) parse('\x1b[?1049h');
          parse(prelude);
          const records = [], events = [];
          let armed = true, calls = 0;
          const original = new Error('line feed option sentinel');
          core.onLineFeed(() => events.push('L'));
          core.onCursorMove(() => events.push('C'));
          core.onScroll(position => {
            events.push(`S${position}`);
            if (action === 'scroll-cursor') {
              core.buffer.y = 0;
              core.buffer.scrollTop = 1;
              core.buffer.scrollBottom = 2;
            }
            if (action === 'scroll-error') throw original;
          });
          core._bufferService.onScroll(() => {
            if (action === 'scroll-later') {
              core.buffer.scrollTop = 1;
              core.buffer.scrollBottom = 1;
            }
          });
          core.onRender(({start, end}) => events.push(`R${start}:${end}`));
          core.onResize(({cols, rows}) => events.push(`Z${cols}:${rows}`));
          Object.defineProperty(core.optionsService.rawOptions, 'convertEol', {configurable: true,
            get() {
              calls++;
              if (!armed) return convertEol;
              armed = false;
              records.push(snapshot(terminal));
              if (action === 'error') throw original;
              if (action === 'reset') terminal.reset();
              if (action === 'alt') core.buffers.activateAltBuffer();
              if (action === 'normal') core.buffers.activateNormalBuffer();
              if (action.startsWith('scroll-')) {
                if (alternate) core.buffers.activateNormalBuffer();
                else core.buffers.activateAltBuffer();
              }
              if (action === 'resize') terminal.resize(7, 5);
              if (action === 'cursor' || action === 'mutate-error') {
                core.buffer.x = 3;
                core.buffer.y = 0;
              }
              if (action === 'nested') core._inputHandler.lineFeed();
              records.push(snapshot(terminal));
              if (action === 'mutate-error') throw original;
              return convertEol;
            }});
          let failure;
          try {parse(command + 'Z');} catch (error) {failure = error;}
          assert.equal(failure, action.endsWith('error') ? original : undefined);
          output.push([hex(command), +alternate, +convertEol, action, calls,
            records.join('~'), snapshot(terminal), events.join('/')].join('\t'));
          terminal.dispose();
        }
      }
    }
  }
  const fixture = 'crates/xterm-core/tests/fixtures/line-feed-option.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'LF option fixture drift');
  verifyReference();
  mkdirSync('results/line-feed-option', {recursive: true});
  writeFileSync('results/line-feed-option/source.json', JSON.stringify({
    command: process.argv.slice(1), node: process.version, platform: process.platform,
    source, referenceFilesVerifiedBeforeAndAfter: manifest.length, cases: output.length,
    fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
    scope: 'LF/VT/FF raw convertEol getter callbacks, two buffers and truth values. Not full equivalence.'
  }, null, 2) + '\n');
  console.log(`line-feed option source cases=${output.length} passed=${output.length}; reference=${manifest.length} pre/post PASS`);
} finally {
  esbuild.stop();
  rmSync(scratch, {recursive: true, force: true});
}
