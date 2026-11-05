// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Exercise untouched InputHandler.print through its synchronous parser path.
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
const scratch = mkdtempSync('tmp/print-option-');
const output = [];
const prelude = '\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\x1b[3;5H';
try {
  await esbuild.build({entryPoints: [join(source, 'src/headless/public/Terminal.ts')],
    outfile: join(scratch, 'source.cjs'), bundle: true, platform: 'node', format: 'cjs',
    tsconfig: join(source, 'src/headless/tsconfig.json'), logLevel: 'silent'});
  const {Terminal} = require(resolve(scratch, 'source.cjs'));
  for (const text of ['qABCD', '界qBC', '\u00ad']) {
    for (const alternate of [false, true]) {
      for (const a11y of [false, true]) {
        for (const action of ['inspect', 'reset', 'alt', 'normal', 'resize', 'cursor',
          'modes', 'attributes', 'charset-replace', 'charset-mutate', 'nested-lf', 'error', 'mutate-error',
          'remove-row', 'missing-inspect', 'missing-reset', 'missing-error']) {
          const terminal = new Terminal({cols: 5, rows: 4, scrollback: 2, allowProposedApi: true});
          const core = terminal._core;
          const parse = value => core._inputHandler.parse(value);
          if (alternate) parse('\x1b[?1049h');
          parse(prelude);
          const charset = {q: 'Q'};
          core._charsetService.charset = charset;
          const removeRow = () => {
            core.buffer.lines.length = 0;
            core.buffer.lines.set(core.buffer.ybase + core.buffer.y, undefined);
          };
          if (action.startsWith('missing-')) removeRow();
          const records = [], events = [];
          let armed = true, calls = 0;
          const original = new Error('print option sentinel');
          core._inputHandler.onA11yChar(value => events.push(`A${hex(value)}`));
          core.onLineFeed(() => events.push('L'));
          core.onCursorMove(() => events.push('C'));
          core.onScroll(position => events.push(`S${position}`));
          core.onRender(({start, end}) => events.push(`R${start}:${end}`));
          core.onResize(({cols, rows}) => events.push(`Z${cols}:${rows}`));
          Object.defineProperty(core.optionsService.rawOptions, 'screenReaderMode', {configurable: true,
            get() {
              calls++;
              if (!armed) return a11y;
              armed = false;
              records.push(snapshot(terminal));
              if (action === 'error' || action === 'missing-error') throw original;
              if (action === 'reset' || action === 'missing-reset') terminal.reset();
              if (action === 'remove-row') removeRow();
              if (action === 'alt') core.buffers.activateAltBuffer();
              if (action === 'normal') core.buffers.activateNormalBuffer();
              if (action === 'resize') terminal.resize(7, 5);
              if (action === 'cursor' || action === 'mutate-error') {
                core.buffer.x = 1;
                core.buffer.y = 0;
              }
              if (action === 'modes') {
                core.coreService.modes.insertMode = true;
                core.coreService.decPrivateModes.wraparound = false;
              }
              if (action === 'attributes') {
                core._inputHandler.getAttrData().fg = 0x3001234;
                core._inputHandler.getAttrData().bg = 0x3005678;
              }
              if (action === 'charset-replace') core._charsetService.charset = {q: 'Z'};
              if (action === 'charset-mutate') charset.q = 'M';
              if (action === 'nested-lf') core._inputHandler.lineFeed();
              records.push(snapshot(terminal));
              if (action === 'mutate-error') throw original;
              return a11y;
            }});
          let failure;
          try {parse(text);} catch (error) {failure = error;}
          assert.equal(failure, action.endsWith('error') ? original : undefined);
          output.push([hex(text), +alternate, +a11y, action, calls,
            records.join('~'), snapshot(terminal), events.join('/')].join('\t'));
          terminal.dispose();
        }
      }
    }
  }
  const fixture = 'crates/xterm-core/tests/fixtures/print-option.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'print option fixture drift');
  verifyReference();
  mkdirSync('results/print-option', {recursive: true});
  writeFileSync('results/print-option/source.json', JSON.stringify({
    command: process.argv.slice(1), node: process.version, platform: process.platform,
    source, referenceFilesVerifiedBeforeAndAfter: manifest.length, cases: output.length,
    fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
    scope: 'Print screenReaderMode startup getter callbacks, both buffers/truth values; not full equivalence.'
  }, null, 2) + '\n');
  console.log(`print option source cases=${output.length} passed=${output.length}; reference=${manifest.length} pre/post PASS`);
} finally {
  esbuild.stop();
  rmSync(scratch, {recursive: true, force: true});
}
