// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Uses read-only upstream source and assertions.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { parserOracle, utf16Hex as units } from './parser-oracle-common.mjs';

const oracle = await parserOracle('Apc');
const { context } = oracle;

const seed = 0xa9c2025;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
const dataToString = data => oracle.utf32String(data);
const lines = [];
const cases = 256;
const steps = 80;
for (let index = 0; index < cases; index++) {
  const parser = new context.Parser();
  const output = [];
  const tokens = [];
  const ops = [];
  const attachFallback = () => parser.setHandlerFallback((id, action, data) => {
    output.push(`f:${id}:${action === 'START' ? 's' : action === 'END' ? `e:${Number(data)}` : `p:${units(data)}`}`);
  });
  attachFallback();
  for (let step = 0; step < steps; step++) {
    const kind = random() % 20;
    const id = random() % 3 + 65;
    if (kind < 4) {
      const name = tokens.length;
      const handled = random() % 2;
      ops.push(`r:${id}:${name}:${handled}`);
      tokens.push(parser.registerHandler(id, {
        start() { output.push(`h:${name}`); },
        put(data, start, end) { output.push(`p:${name}:${units(dataToString(Array.from(data.slice(start, end))))}`); },
        end(success) { output.push(`e:${name}:${Number(success)}`); return !!handled; },
      }));
    } else if (kind < 7) {
      ops.push(`s:${id}`); parser.start(id);
    } else if (kind < 12) {
      const data = [[], [65], [0xd800], [0x1f600, 0x4e2d], [0xfeff, 0, 0x10ffff, 0xffffffff]][random() % 5];
      ops.push(`p:${data.map(x => x.toString(16)).join(',')}`);
      parser.put(Uint32Array.from(data), 0, data.length);
    } else if (kind < 15) {
      const success = random() % 2;
      ops.push(`e:${success}`); parser.end(!!success);
    } else if (kind === 15) {
      ops.push('x'); parser.reset();
    } else if (kind === 16) {
      ops.push(`c:${id}`); parser.clearHandler(id);
    } else if (kind < 19 && tokens.length) {
      const token = random() % tokens.length;
      ops.push(`d:${token}`); tokens[token].dispose();
    } else {
      ops.push('q'); parser.dispose(); attachFallback();
    }
  }
  lines.push([ops.join(';'), output.join('|')].join('\t'));
}
const corpus = lines.join('\n') + '\n';
const fixture = 'crates/xterm-core/tests/fixtures/apc-oracle.tsv';
await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
if (process.argv.includes('--generate')) await writeFile(fixture, corpus);
else assert.equal(await readFile(fixture, 'utf8'), corpus, 'APC oracle drift');
const mutableListErrors = [];
for (const stage of ['start', 'put', 'end', 'reset', 'shift']) {
  const parser = new context.Parser();
  const log = [];
  const tokens = [];
  const handler = name => ({
    start() { log.push(`${name}:s`); if (name === 'b' && ['start', 'shift'].includes(stage)) remove(); },
    put() { log.push(`${name}:p`); if (name === 'b' && stage === 'put') remove(); },
    end(success) { log.push(`${name}:e:${Number(success)}`); if (name === 'b' && ['end', 'reset'].includes(stage)) remove(); return false; },
  });
  function remove() { for (const token of stage === 'shift' ? tokens.slice(0, 1) : tokens) token.dispose(); }
  tokens.push(parser.registerHandler(71, handler('a')));
  tokens.push(parser.registerHandler(71, handler('b')));
  let error;
  try {
    parser.start(71);
    if (stage === 'put') parser.put(Uint32Array.from([65]), 0, 1);
    if (stage === 'end') parser.end(true);
    if (stage === 'reset') parser.reset();
  } catch (caught) { error = { name: caught.name, message: caught.message }; }
  assert.equal(error?.name, stage === 'shift' ? undefined : 'TypeError', `${stage} must preserve source reverse-index behavior`);
  mutableListErrors.push({ stage, log, error });
}
const mutationFixture = mutableListErrors.map(row => `${row.stage}\t${row.log.join('|')}\t${row.error?.name ?? 'none'}`).join('\n') + '\n';
const mutationPath = 'crates/xterm-core/tests/fixtures/apc-mutation.tsv';
if (process.argv.includes('--generate')) await writeFile(mutationPath, mutationFixture);
else assert.equal(await readFile(mutationPath, 'utf8'), mutationFixture, 'APC mutable-list oracle drift');
const report = {
  node: process.version, seed, randomizedCases: cases, stepsPerCase: steps,
  baseline: { ...oracle.baseline, unrun: 0 },
  sources: oracle.sources,
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'),
  mutationFixtureSha256: createHash('sha256').update(mutationFixture).digest('hex'),
  mutableListErrors,
  status: process.argv.includes('--generate') ? 'generated' : 'matched',
};
await mkdir('results/apc', { recursive: true });
await writeFile('results/apc/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...report, baseline: { ...report.baseline, outcomes: undefined } }, null, 2));
assert.equal(report.baseline.failed, 0, 'upstream APC baseline failed');
assert.equal(report.baseline.discovered, 20, 'upstream APC inventory changed');
