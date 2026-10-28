// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Original assertions run unchanged. The trace oracle runs the real JS engine.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { hex, snapshot } from './terminal-snapshot.mjs';

const root = resolve(import.meta.dirname, '..');
process.chdir(root);
const require = createRequire(resolve('source_repo/package.json'));
const esbuild = require('esbuild');
const Mocha = require('mocha');
const chai = require('chai');
const temporary = 'tmp/input-handler-core';
mkdirSync(temporary, { recursive: true });
mkdirSync('results/input-handler-core', { recursive: true });
const sourceHashes = {};
async function bundle(entry, name) {
  const output = `${temporary}/${name}.cjs`;
  const result = await esbuild.build({
    entryPoints: [entry], outfile: output, bundle: true, platform: 'node', format: 'cjs',
    target: 'node24', tsconfig: 'source_repo/src/common/tsconfig.json',
    nodePaths: [resolve('source_repo/src')], metafile: true, logLevel: 'silent',
    plugins: [{ name: 'original-chai', setup(build) {
      build.onResolve({ filter: /^chai$/ }, () => ({ path: require.resolve('chai'), external: true }));
    } }]
  });
  for (const path of Object.keys(result.metafile.inputs).sort()) {
    if (path.startsWith('source_repo/') && !path.includes('/node_modules/')) {
      sourceHashes[path] = createHash('sha256').update(readFileSync(path)).digest('hex');
    }
  }
  return resolve(output);
}
const sourceTest = await bundle('source_repo/src/common/InputHandler.test.ts', 'original-tests');
const outcomes = [];
let current = null;
const methods = ['strictEqual', 'equal', 'deepEqual', 'ok'];
const saved = Object.fromEntries(methods.map(name => [name, chai.assert[name]]));
for (const name of methods) chai.assert[name] = (...args) => {
  if (current) current.assertions++;
  return saved[name](...args);
};
const family = /^InputHandler (?:print |ISO-2022 character sets |BS with reverseWraparound set\/unset |should correctly reset cells taken by wide chars )/;
class Reporter {
  constructor(runner) {
    runner.on('test', test => { current = { name: test.fullTitle(), assertions: 0 }; });
    runner.on('fail', (test, error) => {
      if (current) current.error = error.stack ?? String(error);
      else outcomes.push({ name: test.fullTitle(), assertions: 0, status: 'failed', error: String(error) });
    });
    runner.on('test end', test => {
      outcomes.push({ ...current, status: test.state });
      current = null;
    });
  }
}
const mocha = new Mocha({ reporter: Reporter, grep: family, timeout: 10_000 });
mocha.addFile(sourceTest);
let baselineFailures;
try { baselineFailures = await new Promise(done => mocha.run(done)); }
finally { for (const name of methods) chai.assert[name] = saved[name]; }
assert.equal(outcomes.length, 24, 'Original print/BS/wide/charset selection changed; review the assertion mapping');
const headless = await bundle('source_repo/src/headless/public/Terminal.ts', 'headless');
const { Terminal } = require(headless);
const traces = [];
const add = (id, operations, options = {}) => traces.push({ id, cols: 5, rows: 5, scrollback: 1, options, operations });
for (const input of ['\u200b', '\u0301', 'e\u0301', 'Soft\u00adhy\u00adphen', '￥￥￥', '🙂🙂🙂', '\tX', '\x07', 'a\n\vb\fc\rZ']) {
  add(`direct-${traces.length}`, [['W', input]]);
}
for (const modes of ['', '\x1b[?45h', '\x1b[?1045h', '\x1b[?45;1045h', '\x1b[?45;1045l']) {
  for (const prefix of ['', '12345'.repeat(2), '12345\r\n12345', '#####abcdefghijklmnopqrstuvwxy']) {
    for (const margins of ['', '\x1b[2;4r']) {
      add(`reverse-${traces.length}`, [['W', modes], ['W', prefix], ['W', margins], ['W', '\r'], ['W', '\x08 \x08'.repeat(100)]]);
    }
  }
}
add('a11y-before-cell', [['W', 'ab界'], ['W', '\tZ']], { screenReaderMode: true });
add('nowrap-wide', [['W', '\x1b[?7l1234'], ['W', '￥x']]);
add('insert-combining', [['W', 'abcde'], ['W', '\r\x1b[4he\u0301']]);
add('charset-control', [['W', '\x1b)0\x0eq\x0f'], ['W', 'q\x1b7\x0eq\x1b8q']]);
add('chunk-boundaries', [['W', 'e'], ['W', '\u0301'], ['W', '\ud83d'], ['W', '\ude42']]);
const rows = [];
for (const trace of traces) {
  const terminal = new Terminal({ cols: trace.cols, rows: trace.rows, scrollback: trace.scrollback,
    allowProposedApi: true, ...trace.options });
  const events = [];
  const callbackStates = [];
  terminal.onBell(() => events.push('B'));
  terminal.onCursorMove(() => events.push('C'));
  terminal.onLineFeed(() => events.push('L'));
  terminal.onScroll(y => events.push(`S${y}`));
  terminal.onData(text => events.push(`D${hex(text)}`));
  terminal._core.onRender(event => events.push(`R${event.start}:${event.end}`));
  terminal._core.onA11yChar(text => callbackStates.push(`C${hex(text)}:${snapshot(terminal)}`));
  terminal._core.onA11yTab(count => callbackStates.push(`T${count}:${snapshot(terminal)}`));
  for (const [kind, input] of trace.operations) {
    events.length = 0;
    callbackStates.length = 0;
    await new Promise(done => terminal.write(input, done));
    rows.push([trace.id, trace.cols, trace.rows, trace.scrollback, +!!trace.options.screenReaderMode,
      kind, hex(input), snapshot(terminal), events.join('/'), callbackStates.join('~')].join('\t'));
  }
  terminal.dispose();
}
const text = rows.join('\n') + '\n';
const fixture = 'crates/xterm-core/tests/fixtures/input-handler-core.tsv';
if (process.argv.includes('--generate')) writeFileSync(fixture, text);
else assert.equal(readFileSync(fixture, 'utf8').replaceAll('\r\n', '\n'), text, 'input handler source trace drift');
for (const [path, digest] of Object.entries(sourceHashes)) {
  assert.equal(createHash('sha256').update(readFileSync(path)).digest('hex'), digest, `Source changed: ${path}`);
}
const report = {
  overallEquivalence: 'NOT PROVEN', command: ['node', 'scripts/input-handler-core-oracle.mjs', ...process.argv.slice(2)],
  node: process.version, platform: `${process.platform}-${process.arch}`, sourceHashes,
  baseline: { discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(x => x.status === 'passed').length,
    failed: baselineFailures, skipped: outcomes.filter(x => x.status === 'pending').length,
    assertions: outcomes.reduce((total, item) => total + item.assertions, 0), outcomes },
  traceScope: 'Real headless engine, full cells/both buffers/cursor/margins/modes/ordered events and a11y callback-time state',
  traces: traces.length, snapshots: rows.length, fixture,
  fixtureSha256: createHash('sha256').update(text).digest('hex')
};
writeFileSync('results/input-handler-core/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ baseline: report.baseline, traces: traces.length, snapshots: rows.length }, null, 2));
if (baselineFailures) process.exitCode = 1;
