import {lstat, readdir, realpath, readFile, writeFile, rm} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve} from 'node:path';
const root = resolve(fileURLToPath(new URL('../../', import.meta.url)));
if (!/^O:\\xterm-rust-swarm-16\\tmp\\Orompoto306$/i.test(root)) throw Error('Unexpected owned cleanup root: ' + root);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const checkpoint = JSON.parse(await readFile(resolve(root, 'results/webgl-glyph-backgrounds/checkpoint.json'), 'utf8'));
for (const pin of checkpoint.pins) {
  if (hash(await readFile(resolve(root, pin.path))) !== pin.sha256) throw Error('New pin changed: ' + pin.path);
}
const browserPins = [];
for (const file of ['negative-browser.json','final-browser.json']) {
  const receipt = JSON.parse(await readFile(resolve(root, 'results/webgl-glyph-backgrounds', file), 'utf8'));
  for (const key of ['compiler','wasm','bindings']) {
    const pin = receipt.artifacts[key];
    const bytes = await readFile(resolve(root, pin.path));
    if (hash(bytes) !== pin.sha256) throw Error('Browser artifact changed: ' + key);
    browserPins.push({receipt: file, key, sha256: pin.sha256});
  }
}
let files = 0, directories = 0;
async function scan(path) {
  const info = await lstat(path);
  if (info.isSymbolicLink()) throw Error('Refuse any cleanup with a link: ' + path);
  if (info.isDirectory()) {
    if (resolve(await realpath(path)).toLowerCase() !== resolve(path).toLowerCase()) throw Error('Redirected directory: ' + path);
    directories++;
    for (const entry of await readdir(path)) await scan(resolve(path, entry));
  } else if (info.isFile()) files++;
  else throw Error('Unexpected filesystem object: ' + path);
}
await scan(root);
const command = `@(Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(cargo|rustc|node|chrome|firefox|MiniBrowser|msedge|playwright).*' -and $_.CommandLine -like '*Orompoto306*' -and $_.ProcessId -ne ${process.pid} } | Select-Object ProcessId,Name) | ConvertTo-Json -Compress`;
const raw = execFileSync('C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe', ['-NoProfile','-Command',command], {timeout: 30000, encoding: 'utf8'}).trim();
const processes = raw ? JSON.parse(raw) : [];
if (Array.isArray(processes) ? processes.length : processes.ProcessId) throw Error('Owned processes still live: ' + raw);
const report = {root, files, directories, links: 0, redirects: 0, processes: [], newPins: checkpoint.pins.length, browserPins,
  disposable: ['target','tmp','results/browser-host/wasm_webgl_glyphs','results/browser-host/wasm_webgl_glyphs-results.json'],
  gitWorktreeRemoval: 'Only after this report/script/handoff are committed and tracked status is clean. Use normal git worktree remove, no force.'};
await writeFile(resolve(root, 'results/webgl-glyph-backgrounds/cleanup-preflight.json'), JSON.stringify(report, null, 2) + '\n');
console.log('Preflight PASS', files, 'files', directories, 'directories, zero links/redirects/processes');
if (process.argv.includes('--remove-disposable')) {
  for (const path of report.disposable) await rm(resolve(root, path), {recursive: true, force: false});
  console.log('Removed only enumerated owned disposable outputs; source worktree still present');
}
