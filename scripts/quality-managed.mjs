import { spawn } from 'node:child_process';
import { openSync, closeSync, writeSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { stopProcessTree } from './quality-tree.mjs';
import { failureText } from './quality-failure.mjs';

/** Own a server until stop(), deadline, resource failure or unexpected exit.
 * ready returns false only for expected not-yet-ready states. Rejections fail
 * startup. Probes are bounded by startupTimeoutMs and receive an AbortSignal. */
export function startManagedProcess(command, args, {
  cwd, env, log, timeoutMs, startupTimeoutMs = 15_000, resourceCheck,
  ready, readyPattern, probeIntervalMs = 100
}) {
  for (const value of [timeoutMs, startupTimeoutMs, probeIntervalMs]) {
    if (!Number.isSafeInteger(value) || value < 1) throw new Error('Managed process deadlines and probe interval must be positive integers');
  }
  if (typeof ready !== 'function' && !(readyPattern instanceof RegExp)) throw new Error('A readiness callback or pattern is required');
  const fd = openSync(log, 'wx');
  const started = performance.now();
  const abort = new AbortController();
  const child = spawn(command, args, { cwd, env, windowsHide: true, detached: process.platform !== 'win32', stdio: ['ignore', 'pipe', 'pipe'] });
  let resolveReady;
  let rejectReady;
  const readiness = new Promise((resolve, reject) => { resolveReady = resolve; rejectReady = reject; });
  let resolveFinished;
  const finished = new Promise(resolve => { resolveFinished = resolve; });
  let becameReady = false;
  let stopping = false;
  let closed = false;
  let error = null;
  let terminationError = null;
  let timedOut = false;
  let resourceError = null;
  let loggingError = null;
  let readinessProbes = 0;
  let text = '';
  function stop(reason = null) {
    if (!closed && !stopping) {
      stopping = true;
      if (reason !== null) error = failureText(reason);
      abort.abort(new Error(reason ?? 'Owned server stopped'));
      terminationError = stopProcessTree(child);
    }
    return finished;
  }
  const lifetime = setTimeout(() => {
    timedOut = true;
    void stop('Managed process lifetime deadline exceeded');
  }, timeoutMs);
  const startup = setTimeout(() => {
    if (!becameReady) { timedOut = true; void stop('Managed process readiness deadline exceeded'); }
  }, startupTimeoutMs);
  function markReady() {
    if (closed || stopping || becameReady) return;
    becameReady = true;
    clearTimeout(startup);
    resolveReady();
  }
  function output(chunk) {
    if (loggingError !== null) return;
    try { writeSync(fd, chunk); }
    catch (value) {
      loggingError = failureText(value);
      void stop(`Managed process logging failed: ${loggingError}`);
      return;
    }
    if (readyPattern && !becameReady) {
      text = (text + chunk.toString('utf8')).slice(-65_536);
      readyPattern.lastIndex = 0;
      if (readyPattern.test(text)) markReady();
    }
  }
  child.stdout.on('data', output);
  child.stderr.on('data', output);
  child.on('error', value => { error = value.message; });
  let probing = false;
  const probe = setInterval(async () => {
    if (closed || stopping || probing) return;
    probing = true;
    let checkingResource = true;
    try {
      if (resourceCheck) await resourceCheck();
      checkingResource = false;
      if (!becameReady && ready) {
        readinessProbes++;
        if (await ready({ signal: abort.signal })) markReady();
      }
    } catch (value) {
      if (!closed && !stopping) {
        if (checkingResource) resourceError = failureText(value);
        void stop(failureText(value));
      }
    } finally { probing = false; }
  }, probeIntervalMs);
  child.once('close', (exitCode, signal) => {
    closed = true;
    clearTimeout(lifetime);
    clearTimeout(startup);
    clearInterval(probe);
    abort.abort(new Error('Owned server exited'));
    try { closeSync(fd); }
    catch (value) { loggingError = failureText(value); error ??= `Managed process log close failed: ${loggingError}`; }
    const result = {
      command: [command, ...args], pid: child.pid ?? null, exitCode, signal,
      ready: becameReady, readinessProbes, timedOut, error, resourceError, loggingError, terminationError,
      elapsedMs: performance.now() - started,
      status: stopping && error === null && !timedOut && resourceError === null && loggingError === null && terminationError === null ? 'stopped' : 'failed'
    };
    if (!becameReady) rejectReady(new Error(error ?? `Managed process exited before readiness with status ${exitCode}`));
    resolveFinished(result);
  });
  return { pid: child.pid ?? null, ready: readiness, finished, stop };
}
