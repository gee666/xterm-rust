import { mkdir, readFile, writeFile, readdir, rm, statfs, rename, lstat } from 'node:fs/promises';
import { watch } from 'node:fs';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { performance } from 'node:perf_hooks';
import { runCommand } from './quality-process.mjs';
import { assertBuildEnvironment, testCounts } from './quality-policy.mjs';

const MIN_FREE_BYTES = 1024n * 1024n * 1024n;
const MAX_DEADLINE_MS = 1_800_000;
async function boundedQueuedCallback(action, metadata, milliseconds, signal) {
  if (signal?.aborted) throw signal.reason ?? new Error('Build slot cancelled');
  let timer;
  let abort;
  const boundary = new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error('Build slot queue callback deadline exceeded')), Math.max(1, milliseconds));
    abort = () => reject(signal.reason ?? new Error('Build slot cancelled'));
    signal?.addEventListener('abort', abort, { once: true });
  });
  try { await Promise.race([Promise.resolve().then(() => action(metadata)), boundary]); }
  finally { clearTimeout(timer); signal?.removeEventListener('abort', abort); }
}
function alive(pid) {
  try { process.kill(pid, 0); return true; }
  catch (error) { if (error.code === 'ESRCH') return false; if (error.code === 'EPERM') return true; throw error; }
}
function nextChange(watcher, milliseconds, signal) {
  let finish;
  const promise = new Promise(resolve => { finish = resolve; });
  const complete = value => { clearTimeout(timer); watcher.off('change', change); watcher.off('error', error); signal?.removeEventListener('abort', abort); finish(value); };
  const change = () => complete({ kind: 'change' });
  const error = value => complete({ kind: 'error', error: value });
  const abort = () => complete({ kind: 'abort', error: signal.reason ?? new Error('Build slot cancelled') });
  const timer = setTimeout(() => complete({ kind: 'deadline' }), milliseconds);
  watcher.once('change', change);
  watcher.once('error', error);
  signal?.addEventListener('abort', abort, { once: true });
  if (signal?.aborted) abort();
  return { promise, cancel: () => complete({ kind: 'cancelled' }) };
}

/** FIFO by host-monotonic enqueue ticks/id within one native or WASM lane. No stale active lease
 * is stolen: its child may survive a crashed owner and requires coordinated recovery. */
