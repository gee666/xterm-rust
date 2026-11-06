// Read-only checks on the preserved producer, before its temporary target is removed.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {resolve, join} from 'node:path';
const root = resolve(import.meta.dirname, '../..');
const read = file => readFileSync(resolve(root, file));
const json = file => JSON.parse(read(`results/backspace-sparse/${file}`));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const pins = json('input-pins.json');
for (const [file, expected] of Object.entries(pins.sources)) assert.equal(sha(read(file)), expected, file);
for (const {file, sha256, bytes} of pins.executables) {
  const actual = readFileSync(file);
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
assert.equal(source.cases, 480);
assert.deepEqual(source.sourceIntegrity, {before: 776, after: 776});
assert.equal(source.upstream.passed, 8);
assert.equal(source.upstream.executed, 8);
for (const key of ['failed', 'skipped', 'unrun']) assert.equal(source.upstream[key], 0);
for (const outcome of source.upstream.outcomes) assert.equal(outcome.status, 'passed');
assert.equal(sha(read('crates/xterm-core/tests/fixtures/backspace-sparse.tsv')), source.fixtureSha256);
const restoration = json('negative-restoration.json');
assert.equal(restoration.fixed, restoration.restored);
assert.equal(restoration.fixed, pins.sources['crates/xterm-core/src/terminal/controls.rs']);
assert.notEqual(restoration.fixed, restoration.old);
assert.match(read('results/backspace-sparse/negative-native-summary.txt').toString(), /cases=480\npassed=472\nfailed=8\nexpected_previous_row_errors=8\n/);
assert.match(read('results/backspace-sparse/native-summary.txt').toString(), /cases=480\npassed=480\nfailed=0\nexpected_previous_row_errors=8\n/);
for (const gate of json('gates.json')) assert.equal(gate.exit, 0, gate.name);
const formats = json('format.json');
assert.equal(formats.length, 6);
assert.equal(formats[0].exit, 1);
assert.match(read('results/backspace-sparse/fmt-exact.log').toString(), /os error 206/);
for (const gate of formats.slice(1)) assert.equal(gate.exit, 0, gate.name);
for (const {copy, sha256} of json('generated-report-restoration.json')) assert.equal(sha(read(copy)), sha256, copy);
console.log(JSON.stringify({status: 'passed', sources: Object.keys(pins.sources).length,
  executables: pins.executables.length, reference: manifest.length, sourceCases: source.cases,
  originalTests: source.upstream.passed, caveat: 'Windows aggregate fmt OS206 remains failed; no browser/Linux/macOS or whole-port proof.'}));
