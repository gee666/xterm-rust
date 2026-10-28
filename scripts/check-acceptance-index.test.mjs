import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { auditRegistry, validateRegistry } from './check-acceptance-index.mjs';

function registry(changes = {}) {
  return { criteria: [{ id: 12, group: 'Test', title: 'Exact source contract', body: 'See docs/acceptance/contract.md#edge', verification: 'Run exact tests', source: 'source_repo/src/common/Module.ts', taskIds: [1], ...changes }] };
}

test('extract exact source paths and documents without claiming behavior success', () => {
  assert.deepEqual(validateRegistry(registry()), [{ id: 12, group: 'Test', title: 'Exact source contract', taskIds: [1], documents: ['docs/acceptance/contract.md'], sourceFiles: ['source_repo/src/common/Module.ts'], aggregate: false }]);
});

test('reject empty registries, missing fields and invalid task identifiers', () => {
  for (const value of [null, {}, { criteria: [] }, registry({ source: '' }), registry({ taskIds: [-1] }), registry({ taskIds: ['1'] })]) {
    assert.throws(() => validateRegistry(value));
  }
});

test('reject duplicate and invalid criterion identifiers', () => {
  const value = registry();
  value.criteria.push({ ...value.criteria[0] });
  assert.throws(() => validateRegistry(value), /duplicate/);
  assert.throws(() => validateRegistry(registry({ id: Number.NaN })), /invalid/);
});

test('reject parent traversal references', () => {
  assert.throws(() => validateRegistry(registry({ body: 'docs/../private.md' })), /unsafe/);
  assert.throws(() => validateRegistry(registry({ source: 'source_repo/../private.ts' })), /unsafe/);
});

test('record unlinked tasks and distinguish aggregate metadata', async () => {
  const report = await auditRegistry('.', registry({ id: 2, taskIds: [] }), async () => ({ isFile: () => true }));
  assert.deepEqual(report.unlinked, [2]);
  assert.equal(report.criteria[0].aggregate, true);
  assert.match(report.scope, /not behavior coverage/);
  assert.equal(report.requiresRefreshAfterGraphChange, true);
});

test('missing and nonregular references are reported, never counted as passes', async () => {
  let calls = 0;
  const report = await auditRegistry('.', registry(), async () => {
    if (++calls === 1) { const error = new Error('missing'); error.code = 'ENOENT'; throw error; }
    return { isFile: () => false };
  });
  assert.equal(report.missingReferences.length, 2);
  assert.equal(report.missingReferences[0].reason, 'missing');
  assert.equal(report.missingReferences[1].reason, 'not a regular file');
});

test('unexpected I/O failures propagate', async () => {
  await assert.rejects(auditRegistry('.', registry(), async () => { const error = new Error('denied'); error.code = 'EACCES'; throw error; }), /denied/);
});

test('document-only criteria do not receive guessed source behavior paths', async () => {
  const report = await auditRegistry('.', registry({ source: 'PORT_REQUIREMENTS.md', body: 'Cross-project requirement' }), async () => ({ isFile: () => true }));
  assert.deepEqual(report.criteria[0].sourceFiles, []);
  assert.deepEqual(report.focusedWithoutChecklistDocument, [12]);
});