export async function acquireSlot({ root, owner, lane, signal, waitTimeoutMs = 120_000, onQueued }) {
  if (!['native', 'wasm'].includes(lane) || !owner?.trim()) throw new Error('Owner and native|wasm lane are required');
  if (!Number.isSafeInteger(waitTimeoutMs) || waitTimeoutMs < 1 || waitTimeoutMs > MAX_DEADLINE_MS) throw new Error('Finite positive queue deadline required');
  const directory = join(root, 'tmp', 'build-slots', lane);
  await mkdir(directory, { recursive: true });
  const id = `${process.hrtime.bigint().toString().padStart(24, '0')}-${process.pid}-${randomUUID()}`;
  const request = join(directory, `${id}.request.json`);
  const active = join(directory, 'active.lock');
  const metadata = { id, owner, lane, pid: process.pid, queuedAt: new Date().toISOString() };
  const watcher = watch(directory);
  const deadline = performance.now() + waitTimeoutMs;
  let granted = false;
  try {
    await writeFile(request, JSON.stringify(metadata), { flag: 'wx' });
    if (onQueued) await boundedQueuedCallback(onQueued, metadata, deadline - performance.now(), signal);
    while (performance.now() < deadline) {
      if (signal?.aborted) throw signal.reason ?? new Error('Build slot cancelled');
      const pending = nextChange(watcher, Math.max(1, deadline - performance.now()), signal);
      try {
        const requests = (await readdir(directory)).filter(name => name.endsWith('.request.json')).sort();
        for (const name of requests) {
          const match = name.match(/^\d+-(\d+)-[\w-]+\.request\.json$/);
          if (!match) throw new Error('Malformed build queue entry; coordinator inspection required');
          if (!alive(Number(match[1]))) await rm(join(directory, name));
        }
        const first = (await readdir(directory)).filter(name => name.endsWith('.request.json')).sort()[0];
        if (first === `${id}.request.json`) {
          let created = false;
          try { await mkdir(active); created = true; }
          catch (error) { if (error.code !== 'EEXIST') throw error; }
          if (created) {
            try {
              await writeFile(join(active, 'owner.pending'), JSON.stringify(metadata), { flag: 'wx' });
              await rename(join(active, 'owner.pending'), join(active, 'owner.json'));
            } catch (error) { await rm(active, { recursive: true }); throw error; }
            granted = true;
            pending.cancel();
            let released = false;
            return { ...metadata, async release() {
              if (released) return;
              const current = JSON.parse(await readFile(join(active, 'owner.json'), 'utf8'));
              if (current.id !== id) throw new Error('Build lease ownership changed; refusing to release another job');
              await rm(active, { recursive: true });
              await rm(request);
              released = true;
            } };
          }
          let holder;
          try { holder = JSON.parse(await readFile(join(active, 'owner.json'), 'utf8')); }
          catch (error) {
            // Directory creation precedes atomic metadata publication. A missing
            // owner is pending until the same finite queue deadline, not stolen.
            if (error.code !== 'ENOENT') throw error;
          }
          if (holder && (!Number.isSafeInteger(holder.pid) || holder.pid < 1 || typeof holder.id !== 'string')) throw new Error('Invalid active build lease metadata; coordinator inspection required');
          if (holder && !alive(holder.pid)) throw new Error(`Stale active build lease for ${holder.owner}; coordinate recovery, do not steal it`);
        }
      } catch (error) { pending.cancel(); throw error; }
      const event = await pending.promise;
      if (event.kind === 'abort' || event.kind === 'error') throw event.error;
    }
    throw new Error('Build slot wait deadline exceeded without launching a command');
  } finally {
    watcher.close();
    if (!granted) await rm(request, { force: true });
  }
}

export async function runWithSlot({ root, owner, lane, command, args = [], signal, waitTimeoutMs = 120_000, timeoutMs = 180_000, onQueued }) {
  assertBuildEnvironment(process.env);
  if (!command || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_DEADLINE_MS) throw new Error('Command and finite positive execution deadline required');
  const directory = join(root, 'results', 'build-slots', `${Date.now()}-${process.pid}-${randomUUID()}`);
  await mkdir(directory, { recursive: true });
  const report = { owner, lane, command: [command, ...args], status: 'failed', startedAt: new Date().toISOString(), disk: [] };
  let lease;
  async function resources() {
    try {
      await lstat(join(root, 'tmp', 'build-slots', 'paused.json'));
      throw new Error('Build queue paused by coordinator; no new command may start');
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    for (const location of [root, join(root, 'target')]) {
      let value;
      try { value = await statfs(location, { bigint: true }); }
      catch (error) { if (location !== root && error.code === 'ENOENT') continue; throw error; }
      const bytes = value.bavail * value.bsize;
      report.disk.push({ location, freeBytes: bytes.toString(), observedAt: new Date().toISOString() });
      if (bytes < MIN_FREE_BYTES) throw new Error('Build slot requires at least 1 GiB free; command stopped or not started');
    }
  }
  try {
    await resources();
    lease = await acquireSlot({ root, owner, lane, signal, waitTimeoutMs, onQueued });
    report.lease = { id: lease.id, acquiredAt: new Date().toISOString() };
    await resources();
    report.process = await runCommand(command, args, { cwd: root, env: process.env, log: join(directory, 'command.log'), timeoutMs, signal, resourceCheck: resources });
    report.testSummaries = testCounts(await readFile(join(directory, 'command.log'), 'utf8'));
    await resources();
    report.status = report.process.status;
  } catch (error) { report.error = error.message ?? String(error); }
  finally {
    if (lease) {
      try { await lease.release(); }
      catch (error) { report.error = error.message; report.status = 'failed'; }
    }
  }
  report.finishedAt = new Date().toISOString();
  await writeFile(join(directory, 'receipt.json'), JSON.stringify(report, null, 2) + '\n');
  return { ...report, receiptDirectory: directory };
}
