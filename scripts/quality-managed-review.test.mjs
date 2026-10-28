// Independent failure-shape checks. Tiny owned Node processes only.
import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, mkdir } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { startManagedProcess } from './quality-managed.mjs';
const repository = resolve(import.meta.dirname, '..');
async function rejection_case(value, expected) {
  await mkdir(join(repository, 'tmp'), { recursive: true });
  const root = await mkdtemp(join(repository, 'tmp/managed-review-'));
  const handle = startManagedProcess(process.execPath, ['-e', 'setInterval(()=>{},60000)'], {
    cwd: repository, env: process.env, log: join(root, 'server.log'),
    timeoutMs: 1500, startupTimeoutMs: 1000, probeIntervalMs: 10,
    ready: async () => { throw value; },
  });
  try {
    const rejected = await handle.ready.then(() => null, error => error);
    const result = await handle.finished;
    assert(rejected, 'throwing readiness callback must reject readiness');
    assert.equal(result.status, 'failed', 'callback failure cannot become intentional stopped cleanup');
    assert.notEqual(result.error, null, 'failure must not disappear into null/undefined metadata');
    if (expected) assert.match(String(result.error), expected);
    assert.equal(result.terminationError, null);
  } finally { await handle.stop(); }
}
test('string readiness rejection remains a failed receipt with its cause', { timeout: 5000 }, async () => {
  await rejection_case('readiness-string-sentinel', /readiness-string-sentinel/);
});
test('an Error with an empty message is still failure, not successful cleanup', { timeout: 5000 }, async () => {
  await rejection_case(new Error(''));
});
