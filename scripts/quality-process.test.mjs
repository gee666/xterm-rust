import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runCommand } from './quality-process.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
async function fixture(fn) {
  const parent = join(root, 'tmp');
  await mkdir(parent, { recursive: true });
  const directory = await mkdtemp(join(parent, 'quality-runner-'));
  try { await fn(directory); }
  finally { await rm(directory, { recursive: true, force: true }); }
}
async function run(directory, code, timeoutMs = 10_000) {
  return runCommand(process.execPath, ['-e', code], {
    cwd: root, env: process.env, log: join(directory, 'command.log'), timeoutMs
  });
}
test('successful command retains stdout and stderr without a shell', () => fixture(async directory => {
  const result = await run(directory, 'console.log("stdout"); console.error("stderr")');
  assert.equal(result.status, 'passed');
  assert.equal(result.exitCode, 0);
  const log = await readFile(join(directory, 'command.log'), 'utf8');
  assert.match(log, /stdout/);
  assert.match(log, /stderr/);
}));
test('failed command preserves exit status and output', () => fixture(async directory => {
  const result = await run(directory, 'console.error("retained failure"); process.exit(7)');
  assert.equal(result.status, 'failed');
  assert.equal(result.exitCode, 7);
  assert.match(await readFile(join(directory, 'command.log'), 'utf8'), /retained failure/);
}));
test('missing executable is a recorded failure, not an uncaught spawn error', () => fixture(async directory => {
  const result = await runCommand(join(directory, 'missing-executable'), [], {
    cwd: root, env: process.env, log: join(directory, 'command.log'), timeoutMs: 10_000
  });
  assert.equal(result.status, 'failed');
  assert.match(result.error, /ENOENT/);
}));
test('timeout terminates a blocked command and records the failure', () => fixture(async directory => {
  const result = await run(directory, 'process.stdin.resume(); setInterval(() => {}, 60000)', 200);
  assert.equal(result.status, 'failed');
  assert.equal(result.timedOut, true);
  assert.equal(result.terminationError, null);
  assert.ok(result.elapsedMs < 10_000);
}));
test('disk monitor terminates a command and records its resource failure', () => fixture(async directory => {
  const result = await runCommand(process.execPath, ['-e', 'setInterval(() => {}, 60000)'], {
    cwd: root, env: process.env, log: join(directory, 'command.log'), timeoutMs: 10_000,
    monitorMs: 100, resourceCheck: async () => { throw new Error('low disk test'); }
  });
  assert.equal(result.status, 'failed');
  assert.equal(result.resourceError, 'low disk test');
  assert.equal(result.timedOut, false);
  assert.equal(result.terminationError, null);
}));
test('an already-aborted signal does not start an executable', () => fixture(async directory => {
  const controller = new AbortController();
  controller.abort(new Error('cancel before spawn'));
  const result = await runCommand(join(directory, 'would-not-exist'), [], {
    cwd: root, env: process.env, log: join(directory, 'command.log'), timeoutMs: 10_000, signal: controller.signal
  });
  assert.equal(result.aborted, true);
  assert.equal(result.abortError, 'cancel before spawn');
  assert.equal(result.error, null);
  assert.equal(result.exitCode, null);
  assert.equal(result.status, 'failed');
}));
test('abort stops a running owned tree without reporting a timeout', () => fixture(async directory => {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(new Error('caller cancellation')), 200);
  try {
    const result = await runCommand(process.execPath, ['-e', 'setInterval(()=>{},60000)'], {
      cwd: root, env: process.env, log: join(directory, 'command.log'), timeoutMs: 10_000, signal: controller.signal
    });
    assert.equal(result.aborted, true);
    assert.equal(result.abortError, 'caller cancellation');
    assert.equal(result.timedOut, false);
    assert.equal(result.terminationError, null);
    assert.equal(result.status, 'failed');
  } finally { clearTimeout(timer); }
}));
for (const detached of [false, true]) test(`timeout terminates descendants with detached=${detached}`, () => fixture(async directory => {
  const code = `const {spawn}=require("node:child_process"); const c=spawn(process.execPath,["-e","setInterval(()=>{},60000)"],{stdio:"ignore",detached:${detached}}); console.log(c.pid); setInterval(()=>{},60000)`;
  const result = await run(directory, code, 1000);
  assert.equal(result.status, 'failed');
  assert.equal(result.terminationError, null);
  const pid = Number((await readFile(join(directory, 'command.log'), 'utf8')).trim());
  assert.ok(pid > 0);
  let absent = false;
  try { process.kill(pid, 0); }
  catch (error) { assert.equal(error.code, 'ESRCH'); absent = true; }
  if (!absent) {
    try {
      // Unix can retain a dead orphan's PID until init reaps it. A zombie is
      // terminated, but any still-executing descendant fails the assertion.
      assert.notEqual(process.platform, 'win32');
      const state = spawnSync('ps', ['-o', 'stat=', '-p', String(pid)], { encoding: 'utf8', timeout: 5000 });
      assert.equal(state.error, undefined);
      assert.ok((state.status === 1 && state.stdout.trim() === '') || /^Z/.test(state.stdout.trim()));
    } finally {
      // Preserve cleanup even when this regression detects an escaped child.
      try { process.kill(pid, 'SIGKILL'); }
      catch (error) { if (error.code !== 'ESRCH') throw error; }
    }
  }
}));
