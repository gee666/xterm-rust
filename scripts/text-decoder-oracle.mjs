// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Uses the unchanged upstream TextDecoder oracle.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { strict as assert } from 'node:assert';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo/src/common/input/TextDecoder.ts');
const sourceTest = path.join(root, 'source_repo/src/common/input/TextDecoder.test.ts');
const decoder = await import(pathToFileURL(source).href);
const hash = async file => createHash('sha256').update(await readFile(file)).digest('hex');
const fixtureDir = path.join(root, 'crates/xterm-core/tests/fixtures');
const resultsDir = path.join(root, 'results/text-decoder');
await mkdir(fixtureDir, { recursive: true });
await mkdir(resultsDir, { recursive: true });
await mkdir(path.join(root, 'tmp'), { recursive: true });

// The suite is synchronous and uses only describe/it and chai strictEqual.
// Replace the assertion import with Node's strictEqual, without editing assertions.
const testText = (await readFile(sourceTest, 'utf8'))
  .replace("import { assert } from 'chai';", "import { strict as assert } from 'node:assert';")
  .replace("from './TextDecoder'", `from '${pathToFileURL(source).href}'`);
const outcomes = [];
const suites = [];
globalThis.describe = (name, callback) => {
  suites.push(name);
  try { callback(); } finally { suites.pop(); }
};
globalThis.it = (name, callback) => {
  try {
    callback.call({});
    outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' });
  } catch (error) {
    outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) });
  }
};
const baselineModule = path.join(root, 'tmp/text-decoder-baseline.mts');
await writeFile(baselineModule, testText);
await import(pathToFileURL(baselineModule).href);
delete globalThis.describe;
delete globalThis.it;

const seed = 0xdec0de;
let state = seed;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
const hexList = values => Array.from(values, value => value.toString(16)).join(',');
const fixtures = [];
function trace(kind, chunks) {
  const instance = kind === '8' ? new decoder.Utf8ToUtf32() : new decoder.StringToUtf32();
  const outputs = [];
  const states = [];
  for (const chunk of chunks) {
    // A pending UTF-16 high surrogate can add one extra output value.
    const target = new Uint32Array(chunk.length + 1);
    const input = kind === '8' ? Uint8Array.from(chunk) : String.fromCharCode(...chunk);
    const written = instance.decode(input, target);
    outputs.push(hexList(target.subarray(0, written)));
    if (kind === '8') states.push(hexList(instance.interim));
  }
  fixtures.push([kind, chunks.map(hexList).join(';'), outputs.join(';'), states.join(';')].join('\t'));
}
for (const units of [
  [0xd800, 0xd800, 0xdc00], [0xd800, 0xfeff, 0x41], [0xdc00, 0xdfff],
  [0xfeff, 0x41, 0xfeff], [0xdbff, 0xdfff], [0xd800, 0xd800, 0xd800, 0xdc00]
]) {
  for (let step = 1; step <= units.length; step++) {
    const chunks = [];
    for (let i = 0; i < units.length; i += step) chunks.push(units.slice(i, i + step));
    trace('16', chunks);
  }
}
for (const bytes of [
  [0xc0, 0x80, 0x41], [0xc1, 0xbf, 0x41], [0xed, 0xa0, 0x80],
  [0xe0, 0x80, 0x80], [0xf0, 0x80, 0x80, 0x80], [0xf4, 0x90, 0x80, 0x80],
  [0xef, 0xbb, 0xbf], [0xf0, 0xa0, 0x9c, 0x8e], [0xf0, 0x90, 0x80, 0x80],
  [0xe2, 0xc3, 0x84, 0x41], [0xf0, 0x80, 0x41], [0xff, 0x80, 0xfe, 0x7f]
]) {
  for (let step = 1; step <= bytes.length; step++) {
    const chunks = [];
    for (let i = 0; i < bytes.length; i += step) chunks.push(bytes.slice(i, i + step));
    trace('8', chunks);
  }
}
for (const kind of ['8', '16']) {
  const interesting = kind === '8'
    ? [0, 0x41, 0x7f, 0x80, 0xbf, 0xc0, 0xc3, 0xe0, 0xe2, 0xed, 0xef, 0xf0, 0xf4, 0xf7, 0xff]
    : [0, 0x41, 0xd800, 0xd801, 0xdbff, 0xdc00, 0xdc01, 0xdfff, 0xfeff, 0xffff];
  for (let iteration = 0; iteration < 1024; iteration++) {
    const chunks = [];
    for (let c = 0, count = 1 + random() % 12; c < count; c++) {
      const chunk = [];
      for (let i = 0, size = random() % 16; i < size; i++) {
        chunk.push(random() % 2 ? interesting[random() % interesting.length] : random() % (kind === '8' ? 256 : 65536));
      }
      chunks.push(chunk);
    }
    trace(kind, chunks);
  }
}
async function fixture(name, content) {
  const file = path.join(fixtureDir, name);
  if (process.argv.includes('--generate')) await writeFile(file, content);
  else assert.equal((await readFile(file, 'utf8')).replace(/\r\n/g, '\n'), content, name);
}
await fixture('text-decoder-traces.tsv', fixtures.join('\n') + '\n');
// Preserve the exact upstream multilingual strings, including all code units.
const strings = (await readFile(sourceTest, 'utf8')).match(/const TEST_STRINGS = \[([\s\S]*?)\r?\n\];/)[1];
const samples = Function(`return [${strings}];`)();
await fixture('text-decoder-strings.txt', samples.join('\n') + '\n');
const conversionValues = [0, 1, 0xfeff, 0xd800, 0xdbff, 0xdc00, 0xdfff, 0xffff,
  0x10000, 0x10ffff, 0x110000, 0x80000000, 0x80010000, 0xffffffff];
const conversionRows = conversionValues.map(value => {
  const converted = decoder.stringFromCodePoint(value);
  return `${value.toString(16)}\t${hexList(Array.from({length: converted.length}, (_, i) => converted.charCodeAt(i)))}`;
});
await fixture('text-decoder-conversion.tsv', conversionRows.join('\n') + '\n');
const report = {
  source: 'source_repo/src/common/input/TextDecoder.ts', sourceSha256: await hash(source),
  test: 'source_repo/src/common/input/TextDecoder.test.ts', testSha256: await hash(sourceTest),
  node: process.version, platform: process.platform, arch: process.arch,
  harness: 'Native Node TypeScript stripping, synchronous describe/it, Node strictEqual replacing chai strictEqual',
  seed, generatedTraces: fixtures.length, conversionCases: conversionValues.length,
  baseline: { discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(x => x.status === 'passed').length,
    failed: outcomes.filter(x => x.status === 'failed').length, skipped: 0, unrun: 0, outcomes }
};
await writeFile(path.join(resultsDir, 'oracle.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ traces: fixtures.length, baseline: report.baseline.passed, failed: report.baseline.failed, seed }));
if (report.baseline.failed) process.exitCode = 1;
