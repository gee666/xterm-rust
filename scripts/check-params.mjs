// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Generate a deterministic corpus from the unchanged upstream implementation.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';

const sourcePath = 'source_repo/src/common/parser/Params.ts';
const source = await readFile(sourcePath, 'utf8');
const importLine = /^import \{ IParams, ParamsArray \} from '\.\/Types';\r?\n/m;
assert.match(source, importLine);
// These interfaces have no runtime implementation. Enum transformation retains
// the source constants and all method bodies, including Uint16 offset behavior.
const js = stripTypeScriptTypes(source.replace(importLine, ''), { mode: 'transform' });
const { Params } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const seed = 0x2389abcd;
let state = seed;
function next() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
const cases = 512;
const steps = 96;
const lines = [];
let operations = 0;
const values = [-2, -1, 0, 1, 9, 23, 2147483647, 2147483648, 9007199254740991];
for (let index = 0; index < cases; index++) {
  const capacity = [1, 3, 12, 32][next() % 4];
  const subCapacity = [0, 1, 3, 32, 255, 256][next() % 6];
  let params = new Params(capacity, subCapacity);
  const ops = [];
  const snapshots = [];
  for (let step = 0; step < steps; step++) {
    const choice = next() % 20;
    let op;
    let error = '';
    try {
      if (choice < 6) {
        const value = values[next() % values.length];
        op = `p${value}`;
        params.addParam(value);
      } else if (choice < 12) {
        const value = values[next() % values.length];
        op = `s${value}`;
        params.addSubParam(value);
      } else if (choice < 17) {
        const digit = next() % 10;
        op = `d${digit}`;
        params.addDigit(digit);
      } else if (choice === 17) {
        op = 'r';
        params.reset();
      } else if (choice === 18) {
        op = 'z';
        params.resetZdm();
      } else {
        op = 'c';
        params = params.clone();
      }
    } catch (caught) {
      assert.equal(caught.message, 'values less than -1 are not allowed');
      error = '!';
    }
    ops.push(op);
    snapshots.push(`${error}${JSON.stringify(params.toArray())}`);
    // Exercise borrowed access, owned snapshots, and logical lengths too.
    const all = params.getSubParamsAll();
    for (let i = 0; i < params.length; i++) {
      assert.equal(params.hasSubParams(i), !!all[i]);
      assert.deepEqual(params.getSubParams(i), all[i] ?? null);
    }
    operations++;
  }
  lines.push([capacity, subCapacity, ops.join(','), snapshots.join('|')].join('\t'));
}
// Explicitly reach the 256-subparameter packed-offset boundary.
const boundary = new Params(2, 256);
boundary.addParam(1);
const boundaryOps = ['p1'];
const boundarySnapshots = ['[1]'];
for (let i = 0; i < 256; i++) {
  boundary.addSubParam(2);
  boundaryOps.push('s2');
  boundarySnapshots.push(JSON.stringify(boundary.toArray()));
}
lines.push([2, 256, boundaryOps.join(','), boundarySnapshots.join('|')].join('\t'));
const corpus = lines.join('\n') + '\n';
const path = 'crates/xterm-core/tests/fixtures/params-oracle.tsv';
if (process.argv.includes('--generate')) {
  await mkdir('crates/xterm-core/tests/fixtures', { recursive: true });
  await writeFile(path, corpus);
} else {
  const committed = (await readFile(path, 'utf8')).replace(/\r\n/g, '\n');
  assert.equal(committed, corpus, 'committed oracle corpus differs');
}
const summary = {
  sourcePath,
  sourceSha256: createHash('sha256').update(source).digest('hex'),
  node: process.version,
  seed,
  randomizedCases: cases,
  stepsPerCase: steps,
  boundaryCases: 1,
  comparedSnapshots: operations + 257,
  corpusSha256: createHash('sha256').update(corpus).digest('hex'),
  status: process.argv.includes('--generate') ? 'generated' : 'matched'
};
await mkdir('results/params', { recursive: true });
await writeFile('results/params/oracle.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
