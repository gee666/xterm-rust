// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Independent JS event-loop oracle for two WriteBuffers sharing one host.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { createHash, randomUUID } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const runId = randomUUID();
const resultDir = `results/native-queue-order/runs/${runId}`;
const temporary = `tmp/native-queue-order/${runId}`;
await mkdir(path.join(root, resultDir), { recursive: true });
const report = {
  scope: 'Original WriteBuffer host microtask ordering only; Rust/browser parity NOT PROVEN',
  runId, status: 'running', command: ['node', 'scripts/native-queue-order-oracle.mjs'],
  environment: { node: process.version, platform: process.platform, arch: process.arch },
  sourceHashes: {}, scenarios: [], rustExecution: 'unrun'
};
async function retain() {
  await writeFile(path.join(root, resultDir, 'oracle.json'), `${JSON.stringify(report, null, 2)}\n`);
  await writeFile(path.join(root, 'results/native-queue-order/latest.json'), `${JSON.stringify({ runId, status: report.status, summary: `runs/${runId}/oracle.json` }, null, 2)}\n`);
}
await retain();
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
async function source(relative) {
  const bytes = await readFile(path.join(root, relative));
  report.sourceHashes[relative] = hash(bytes);
  return bytes.toString('utf8');
}
async function emit(name, text) {
  const target = path.join(root, temporary, `${name}.mjs`);
  await writeFile(target, stripTypeScriptTypes(text, { mode: 'transform' }));
  return pathToFileURL(target).href;
}
async function referenceWriteBuffer() {
  await mkdir(path.join(root, temporary), { recursive: true });
  const base = 'source_repo/src/common/';
  // Same import/type adaptation as write-buffer-oracle.mjs, deliberately local:
  // importing that executable would run its suites and overwrite its fixtures.
  const lifecycle = await emit('lifecycle', await source(`${base}Lifecycle.ts`));
  const event = await emit('event', (await source(`${base}Event.ts`))
    .replace('IDisposable, ', '').replace("'./Lifecycle'", JSON.stringify(lifecycle)));
  const timers = await emit('async', (await source(`${base}Async.ts`))
    .replace(', IDisposable', '').replace("'./Lifecycle'", JSON.stringify(lifecycle)));
  const buffer = await emit('write-buffer', (await source(`${base}input/WriteBuffer.ts`))
    .replace("'../Lifecycle'", JSON.stringify(lifecycle))
    .replace("'../Event'", JSON.stringify(event)).replace("'../Async'", JSON.stringify(timers)));
  return (await import(buffer)).WriteBuffer;
}
async function scenario(WriteBuffer, rejection) {
  const log = [];
  const unexpectedErrors = [];
  const tasks = new Map();
  let next = 0;
  const saved = { timeout: globalThis.setTimeout, clear: globalThis.clearTimeout,
    performance: Object.getOwnPropertyDescriptor(globalThis, 'performance') };
  const expectedError = new Error('A pending handler rejected');
  const onError = error => {
    if (error === expectedError) log.push('error:A');
    else unexpectedErrors.push(String(error.stack ?? error));
  };
  globalThis.setTimeout = callback => { const id = ++next; tasks.set(id, callback); return id; };
  globalThis.clearTimeout = id => tasks.delete(id);
  Object.defineProperty(globalThis, 'performance', { configurable: true, value: { now: () => 100 } });
  process.on('uncaughtException', onError);
  let a;
  let b;
  const fire = id => {
    const callback = tasks.get(id);
    assert(callback, `Missing scheduled source timer ${id}`);
    tasks.delete(id);
    callback();
  };
  // A real host macrotask fence drains the entire Promise/queueMicrotask queue.
  // An arbitrary number of `await Promise.resolve()` calls would not prove that.
  const checkpoint = () => new Promise(resolve => setImmediate(resolve));
  try {
    let first = true;
    a = new WriteBuffer((_data, result) => {
      log.push(`A:${result}`);
      if (first) {
        first = false;
        return rejection ? Promise.reject(expectedError) : Promise.resolve(true);
      }
    });
    b = new WriteBuffer((_data, result) => { log.push(`B:${result}`); });
    a.onWriteParsed(() => log.push('parsed:A'));
    b.onWriteParsed(() => log.push('parsed:B'));
    a.write('a', () => log.push('callback:A'));
    b.write('b', () => log.push('callback:B'));
    assert.equal(tasks.size, 2);
    const firstId = a._innerWriteTimer._token;
    const secondId = b._innerWriteTimer._token;
    fire(firstId);
    await checkpoint();
    const afterFirstTimerAndMicrotasks = [...log];
    fire(secondId);
    await checkpoint();
    const expected = rejection
      ? ['A:true', 'error:A', 'A:false', 'callback:A', 'parsed:A', 'B:true', 'callback:B', 'parsed:B']
      : ['A:true', 'A:true', 'callback:A', 'parsed:A', 'B:true', 'callback:B', 'parsed:B'];
    const observation = { name: rejection ? 'rejected-A-before-B' : 'resolved-A-before-B',
      log, afterFirstTimerAndMicrotasks, expected, pendingA: a._pendingData, pendingB: b._pendingData,
      scheduledTasks: tasks.size, unexpectedErrors };
    report.scenarios.push(observation);
    assert.deepEqual(unexpectedErrors, []);
    assert.deepEqual(log, expected);
    assert.deepEqual(afterFirstTimerAndMicrotasks, expected.slice(0, expected.indexOf('B:true')));
    assert.equal(a._pendingData, 0);
    assert.equal(b._pendingData, 0);
    assert.equal(tasks.size, 0);
  } finally {
    a?.dispose();
    b?.dispose();
    process.removeListener('uncaughtException', onError);
    globalThis.setTimeout = saved.timeout;
    globalThis.clearTimeout = saved.clear;
    Object.defineProperty(globalThis, 'performance', saved.performance);
  }
}
try {
  const WriteBuffer = await referenceWriteBuffer();
  await scenario(WriteBuffer, false);
  await scenario(WriteBuffer, true);
  report.status = 'passed';
  report.counts = { discovered: 2, executed: report.scenarios.length, passed: 2, failed: 0, skipped: 0, unrun: 0 };
} catch (error) {
  report.status = 'failed';
  report.failure = String(error.stack ?? error);
  process.exitCode = 1;
} finally {
  await retain();
  console.log(JSON.stringify(report, null, 2));
}
