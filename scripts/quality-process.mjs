import { spawn } from 'node:child_process';
import { openSync, closeSync } from 'node:fs';
import { stopProcessTree } from './quality-tree.mjs';
export { startManagedProcess } from './quality-managed.mjs';
export { stopProcessTree } from './quality-tree.mjs';
import { performance } from 'node:perf_hooks';

/** Run without a shell, retaining failures and bounding lifetime and disk growth. */
export async function runCommand(command, args, { cwd, env, log, timeoutMs, resourceCheck, monitorMs = 1000 }) {
  const fd = openSync(log, 'w');
  const started = performance.now();
  const child = spawn(command, args, {
    cwd, env, windowsHide: true, detached: process.platform !== 'win32',
    stdio: ['ignore', fd, fd]
  });
  let error = null;
  let timedOut = false;
  let terminationError = null;
  let resourceError = null;
  let closed = false;
  let terminating = false;
  function terminate() {
    if (closed || terminating || !child.pid) return;
    terminating = true;
    terminationError = stopProcessTree(child);
  }
  child.on('error', value => { error = value.message; });
  const timer = setTimeout(() => { timedOut = true; terminate(); }, timeoutMs);
  let probing = false;
  const monitor = resourceCheck ? setInterval(async () => {
    if (probing || closed || terminating) return;
    probing = true;
    try { await resourceCheck(); }
    catch (value) {
      if (!closed) { resourceError = value.message; terminate(); }
    } finally { probing = false; }
  }, monitorMs) : null;
  const [exitCode, signal] = await new Promise(resolve => {
    child.once('close', (code, value) => { closed = true; resolve([code, value]); });
  });
  clearTimeout(timer);
  if (monitor) clearInterval(monitor);
  closeSync(fd);
  return {
    command: [command, ...args], exitCode, signal, error, timedOut, terminationError, resourceError,
    elapsedMs: performance.now() - started,
    status: exitCode === 0 && !error && !timedOut && !resourceError ? 'passed' : 'failed'
  };
}
