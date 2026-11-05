// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Raw getters must run through the untouched InputHandler, including thrown identity.
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
const Mocha = require('mocha');
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
const scratch = mkdtempSync('tmp/mode-query-option-');
const output = [];
const prelude = '\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\x1b[3;5H';
const values = {undefined, null: null, false: false, true: true, zero: 0, nan: NaN, empty: '', text: 'false', object: {}};
try {
  const testsFile = resolve(scratch, 'source-tests.cjs');
  await esbuild.build({entryPoints: [join(source, 'src/common/InputHandler.test.ts')],
    outfile: testsFile, bundle: true, platform: 'node', format: 'cjs',
    tsconfig: join(source, 'src/common/tsconfig.json'), logLevel: 'silent'});
  const upstream = [];
  const mocha = new Mocha({timeout: 10000, grep: /^InputHandler DECRQM /, reporter: 'dot'});
  mocha.addFile(testsFile);
  const failures = await new Promise(resolve => {
    const runner = mocha.run(resolve);
    runner.on('pass', test => upstream.push({name: test.fullTitle(), status: 'passed'}));
    runner.on('fail', (test, error) => upstream.push({name: test.fullTitle(), status: 'failed', error: String(error.stack)}));
    runner.on('pending', test => upstream.push({name: test.fullTitle(), status: 'pending'}));
  });
  assert.equal(upstream.length, 8, 'DECRQM upstream inventory changed');
  assert.equal(failures, 0, 'Unchanged upstream DECRQM assertions failed');
  assert.ok(upstream.every(test => test.status === 'passed'), 'No pending upstream cases allowed');
  await esbuild.build({entryPoints: [join(source, 'src/headless/public/Terminal.ts')],
    outfile: join(scratch, 'source.cjs'), bundle: true, platform: 'node', format: 'cjs',
    tsconfig: join(source, 'src/headless/tsconfig.json'), logLevel: 'silent'});
  const {Terminal} = require(resolve(scratch, 'source.cjs'));
  for (const [command, key] of [['\x1b[20;4$p', 'convertEol'], ['\x1b[?12;1$p', 'cursorBlink']]) {
    for (const alternate of [false, true]) {
      for (const [valueName, value] of Object.entries(values)) {
        for (const action of ['inspect', 'reset', 'alt', 'normal', 'resize', 'cursor', 'modes', 'disable', 'error', 'mutate-error', 'reply-reset', 'reply-error']) {
          const terminal = new Terminal({cols: 5, rows: 4, scrollback: 2, allowProposedApi: true});
          const core = terminal._core;
          const parse = text => core._inputHandler.parse(text);
          if (alternate) parse('\x1b[?1049h');
          parse(prelude);
          const records = [], events = [];
          let armed = true, calls = 0, firstReply = true;
          const original = new Error('mode query option sentinel');
          core.onLineFeed(() => events.push('L'));
          core.onCursorMove(() => events.push('C'));
          core.onScroll(position => events.push(`S${position}`));
          core.onRender(({start, end}) => events.push(`R${start}:${end}`));
          core.onResize(({cols, rows}) => events.push(`Z${cols}:${rows}`));
          terminal.onData(data => {
            events.push(`D${hex(data)}`);
            records.push(`reply:${snapshot(terminal)}`);
            if (!firstReply) return;
            firstReply = false;
            if (action === 'reply-reset') terminal.reset();
            if (action === 'reply-error') throw original;
          });
          Object.defineProperty(core.optionsService.rawOptions, key, {configurable: true,
            get() {
              calls++;
              if (!armed) return value;
              armed = false;
              records.push(`get:${snapshot(terminal)}`);
              if (action === 'error') throw original;
              if (action === 'reset') terminal.reset();
              if (action === 'alt') core.buffers.activateAltBuffer();
              if (action === 'normal') core.buffers.activateNormalBuffer();
              if (action === 'resize') terminal.resize(7, 5);
              if (action === 'cursor' || action === 'mutate-error') {
                core.buffer.x = 3;
                core.buffer.y = 0;
              }
              if (action === 'modes') {
                core.coreService.modes.insertMode = true;
                core.coreService.decPrivateModes.applicationCursorKeys = true;
              }
              if (action === 'disable') core.optionsService.rawOptions.disableStdin = true;
              records.push(`get:${snapshot(terminal)}`);
              if (action === 'mutate-error') throw original;
              return value;
            }});
          let failure;
          try {parse(command + '\x1b[?1$pZ');} catch (error) {failure = error;}
          assert.equal(failure, action.endsWith('error') ? original : undefined);
          output.push([hex(command), key, +alternate, valueName, action, calls,
            records.join('~'), snapshot(terminal), events.join('/')].join('\t'));
          terminal.dispose();
        }
      }
    }
  }
  assert.equal(output.length, 432, 'DECRQM callback case inventory changed');
  const fixture = 'crates/xterm-core/tests/fixtures/mode-query-option.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'DECRQM option fixture drift');
  verifyReference();
  mkdirSync('results/mode-query-option', {recursive: true});
  writeFileSync('results/mode-query-option/source.json', JSON.stringify({
    command: process.argv.slice(1), node: process.version, platform: process.platform,
    source, referenceFilesVerifiedBeforeAndAfter: manifest.length, cases: output.length,
    upstream: {discovered: upstream.length, executed: upstream.length, passed: upstream.length,
      failed: failures, pending: 0, skipped: 0, unrun: 0, selected: 'InputHandler DECRQM only', outcomes: upstream},
    fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
    scope: 'ANSI20/DEC12 DECRQM raw option getters, two buffers, nine raw values, twelve callback actions. Not full equivalence.'
  }, null, 2) + '\n');
  console.log(`mode-query option source cases=${output.length} passed=${output.length}; unchanged upstream=${upstream.length}; reference=${manifest.length} pre/post PASS`);
} finally {
  esbuild.stop();
  rmSync(scratch, {recursive: true, force: true});
}
