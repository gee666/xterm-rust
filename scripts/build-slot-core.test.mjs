import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, writeFile, readdir, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { acquireSlot, runWithSlot } from './build-slot-core.mjs';
const project = fileURLToPath(new URL('../', import.meta.url));
async function fixture(fn) {
  await mkdir(join(project, 'tmp'), { recursive: true });
  const root = await mkdtemp(join(project, 'tmp', 'build-slot-test-'));
  try { await fn(root); }
  finally { await rm(root, { recursive: true, force: true }); }
}
test('native FIFO preserves active ownership until release', () => fixture(async root => {
  const first = await acquireSlot({ root, owner: 'first', lane: 'native' });
  let queued;
  const observed = new Promise(resolve => { queued = resolve; });
  const waiting = acquireSlot({ root, owner: 'second', lane: 'native', onQueued: queued });
  await observed;
  const path = join(root, 'tmp/build-slots/native/active.lock/owner.json');
  assert.equal(JSON.parse(await readFile(path, 'utf8')).owner, 'first');
  await first.release();
  const second = await waiting;
  assert.equal(JSON.parse(await readFile(path, 'utf8')).owner, 'second');
  await second.release();
  await second.release();
  assert.deepEqual(await readdir(join(root, 'tmp/build-slots/native')), []);
}));
test('native and WASM are independent resource lanes', () => fixture(async root => {
  const native = await acquireSlot({ root, owner: 'native-owner', lane: 'native' });
  const wasm = await acquireSlot({ root, owner: 'wasm-owner', lane: 'wasm', waitTimeoutMs: 1000 });
  assert.notEqual(native.id, wasm.id);
  await native.release();
  await wasm.release();
}));
test('wait deadline does not steal the active lease', () => fixture(async root => {
  const first = await acquireSlot({ root, owner: 'first', lane: 'native' });
  await assert.rejects(acquireSlot({ root, owner: 'timed-out', lane: 'native', waitTimeoutMs: 50 }), /deadline/);
  assert.equal(JSON.parse(await readFile(join(root, 'tmp/build-slots/native/active.lock/owner.json'), 'utf8')).id, first.id);
  await first.release();
}));
test('queued cancellation and callback errors remove only their own request', () => fixture(async root => {
  const first = await acquireSlot({ root, owner: 'first', lane: 'native' });
  const controller = new AbortController();
  controller.abort(new Error('caller cancelled'));
  await assert.rejects(acquireSlot({ root, owner: 'cancelled', lane: 'native', signal: controller.signal }), /caller cancelled/);
  await assert.rejects(acquireSlot({ root, owner: 'bad callback', lane: 'native', onQueued: () => { throw new Error('queue callback failed'); } }), /queue callback failed/);
  assert.equal((await readdir(join(root, 'tmp/build-slots/native'))).filter(name => name.endsWith('.request.json')).length, 1);
  await first.release();
}));
test('an asynchronous queue progress callback cannot bypass the wait deadline', () => fixture(async root => {
  await assert.rejects(acquireSlot({ root, owner: 'hung progress', lane: 'native', waitTimeoutMs: 50,
    onQueued: () => new Promise(() => {}) }), /callback deadline/);
  assert.deepEqual(await readdir(join(root, 'tmp/build-slots/native')), []);
}));
test('stale active owners fail closed rather than losing child ownership', () => fixture(async root => {
  const directory = join(root, 'tmp/build-slots/native');
  await mkdir(join(directory, 'active.lock'), { recursive: true });
  const holder = { id: 'old', owner: 'crashed owner', pid: 2147483647 };
  await writeFile(join(directory, 'active.lock/owner.json'), JSON.stringify(holder));
  await assert.rejects(acquireSlot({ root, owner: 'new', lane: 'native' }), /Stale active build lease/);
  assert.deepEqual(JSON.parse(await readFile(join(directory, 'active.lock/owner.json'), 'utf8')), holder);
}));
test('completed failures retain output, free the lease and allow a later job', () => fixture(async root => {
  const failed = await runWithSlot({ root, owner: 'failed', lane: 'native', command: process.execPath,
    args: ['-e', 'console.error("retained command failure");process.exit(3)'] });
  assert.equal(failed.status, 'failed');
  assert.equal(failed.process.exitCode, 3);
  assert.match(await readFile(join(failed.receiptDirectory, 'command.log'), 'utf8'), /retained command failure/);
  const passed = await runWithSlot({ root, owner: 'passed', lane: 'native', command: process.execPath, args: ['-e', 'console.log("done")'] });
  assert.equal(passed.status, 'passed');
  assert.notEqual(failed.receiptDirectory, passed.receiptDirectory);
  assert.deepEqual(await readdir(join(root, 'tmp/build-slots/native')), []);
}));
test('command timeout cleans its lease without becoming a successful skip', () => fixture(async root => {
  const result = await runWithSlot({ root, owner: 'hung', lane: 'native', timeoutMs: 100,
    command: process.execPath, args: ['-e', 'setInterval(()=>{},60000)'] });
  assert.equal(result.status, 'failed');
  assert.equal(result.process.timedOut, true);
  assert.deepEqual(await readdir(join(root, 'tmp/build-slots/native')), []);
}));
