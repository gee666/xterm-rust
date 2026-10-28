// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Generate traces from unchanged upstream OSC code.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { parserOracle, utf16Hex } from './parser-oracle-common.mjs';
import { canonicalTsvRecords } from './tsv-records.mjs';

const oracle = await parserOracle('Osc');
const seed = 0x05c2019;
let state = seed;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
const mutationChecks = [];
for (const phase of ['start', 'put', 'end', 'reset']) {
  const parser = new oracle.context.Parser();
  const tokens = [];
  const remove = method => { if (method === phase || phase === 'reset' && method === 'end') tokens.forEach(t => t.dispose()); };
  const handler = { start() { remove('start'); }, put() { remove('put'); },
    end() { remove('end'); return false; } };
  tokens.push(parser.registerHandler(1, handler), parser.registerHandler(1, handler));
  parser.start();
  const put = () => parser.put(Uint32Array.from([49, 59, 65]), 0, 3);
  const operation = phase === 'start' || phase === 'put' ? put : () => {
    put(); if (phase === 'end') parser.end(true); else parser.reset();
  };
  assert.throws(operation, { name: 'TypeError' });
  mutationChecks.push({ phase, error: 'TypeError', rustError: 'DispatchError::ActiveHandlerRemoved', status: 'passed' });
}
{
  const parser = new oracle.context.Parser();
  let calls = 0;
  const handler = { start() {}, put() {}, end() { calls++; return false; } };
  const token = parser.registerHandler(1, handler);
  parser.registerHandler(1, handler);
  token.dispose(); token.dispose();
  parser.start(); parser.put(Uint32Array.from([49, 59, 65]), 0, 3); parser.end(true);
  assert.equal(calls, 0);
  mutationChecks.push({ phase: 'repeated disposal with duplicate identity', status: 'passed' });
}
for (const clear of [false, true]) {
  const parser = new oracle.context.Parser();
  const log = [];
  parser.registerHandler(2, new oracle.context.Handler(data => {
    log.push(`old:${utf16Hex(data)}`);
    if (clear) parser.clearHandler(2);
    parser.registerHandler(2, new oracle.context.Handler(data => { log.push(`new:${utf16Hex(data)}`); return true; }));
    return true;
  }));
  for (const code of [65, 66]) {
    parser.start(); parser.put(Uint32Array.from([50, 59, code]), 0, 3); parser.end(true);
  }
  assert.deepEqual(log, ['old:41', 'new:42']);
  mutationChecks.push({ phase: clear ? 'clear then register from callback' : 'register from callback', status: 'passed' });
}
const callbackFailureChecks = [];
for (const asynchronous of [false, true]) {
  const payloads = [];
  const callback = data => {
    payloads.push(utf16Hex(data));
    const run = () => { if (payloads.length === 1) throw new Error('listener failed'); return true; };
    return asynchronous ? oracle.resolvedPromise(run) : run();
  };
  const handler = new oracle.context.Handler(callback);
  handler.start(); handler.put(Uint32Array.from([65]), 0, 1);
  if (asynchronous) await assert.rejects(handler.end(true), /listener failed/);
  else assert.throws(() => handler.end(true), /listener failed/);
  assert.equal(await handler.end(true), true);
  assert.equal(await handler.end(true), true);
  assert.deepEqual(payloads, ['41', '41', '']);
  callbackFailureChecks.push({ asynchronous, path: 'buffer retained on error; cleared on success', status: 'passed' });
}
for (const stage of ['end', 'reset', 'cleanup']) {
  const parser = new oracle.context.Parser();
  const log = [];
  for (const name of [1, 2]) parser.registerHandler(2, {
    start() {}, put() {}, end(success) {
      log.push([name, success]); if (name === 2) throw new Error('listener failed'); return false;
    },
  });
  if (stage === 'cleanup') parser.registerHandler(2, { start() {}, put() {}, end() { return true; } });
  parser.start(); parser.put(Uint32Array.from([50, 59, 65]), 0, 3);
  assert.throws(() => stage === 'reset' ? parser.reset() : parser.end(true), /listener failed/);
  assert.deepEqual(log, [[2, stage === 'end']]);
  callbackFailureChecks.push({ stage, path: 'failure stops older dispatch with partial state', status: 'passed' });
}
{
  const parser = new oracle.context.Parser(); const log = []; const marker = {};
  for (const name of [0,1,2]) parser.registerHandler(2, {
    start() {}, put() {}, end(success) {
      const run = () => { log.push([name,success]); if (name === 1 && success) throw marker; return false; };
      return name === 2 ? oracle.resolvedPromise(run) : run();
    },
  });
  parser.start(); parser.put(Uint32Array.from([50,59,65]),0,3);
  assert.equal(await parser.end(true), false);
  assert.throws(() => parser.end(true,false), error => error === marker);
  assert.equal(parser._stack.paused, false);
  parser.reset(); await oracle.resolvedPromise(() => {});
  assert.deepEqual(log, [[2,true],[1,true],[1,false],[0,false],[2,false]]);
  callbackFailureChecks.push({ stage: 'resume-lower-error then reset', log, innerPausedAfterError: false, status: 'passed' });
}
const fallbackFailureChecks = [];
for (const scenario of ['start_put', 'start_end', 'put', 'end_payload', 'end_id']) {
  const parser = new oracle.context.Parser(); const log = [];
  const marker = { scenario }; let remaining = true;
  const phase = scenario.startsWith('start') ? 's' : scenario === 'put' ? 'p' : 'e';
  parser.setHandlerFallback((id, action, data) => {
    const message = action === 'START' ? 's' : action === 'PUT' ? `p:${utf16Hex(data)}` : `e:${Number(data)}`;
    log.push(`${id}:${message}`);
    if (remaining && message[0] === phase) { remaining = false; throw marker; }
  });
  parser.start();
  const put = values => parser.put(Uint32Array.from(values), 0, values.length);
  const fail = callback => assert.throws(callback, error => error === marker);
  if (scenario === 'start_put') { fail(() => put([50,59,65])); put([66]); parser.end(true); }
  else if (scenario === 'start_end') { put([50]); fail(() => parser.end(true)); parser.end(true); }
  else if (scenario === 'put') { put([50,59]); fail(() => put([65])); put([66]); parser.end(true); }
  else if (scenario === 'end_payload') { put([50,59,65]); fail(() => parser.end(true)); parser.end(false); }
  else { put([50]); fail(() => parser.end(true)); parser.end(true); }
  const expected = {
    start_put: ['2:s','2:p:42','2:e:1'], start_end: ['2:s','2:s','2:e:1'],
    put: ['2:s','2:p:41','2:p:42','2:e:1'], end_payload: ['2:s','2:p:41','2:e:1','2:e:0'],
    end_id: ['2:s','2:e:1','2:s','2:e:1'],
  }[scenario];
  assert.deepEqual(log, expected); assert.equal(parser._state, 0); assert.equal(parser._id, -1);
  fallbackFailureChecks.push({ scenario, log, errorIdentity: true, status: 'passed' });
}
for (const phase of ['s','p','e']) {
  const parser = new oracle.context.Parser(); const log = []; const marker = { phase };
  parser.setHandlerFallback((id, action, data) => {
    const current = action === 'START' ? 's' : action === 'PUT' ? 'p' : 'e';
    log.push(`${id}:${current}`); if (current === phase) throw marker;
  });
  parser.start();
  const put = () => parser.put(Uint32Array.from([50,59,65]), 0, 3);
  assert.throws(() => { put(); if (phase === 'e') parser.end(true); }, error => error === marker);
  const length = log.length; parser.reset(); assert.equal(log.length, length);
  parser.dispose(); parser.start(); put(); parser.end(true); assert.equal(log.length, length);
  fallbackFailureChecks.push({ scenario: `reset/dispose after ${phase}`, log, errorIdentity: true, status: 'passed' });
}
const rawHandlerFailureChecks = [];
const rawHandlerFixtures = [];
for (const scenario of ['s','p','i']) for (const failing of [0,1,2]) for (const reset of [false,true]) {
  const phase = scenario === 'i' ? 's' : scenario;
  const parser = new oracle.context.Parser(); const log = []; const marker = { scenario, failing, reset }; let remaining = true;
  for (const name of [0,1,2]) {
    const report = (current, message) => {
      log.push(message);
      if (remaining && name === failing && phase === current) { remaining = false; throw marker; }
    };
    parser.registerHandler(2, {
      start() { report('s', `s${name}`); },
      put(data,start,end) { report('p', `p${name}:${utf16Hex(oracle.utf32String(Array.from(data.slice(start,end))))}`); },
      end(success) { log.push(`e${name}:${Number(success)}`); return true; },
    });
  }
  parser.start();
  if (scenario === 'i') { parser.put(Uint32Array.of(50),0,1); assert.throws(() => parser.end(true), error => error === marker); }
  else assert.throws(() => parser.put(Uint32Array.from([50,59,65]),0,3), error => error === marker);
  if (reset) { parser.reset(); parser.start(); parser.put(Uint32Array.from([50,59,66]),0,3); }
  else if (scenario !== 'i') parser.put(Uint32Array.of(66),0,1);
  assert.equal(parser.end(true), undefined); assert.equal(parser._state, 0); assert.equal(parser._id, -1);
  rawHandlerFixtures.push([scenario,failing,Number(reset),log.join('|')].join('\t'));
  rawHandlerFailureChecks.push({ scenario, phase, failing, reset, log, errorIdentity: true, status: 'passed' });
}
const rawHandlerCorpus = `${rawHandlerFixtures.join('\n')}\n`;
const rawHandlerFixture = 'crates/xterm-core/tests/fixtures/osc-raw-failures.tsv';
await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(rawHandlerFixture, rawHandlerCorpus);
const rawHandlerFixtureBytes = await readFile(rawHandlerFixture);
assert.equal(canonicalTsvRecords(rawHandlerFixtureBytes.toString('utf8')), rawHandlerCorpus, 'OSC raw failure fixture drift');
const lines = [];
const cases = 512;
const steps = 96;
for (let index = 0; index < cases; index++) {
  const parser = new oracle.context.Parser();
  const output = [];
  const tokens = [];
  const ops = [];
  const attachFallback = () => parser.setHandlerFallback((id, action, data) => {
    output.push(`f:${id}:${action === 'START' ? 's' : action === 'END' ? `e:${Number(data)}` : `p:${utf16Hex(data)}`}`);
  });
  attachFallback();
  for (let step = 0; step < steps; step++) {
    const kind = random() % 24;
    const id = random() % 3;
    if (kind < 4) {
      const name = tokens.length;
      const handled = random() % 2;
      ops.push(`r:${id}:${name}:${handled}`);
      tokens.push(parser.registerHandler(id, {
        start() { output.push(`h:${name}`); },
        put(data, start, end) {
          output.push(`p:${name}:${utf16Hex(oracle.utf32String(Array.from(data.slice(start, end))))}`);
        },
        end(success) { output.push(`e:${name}:${Number(success)}`); return !!handled; },
      }));
    } else if (kind < 7) {
      ops.push('s'); parser.start();
    } else if (kind < 17) {
      const data = [[], [48], [49], [50], [59], [48, 59, 65], [0xd800],
        [0x1f600, 0x4e2d], [0xfeff, 0, 0x10ffff], [49, 50], [65],
        Array(320).fill(57)][random() % 12];
      ops.push(`p:${data.map(x => x.toString(16)).join(',')}`);
      parser.put(Uint32Array.from(data), 0, data.length);
    } else if (kind < 20) {
      const success = random() % 2;
      ops.push(`e:${success}`); parser.end(!!success);
    } else if (kind === 20) {
      ops.push('x'); parser.reset();
    } else if (kind === 21) {
      ops.push(`c:${id}`); parser.clearHandler(id);
    } else if (kind === 22 && tokens.length) {
      const token = random() % tokens.length;
      ops.push(`d:${token}`); tokens[token].dispose();
    } else {
      ops.push('q'); parser.dispose(); attachFallback();
    }
  }
  lines.push([ops.join(';'), output.join('|')].join('\t'));
}
const corpus = lines.join('\n') + '\n';
const fixture = 'crates/xterm-core/tests/fixtures/osc-oracle.tsv';
await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
const fixtureBytes = await readFile(fixture);
assert.equal(canonicalTsvRecords(fixtureBytes.toString('utf8')), corpus, 'OSC oracle drift');
const report = {
  node: process.version, seed, randomizedCases: cases, stepsPerCase: steps,
  baseline: oracle.baseline, mutationChecks, callbackFailureChecks, fallbackFailureChecks, rawHandlerFailureChecks,
  rawHandlerFixtureSha256: createHash('sha256').update(rawHandlerFixtureBytes).digest('hex'),
  rawHandlerCanonicalSha256: createHash('sha256').update(rawHandlerCorpus).digest('hex'), sources: oracle.sources,
  fixtureSha256: createHash('sha256').update(fixtureBytes).digest('hex'),
  fixtureCanonicalSha256: createHash('sha256').update(corpus).digest('hex'),
  status: process.argv.includes('--generate') ? 'generated' : 'matched',
};
await mkdir('results/osc', { recursive: true });
await writeFile('results/osc/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...report, baseline: { ...report.baseline, outcomes: undefined } }, null, 2));
assert.equal(report.baseline.failed, 0, 'upstream OSC baseline failed');
