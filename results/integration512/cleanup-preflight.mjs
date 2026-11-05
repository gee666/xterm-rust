// Reuses accepted integration463's read-only link/process/reference preflight.
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root = process.cwd(), sha = bytes => createHash('sha256').update(bytes).digest('hex');
const norm = p => path.resolve(p).toLowerCase().replaceAll('\\', '/').replace(/^\/\/\?\//, '');
if (norm(root) !== 'o:/xterm-rust-swarm-16/tmp/worktrees/scottnext') throw Error('Not owned worktree');
let files = 0, directories = 0;
const links = [], redirects = [];
function visit(file) {
  const stat = fs.lstatSync(file);
  if (stat.isSymbolicLink()) { links.push(file); return; }
  if (norm(fs.realpathSync.native(file)) !== norm(file)) { redirects.push(file); return; }
  if (stat.isDirectory()) {
    directories++;
    for (const name of fs.readdirSync(file)) visit(path.join(file, name));
  } else if (stat.isFile()) files++;
  else throw Error(`Unexpected entry ${file}`);
}
visit(root);
if (links.length || redirects.length) throw Error('Unsafe linked cleanup tree');
const powershell = path.join(process.env.SystemRoot, 'System32/WindowsPowerShell/v1.0/powershell.exe');
const command = `@(Get-CimInstance Win32_Process | Where-Object { $_.ProcessId -ne $PID -and $_.ProcessId -ne ${process.pid} -and $_.CommandLine -and $_.Name -match '^(cargo|rustc|clippy-driver|node|chrome|firefox|MiniBrowser|WebKitNetworkProcess|WebKitWebProcess)(.exe)?$' -and $_.CommandLine -match 'ScottNext' } | Select-Object ProcessId,Name,CommandLine) | ConvertTo-Json -Compress`;
const output = execFileSync(powershell, ['-NoProfile', '-Command', command], {encoding: 'utf8', timeout: 30000}).trim();
const decoded = output ? JSON.parse(output) : [];
const processes = Array.isArray(decoded) ? decoded : [decoded];
if (processes.length) throw Error(`Owned processes remain: ${output}`);
const manifest = JSON.parse(fs.readFileSync('docs/baseline/reference-manifest.json'));
for (const base of ['source_repo', 'P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo']) {
  for (const entry of manifest) {
    const bytes = fs.readFileSync(path.join(base, entry.path));
    if (bytes.length !== entry.bytes || sha(bytes) !== entry.sha256) throw Error(`Reference changed ${base}/${entry.path}`);
  }
}
const report = {root, files, directories, links, redirects, ownedProcesses: processes,
  referenceFilesEach: manifest.length, heavyGrantsHeld: 0, privateTargetRemoved: !fs.existsSync('target'),
  branch: 'swarm/ScottNext', canonicalMain: '1106d4806217af25e3ebc5831093447002bd8596',
  scope: 'Preserve committed source/evidence and branches; remove only this owned root through normal git worktree remove after review. No force, unknown PID action or reference-link traversal.'};
fs.writeFileSync('results/integration512/cleanup-preflight.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report));
