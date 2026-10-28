// Copyright (c) 2019 The xterm.js authors. MIT license.
// Actual source typed-array writes, including silently discarded overflow.
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { strict as assert } from 'node:assert';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import path from 'node:path';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo/src/common/input/TextDecoder.ts');
const api = await import(pathToFileURL(source).href);
const fixture = await readFile(path.join(root, 'crates/xterm-core/tests/fixtures/text-decoder-traces.tsv'), 'utf8');
const parse = text => text ? text.split(',').map(value => parseInt(value, 16)) : [];
const capacities = [0, 1, 2, 5, 13];
let traces = 0, chunks = 0, discarded = 0;
for (const line of fixture.split(/\r?\n/).filter(line => line.length > 0)) {
  const [kind, inputText, expectedText, stateText] = line.split('\t');
  const inputs = inputText.split(';').map(parse), expected = expectedText.split(';').map(parse);
  const states = stateText.split(';').map(parse);
  for (const capacity of capacities) {
    const decoder = kind === '8' ? new api.Utf8ToUtf32() : new api.StringToUtf32();
    for (let step = 0; step < inputs.length; step++) {
      const target = new Uint32Array(capacity).fill(0xdeadbeef);
      const input = kind === '8' ? Uint8Array.from(inputs[step]) : String.fromCharCode(...inputs[step]);
      const count = decoder.decode(input, target);
      assert.equal(count, expected[step].length);
      assert.deepEqual(Array.from(target), Array.from({length: capacity}, (_, i) => expected[step][i] ?? 0xdeadbeef));
      if (kind === '8') assert.deepEqual(Array.from(decoder.interim), states[step]);
      discarded += Math.max(0, count - capacity);
      chunks++;
    }
    traces++;
  }
}
assert.equal(traces, 2107 * capacities.length);
assert.ok(chunks > 60000);
assert.ok(discarded > 0);
const report = { scope: 'Fixed-target text decoder behavior, not terminal equivalence',
  node: process.version, platform: process.platform, architecture: process.arch,
  sourceSha256: createHash('sha256').update(await readFile(source)).digest('hex'),
  fixtureSha256: createHash('sha256').update(fixture.replace(/\r\n/g, '\n')).digest('hex'),
  capacities, inputTraces: 2107, comparedTraces: traces, comparedChunks: chunks,
  discardedWrites: discarded, failed: 0 };
await mkdir(path.join(root, 'results/text-decoder'), { recursive: true });
await writeFile(path.join(root, 'results/text-decoder/fixed-target.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
