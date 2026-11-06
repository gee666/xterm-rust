// Native Windows WebKit process-lifetime proof, not terminal functionality.
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { watch } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import { runCommand } from './quality-process.mjs';

if (process.platform !== 'win32') throw new Error('Native Windows WebKit lifecycle check required');
const root = fileURLToPath(new URL('../', import.meta.url));
const cli = process.env.PLAYWRIGHT_CLI ?? join(root, 'source_repo/node_modules/@playwright/test/cli.js');
await readFile(cli);
const output = join(root, 'results/owned-process562', `${Date.now()}-webkit-${randomUUID()}`);
await mkdir(output, { recursive: true });
await mkdir(join(root, 'tmp'), { recursive: true });
const temporary = await mkdtemp(join(root, 'tmp', 'owned-webkit-'));
const report = { scope: 'WebKit cleanup only, no terminal engine assertions', platform: process.platform,
  node: process.version, cli, temporary, attempts: [], status: 'failed' };
const outer = new AbortController();
const interrupt = () => outer.abort(new Error('WebKit lifecycle checks interrupted'));
process.once('SIGINT', interrupt); process.once('SIGTERM', interrupt);
function inventory(pid, script) {
  const result = spawnSync(join(process.env.SystemRoot, 'System32/WindowsPowerShell/v1.0/powershell.exe'),
    ['-NoProfile', '-NonInteractive', '-File', script, String(pid)],
    { encoding: 'utf8', timeout: 5000, windowsHide: true });
  assert.equal(result.error, undefined); assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
function alive(pid) {
  try { process.kill(pid, 0); return true; }
  catch (error) { if (error.code !== 'ESRCH') throw error; return false; }
}
try {
  const snapshot = join(temporary, 'snapshot.ps1');
  await writeFile(snapshot, `param([int]$ownerId)
$ErrorActionPreference='Stop'
$all=@(Get-CimInstance Win32_Process)
$ids=[Collections.Generic.HashSet[int]]::new()
[void]$ids.Add($ownerId)
do {
 $added=$false
 foreach($item in $all) {
  if($ids.Contains([int]$item.ParentProcessId) -and $ids.Add([int]$item.ProcessId)) { $added=$true }
 }
} while($added)
$found=@($all | Where-Object { $ids.Contains([int]$_.ProcessId) } | ForEach-Object {
 [ordered]@{pid=[int]$_.ProcessId;parent=[int]$_.ParentProcessId;name=$_.Name;created=$_.CreationDate.ToUniversalTime().ToString('o')}
})
ConvertTo-Json -InputObject $found -Depth 4
`);
  for (const mode of ['normal', 'global-timeout', 'abort']) {
    const attempt = join(temporary, mode);
    await mkdir(attempt);
    const ready = join(attempt, 'ready.json'); const gate = join(attempt, 'release');
    const spec = join(attempt, 'cleanup.spec.cjs'); const config = join(attempt, 'playwright.cjs');
    await writeFile(spec, `const {test}=require(${JSON.stringify(join(dirname(cli), 'index.js'))});
const fs=require('node:fs');const http=require('node:http');const {once}=require('node:events');
test('real WebKit network helper lifecycle',async({browser})=>{
 const server=http.createServer((req,res)=>res.end('<title>owned</title>'));
 server.listen(0,'127.0.0.1');await once(server,'listening');
 const page=await browser.newPage();await page.goto('http://127.0.0.1:'+server.address().port);
 const released=new Promise(resolve=>{const w=fs.watch(${JSON.stringify(attempt)},(e,n)=>{
  if(n==='release'){w.close();resolve()}
 });});
 fs.writeFileSync(${JSON.stringify(ready + '.writing')},JSON.stringify({pid:process.pid,version:browser.version()}));
 fs.renameSync(${JSON.stringify(ready + '.writing')},${JSON.stringify(ready)});
 await released;
 ${mode === 'normal' ? 'await page.close();await new Promise(resolve=>server.close(resolve));' : 'await new Promise(()=>{});'}
});
`);
    await writeFile(config, `module.exports={testDir:${JSON.stringify(attempt)},testMatch:'cleanup.spec.cjs',
 projects:[{name:'WebKit',use:{browserName:'webkit',headless:true}}],workers:1,retries:0,
 timeout:30000,globalTimeout:12000};`);
    const controller = new AbortController();
    const forward = () => controller.abort(outer.signal.reason);
    outer.signal.addEventListener('abort', forward, { once: true });
    if (outer.signal.aborted) forward();
    let observerError; let observed; let probing = false;
    const observer = watch(attempt, (event, filename) => {
      if (filename !== 'ready.json' || probing) return;
      probing = true;
      void (async () => {
        try {
          const readyState = JSON.parse(await readFile(ready, 'utf8'));
          observed = { ...readyState, processes: inventory(readyState.pid, snapshot) };
          assert.ok(observed.processes.some(item => item.name === 'WebKitNetworkProcess.exe'), 'real network helper must be observed');
          if (mode === 'abort') controller.abort(new Error('controlled real WebKit cancellation'));
          else await writeFile(gate, 'release');
        } catch (error) { observerError = error; controller.abort(error); }
      })();
    });
    let result;
    try {
      result = await runCommand(process.execPath, [cli, 'test', `--config=${config}`, '--reporter=json'], {
        cwd: attempt, env: { ...process.env, TEMP: attempt, TMP: attempt, TMPDIR: attempt,
          PLAYWRIGHT_JSON_OUTPUT_FILE: join(output, `${mode}.json`) },
        log: join(output, `${mode}.log`), timeoutMs: 20_000, signal: controller.signal, containDescendants: true
      });
    } finally { observer.close(); outer.signal.removeEventListener('abort', forward); }
    const entry = { mode, ...result, observed, observerError: observerError?.stack };
    report.attempts.push(entry);
    assert.equal(observerError, undefined); assert.ok(observed);
    entry.stillExecuting = observed.processes.filter(item => alive(item.pid));
    assert.deepEqual(entry.stillExecuting, []);
    assert.equal(result.timedOut, false);
    assert.equal(result.aborted, mode === 'abort'); assert.equal(result.terminationError, null);
    assert.equal(result.exitCode, mode === 'normal' ? 0 : 1);
    if (mode === 'global-timeout') {
      const raw = JSON.parse(await readFile(join(output, `${mode}.json`), 'utf8'));
      assert.match(JSON.stringify(raw.errors), /Timed out waiting/);
    }
    await rm(attempt, { recursive: true }); entry.directoryRemoved = true;
  }
  report.status = 'passed normal, actual Playwright global timeout and abort cleanup';
} catch (error) { report.error = error.stack ?? String(error); process.exitCode = 1; }
finally {
  process.off('SIGINT', interrupt); process.off('SIGTERM', interrupt);
  try { await rm(temporary, { recursive: true }); report.temporaryRemoved = true; }
  catch (error) { report.cleanupError = String(error); process.exitCode = 1; }
  await writeFile(join(output, 'summary.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(JSON.stringify(report, null, 2));
