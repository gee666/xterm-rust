// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
// Run with Node 24 --experimental-transform-types. Reference sources stay unchanged.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo/src/common/StringBuilder.ts');
const tests = path.join(root, 'source_repo/src/common/StringBuilder.test.ts');
const { StringBuilder, LimitedStringBuilder } = await import(pathToFileURL(source));
const hash = async file => createHash('sha256').update(await readFile(file)).digest('hex');
const resultDir = path.join(root, 'results/string-builder');
const fixture = path.join(root, 'crates/xterm-core/tests/fixtures/string-builder-traces.tsv');
await Promise.all([mkdir(resultDir, { recursive: true }), mkdir(path.dirname(fixture), { recursive: true }),
  mkdir(path.join(root, 'tmp'), { recursive: true })]);

// Only the assertion-library/module imports change in the temporary test copy.
// Test bodies and assertions are executed unchanged, against the real source.
const testText = (await readFile(tests, 'utf8'))
  .replace("import { assert } from 'chai';", "import { strict as assert } from 'node:assert';")
  .replace("from './StringBuilder'", `from '${pathToFileURL(source).href}'`);
const outcomes = [];
const suites = [];
globalThis.describe = (name, fn) => {
  suites.push(name);
  try { fn(); } finally { suites.pop(); }
};
globalThis.it = (name, fn) => {
  try {
    fn();
    outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' });
  } catch (error) {
    outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) });
  }
};
const baseline = path.join(root, 'tmp/string-builder-baseline.mts');
await writeFile(baseline, testText);
await import(pathToFileURL(baseline));
delete globalThis.describe;
delete globalThis.it;

const seed = 0x51b01d;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
const hex = units => Array.from(units, unit => unit.toString(16)).join(',');
const unitsOf = str => Array.from({ length: str.length }, (_, i) => str.charCodeAt(i));
const rows = [];
let traceCount = 0;
function trace(limit, operations, javascriptLimit = false) {
  const id = traceCount++;
  const builder = limit === '-' ? new StringBuilder() : new LimitedStringBuilder(limit);
  for (const units of operations) {
    let overflow = false;
    if (units === null) builder.reset();
    else overflow = builder.append(String.fromCharCode(...units)) === true;
    const limitLabel = javascriptLimit ? `j:${Object.is(limit, -0) ? '-0' : String(limit)}` : limit;
    rows.push([id, limitLabel, units === null ? 'reset' : 'append', hex(units ?? []),
      Number(overflow), builder.length, hex(unitsOf(builder.toString()))].join('\t'));
  }
}
for (const limit of ['-', 0, 1, 2, 3, 5, 10, 65536]) {
  trace(limit, [[], [0xd800], [0xdc00], [0], [0xffff], null,
    [0x41, 0xd83d, 0xde00], [0x301], [], null, [0xdc00], [0xd800]]);
}
const interesting = [0, 0x41, 0x301, 0xd800, 0xdbff, 0xdc00, 0xdfff, 0xfeff, 0xffff];
for (let i = 0; i < 256; i++) {
  const limit = i % 4 === 0 ? '-' : random() % 65;
  const operations = [];
  for (let j = 0; j < 64; j++) {
    if (random() % 10 === 0) operations.push(null);
    else operations.push(Array.from({ length: random() % 9 }, () =>
      random() % 2 ? interesting[random() % interesting.length] : random() % 65536));
  }
  trace(limit, operations);
}
const numericLimits = [-0, 0, 0.5, 1, 1.5, -0.5, -1, NaN, Infinity, -Infinity,
  Number.MAX_SAFE_INTEGER, 2 ** 53, Number.MAX_VALUE, Number.MIN_VALUE];
for (const limit of numericLimits) {
  trace(limit, [[], [0x41], [0x42], [], null, [], [0xd800], [0xdc00],
    null, [0x41, 0xd83d, 0xde00], [0x301], []], true);
}
// Every single JS code unit is legal, including lone surrogates.
for (let unit = 0; unit <= 0xffff; unit++) {
  const builder = new StringBuilder();
  builder.append(String.fromCharCode(unit));
  if (builder.length !== 1 || builder.toString().charCodeAt(0) !== unit) {
    throw new Error(`single-unit mismatch ${unit}`);
  }
}
await writeFile(fixture, rows.join('\n') + '\n');
const report = {
  source: path.relative(root, source), sourceSha256: await hash(source),
  tests: path.relative(root, tests), testsSha256: await hash(tests),
  node: process.version, platform: process.platform, arch: process.arch,
  harness: 'Node TypeScript transform; synchronous describe/it; Node strict assertions replace chai',
  seed, traces: traceCount, operations: rows.length,
  numericLimitTraces: numericLimits.length, exhaustiveSingleUnits: 65536,
  fixture: path.relative(root, fixture), fixtureSha256: await hash(fixture),
  baseline: { discovered: outcomes.length, executed: outcomes.length,
    passed: outcomes.filter(o => o.status === 'passed').length,
    failed: outcomes.filter(o => o.status === 'failed').length, skipped: 0, unrun: 0, outcomes }
};
await writeFile(path.join(resultDir, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
if (report.baseline.failed) process.exitCode = 1;
