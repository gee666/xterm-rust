// Use when inspecting this task's owned root before normal worktree removal.
// This script scans and records evidence only; it does not delete anything.
import {readdir,lstat,realpath,readFile,writeFile} from 'node:fs/promises';
import {resolve,join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=resolve('O:/xterm-rust-swarm-16/tmp/worktrees/HypatiaImageRows');
if(resolve(process.cwd()).toLowerCase()!==root.toLowerCase())throw Error('Wrong owned root');
const git=(...args)=>execFileSync(process.env.GIT??'git',args,{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024}).trim();
if(git('branch','--show-current')!=='swarm/HypatiaImageRows')throw Error('Wrong owner branch');
if(git('status','--porcelain','--untracked-files=no'))throw Error('Tracked changes remain');
const report={root,branch:'swarm/HypatiaImageRows',head:git('rev-parse','HEAD'),files:0,directories:0,links:[],redirects:[],ownedJobs:[],referenceFiles:0};
async function scan(path) {
  const stat=await lstat(path);
  if(stat.isSymbolicLink()){report.links.push(path);return;}
  if((await realpath(path)).toLowerCase()!==resolve(path).toLowerCase()){report.redirects.push(path);return;}
  if(stat.isDirectory()){report.directories++;for(const name of await readdir(path))await scan(join(path,name));}
  else if(stat.isFile())report.files++;
  else throw Error('Unexpected file kind '+path);
}
await scan(root);
const command=`$ErrorActionPreference='Stop'; @(Get-CimInstance Win32_Process | Where-Object {$_.CommandLine -like '*HypatiaImageRows*' -and $_.ProcessId -ne $PID -and $_.ProcessId -ne ${process.pid} -and $_.ProcessId -ne ${process.ppid}} | Select-Object ProcessId,ParentProcessId,Name,CommandLine) | ConvertTo-Json -Compress`;
const output=execFileSync('C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe',['-NoProfile','-NonInteractive','-Command',command],{encoding:'utf8',timeout:30000}).trim();
report.ownedJobs=output?JSON.parse(output):[];
if(!Array.isArray(report.ownedJobs))report.ownedJobs=[report.ownedJobs];
const manifest=JSON.parse(await readFile('docs/baseline/reference-manifest.json','utf8'));
for(const pin of manifest) {
  const bytes=await readFile(join('P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo',pin.path));
  if(createHash('sha256').update(bytes).digest('hex')!==pin.sha256)throw Error('Original reference changed '+pin.path);
  report.referenceFiles++;
}
if(report.links.length||report.redirects.length||report.ownedJobs.length)throw Error('Unsafe cleanup preflight '+JSON.stringify(report));
await writeFile('results/image-row-coordinates/cleanup-preflight.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
