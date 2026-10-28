// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// DCS integration traces execute a fresh bundle of read-only reference sources.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { escapeReference } from './escape-reference.mjs';

const reference = await escapeReference();
const { EscapeSequenceParser } = reference.context;
const sources = reference.sources;
const hex = values => Array.from(values, value => value.toString(16)).join(',');
const units = text => hex(Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)));
const params = value => value.toArray().map(v => Array.isArray(v) ? `s${v.join(',')}` : `p${v}`).join(';');
const ids = [
  ['q', { final: 'q' }],
  ['+q', { intermediates: '+', final: 'q' }],
  ['?+q', { prefix: '?', intermediates: '+', final: 'q' }],
];
const fixtures = [];
let parseCalls = 0;
let pauses = 0;
async function trace(config, operations) {
  const parser = new EscapeSequenceParser();
  const reports = [];
  parser.setPrintHandler((data, start, end) => reports.push(`P:${hex(data.slice(start, end))}`));
  parser.setExecuteHandlerFallback(code => reports.push(`X:${code}`));
  parser.setEscHandlerFallback(ident => reports.push(`E:${ident >>> 0}`));
  parser.setCsiHandlerFallback((ident, value) => reports.push(`C:${ident >>> 0}:${params(value)}`));
  parser.setDcsHandlerFallback((ident, action, value) => reports.push(`F:${ident >>> 0}:${
    action === 'HOOK' ? `H:${params(value)}` : action === 'PUT' ? `P:${units(value)}` : `U:${+value}`
  }`));
  for (const [label, id] of ids) {
    const count = config === 0 ? 1 : 3;
    for (let index = 0; index < count; index++) {
      const name = `${label}${index}`;
      parser.registerDcsHandler(id, {
        hook(value) { reports.push(`H:${name}:${params(value)}`); },
        put(data, start, end) { reports.push(`D:${name}:${hex(data.slice(start, end))}`); },
        unhook(success) {
          const report = () => { reports.push(`U:${name}:${+success}`); return config === 1 ? index === 0 : true; };
          return config && index === 1 ? reference.resolvedPromise(report) : report();
        }
      });
    }
  }
  const snapshots = [];
  for (const operation of operations) {
    reports.length = 0;
    if (operation === 'r') { parser.reset(); await Promise.resolve(); }
    else if (operation === 'd') parser.dispose();
    else {
      const data = Uint32Array.from(operation.slice(1).split(',').filter(Boolean).map(value => parseInt(value, 16)));
      let previous;
      let pending;
      do {
        pending = parser.parse(data, data.length, previous);
        if (pending) { pauses++; previous = await pending; }
      } while (pending);
      parseCalls++;
    }
    snapshots.push(`${parser.currentState}/${parser._collect >>> 0}/${params(parser._params)}/${parser.precedingJoinState}/${reports.join('|')}`);
  }
  fixtures.push(`${config}\t${operations.join('|')}\t${snapshots.join('~')}`);
}
const streams = [
  '\x1bP1;2:3+qabc😀\x1b\\Z', '\x1bP?1;2::3+q€\x9cZ', '\x90qabc\x9cZ',
  '\x1bPqA\x07B\x1b\\Z', '\x1bPqA\x7fB\x1b\\Z', '\x1bPqA\0\b\t\n\rB\x9cZ',
  '\x1bPqpartial\x18Z', '\x1bPqpartial\x1aZ', '\x1bPqabc\x1b[31mZ',
  '\x1bP1;:2q\x1b\\Z', '\x1bP?1+q\ud800😀\x9cZ', '\x1bP1:2?+qignored\x9cZ',
  '\x1bPzunknown\x9cZ', '\x1bPq\x9cZ', '\x1bPqabc\x9c\x1bPqdef\x9cZ',
];
for (const text of streams) {
  const data = Array.from(text, character => character.codePointAt(0));
  for (const config of [0, 1, 2]) {
    await trace(config, [`p${hex(data)}`]);
    await trace(config, data.map(code => `p${code.toString(16)}`));
    for (let split = 0; split <= data.length; split++) {
      await trace(config, [`p${hex(data.slice(0, split))}`, `p${hex(data.slice(split))}`]);
    }
  }
}
for (const config of [0, 1, 2]) {
  await trace(config, ['p1b,50,71,41', 'r', 'p5a,1b,50,71,42,9c,59']);
  await trace(config, ['p1b,50,71,41', 'd', 'p9c,5a']);
}
const regressions = [];
{
  const parser = new EscapeSequenceParser();
  const calls = [];
  parser.setPrintHandler((data, start, end) => calls.push(`P:${hex(data.slice(start, end))}`));
  for (let index = 0; index < 3; index++) parser.registerDcsHandler({ final: 'q' }, {
    hook(value) { calls.push(`H:q${index}:${params(value)}`); },
    put(data, start, end) { calls.push(`D:q${index}:${hex(data.slice(start, end))}`); },
    unhook(success) {
      const report = () => { calls.push(`U:q${index}:${+success}`); return index === 0; };
      return index === 1 ? reference.resolvedPromise(report) : report();
    }
  });
  const data = Uint32Array.from([0x1b, 0x50, 0x71, 65, 0x9c, 90]);
  assert.equal(await parser.parse(data, data.length), false);
  parser.reset();
  assert.equal(parser.parse(data, data.length, false), undefined);
  assert.deepEqual(calls, ['H:q2:p0', 'H:q1:p0', 'H:q0:p0', 'D:q2:41', 'D:q1:41', 'D:q0:41', 'U:q2:1', 'U:q1:1', 'U:q0:0', 'P:5a']);
  regressions.push({ name: 'reset paused DCS resumes suffix exactly once', calls, passed: true });
}
for (const asynchronous of [false, true]) {
  const parser = new EscapeSequenceParser();
  const printed = [];
  parser.setPrintHandler((data, start, end) => printed.push(...data.slice(start, end)));
  parser.registerDcsHandler({ final: 'q' }, { hook() {}, put() {}, unhook(success) {
    if (!success) return false;
    if (asynchronous) return reference.resolvedPromise(() => { throw new Error('DCS rejected'); });
    throw new Error('DCS threw');
  } });
  const data = Uint32Array.from(Array.from('\x1bPqpayload\x9cSUFFIX', char => char.codePointAt(0)));
  if (asynchronous) await assert.rejects(parser.parse(data, data.length), /DCS rejected/);
  else assert.throws(() => parser.parse(data, data.length), /DCS threw/);
  assert.deepEqual(printed, []);
  parser.reset(); parser.clearDcsHandler({ final: 'q' });
  if (asynchronous) parser.parse(data, data.length, false);
  parser.parse(Uint32Array.of(65), 1);
  assert.equal(String.fromCodePoint(...printed), asynchronous ? 'SUFFIXA' : 'A');
  regressions.push({ name: asynchronous ? 'rejection retains saved chunk for reset/resume' : 'exception stops suffix, reset recovers', printed, passed: true });
}
{
  const parser = new EscapeSequenceParser();
  const tokens = [];
  const printed = [];
  parser.setPrintHandler((data, start, end) => printed.push(...data.slice(start, end)));
  for (let index = 0; index < 3; index++) tokens.push(parser.registerDcsHandler({ final: 'q' }, {
    hook() {}, put() {}, unhook() { if (index === 2) for (const token of tokens) token.dispose(); return false; }
  }));
  const data = Uint32Array.from([0x1b, 0x50, 0x71, 65, 0x9c, 90]);
  assert.throws(() => parser.parse(data, data.length), { name: 'TypeError' });
  assert.deepEqual(printed, []);
  parser.reset(); parser.parse(Uint32Array.of(65), 1);
  assert.deepEqual(printed, [65]);
  regressions.push({ name: 'removed active index propagates error and reset recovers', printed, passed: true });
}
const corpus = `${fixtures.join('\n')}\n`;
const fixture = 'crates/xterm-core/tests/fixtures/escape-dcs.tsv';
await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal(await readFile(fixture, 'utf8'), corpus, 'DCS integration trace drift');
const report = {
  node: process.version, platform: process.platform, sources,
  streams: streams.length, configurations: 3, traces: fixtures.length, parseCalls, pauses, regressions,
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'), fixtureBytes: Buffer.byteLength(corpus),
  scope: 'actual outer parser plus registered DCS, sync/async bubbling, all split boundaries, ordered callbacks and state snapshots'
};
await mkdir('results/dcs-integration', { recursive: true });
await writeFile('results/dcs-integration/oracle.json', `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
