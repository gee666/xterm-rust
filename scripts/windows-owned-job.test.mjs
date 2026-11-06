// Real Windows Job ownership regressions. Other OS process trees are unchanged.
import test from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdir, mkdtemp, readFile, rm } from 'node:fs/promises';
import { watch } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { runCommand } from './quality-process.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const leaf = 'process.send({pid:process.pid});process.disconnect();setInterval(()=>{},60000)';
const middle = `const {spawn}=require('node:child_process');
const child=spawn(process.execPath,['-e',${JSON.stringify(leaf)},process.argv[1]],
 {stdio:['ignore','ignore','ignore','ipc'],detached:true,cwd:process.cwd()});
child.on('message',value=>{process.send(value);child.unref();process.exit(0)});`;
function parent(mode, ready, nonce) {
  return `const {spawn}=require('node:child_process');const fs=require('node:fs');let leaf;
const middle=spawn(process.execPath,['-e',${JSON.stringify(middle)},${JSON.stringify(nonce)}],
 {stdio:['ignore','ignore','ignore','ipc'],cwd:process.cwd()});
middle.on('message',value=>{leaf=value});
middle.on('exit',()=>{if(!leaf)process.exit(9);
 fs.writeFileSync(${JSON.stringify(ready + '.writing')},JSON.stringify({...leaf,middle:middle.pid}));
 fs.renameSync(${JSON.stringify(ready + '.writing')},${JSON.stringify(ready)});
 ${mode === 'normal' ? 'process.exit(7)' : 'setInterval(()=>{},60000)'};});`;
}
function alive(pid) {
  try { process.kill(pid, 0); return true; }
  catch (error) { if (error.code !== 'ESRCH') throw error; return false; }
}
if (process.env.XTERM_OWNED_JOB_NEGATIVE === '1' && process.env.XTERM_OWNED_JOB_NEGATIVE_CONTAINED !== '1') {
  throw new Error('Negative control requires the outer owned-command-check Job');
}
if (process.platform === 'win32') for (const mode of ['normal', 'timeout', 'abort']) {
  test(`Windows Job removes dead-parent detached helpers on ${mode}`, async () => {
    const base = process.env.XTERM_OWNED_JOB_FIXTURE_ROOT ?? join(root, 'tmp');
    await mkdir(base, { recursive: true });
    const directory = await mkdtemp(join(base, 'owned-job-tree-'));
    const attempt = join(directory, 'attempt');
    await mkdir(attempt);
    const ready = join(attempt, 'ready.json');
    const controller = new AbortController();
    const sentinel = spawn(process.execPath, ['-e', 'setInterval(()=>{},60000)'], {
      cwd: directory, stdio: 'ignore'
    });
    const sentinelExit = once(sentinel, 'exit', { signal: AbortSignal.timeout(30_000) });
    const observer = watch(attempt, (event, filename) => {
      if (mode === 'abort' && filename === 'ready.json') controller.abort(new Error('fixture cancellation'));
    });
    try {
      const result = await runCommand(process.execPath, ['-e', parent(mode, ready, randomUUID())], {
        cwd: attempt, env: process.env, log: join(directory, 'command.log'),
        timeoutMs: mode === 'timeout' ? 15_000 : 20_000, signal: controller.signal,
        containDescendants: process.env.XTERM_OWNED_JOB_NEGATIVE !== '1'
      });
      const owned = JSON.parse(await readFile(ready, 'utf8'));
      assert.ok(owned.pid > 0 && owned.middle > 0);
      assert.equal(alive(owned.middle), false, 'intermediate parent has already exited');
      assert.equal(alive(owned.pid), false, 'detached orphan must not outlive the command');
      assert.equal(alive(sentinel.pid), true, 'an unrelated owned test process remains untouched');
      assert.equal(result.timedOut, mode === 'timeout');
      assert.equal(result.aborted, mode === 'abort');
      assert.equal(result.terminationError, null);
      assert.equal(result.status, 'failed');
      if (mode === 'normal') assert.equal(result.exitCode, 7);
      observer.close();
      await rm(attempt, { recursive: true });
    } finally {
      observer.close();
      if (sentinel.exitCode === null) sentinel.kill('SIGKILL');
      await sentinelExit;
      // A negative control runs inside an OUTER Job. Do not kill orphan PIDs
      // here: the outer job closes them when this test process exits, then its
      // caller removes the uniquely owned fixture parent. Positive cleanup is
      // asserted here without that outer job's help.
      if (process.env.XTERM_OWNED_JOB_NEGATIVE !== '1') await rm(directory, { recursive: true });
    }
  });
}
