import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { referenceIntegrity, sha256, unitCounts, createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const tempRoot = path.join(root, 'tmp');
fs.mkdirSync(tempRoot, { recursive: true });

test('manifest detects same-length modifications, growth and missing files', () => {
  const directory = fs.mkdtempSync(path.join(tempRoot, 'upstream-baseline-test-'));
  try {
    const file = path.join(directory, 'reference.ts');
    const original = Buffer.from('assert.equal(1, 1);\n');
    const manifest = [{ path: 'reference.ts', bytes: original.length, sha256: sha256(original) }];
    fs.writeFileSync(file, original);
    assert.deepEqual(referenceIntegrity(directory, manifest), { files: 1, mismatches: [] });
    fs.writeFileSync(file, 'assert.equal(1, 2);\n');
    assert.equal(referenceIntegrity(directory, manifest).mismatches.length, 1);
    fs.writeFileSync(file, Buffer.concat([original, Buffer.from('// changed\n')]));
    assert.equal(referenceIntegrity(directory, manifest).mismatches.length, 1);
    fs.unlinkSync(file);
    assert.deepEqual(referenceIntegrity(directory, manifest).mismatches, [{ path: 'reference.ts', error: 'missing' }]);
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});

test('manifest paths cannot traverse outside the reference root', () => {
  for (const name of ['../reference.ts', 'src/../../reference.ts', '..\\reference.ts']) {
    assert.throws(() => referenceIntegrity(tempRoot, [{ path: name }]), /escapes/);
  }
  assert.throws(() => referenceIntegrity(tempRoot, [{ path: path.resolve(root, 'reference.ts') }]), /escapes/);
});

test('new, interrupted and failed attempts never reuse or erase older success', () => {
  const directory = fs.mkdtempSync(path.join(tempRoot, 'upstream-history-test-'));
  const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
  try {
    const first = createEvidenceRun(directory);
    persistEvidence(first, { status: 'passed', caseCount: 2460 });
    const firstSummary = fs.readFileSync(path.join(first.directory, 'summary.json'), 'utf8');
    const raw = path.join(first.directory, 'raw.json');
    fs.writeFileSync(raw, '{"cases":2460}');
    const second = createEvidenceRun(directory);
    assert.notEqual(second.id, first.id);
    assert.equal(read(path.join(directory, 'latest.json')).status, 'incomplete');
    assert.equal(read(path.join(second.directory, 'summary.json')).status, 'incomplete');
    assert.equal(fs.readFileSync(path.join(first.directory, 'summary.json'), 'utf8'), firstSummary);
    assert.equal(fs.readFileSync(raw, 'utf8'), '{"cases":2460}');
    persistEvidence(second, { status: 'failed', error: 'simulated missing dependency' });
    const latest = read(path.join(directory, 'latest.json'));
    assert.equal(latest.status, 'failed');
    assert.equal(latest.runId, second.id);
    assert.equal(read(path.join(directory, latest.summary)).error, 'simulated missing dependency');
    assert.equal(fs.readFileSync(path.join(first.directory, 'summary.json'), 'utf8'), firstSummary);
    const third = createEvidenceRun(directory);
    assert.equal(read(path.join(directory, 'latest.json')).runId, third.id);
    assert.equal(read(path.join(directory, 'latest.json')).status, 'incomplete');
    assert.equal(read(path.join(second.directory, 'summary.json')).status, 'failed');
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});

test('older completion cannot replace newer failed or incomplete attempt pointers', () => {
  const directory = fs.mkdtempSync(path.join(tempRoot, 'upstream-order-test-'));
  const readLatest = () => JSON.parse(fs.readFileSync(path.join(directory, 'latest.json'), 'utf8'));
  try {
    for (const newerStatus of ['failed', 'incomplete', 'passed']) {
      const older = createEvidenceRun(directory);
      const newer = createEvidenceRun(directory);
      if (newerStatus !== 'incomplete') persistEvidence(newer, { status: newerStatus });
      const before = fs.readFileSync(path.join(directory, 'latest.json'), 'utf8');
      persistEvidence(older, { status: 'passed' });
      assert.equal(readLatest().runId, newer.id);
      assert.equal(readLatest().status, newerStatus);
      assert.equal(fs.readFileSync(path.join(directory, 'latest.json'), 'utf8'), before);
      assert.equal(JSON.parse(fs.readFileSync(path.join(older.directory, 'summary.json'), 'utf8')).status, 'passed');
    }
  } finally { fs.rmSync(directory, { recursive: true }); }
});

test('pointer lock contention fails explicitly and callback errors release ownership', () => {
  const directory = fs.mkdtempSync(path.join(tempRoot, 'upstream-lock-test-'));
  let ownedLock;
  try {
    const attempt = createEvidenceRun(directory);
    const canonical = fs.realpathSync(directory);
    const key = sha256(process.platform === 'win32' ? canonical.toLowerCase() : canonical);
    ownedLock = path.join(tempRoot, 'evidence-pointer-locks', `${key}.lock`);
    fs.writeFileSync(ownedLock, 'test-owned lock', { flag: 'wx' });
    const before = fs.readFileSync(path.join(directory, 'latest.json'), 'utf8');
    assert.throws(() => createEvidenceRun(directory), /Evidence pointer is locked/);
    assert.throws(() => persistEvidence(attempt, { status: 'passed' }), /Evidence pointer is locked/);
    assert.equal(fs.readFileSync(path.join(directory, 'latest.json'), 'utf8'), before);
    fs.unlinkSync(ownedLock);
    ownedLock = undefined;
    const circular = { status: 'failed' };
    circular.circular = circular;
    assert.throws(() => persistEvidence(attempt, circular), /circular/i);
    const recovered = createEvidenceRun(directory);
    persistEvidence(recovered, { status: 'passed' });
    assert.equal(JSON.parse(fs.readFileSync(path.join(directory, 'latest.json'), 'utf8')).runId, recovered.id);
  } finally {
    if (ownedLock) fs.unlinkSync(ownedLock);
    fs.rmSync(directory, { recursive: true });
  }
});

test('pending cases are not executed or passed and mismatched totals fail', () => {
  assert.deepEqual(unitCounts(5, 2, 1, 2), {
    discovered: 5, executed: 3, passed: 2, failed: 1, pending: 2, skipped: 0, unrun: 0
  });
  assert.throws(() => unitCounts(5, 2, 0, 2), /disagree/);
});
