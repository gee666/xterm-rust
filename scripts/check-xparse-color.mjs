// Copyright (c) 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { runScopedOracle, buildNativeOracle, recordSyncCommand } from './oracle-evidence.mjs';

const root = resolve(import.meta.dirname, '..');
process.chdir(root);
const attemptReport = await runScopedOracle(resolve('results/xparse-color'), {
  kind: 'xparse-color', command: 'node scripts/check-xparse-color.mjs', scope: 'XParseColor only; full equivalence NOT PROVEN'
}, async context => {
const resultDir = context.directory;
const { parseColor, toRgbString } = await import('../source_repo/src/common/input/XParseColor.ts');

// Execute every original assertion. Only the test runner and import plumbing
// change. These assertions use no Chai-specific behavior, so strict Node assert
// preserves their equality checks without installing the browser dependencies.
const originalTests = readFileSync('source_repo/src/common/input/XParseColor.test.ts', 'utf8');
let baselineCases = 0;
let baselineAssertions = 0;
const countedAssert = {
  deepEqual: (...args) => { baselineAssertions++; assert.deepEqual(...args); },
  equal: (...args) => { baselineAssertions++; assert.equal(...args); }
};
const baseline = originalTests
  .replace("import { assert } from 'chai';", '')
  .replace("import { parseColor, toRgbString } from './XParseColor';", '');
const jsTests = stripTypeScriptTypes(baseline);
new Function('assert', 'parseColor', 'toRgbString', 'describe', 'it', jsTests)(
  countedAssert, parseColor, toRgbString, (_name, fn) => fn(),
  (_name, fn) => { fn(); baselineCases++; }
);

const operations = [];
const expected = [];
let parseCases = 0;
let formatCases = 0;
function parse(spec) {
  operations.push(`p ${Buffer.from(spec).toString('hex')}`);
  expected.push(parseColor(spec)?.join(',') ?? 'undefined');
  parseCases++;
}
function format(color, bits) {
  operations.push(`f ${color.join(',')},${bits ?? 'default'}`);
  expected.push(toRgbString(color, bits));
  formatCases++;
}
// Every possible input-channel value at each supported width. Other channels
// vary in the same case to exercise mixed values and parsing offsets.
for (let width = 1; width <= 4; width++) {
  const max = 16 ** width;
  for (let value = 0; value < max; value++) {
    const channels = [value, max - value - 1, (value * 313) % max]
      .map(v => v.toString(16).padStart(width, '0'));
    parse(`rgb:${channels.join('/')}`);
    parse(`#${channels.join('')}`);
  }
}
for (let channel = 0; channel < 256; channel++) {
  for (const depth of [undefined, 0, 4, 8, 12, 16, 17, 4294967295]) {
    format([channel, 255 - channel, (channel * 13) % 256], depth);
  }
}
for (const spec of ['', '#', 'red', '#123\n', '#123\r', 'rgb:1/2/3\n',
  'rgb:1/2/3\u2028', '#１２３', 'rgb:α/β/γ', 'RGB:0/A/F', '#FFF',
  'rgb:0/11/222', 'rgbi:00/11/22', '#aabbbcc', '#aabbgg', 'rgb:aa/bb/gg']) parse(spec);

const seed = 0x58434f4c;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
const alphabet = [...'0123456789abcdefABCDEF#rgb:/ xyz\n\r\t', 'é', '😀', '１'];
for (let i = 0; i < 12000; i++) {
  const length = random() % 24;
  let spec = ['', '#', 'rgb:', 'RGB:'][random() % 4];
  for (let j = 0; j < length; j++) spec += alphabet[random() % alphabet.length];
  parse(spec);
}

context.checkpoint({ baselineCases, baselineAssertions, phase: 'native-build' });
const artifact = await buildNativeOracle(context, root, 'xparse_color_oracle');
const run = spawnSync(artifact.executable, [], {
  encoding: 'utf8', input: operations.join('\n') + '\n',
  maxBuffer: 32 * 1024 * 1024, timeout: 120000
});
writeFileSync(`${resultDir}/native-stderr.log`, run.stderr ?? '');
recordSyncCommand(context, [artifact.executable], run, { stderr: `${resultDir}/native-stderr.log` });
if (run.error) throw run.error;
assert.equal(run.status, 0, run.stderr);
context.checkpoint({ phase: 'native-output-comparison' });
const actual = run.stdout.trimEnd().split(/\r?\n/);
assert.equal(actual.length, expected.length);
for (let i = 0; i < actual.length; i++) {
  if (actual[i] !== expected[i]) {
    writeFileSync(`${resultDir}/counterexample.json`, JSON.stringify({ seed, index: i, operation: operations[i], expected: expected[i], actual: actual[i] }, null, 2));
    assert.fail(`Mismatch at operation ${i}; see ${resultDir}/counterexample.json`);
  }
}
const source = readFileSync('source_repo/src/common/input/XParseColor.ts');
const summary = {
  scope: 'XParseColor only; full terminal equivalence NOT PROVEN',
  command: 'node scripts/check-xparse-color.mjs',
  node: process.version, rustc: artifact.rustc, platform: process.platform,
  arch: process.arch, seed, baselineCases, baselineAssertions,
  parseCases, formatCases, totalCompared: actual.length, mismatches: 0,
  upstreamSha256: createHash('sha256').update(source).digest('hex'),
  operationSha256: createHash('sha256').update(operations.join('\n')).digest('hex'),
  outputSha256: createHash('sha256').update(actual.join('\n')).digest('hex')
};
return { ...summary, status: 'passed' };
});
console.log(JSON.stringify(attemptReport, null, 2));
