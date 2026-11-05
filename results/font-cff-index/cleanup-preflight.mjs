// Read-only ownership/link/process/hash audit. Deletion is a separate action.
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=process.cwd(),hash=b=>createHash('sha256').update(b).digest('hex');
const norm=p=>{const s=path.resolve(p).toLowerCase().replaceAll('\\','/');return s.startsWith('//?/')?s.slice(4):s;};
if(!norm(root).endsWith('/tmp/worktrees/hippolytacff'))throw new Error('Not owned worktree');
let files=0,directories=0;const links=[],redirects=[];
function visit(p){const s=fs.lstatSync(p);if(s.isSymbolicLink()){links.push(p);return;}if(norm(fs.realpathSync.native(p))!==norm(p)){redirects.push(p);return;}if(s.isDirectory()){directories++;for(const e of fs.readdirSync(p))visit(path.join(p,e));}else if(s.isFile())files++;else throw new Error(`Unexpected file ${p}`);}
visit(root);if(links.length||redirects.length)throw new Error('Unsafe cleanup tree');
if(!process.env.SystemRoot)throw new Error('Windows SystemRoot required');
const ps=path.join(process.env.SystemRoot,'System32/WindowsPowerShell/v1.0/powershell.exe');
const query="Get-CimInstance Win32_Process | Where-Object { $_.ProcessId -ne $PID -and $_.Name -match '^(cargo|rustc|node|chrome|firefox|MiniBrowser|WebKitNetworkProcess|WebKitWebProcess)(.exe)?$' -and $_.CommandLine -match 'HippolytaCff' } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress";
const text=execFileSync(ps,['-NoProfile','-Command',query],{encoding:'utf8',timeout:30000}).trim(),processes=text?JSON.parse(text):[];
// The audit's own Node invocation mentions this file only, not the worktree.
if((Array.isArray(processes)?processes:[processes]).length)throw new Error('Owned process still active '+text);
const manifest=JSON.parse(fs.readFileSync('docs/baseline/reference-manifest.json'));
for(const base of ['source_repo','P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo'])for(const entry of manifest)if(hash(fs.readFileSync(path.join(base,entry.path)))!==entry.sha256)throw new Error(`Reference mismatch ${base}/${entry.path}`);
const pins=JSON.parse(fs.readFileSync('results/font-cff-index/producer-hashes.json'));
for(const [file,sha]of Object.entries(pins))if(hash(fs.readFileSync(file))!==sha)throw new Error(`Producer changed ${file}`);
const report={root,files,directories,links,redirects,ownedProcesses:processes,producerPins:Object.keys(pins).length,referenceFilesEach:manifest.length,scope:'Only this owned regular worktree. Preserve all source/fixtures/concise receipts and pinned compiler WASM before removal; no main refs, dependency caches or reference junctions are cleanup targets.'};
fs.writeFileSync('results/font-cff-index/cleanup-preflight.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
