// Owned receipt automation adapted from accepted canvas-layer cleanup-preflight.
// It cannot target another worktree or traverse any linked/reference directory.
import {readdir,lstat,realpath,rm,writeFile,readFile,access} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import assert from 'node:assert/strict';
const root=process.cwd();
assert.equal(root.replaceAll('\\','/').toLowerCase(),'o:/xterm-rust-swarm-16/tmp/worktrees/rebeccawebgl');
const git=(...args)=>{const r=spawnSync('C:/Program Files/Git/cmd/git.exe',args,{cwd:root,encoding:'utf8',timeout:30000});if(r.status!==0)throw Error(r.stderr);return r.stdout.trim();};
assert.equal(git('branch','--show-current'),'swarm/RebeccaWebgl');
assert.equal(git('status','--porcelain','--untracked-files=no'),'','commit owned tracked changes before cleanup');
const mode=process.argv[2]??'preflight';assert.ok(['preflight','remove-runtime','whole-preflight'].includes(mode));
const proof={root,head:git('rev-parse','HEAD'),mode,files:0,directories:0,links:[],redirects:[],workers:[],removed:[]};
async function tree(dir){for(const e of await readdir(dir,{withFileTypes:true})){
 const file=path.join(dir,e.name),info=await lstat(file);
 if(info.isSymbolicLink()){proof.links.push(file);continue;}
 if(info.isDirectory()){
  const actual=await realpath(file);if(path.resolve(actual).toLowerCase()!==path.resolve(file).toLowerCase()){proof.redirects.push({file,actual});continue;}
  proof.directories++;await tree(file);
 }else if(info.isFile())proof.files++;
}}
await tree(root);assert.deepEqual(proof.links,[]);assert.deepEqual(proof.redirects,[]);
const query=`Get-CimInstance Win32_Process | Where-Object { $_.ProcessId -ne ${process.pid} -and $_.Name -match '^(node|cargo|rustc|chrome|firefox|MiniBrowser|playwright).*' -and $_.CommandLine -like '*${root.replaceAll("'","''")}*' } | Select-Object ProcessId,ParentProcessId,Name,CommandLine | ConvertTo-Json -Depth 3`;
const r=spawnSync('C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe',['-NoProfile','-Command',query],{encoding:'utf8',timeout:30000});
assert.equal(r.status,0);if(r.stdout.trim()){const rows=JSON.parse(r.stdout);proof.workers=Array.isArray(rows)?rows:[rows];}
assert.deepEqual(proof.workers,[],'stop only owned workers before cleanup; never kill unknown PIDs');
if(mode==='remove-runtime'){
 const audit=spawnSync(process.execPath,['scripts/webgl-frame-audit.mjs','--runtime'],{cwd:root,env:process.env,encoding:'utf8',timeout:60000});
 assert.equal(audit.status,0,audit.stderr);proof.lastRuntimeAudit=audit.stdout.trim();
 const key='results/browser-host/wasm_webgl_frame',pins=new Set();
 for(const e of await readdir(key,{withFileTypes:true}))if(e.isDirectory()){
  const report=JSON.parse(await readFile(path.join(key,e.name,'wasm_webgl_frame-results.json'),'utf8'));
  if(report.artifacts?.compiler){const compiler=report.artifacts.compiler.path;
   assert.match(compiler,/^results\/acceptance-artifacts\/[a-f0-9]{64}\/artifact\.wasm$/);pins.add(path.dirname(compiler));
  }
 }
 for(const relative of ['target','tmp',key,`${key}-results.json`,...pins]){
  assert.equal(git('ls-files','--',relative),'','never remove tracked source/evidence');
  const file=path.join(root,relative);try{await access(file);}catch(e){if(e.code==='ENOENT')continue;throw e;}
  await rm(file,{recursive:true});proof.removed.push(relative);
 }
}
await writeFile(`results/webgl-frame/cleanup-${mode}.json`,JSON.stringify(proof,null,2)+'\n');
console.log(JSON.stringify(proof,null,2));
