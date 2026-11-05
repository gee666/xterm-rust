import {readdir,lstat,realpath,rm,writeFile,access} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import assert from 'node:assert/strict';
const root=process.cwd();
assert.equal(root.replaceAll('\\','/').toLowerCase(),'o:/xterm-rust-swarm-16/tmp/worktrees/geraldatlascache');
const git=(...args)=>{const r=spawnSync('C:/Program Files/Git/cmd/git.exe',args,{cwd:root,encoding:'utf8',timeout:30000});if(r.status!==0)throw new Error(r.stderr);return r.stdout.trim();};
assert.equal(git('branch','--show-current'),'swarm/GeraldAtlasCache');
assert.equal(git('status','--porcelain','--untracked-files=no'),'','tracked edits must be preserved before cleanup');
const proof={root,head:git('rev-parse','HEAD'),files:0,directories:0,links:[],redirects:[],workers:[],removed:[],mode:process.argv[2]??'preflight'};
async function tree(dir){
  const entries=await readdir(dir,{withFileTypes:true});
  for(const entry of entries){const file=path.join(dir,entry.name),info=await lstat(file);
    if(info.isSymbolicLink()){proof.links.push(file);continue;}
    if(info.isDirectory()){
      const actual=await realpath(file);if(path.resolve(actual).toLowerCase()!==path.resolve(file).toLowerCase()){proof.redirects.push({file,actual});continue;}
      proof.directories++;await tree(file);
    }else if(info.isFile())proof.files++;
  }
}
await tree(root);
assert.deepEqual(proof.links,[]);assert.deepEqual(proof.redirects,[]);
const escaped=root.replaceAll("'","''");
const query=`Get-CimInstance Win32_Process | Where-Object { $_.ProcessId -ne ${process.pid} -and $_.Name -match '^(node|cargo|rustc|chrome|firefox|MiniBrowser|playwright).*' -and ($_.CommandLine -like '*${escaped}*' -or $_.CommandLine -like '*run516.mjs*') } | Select-Object ProcessId,ParentProcessId,Name,CommandLine | ConvertTo-Json -Depth 3`;
const result=spawnSync('C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe',['-NoProfile','-Command',query],{encoding:'utf8',timeout:30000});
assert.equal(result.status,0);if(result.stdout.trim()){const rows=JSON.parse(result.stdout);proof.workers=Array.isArray(rows)?rows:[rows];}
assert.deepEqual(proof.workers,[],'owned processes must stop first; do not kill unknown workers');
if(proof.mode==='remove-runtime'){
  // All are new owned directories, preflighted without following links above.
  for(const relative of ['target','results/browser-host','results/acceptance-artifacts']){
    const file=path.join(root,relative);try{await access(file);}catch(e){if(e.code==='ENOENT')continue;throw e;}
    await rm(file,{recursive:true});proof.removed.push(relative);
  }
  // Leave tmp until this process exits. Its caller removes that exact directory.
}
await writeFile('results/browser-atlas-cache/cleanup.json',JSON.stringify(proof,null,2)+'\n');
console.log(JSON.stringify(proof,null,2));
