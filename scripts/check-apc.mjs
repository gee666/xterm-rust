// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Uses read-only upstream source and assertions.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import vm from 'node:vm';

const paths = [
  'source_repo/src/common/parser/Constants.ts',
  'source_repo/src/common/input/TextDecoder.ts',
  'source_repo/src/common/StringBuilder.ts',
  'source_repo/src/common/parser/ApcParser.ts',
  'source_repo/src/common/parser/ApcParser.test.ts',
];
const texts = await Promise.all(paths.map(path => readFile(path, 'utf8')));
// Imports are the only source edits before TypeScript erasure. Assertions and
// implementation bodies remain unchanged and run in their original order.
function erase(text) {
  return stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
}
const suites = [];
const tests = [];
const context = vm.createContext({ assert,
  describe(name, callback) {
    suites.push({ name, before: [], after: [] });
    callback();
    suites.pop();
  },
  beforeEach(callback) { suites.at(-1).before.push(callback); },
  afterEach(callback) { suites.at(-1).after.push(callback); },
  it(name, callback) { tests.push({ name: [...suites.map(s => s.name), name].join(' / '),
    callback, before: suites.flatMap(s => s.before), after: suites.toReversed().flatMap(s => s.after) }); },
});
vm.runInContext(texts.slice(0, -1).map(erase).join('\n') + '\nglobalThis.Parser = ApcParser;', context);
vm.runInContext(erase(texts.at(-1)), context);
const outcomes = [];
for (const test of tests) {
  try {
    for (const before of test.before) await before();
    await test.callback.call({ timeout() {} });
    outcomes.push({ name: test.name, status: 'passed' });
  } catch (error) {
    outcomes.push({ name: test.name, status: 'failed', error: String(error.stack) });
  } finally {
    for (const after of test.after) await after();
  }
}

const seed = 0xa9c2025;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
const units = data => Array.from({ length: data.length }, (_, i) => data.charCodeAt(i).toString(16)).join(',');
const dataToString = data => vm.runInContext(`utf32ToString(Uint32Array.from([${data.join(',')}]))`, context);
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
      const data = [[], [65], [0xd800], [0x1f600, 0x4e2d], [0xfeff, 0, 0x10ffff]][random() % 5];
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
const report = {
  node: process.version, seed, randomizedCases: cases, stepsPerCase: steps,
  baseline: { discovered: outcomes.length, passed: outcomes.filter(x => x.status === 'passed').length,
    failed: outcomes.filter(x => x.status === 'failed').length, skipped: 0, outcomes },
  sources: paths.map((path, index) => ({ path, sha256: createHash('sha256').update(texts[index]).digest('hex') })),
  fixtureSha256: createHash('sha256').update(corpus).digest('hex'),
  status: process.argv.includes('--generate') ? 'generated' : 'matched',
};
await mkdir('results/apc', { recursive: true });
await writeFile('results/apc/oracle.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...report, baseline: { ...report.baseline, outcomes: undefined } }, null, 2));
assert.equal(report.baseline.failed, 0, 'upstream APC baseline failed');
