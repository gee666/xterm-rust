import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { workloads } from './matched-benchmark-workloads.mjs';
import { prepare } from './matched-benchmark-js.mjs';
import { prepared, verify, sha256, checkPairedReceipt } from './matched-benchmark-adapter.mjs';
import { jsonHash } from './benchmark-contract.mjs';

test('workloads preserve byte/text chunk entry points and Unicode provider identities', () => {
  const cases = workloads();
  assert.equal(new Set(cases.map(row => row.id)).size, 7);
  assert.deepEqual(cases.map(row => row.options.unicode.version), ['6', '6', '11', '15-graphemes', '6', '6', '11']);
  for (const row of cases) assert.equal(prepare(row).length, row.operations.length);
  assert.equal(typeof prepare(cases[0])[0].input, 'string');
  assert.ok(prepare(cases[1])[0].input instanceof Uint8Array);
  assert.equal(cases[3].operations.length, 2);
  assert.equal(cases[5].operations.filter(op => op.kind === 'resize').length, 2);
  assert.equal(cases[6].operations.filter(op => op.kind === 'addon').length, 2);
});
test('unsupported settings, providers and addon calls fail before timing', () => {
  const changes = [
    row => { row.options.settings.foo = 1; },
    row => { row.options.unicode.settings.foo = 1; },
    row => { row.options.unicode.version = '15'; },
    row => { row.operations = [{ kind: 'addon', addon: 'search', method: 'findNext', arguments: [] }]; },
    row => { row.operations = [{ kind: 'addon', addon: 'serialize', method: 'serializeAsHTML', arguments: [] }]; },
    row => { row.operations = [{ kind: 'addon', addon: 'serialize', method: 'serialize', arguments: [{}] }]; }
  ];
  for (const change of changes) {
    const row = workloads()[0]; change(row); assert.throws(() => prepare(row));
  }
});
test('native preparation preserves lone surrogates, split UTF8 and exact operations', () => {
  const row = workloads()[0];
  row.operations = [
    { kind: 'write', encoding: 'utf16le', data: Buffer.from([0, 0xd8, 0, 0xdc, 0, 0xd8]).toString('base64') },
    { kind: 'write', encoding: 'utf8-bytes', data: Buffer.from([0xf0, 0x9f]).toString('base64') },
    { kind: 'write', encoding: 'utf8-bytes', data: Buffer.from([0x98, 0x80]).toString('base64') },
    { kind: 'write', encoding: 'utf16le', data: '' }, { kind: 'resize', cols: 4, rows: 2 }
  ];
  const result = prepared({ warmup: 2, repetitions: 10, cases: [row] });
  assert.deepEqual(result.cases[0].operations.map(op => op.units), ['d800,dc00,d800', 'f0,9f', '98,80', '', undefined]);
  assert.deepEqual(result.cases[0].operations.at(-1), row.operations.at(-1));
  assert.equal(prepare(row)[0].input.charCodeAt(2), 0xd800);
});
test('timing rejects failed or stale actual paired workload/source/artifact captures', () => {
  const build = { sourceSha256: 'source', artifactSha256: 'artifact', flags: 'real flags and runtime pins' };
  const receipt = { status: 'passed', manifestSha256: 'manifest', builds: { rust: build } };
  checkPairedReceipt(receipt, 'manifest', build, 'rust');
  assert.throws(() => checkPairedReceipt({ ...receipt, status: 'failed' }, 'manifest', build, 'rust'));
  assert.throws(() => checkPairedReceipt(receipt, 'changed', build, 'rust'));
  assert.throws(() => checkPairedReceipt(receipt, 'manifest', { ...build, sourceSha256: 'changed' }, 'rust'));
  assert.throws(() => checkPairedReceipt(receipt, 'manifest', { ...build, artifactSha256: 'changed' }, 'rust'));
  assert.throws(() => checkPairedReceipt(receipt, 'manifest', { ...build, flags: 'changed' }, 'rust'));
});
test('provenance rehashes sources and artifacts and rejects drift or relabelled digests', () => {
  const scratch = mkdtempSync(join(process.env.TMPDIR ?? 'tmp', 'matched-pins-'));
  try {
    const path = join(scratch, 'fixture'); writeFileSync(path, 'fixture');
    const sources = [{ name: 'fixture', path, sha256: sha256(path) }];
    const artifacts = { native: { path, sha256: sha256(path) } };
    const build = { sourceSha256: jsonHash(sources.map(({ name, sha256 }) => ({ name, sha256 }))),
      artifactSha256: jsonHash(Object.entries(artifacts).map(([name, pin]) => ({ name, sha256: pin.sha256 }))) };
    assert.equal(verify({ sources, artifacts, build }), build);
    assert.throws(() => verify({ sources, artifacts, build: { ...build, sourceSha256: '0'.repeat(64) } }));
    assert.throws(() => verify({ sources, artifacts, build: { ...build, artifactSha256: '0'.repeat(64) } }));
    assert.throws(() => verify({ sources: [], artifacts, build }));
    assert.throws(() => verify({ sources, artifacts: {}, build }));
    assert.throws(() => verify({ sources: [...sources, ...sources], artifacts, build }));
    writeFileSync(path, 'changed'); assert.throws(() => verify({ sources, artifacts, build }));
  } finally { rmSync(scratch, { recursive: true }); }
});
