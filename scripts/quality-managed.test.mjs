import { test, mock } from 'node:test';
import fs from 'node:fs';
import { syncBuiltinESMExports } from 'node:module';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { startManagedProcess, stopProcessTree } from './quality-process.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
async function fixture(fn) {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp', 'managed-process-'));
  try { await fn(directory); }
  finally { await rm(directory, { recursive: true, force: true }); }
}
function start(directory, code, options = {}) {
  return startManagedProcess(process.execPath, ['-e', code], {
    cwd: root, env: process.env, log: join(directory, 'server.log'), timeoutMs: 10_000,
    startupTimeoutMs: 2000, readyPattern: /READY/, ...options
  });
}
test('managed readiness and explicit stop retain output and are idempotent', () => fixture(async directory => {
  const handle = start(directory, 'console.log("READY");setInterval(()=>{},60000)');
  try { await handle.ready; }
  finally { await handle.stop(); }
  const result = await handle.finished;
  assert.equal(result.status, 'stopped');
  assert.equal(result.ready, true);
  assert.equal(await handle.stop(), result);
  assert.match(await readFile(join(directory, 'server.log'), 'utf8'), /READY/);
}));
test('early exit is failed even with a zero exit code', () => fixture(async directory => {
  const handle = start(directory, 'console.log("not ready")');
  await assert.rejects(handle.ready, /before readiness/);
  assert.equal((await handle.finished).status, 'failed');
}));
test('readiness deadline terminates the owned server', () => fixture(async directory => {
  const handle = start(directory, 'setInterval(()=>{},60000)', { startupTimeoutMs: 150 });
  await assert.rejects(handle.ready, /readiness deadline/);
  const result = await handle.finished;
  assert.equal(result.timedOut, true);
  assert.equal(result.status, 'failed');
  assert.equal(result.terminationError, null);
}));
test('readiness callback rejection is retained instead of retried', () => fixture(async directory => {
  const handle = start(directory, 'setInterval(()=>{},60000)', {
    readyPattern: undefined, ready: async () => { throw new Error('invalid readiness contract'); }
  });
  await assert.rejects(handle.ready, /invalid readiness contract/);
  const result = await handle.finished;
  assert.equal(result.readinessProbes, 1);
  assert.equal(result.resourceError, null);
}));
test('disk failure kills the server and retains the exact resource error', () => fixture(async directory => {
  const handle = start(directory, 'setInterval(()=>{},60000)', {
    resourceCheck: async () => { throw new Error('insufficient reserve'); }
  });
  await assert.rejects(handle.ready, /insufficient reserve/);
  const result = await handle.finished;
  assert.equal(result.resourceError, 'insufficient reserve');
  assert.equal(result.status, 'failed');
}));
test('log write failure stops its server rather than throwing out of the data callback', () => fixture(async directory => {
  const failure = new Error('ENOSPC injected log failure');
  const replacement = mock.method(fs, 'writeSync', () => { throw failure; });
  syncBuiltinESMExports();
  const handle = start(directory, 'console.log("READY");setInterval(()=>{},60000)');
  try {
    await assert.rejects(handle.ready, /logging failed.*ENOSPC/);
    const result = await handle.finished;
    assert.equal(result.status, 'failed');
    assert.equal(result.loggingError, failure.message);
    assert.equal(result.terminationError, null);
  } finally {
    replacement.mock.restore();
    syncBuiltinESMExports();
    await handle.stop();
  }
}));
test('expected false readiness responses can become ready within the startup deadline', () => fixture(async directory => {
  let probes = 0;
  const handle = start(directory, 'setInterval(()=>{},60000)', {
    readyPattern: undefined, ready: async () => ++probes >= 2
  });
  try { await handle.ready; }
  finally { await handle.stop(); }
  assert.equal((await handle.finished).readinessProbes, 2);
}));
test('lifetime remains bounded after readiness', () => fixture(async directory => {
  const handle = start(directory, 'console.log("READY");setInterval(()=>{},60000)', { timeoutMs: 500 });
  await handle.ready;
  const result = await handle.finished;
  assert.equal(result.status, 'failed');
  assert.equal(result.timedOut, true);
  assert.match(result.error, /lifetime deadline/);
}));
test('a ready server that exits unexpectedly is not a stopped success', () => fixture(async directory => {
  const handle = start(directory, 'console.log("READY");setImmediate(()=>process.exit(0))');
  await handle.ready;
  const result = await handle.finished;
  assert.equal(result.ready, true);
  assert.equal(result.exitCode, 0);
  assert.equal(result.status, 'failed');
}));
test('managed spawn errors reject readiness and retain a failed receipt', () => fixture(async directory => {
  const handle = startManagedProcess(join(directory, 'missing-executable'), [], {
    cwd: root, env: process.env, log: join(directory, 'server.log'), timeoutMs: 1000, readyPattern: /READY/
  });
  await assert.rejects(handle.ready, /ENOENT/);
  assert.equal((await handle.finished).status, 'failed');
}));
test('tree stop refuses arbitrary PIDs', () => {
  assert.throws(() => stopProcessTree(process.pid), /ChildProcess handle/);
});
