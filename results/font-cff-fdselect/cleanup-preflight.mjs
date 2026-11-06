// Read-only ownership/link/process/hash preflight; deletion is separate.
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=process.cwd(),sha=b=>createHash('sha256').update(b).digest('hex');
const norm=p=>{const s=path.resolve(p).toLowerCase().replaceAll('\\','/');return s.startsWith('//?/')?s.slice(4):s;};
if(!norm(root).endsWith('/tmp/worktrees/juliefdselect'))throw Error('Not owned worktree');
let files=0,directories=0;const links=[],redirects=[];
function visit(p){const s=fs.lstatSync(p);if(s.isSymbolicLink()){links.push(p);return;}if(norm(fs.realpathSync.native(p))!==norm(p)){redirects.push(p);return;}if(s.isDirectory()){directories++;for(const e of fs.readdirSync(p))visit(path.join(p,e));}else if(s.isFile())files++;else throw Error('Unexpected file '+p);}
visit(root);if(links.length||redirects.length)throw Error('Unsafe cleanup tree');
if(!process.env.SystemRoot)throw Error('Windows SystemRoot required');
const powershell=path.join(process.env.SystemRoot,'System32/WindowsPowerShell/v1.0/powershell.exe');
const query="Get-CimInstance Win32_Process | Where-Object { $_.ProcessId -ne $PID -and $_.ProcessId -ne "+process.pid+" -and $_.Name -match '^(cargo|rustc|node|chrome|firefox|MiniBrowser|WebKitNetworkProcess|WebKitWebProcess|xterm_addons-[0-9a-f]+|font_cff_fdselect-[0-9a-f]+)(.exe)?$' -and $_.CommandLine -match 'JulieFdSelect' } | Select-Object ProcessId,Name,CommandLine | ConvertTo-Json -Compress";
const text=execFileSync(powershell,['-NoProfile','-Command',query],{encoding:'utf8',timeout:30000}).trim(),processes=text?JSON.parse(text):[];
if((Array.isArray(processes)?processes:[processes]).length)throw Error('Owned process active '+text);
const manifest=JSON.parse(fs.readFileSync('docs/baseline/reference-manifest.json')),original=process.env.REFERENCE_ROOT;
if(!original)throw Error('Set REFERENCE_ROOT to the original reference');
for(const base of ['source_repo',original])for(const e of manifest)if(sha(fs.readFileSync(path.join(base,e.path)))!==e.sha256)throw Error('Reference drift '+base+'/'+e.path);
const pins=JSON.parse(fs.readFileSync('results/font-cff-fdselect/producer-hashes.json')).files;
for(const [file,pin]of Object.entries(pins)){const bytes=fs.readFileSync(file),canonical=pin.mode==='text'?Buffer.from(bytes.toString().replaceAll('\r\n','\n')):bytes;if(sha(canonical)!==pin.canonicalSha256)throw Error('Producer drift '+file);}
const report={root,files,directories,links,redirects,ownedProcesses:processes,producerPins:Object.keys(pins).length,referenceFilesEach:manifest.length,scope:'Only this owned regular worktree. Preserve accepted commits, source/fixtures/receipts/compiler pins; unlink only owned generated browser outputs after hash audit. No other owners, reference junctions, caches, main or credentials are cleanup targets.'};
fs.writeFileSync('results/font-cff-fdselect/cleanup-preflight.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
