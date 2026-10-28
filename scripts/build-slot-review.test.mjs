// Independent resource-admission checks by Artaxerxes. No compiler/bundler runs.
import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, mkdir, readFile, writeFile, readdir, access } from 'node:fs/promises';
import { watch } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { acquireSlot, runWithSlot } from './build-slot-core.mjs';
const repository = resolve(import.meta.dirname, '..');
async function root() {
  await mkdir(join(repository, 'tmp'), { recursive: true });
  // Retain tiny isolated fixtures, including failed receipts, for diagnosis.
  return mkdtemp(join(repository, 'tmp/build-slot-independent-'));
}
function deferred() {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
}
const laneDirectory = (root, lane = 'native') => join(root, 'tmp/build-slots', lane);
async function absent(path) {
  try { await access(path); return false; }
  catch (error) { if (error.code === 'ENOENT') return true; throw error; }
}
function readyFile(directory, filename) {
  const ready = deferred();
  const watcher = watch(directory, (_event, name) => {
    if (String(name) === filename) { watcher.close(); ready.resolve(); }
  });
  return { promise: ready.promise, close: () => watcher.close() };
}

test('same-lane admissions obey documented timestamp/id FIFO', { timeout: 10_000 }, async () => {
  const directory = await root();
  const first = await acquireSlot({ root: directory, owner: 'first', lane: 'native', waitTimeoutMs: 5000 });
  const queued2 = deferred(), queued3 = deferred();
  const p2 = acquireSlot({ root: directory, owner: 'second', lane: 'native', waitTimeoutMs: 5000, onQueued: queued2.resolve });
  const metadata2 = await queued2.promise;
  const p3 = acquireSlot({ root: directory, owner: 'third', lane: 'native', waitTimeoutMs: 5000, onQueued: queued3.resolve });
  const metadata3 = await queued3.promise;
  const expected = [metadata2, metadata3].sort((a, b) => a.id < b.id ? -1 : 1);
  const holder = JSON.parse(await readFile(join(laneDirectory(directory), 'active.lock/owner.json'), 'utf8'));
  assert.equal(holder.id, first.id);
  await first.release();
  const second = await Promise.race([p2, p3]);
  assert.equal(second.id, expected[0].id);
  await second.release();
  const third = await (second.id === metadata2.id ? p3 : p2);
  assert.equal(third.id, expected[1].id);
  await third.release();
  assert.deepEqual(await readdir(laneDirectory(directory)), []);
});

test('native and WASM lane leases can coexist without stealing either owner', { timeout: 5000 }, async () => {
  const directory = await root();
  const native = await acquireSlot({ root: directory, owner: 'native', lane: 'native' });
  const wasm = await acquireSlot({ root: directory, owner: 'wasm', lane: 'wasm' });
  try {
    for (const lease of [native, wasm]) {
      const holder = JSON.parse(await readFile(join(laneDirectory(directory, lease.lane), 'active.lock/owner.json'), 'utf8'));
      assert.equal(holder.id, lease.id);
    }
  } finally { await wasm.release(); await native.release(); }
});

test('queued cancellation removes only its request and retains the active lease', { timeout: 5000 }, async () => {
  const directory = await root();
  const holder = await acquireSlot({ root: directory, owner: 'holder', lane: 'native' });
  const controller = new AbortController();
  const queued = deferred();
  const pending = acquireSlot({ root: directory, owner: 'cancelled', lane: 'native', signal: controller.signal,
    waitTimeoutMs: 2000, onQueued: queued.resolve });
  await queued.promise;
  const reason = new Error('independent queued cancellation');
  controller.abort(reason);
  await assert.rejects(pending, error => error === reason);
  const entries = await readdir(laneDirectory(directory));
  assert.deepEqual(entries.sort(), [`${holder.id}.request.json`, 'active.lock'].sort());
  await holder.release();
});

test('queue deadline fails without launching the requested process', { timeout: 5000 }, async () => {
  const directory = await root();
  const holder = await acquireSlot({ root: directory, owner: 'holder', lane: 'native' });
  const marker = join(directory, 'must-not-exist.txt');
  try {
    const result = await runWithSlot({ root: directory, owner: 'deadline', lane: 'native', waitTimeoutMs: 100,
      timeoutMs: 1000, command: process.execPath,
      args: ['-e', `require('node:fs').writeFileSync(${JSON.stringify(marker)},'wrongly admitted')`] });
    assert.equal(result.status, 'failed');
    assert.match(result.error, /wait deadline exceeded/);
    assert.equal(await absent(marker), true);
    const receipt = JSON.parse(await readFile(join(result.receiptDirectory, 'receipt.json'), 'utf8'));
    assert.equal(receipt.status, 'failed');
    assert.equal(receipt.process, undefined);
  } finally { await holder.release(); }
});

