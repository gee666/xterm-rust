// Read-only safety checks. Only the durable report is written; nothing is removed.
import assert from 'node:assert/strict';
import { lstat, readdir, readFile, realpath, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { referenceIntegrity } from '../../scripts/upstream-baseline-utils.mjs';
const root = resolve(process.cwd()), normalized = path => resolve(path).replaceAll('\\', '/').toLowerCase();
assert.equal(normalized(root), 'o:/xterm-rust-swarm-16/tmp/worktrees/emeryansicoercion');
const report = { root, revision: execFileSync('git', ['rev-parse', 'HEAD'], { timeout: 30000, encoding: 'utf8' }).trim(),
  files: 0, directories: 0, links: [], workers: [] };
async function walk(directory) {
  for (const entry of await readdir(directory)) {
    const file = join(directory, entry), info = await lstat(file);
    if (info.isSymbolicLink()) report.links.push(file);
    else if (info.isDirectory()) {
      if (normalized(await realpath(file)) !== normalized(file)) report.links.push(file);
      else { report.directories++; await walk(file); }
    } else if (info.isFile()) report.files++;
    else report.links.push(file);
  }
}
await walk(root);
const ps = '$ErrorActionPreference="Stop"; @(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name,CommandLine) | ConvertTo-Json -Compress';
const processes = JSON.parse(execFileSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', ps],
  { timeout: 30000, encoding: 'utf8', maxBuffer: 8 * 1024 ** 2 }));
const owners = new Set(processes.filter(item => item.ProcessId !== process.pid &&
  /^(cargo|rustc|rustdoc|clippy-driver|wasm-bindgen|node|chrome|firefox|msedge|MiniBrowser|Playwright|serialize).*\.exe$/i.test(item.Name) &&
  String(item.CommandLine).replaceAll('\\', '/').toLowerCase().includes(normalized(root))).map(item => item.ProcessId));
let changed;
do {
  changed = false;
  for (const item of processes) if (owners.has(item.ParentProcessId) && !owners.has(item.ProcessId)) {
    owners.add(item.ProcessId); changed = true;
  }
} while (changed);
report.workers = processes.filter(item => owners.has(item.ProcessId));
const manifest = JSON.parse(await readFile(join(root, 'docs/baseline/reference-manifest.json'), 'utf8'));
report.original = referenceIntegrity(process.env.SOURCE_ROOT, manifest);
report.copy = referenceIntegrity(join(root, 'source_repo'), manifest);
report.status = report.links.length === 0 && report.workers.length === 0 && report.original.mismatches.length === 0 && report.copy.mismatches.length === 0 ? 'passed' : 'failed';
await writeFile(join(root, 'results/serialize-ansi-coercion/cleanup-preflight.json'), JSON.stringify(report, null, 2) + '\n');
assert.equal(report.status, 'passed');
console.log(JSON.stringify({ status: report.status, files: report.files, directories: report.directories, links: report.links.length, workers: report.workers.length }));
