// Read-only link, redirect, process, producer and source checks before owned cleanup.
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=process.cwd(),sha=b=>createHash('sha256').update(b).digest('hex');
const norm=p=>{const s=path.resolve(p).toLowerCase().split(String.fromCharCode(92)).join('/');return s.startsWith('//?/')?s.slice(4):s;};
if(!norm(root).endsWith('/tmp/worktrees/abigail457'))throw new Error('Not owned worktree');
let files=0,directories=0;const links=[],redirects=[];
function visit(p){const s=fs.lstatSync(p);if(s.isSymbolicLink()){links.push(p);return;}if(norm(fs.realpathSync.native(p))!==norm(p)){redirects.push(p);return;}if(s.isDirectory()){directories++;for(const e of fs.readdirSync(p))visit(path.join(p,e));}else if(s.isFile())files++;else throw new Error('Unexpected file '+p);}
visit(root);if(links.length||redirects.length)throw new Error('Unsafe cleanup tree');
const powershell="Get-CimInstance Win32_Process | Where-Object { $_.ProcessId -ne $PID -and $_.Name -match '^(cargo|rustc|node|chrome|firefox|MiniBrowser|WebKitNetworkProcess|WebKitWebProcess)(.exe)?$' -and $_.CommandLine -match 'Abigail457' } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress";
if(!process.env.SystemRoot)throw new Error('Windows SystemRoot is required for the native process audit');
const executable=path.join(process.env.SystemRoot,'System32','WindowsPowerShell','v1.0','powershell.exe');
const text=execFileSync(executable,['-NoProfile','-Command',powershell],{encoding:'utf8',timeout:30000}).trim();
const processes=text?JSON.parse(text):[];
if((Array.isArray(processes)?processes:[processes]).length)throw new Error('Owned process still active '+text);
const manifest=JSON.parse(fs.readFileSync('docs/baseline/reference-manifest.json'));
for(const base of ['source_repo','P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo'])for(const m of manifest){const b=fs.readFileSync(path.join(base,m.path));if(b.length!==m.bytes||sha(b)!==m.sha256)throw new Error('Reference mismatch '+base+'/'+m.path);}
const pins=JSON.parse(fs.readFileSync('results/font-cff-header/producer-hashes.json'));
for(const [p,h]of Object.entries(pins))if(sha(fs.readFileSync(p))!==h)throw new Error('Producer changed '+p);
const report={root,files,directories,links,redirects,ownedProcesses:processes,producerPins:Object.keys(pins).length,referenceFilesEach:manifest.length,earlierAttempt:'Inline bash escaping caused a JavaScript syntax error before execution or file access; no cleanup side effects. Handoff commit20c11072 preserved docs only. A second read-only attempt scanned the tree but stopped before process inspection because powershell.exe was absent from PATH, also with no cleanup side effects. This file-based run resolves PowerShell from SystemRoot and is authoritative.',untrackedDisposablePaths:['results/browser-host/xterm-font-addons__font_cff_header_browser-results.json','results/browser-host/xterm-font-addons__font_cff_header_browser','results/browser-host/xterm-font-addons__font_hint_tables_browser-results.json','results/browser-host/xterm-font-addons__font_hint_tables_browser'],scope:'All concise source/fixtures/receipts committed before normal owned worktree removal, no cleanup outside this root'};
fs.writeFileSync('results/font-cff-header/cleanup-preflight.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