test('failed child command is retained and releases its native lease', { timeout: 5000 }, async () => {
  const directory = await root();
  const result = await runWithSlot({ root: directory, owner: 'failing-child', lane: 'native', command: process.execPath,
    args: ['-e', 'process.stderr.write("independent failure sentinel");process.exit(7)'], timeoutMs: 1500 });
  assert.equal(result.status, 'failed');
  assert.equal(result.process.exitCode, 7);
  assert.match(await readFile(join(result.receiptDirectory, 'command.log'), 'utf8'), /failure sentinel/);
  assert.deepEqual(await readdir(laneDirectory(directory)), []);
  const next = await acquireSlot({ root: directory, owner: 'after-failure', lane: 'native' });
  await next.release();
});

test('stale active owner fails closed instead of stealing or deleting its lease', { timeout: 5000 }, async () => {
  const directory = await root();
  const exited = spawnSync(process.execPath, ['-e', 'process.exit(0)'], { timeout: 2000 });
  assert.equal(exited.status, 0);
  assert(exited.pid > 0);
  const active = join(laneDirectory(directory), 'active.lock');
  await mkdir(active, { recursive: true });
  const metadata = { id: 'stale-sentinel', owner: 'dead-owner', lane: 'native', pid: exited.pid };
  const bytes = JSON.stringify(metadata);
  await writeFile(join(active, 'owner.json'), bytes);
  await assert.rejects(acquireSlot({ root: directory, owner: 'must-not-steal', lane: 'native', waitTimeoutMs: 500 }), /Stale active build lease/);
  assert.equal(await readFile(join(active, 'owner.json'), 'utf8'), bytes);
  assert.deepEqual(await readdir(laneDirectory(directory)), ['active.lock']);
});

test('release refuses a foreign owner and does not remove its marker', { timeout: 5000 }, async () => {
  const directory = await root();
  const lease = await acquireSlot({ root: directory, owner: 'original', lane: 'native' });
  const file = join(laneDirectory(directory), 'active.lock/owner.json');
  const original = await readFile(file, 'utf8');
  await writeFile(file, JSON.stringify({ id: 'foreign-sentinel' }));
  await assert.rejects(lease.release(), /ownership changed/);
  assert.equal(JSON.parse(await readFile(file, 'utf8')).id, 'foreign-sentinel');
  await writeFile(file, original);
  await lease.release();
  await lease.release();
  assert.deepEqual(await readdir(laneDirectory(directory)), []);
});

test('active owner cancellation cannot be reported as a successful child completion', { timeout: 5000 }, async () => {
  const directory = await root();
  const controller = new AbortController();
  const filename = 'ready.txt';
  const ready = readyFile(directory, filename);
  const pending = runWithSlot({ root: directory, owner: 'active-cancel', lane: 'native', signal: controller.signal,
    timeoutMs: 1500, command: process.execPath, args: ['-e',
      `require('node:fs').writeFileSync(${JSON.stringify(join(directory, filename))},'ready');setTimeout(()=>process.exit(0),300)`] });
  try {
    await ready.promise;
    controller.abort(new Error('independent active cancellation'));
    const result = await pending;
    assert.equal(result.status, 'failed', 'abort must not turn into a later successful child result');
    assert.equal(result.process.timedOut, false, 'abort must not wait for the command deadline');
    assert.deepEqual(await readdir(laneDirectory(directory)), []);
  } finally { ready.close(); }
});

test('wall-clock rollback cannot extend the bounded queue deadline', { timeout: 2000 }, async () => {
  const directory = await root();
  const holder = await acquireSlot({ root: directory, owner: 'clock-holder', lane: 'native' });
  const originalNow = Date.now;
  const controller = new AbortController();
  let rolledBack = false;
  Date.now = () => originalNow() - (rolledBack ? 3_600_000 : 0);
  // This guard makes an incorrect one-hour wait fail quickly, without leaving a
  // watcher/timer behind. It is not the expected queue timeout under test.
  const guard = setTimeout(() => controller.abort(new Error('clock rollback defeated the finite wait')), 400);
  try {
    await assert.rejects(acquireSlot({ root: directory, owner: 'clock-waiter', lane: 'native',
      waitTimeoutMs: 100, signal: controller.signal, onQueued() { rolledBack = true; } }),
    /wait deadline exceeded/);
  } finally {
    clearTimeout(guard);
    Date.now = originalNow;
    controller.abort(new Error('test cleanup'));
    await holder.release();
  }
});
