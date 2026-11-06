// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mkdirSync, readFileSync, writeFileSync, rmSync, mkdtempSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {hex, snapshot} from './terminal-snapshot.mjs';

process.chdir(resolve(import.meta.dirname, '..'));
const source = resolve(process.env.XTERM_SOURCE_ROOT ?? 'source_repo');
const require = createRequire(join(source, 'package.json'));
const esbuild = require('esbuild'), Mocha = require('mocha');
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
const scratch = mkdtempSync('tmp/dsr-option-');
const output = [];
const prelude = '\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\x1b[3;5H';
const values = {undefined, null: null, false: false, true: true, zero: 0, nan: NaN,
  empty: '', text: 'false', object: {valueOf() {throw new Error('must not coerce');}}};
const actions = ['inspect', 'reset', 'alt', 'normal', 'resize', 'cursor', 'modes', 'disable',
  'replace', 'error', 'mutate-error', 'event-reset', 'event-error'];
try {
  const testsFile = resolve(scratch, 'source-tests.cjs');
  const common = {bundle: true, platform: 'node', format: 'cjs', logLevel: 'silent'};
  await esbuild.build({...common, entryPoints: [join(source, 'src/headless/public/Terminal.test.ts')],
    outfile: testsFile, tsconfig: join(source, 'src/headless/tsconfig.json')});
  const upstream = [];
  const mocha = new Mocha({timeout: 10000, grep: /^Headless API Tests Events onData$/, reporter: 'dot'});
  mocha.addFile(testsFile);
  const failures = await new Promise(done => {
    const runner = mocha.run(done);
    runner.on('pass', test => upstream.push({name: test.fullTitle(), status: 'passed'}));
    runner.on('fail', (test, error) => upstream.push({name: test.fullTitle(), status: 'failed', error: String(error.stack)}));
    runner.on('pending', test => upstream.push({name: test.fullTitle(), status: 'pending'}));
  });
  assert.equal(upstream.length, 1, 'Headless onData inventory changed');
  assert.equal(failures, 0);
  assert.ok(upstream.every(test => test.status === 'passed'));
  await esbuild.build({...common, entryPoints: [join(source, 'src/headless/public/Terminal.ts')],
    outfile: join(scratch, 'source.cjs'), tsconfig: join(source, 'src/headless/tsconfig.json')});
  const {Terminal} = require(resolve(scratch, 'source.cjs'));
  const controls = new Terminal();
  const replies = [];
  controls.onData(data => replies.push(data));
  Object.defineProperty(controls._core.optionsService.rawOptions, 'vtExtensions',
    {get() {throw new Error('non-996 branch must not read options');}});
  for (const command of ['\x1b[996n', '\x1b[?15;996n', '\x1b[?6;996n', '\x1b[5;996n']) {
    controls._core._inputHandler.parse(command);
  }
  assert.deepEqual(replies, ['\x1b[?1;1R', '\x1b[0n']);
  controls.dispose();
  for (const phase of ['bag', 'nested']) for (const alternate of [false, true]) {
    for (const [name, value] of Object.entries(values)) for (const action of actions) {
      const terminal = new Terminal({cols: 5, rows: 4, scrollback: 2, allowProposedApi: true});
      const core = terminal._core, parse = text => core._inputHandler.parse(text);
      if (alternate) parse('\x1b[?1049h');
      parse(prelude);
      const records = [], events = [];
      let armed = true, calls = 0, firstEvent = true;
      const original = new Error('DSR option sentinel');
      const record = tag => records.push(`${tag}:${snapshot(terminal)}`);
      core.onLineFeed(() => events.push('L'));
      core.onCursorMove(() => events.push('C'));
      core.onScroll(position => events.push(`S${position}`));
      core.onRender(({start, end}) => events.push(`R${start}:${end}`));
      core.onResize(({cols, rows}) => events.push(`Z${cols}:${rows}`));
      terminal.onData(data => events.push(`D${hex(data)}`));
      core._inputHandler.onRequestColorSchemeQuery(() => {
        events.push('Q'); record('event');
        if (!firstEvent) return;
        firstEvent = false;
        if (action === 'event-reset') terminal.reset();
        if (action === 'event-error') throw original;
      });
      const getter = () => {
        calls++;
        if (!armed) return;
        armed = false;
        record('get');
        if (action === 'error') throw original;
        if (action === 'reset') terminal.reset();
        if (action === 'alt') core.buffers.activateAltBuffer();
        if (action === 'normal') core.buffers.activateNormalBuffer();
        if (action === 'resize') terminal.resize(7, 5);
        if (action === 'cursor' || action === 'mutate-error') {core.buffer.x = 3; core.buffer.y = 0;}
        if (action === 'modes') {core.coreService.modes.insertMode = true; core.coreService.decPrivateModes.applicationCursorKeys = true;}
        if (action === 'disable') core.optionsService.rawOptions.disableStdin = true;
        if (action === 'replace') Object.defineProperty(core.optionsService.rawOptions, 'vtExtensions',
          {configurable: true, value: {colorSchemeQuery: false}, writable: true});
        record('get');
        if (action === 'mutate-error') throw original;
      };
      if (phase === 'bag') Object.defineProperty(core.optionsService.rawOptions, 'vtExtensions',
        {configurable: true, get() {getter(); return value;}});
      else {
        const bag = {};
        Object.defineProperty(bag, 'colorSchemeQuery', {get() {getter(); return value;}});
        core.optionsService.rawOptions.vtExtensions = bag;
      }
      let failure;
      try {parse('\x1b[?996;6n\x1b[?6nZ');} catch (error) {failure = error;}
      if (failure) assert.equal(failure, original, 'Exact original thrown identity');
      output.push([phase, +alternate, name, action, calls, +!!failure,
        records.join('~'), snapshot(terminal), events.join('/') || '-'].join('\t'));
      terminal.dispose();
    }
  }
  assert.equal(output.length, 468, 'DSR option case inventory changed');
  const fixture = 'crates/xterm-core/tests/fixtures/dsr-option.tsv';
  const text = output.join('\n') + '\n';
  if (process.argv.includes('--generate')) writeFileSync(fixture, text);
  else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'DSR option fixture drift');
  verifyReference();
  mkdirSync('results/dsr-option', {recursive: true});
  writeFileSync('results/dsr-option/source.json', JSON.stringify({command: process.argv.slice(1),
    node: process.version, platform: process.platform, source, referenceFilesVerifiedBeforeAndAfter: manifest.length,
    cases: output.length, non996ShortCircuitCommands: 4, upstream: {discovered: 1, executed: 1, passed: 1, failed: 0, skipped: 0, unrun: 0, outcomes: upstream},
    fixture, fixtureSha256: createHash('sha256').update(text).digest('hex'),
    scope: 'Private DSR raw bag/nested property, two buffers, nine values, thirteen callback actions. No full equivalence claim.'
  }, null, 2) + '\n');
  console.log(`DSR source ${output.length} cases PASS; original headless onData 1 PASS; reference ${manifest.length} pre/post PASS`);
} finally {esbuild.stop(); rmSync(scratch, {recursive: true, force: true});}
