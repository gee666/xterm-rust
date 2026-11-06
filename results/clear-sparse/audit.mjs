// Read-only producer checks. Cached executable checks apply before owned cleanup.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
const root = resolve(import.meta.dirname, '../..');
const read = file => readFileSync(resolve(root, file));
const json = file => JSON.parse(read(`results/clear-sparse/${file}`));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const pins = json('input-pins.json');
for (const [file, expected] of Object.entries(pins.sources)) assert.equal(sha(read(file)), expected, file);
for (const {file, sha256, bytes} of pins.executables) {
  const actual = read(file);
  assert.equal(actual.length, bytes, file);
  assert.equal(sha(actual), sha256, file);
}
const reference = resolve(process.env.XTERM_SOURCE_ROOT ?? join(root, 'source_repo'));
const manifest = JSON.parse(read('docs/baseline/reference-manifest.json'));
for (const entry of manifest) {
  const bytes = readFileSync(join(reference, entry.path));
  assert.equal(bytes.length, entry.bytes, entry.path);
  assert.equal(sha(bytes), entry.sha256, entry.path);
}
const source = json('source.json');
assert.equal(source.cases, 280);
assert.deepEqual(source.sourceIntegrity, {before: 776, after: 776});
assert.equal(source.upstream.passed, 2);
assert.equal(source.upstream.executed, 2);
for (const key of ['failed', 'skipped', 'unrun']) assert.equal(source.upstream[key], 0);
for (const outcome of source.upstream.outcomes) assert.equal(outcome.status, 'passed');
assert.equal(sha(read('crates/xterm-core/tests/fixtures/clear-sparse.tsv')), source.fixtureSha256);
const restoration = json('negative-restoration.json');
assert.equal(restoration.fixed, restoration.restored);
assert.equal(restoration.fixed, pins.sources['crates/xterm-core/src/terminal/operations.rs']);
assert.notEqual(restoration.fixed, restoration.old);
assert.equal(restoration.fixtureBefore, restoration.fixtureAfter);
assert.deepEqual(restoration.negative, {exit: 101, error: null});
assert.deepEqual(restoration.restoredRun, {exit: 0, error: null});
assert.equal(read('results/clear-sparse/negative-native-summary.txt').toString(), 'cases=280\npassed=88\nfailed=192\n');
assert.equal(read('results/clear-sparse/native-summary.txt').toString(), 'cases=280\npassed=280\nfailed=0\n');
assert.equal(read('results/clear-sparse/counterexamples.txt').length, 0);
for (const gate of json('gates.json')) {
  assert.equal(gate.exit, 0, gate.name);
  assert.equal(gate.error, null, gate.name);
  assert.doesNotMatch(read(`results/clear-sparse/${gate.name}.log`).toString(), /^warning:/m, gate.name);
}
const core = read('results/clear-sparse/core-all.log').toString();
const summaries = [...core.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)];
assert.equal(summaries.length, 256);
assert.deepEqual(summaries.reduce((a, r) => a.map((v, i) => v + Number(r[i + 1])), [0, 0, 0, 0, 0]), [1593, 0, 0, 0, 0]);
const formats = json('format.json');
assert.equal(formats.length, 6);
assert.equal(formats[0].exit, 1);
assert.match(read('results/clear-sparse/fmt-exact.log').toString(), /os error 206/);
for (const gate of formats.slice(1)) assert.equal(gate.exit, 0, gate.name);
for (const {copy, sha256} of json('generated-report-restoration.json')) assert.equal(sha(read(copy)), sha256, copy);
console.log(JSON.stringify({status: 'passed', sources: Object.keys(pins.sources).length,
  executables: pins.executables.length, reference: manifest.length, sourceCases: source.cases,
  originalTests: source.upstream.passed, core: 1593,
  caveat: 'Windows aggregate fmt OS206 remains failed. No whole-workspace runtime, browser/Linux/macOS or equivalence proof.'}));
