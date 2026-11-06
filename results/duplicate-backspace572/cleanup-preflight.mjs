// Own worktree cleanup only. Never follow reference/dependency links.
import fs from 'node:fs';
import cp from 'node:child_process';
import path from 'node:path';
import assert from 'node:assert/strict';
const expected = 'O:/xterm-rust-swarm-16/tmp/worktrees/KarnaBackspace';
const cwd = process.cwd();
const normalize = value => path.resolve(value).replaceAll('\\','/').toLowerCase();
assert.equal(normalize(cwd), normalize(expected));
const git = (...args) => cp.execFileSync('C:/Program Files/Git/cmd/git.exe',args,{cwd,encoding:'utf8',timeout:30000}).trim();
assert.equal(git('branch','--show-current'),'swarm/KarnaBackspace');
const scan = {files:0,directories:0,bytes:0,links:[],redirects:[]};
function walk(directory) {
  if (normalize(fs.realpathSync.native(directory)) !== normalize(directory)) {
    scan.redirects.push(directory); return;
  }
  for (const entry of fs.readdirSync(directory,{withFileTypes:true})) {
    const local = path.join(directory,entry.name);
    if (entry.isSymbolicLink()) {scan.links.push(local);continue;}
    if (entry.isDirectory()) {scan.directories++;walk(local);}
    else if (entry.isFile()) {scan.files++;scan.bytes += fs.statSync(local).size;}
    else throw new Error(`Unknown file type ${local}`);
  }
}
walk(cwd);
assert.deepEqual(scan.links,[]); assert.deepEqual(scan.redirects,[]);
const command = '$root="KarnaBackspace"; Get-CimInstance Win32_Process | Where-Object { $_.Name -match "^(node|git|cargo|rustc|rustfmt|clippy-driver|chrome|firefox|Playwright|WebKit).*\\.exe$" -and $_.CommandLine -like "*$root*" } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress';
const live = cp.execFileSync('C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe',['-NoProfile','-Command',command],{cwd,encoding:'utf8',timeout:30000}).trim();
// This script's own Node process is the only allowed match.
let workers = live ? JSON.parse(live) : [];
if (!Array.isArray(workers)) workers = [workers];
workers = workers.filter(worker => worker.ProcessId !== process.pid);
assert.deepEqual(workers,[]);
const result = {root:expected,head:git('rev-parse','HEAD'),branch:git('branch','--show-current'),scan,liveWorkers:workers.length,cargoInvocations:0,browserRuns:0,grants:0,at:new Date().toISOString()};
fs.writeFileSync('results/duplicate-backspace572/cleanup-preflight.json',JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result));
