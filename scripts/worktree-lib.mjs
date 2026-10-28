// Portable git-worktree operations. Builds use private targets, not a global lease.
import {spawnSync} from 'node:child_process';
import {mkdir,statfs,realpath,lstat,rm,unlink,rename,readdir,copyFile,readFile,writeFile,appendFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';

export function run(command,args,cwd,{inherit=false,env=process.env,timeout=600000,raw=false}={}) {
  const result=spawnSync(command,args,{cwd,env,encoding:'utf8',windowsHide:true,
    stdio:inherit?'inherit':'pipe',timeout,maxBuffer:16*1024*1024});
  if(result.error||result.status!==0)throw new Error(`${command} ${args.join(' ')} failed${result.error?`: ${result.error.message}`:''}\n${result.stderr??''}`);
  return raw ? result.stdout??'' : result.stdout?.trim()??'';
}
const git=(cwd,...args)=>run(process.env.GIT??'git',args,cwd);
export function parseArgs(args,allowed) {
  const result={};
  for(let index=0;index<args.length;index++) {
    const key=args[index];
    if(!allowed.includes(key)||(result[key]!==undefined&&key!=='--copy-owned'))throw new Error(`Unknown or repeated option ${key}`);
    const value=args[++index];if(!value||value.startsWith('--'))throw new Error(`${key} needs a value`);
    if(key==='--copy-owned')(result[key]??=[]).push(value);else result[key]=value;
  }
  return result;
}
export function worktrees(cwd) {
  return git(cwd,'worktree','list','--porcelain').split(/\r?\n\r?\n/).filter(Boolean).map(block=>{
    const lines=block.split(/\r?\n/);return {directory:lines.find(line=>line.startsWith('worktree '))?.slice(9),
      branch:lines.find(line=>line.startsWith('branch '))?.slice(7)};
  });
}
function hasRef(cwd,ref) {
  const result=spawnSync(process.env.GIT??'git',['show-ref','--verify','--quiet',ref],{cwd,timeout:30000});
  if(result.status===0)return true;if(result.status===1)return false;
  throw new Error(`Cannot inspect git ref ${ref}`);
}
export function repository(cwd,requestedMain) {
  const root=git(cwd,'rev-parse','--show-toplevel');
  const main=requestedMain??(hasRef(root,'refs/heads/main')?'main':'master');
  if(!hasRef(root,`refs/heads/${main}`))throw new Error(`Main branch ${main} does not exist; pass --main`);
  const primary=worktrees(root).find(item=>item.branch===`refs/heads/${main}`);
  if(!primary)throw new Error(`${main} must be checked out in a primary worktree`);
  return {root,main,primary:primary.directory};
}
export async function ensureSpace(directory,minimum=1024**3) {
  await mkdir(directory,{recursive:true});const disk=await statfs(directory);
  const free=Number(disk.bavail)*Number(disk.bsize);
  if(free<minimum)throw new Error(`${directory}: only ${free} bytes free, need ${minimum}`);
  return free;
}
function within(root,file) {const rel=path.relative(root,file);return rel!==''&&!rel.startsWith(`..${path.sep}`)&&rel!=='..'&&!path.isAbsolute(rel);}
export async function defaultBase(primary) {
  const logical=path.resolve(primary,'target');
  try {
    const physical=await realpath(logical);
    if(!within(primary,physical))return path.join(path.dirname(physical),`${path.basename(physical)}-worktrees`);
  } catch(error) {if(error.code!=='ENOENT')throw error;}
  return path.join(primary,'tmp','worktrees');
}
export function agentName(value) {
  if(!value||!/^[A-Za-z0-9][A-Za-z0-9_-]{0,59}$/.test(value)||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i.test(value))throw new Error('Agent name must be a safe directory/branch name');
  return value;
}
async function localExcludes(root) {
  const common=path.resolve(root,git(root,'rev-parse','--git-common-dir'));
  const file=path.join(common,'info','exclude');await mkdir(path.dirname(file),{recursive:true});
  let previous='';try{previous=await readFile(file,'utf8');}catch(error){if(error.code!=='ENOENT')throw error;}
  const missing=['target/','tmp/','source_repo/'].filter(pattern=>!previous.split(/\r?\n/).includes(pattern));
  if(missing.length)await appendFile(file,`\n# Private agent build/reference paths\n${missing.join('\n')}\n`);
}
async function privateTarget(directory) {
  const target=path.join(directory,'target');
  try {
    const info=await lstat(target);
    if(info.isSymbolicLink()||path.resolve(await realpath(target))!==path.resolve(target))throw new Error(`Target is linked, not private: ${target}`);
    if(!info.isDirectory())throw new Error(`Target is not a directory: ${target}`);
  } catch(error) {if(error.code!=='ENOENT')throw error;await mkdir(target,{recursive:true});}
  return target;
}
async function copyReference(primary,directory) {
  const source=path.join(primary,'source_repo'),destination=path.join(directory,'source_repo');
  try{await lstat(source);}catch(error){if(error.code==='ENOENT')return;throw error;}
  try{const entry=await lstat(destination);if(!entry.isSymbolicLink())return;await unlink(destination);}catch(error){if(error.code!=='ENOENT')throw error;}
  const staging=path.join(directory,'tmp',`reference-copy-${process.pid}-${Date.now()}`);await mkdir(staging,{recursive:true});
  async function copy(relative) {
    const input=path.resolve(source,relative),output=path.resolve(staging,relative);
    if(!within(source,input)||!within(staging,output))throw new Error(`Invalid reference path ${relative}`);
    if(!(await lstat(input)).isFile())throw new Error(`Reference source must be a regular file: ${relative}`);
    await mkdir(path.dirname(output),{recursive:true});await copyFile(input,output);
  }
  async function tree(relative='') {
    for(const entry of await readdir(path.join(source,relative),{withFileTypes:true})) {
      if(['.git','node_modules','target','lib','out','out-esbuild','out-esbuild-test','out-test'].includes(entry.name))continue;
      const local=path.join(relative,entry.name);
      if(entry.isSymbolicLink())throw new Error(`Reference links must not be copied: ${local}`);
      if(entry.isDirectory())await tree(local);else if(entry.isFile())await copy(local);
    }
  }
  try {
    const manifest=path.join(primary,'docs','baseline','reference-manifest.json');
    let files;try{files=JSON.parse(await readFile(manifest,'utf8'));}catch(error){if(error.code!=='ENOENT')throw error;}
    if(files)for(const entry of files)await copy(entry.path);else await tree();
    await rename(staging,destination);
  } catch(error) {await rm(staging,{recursive:true,force:true});throw error;}
}
export async function createWorktree({cwd,agent,base,main,copyOwned:owned=[]}) {
  agent=agentName(agent);const repo=repository(cwd,main);base=path.resolve(base??await defaultBase(repo.primary));
  if(owned.length)await carrySelection(repo.root,owned);
  await ensureSpace(base);await ensureSpace(repo.primary);await localExcludes(repo.root);
  const branch=`swarm/${agent}`;
  const registered=worktrees(repo.root);
  const byBranch=registered.find(item=>item.branch===`refs/heads/${branch}`);
  const directory=byBranch?path.resolve(byBranch.directory):path.join(base,agent);
  const existing=registered.find(item=>path.resolve(item.directory)===directory);
  if(existing) {
    if(existing.branch!==`refs/heads/${branch}`)throw new Error(`${directory} belongs to ${existing.branch}`);
  } else if(hasRef(repo.root,`refs/heads/${branch}`))git(repo.root,'worktree','add',directory,branch);
  else git(repo.root,'worktree','add','-b',branch,directory,repo.main);
  const target=await privateTarget(directory);
  await copyReference(repo.primary,directory);
  if(owned.length)await carryChanges(repo.root,directory,owned);
  return {directory,branch,main:repo.main,target};
}
async function carrySelection(primary,paths) {
  if(!Array.isArray(paths)||paths.some(file=>typeof file!=='string'))throw new Error('Carry list must be a JSON array of relative owned paths');
  for(const file of paths) {
    const resolved=path.resolve(primary,file);
    const normalized=path.relative(primary,resolved).split(path.sep).join('/').toLowerCase();
    const parts=normalized.split('/');
    if(path.isAbsolute(file)||!within(primary,resolved)||
      parts.some(part=>['.git','target','tmp','node_modules','source_repo','auth.json','.npmrc','.netrc','.git-credentials','agents.md'].includes(part)||part==='.env'||part.startsWith('.env.'))||
      (normalized.startsWith('.pi/')&&!normalized.startsWith('.pi/swarm/skills/')))
      throw new Error(`Refusing private/reference/generated/rules/outside path ${file}`);
  }
  const selected=git(primary,'ls-files','--cached','--others','--exclude-standard','-z','--',...paths).split('\0').filter(Boolean);
  if(!selected.length)throw new Error('No owned files matched the carry list');
  for(const file of selected) {
    const parts=file.replaceAll('\\\\','/').toLowerCase().split('/');
    if(parts.some(part=>['.git','target','tmp','node_modules','source_repo','auth.json','.npmrc','.netrc','.git-credentials','agents.md'].includes(part)||part==='.env'||part.startsWith('.env.'))||
      (file.startsWith('.pi/')&&!file.startsWith('.pi/swarm/skills/')))throw new Error(`Carry selection contains a private/generated/rules file: ${file}`);
    try{if((await lstat(path.join(primary,file))).isSymbolicLink())throw new Error(`Carry regular files, not links: ${file}`);}catch(error){if(error.code!=='ENOENT')throw error;}
  }
  return selected;
}
export async function carryChanges(primary,directory,paths,{move=false}={}) {
  const selected=await carrySelection(primary,paths);
  const tracked=new Set(git(primary,'ls-files','-z').split('\0'));
  const fingerprint=async file=>{try{return createHash('sha256').update(await readFile(file)).digest('hex');}catch(error){if(error.code==='ENOENT')return null;throw error;}};
  const before=new Map();for(const file of selected)before.set(file,await fingerprint(path.join(primary,file)));
  const patch=run(process.env.GIT??'git',['diff','--binary','HEAD','--',...paths],primary,{raw:true});
  if(patch) {
    const temporary=path.join(directory,'tmp','carry.patch');await mkdir(path.dirname(temporary),{recursive:true});await writeFile(temporary,patch);
    git(directory,'apply','--binary',temporary);await rm(temporary);
  }
  // git apply may honor core.autocrlf. Restore exact source working bytes only
  // when the destination differs solely in record terminators, never content.
  for(const file of selected.filter(file=>tracked.has(file))) {
    const source=path.join(primary,file),target=path.join(directory,file);
    let bytes;try{bytes=await readFile(source);}catch(error){if(error.code==='ENOENT')continue;throw error;}
    const output=await readFile(target);
    if(!bytes.equals(output)) {
      const records=value=>value.toString('latin1').replaceAll('\r\n','\n');
      if(records(bytes)!==records(output))throw new Error(`Carry destination has other content changes: ${file}; source was not removed`);
      await writeFile(target,bytes);
    }
  }
  for(const file of selected.filter(file=>!tracked.has(file))) {
    const target=path.join(directory,file);await mkdir(path.dirname(target),{recursive:true});
    try{await lstat(target);throw new Error(`Carry would overwrite ${target}`);}catch(error){if(error.code!=='ENOENT')throw error;}
    await writeFile(target,await readFile(path.join(primary,file)));
  }
  for(const file of selected)if(await fingerprint(path.join(primary,file))!==before.get(file))throw new Error(`Primary changed during carry: ${file}; nothing was removed`);
  if(move) {
    const existing=selected.filter(file=>tracked.has(file));if(existing.length)git(primary,'restore','--source=HEAD','--staged','--worktree','--',...existing);
    for(const file of selected.filter(file=>!tracked.has(file)))await rm(path.join(primary,file));
  }
  return selected.length;
}
function cleanTracked(cwd) {
  if(git(cwd,'status','--porcelain','--untracked-files=no'))throw new Error(`${cwd} has uncommitted tracked changes; commit your own files, never stash other agents' work`);
}
export function changedPackages(metadata,files,cwd) {
  const members=metadata.packages.filter(pkg=>metadata.workspace_members.includes(pkg.id));
  if(files.some(file=>['Cargo.toml','Cargo.lock'].includes(file)||file.startsWith('.cargo/')||file.endsWith('/Cargo.toml')))return members.map(pkg=>pkg.name).sort();
  return members.filter(pkg=>{
    const prefix=path.relative(cwd,path.dirname(pkg.manifest_path)).split(path.sep).join('/');
    return files.some(file=>!prefix||file.startsWith(`${prefix}/`));
  }).map(pkg=>pkg.name).sort();
}
export function affectedPackages(metadata,packages) {
  const members=metadata.packages.filter(pkg=>metadata.workspace_members.includes(pkg.id));
  const affected=new Set(packages);let grew;
  do {grew=false;for(const pkg of members)if(!affected.has(pkg.name)&&(pkg.dependencies??[]).some(dep=>affected.has(dep.name))){affected.add(pkg.name);grew=true;}}while(grew);
  return members.filter(pkg=>affected.has(pkg.name));
}
function cleanNewFiles(cwd) {
  const files=git(cwd,'ls-files','--others','--exclude-standard','-z').split('\0').filter(Boolean)
    .filter(file=>!['target','tmp','source_repo','results'].some(name=>file===name||file.startsWith(`${name}/`)));
  if(files.length)throw new Error(`Commit new source files before merging: ${files.slice(0,8).join(', ')}`);
}
export async function deletePrivateTarget(directory) {
  const target=path.join(directory,'target');
  try {
    const info=await lstat(target);if(info.isSymbolicLink())throw new Error(`Refusing to delete linked target ${target}`);
    if(path.resolve(await realpath(target))!==path.resolve(target))throw new Error(`Target is not private: ${target}`);
    await rm(target,{recursive:true,force:true});
  } catch(error) {if(error.code!=='ENOENT')throw error;}
}
export async function mergeWorktree({cwd,main,check=run}) {
  const repo=repository(cwd,main);const directory=repo.root;const branch=git(directory,'branch','--show-current');
  if(!branch.startsWith('swarm/')||path.resolve(directory)===path.resolve(repo.primary))throw new Error('Run merge from the agent worktree, not the primary branch');
  cleanTracked(directory);cleanTracked(repo.primary);
  cleanNewFiles(directory);
  git(directory,'rebase',repo.main); // Conflicts stay with the branch owner.
  const base=git(directory,'rev-parse',repo.main);const head=git(directory,'rev-parse','HEAD');
  const files=git(directory,'diff','--name-only','--no-renames','--diff-filter=ACDMR',`${base}...${head}`).split(/\r?\n/).filter(Boolean);
  await ensureSpace(directory);
  const env={...process.env,CARGO_TARGET_DIR:path.join(directory,'target'),CARGO_BUILD_JOBS:process.env.CARGO_BUILD_JOBS??'2'};
  const cargo=process.env.CARGO??'cargo';
  const metadata=JSON.parse(check(cargo,['metadata','--locked','--no-deps','--format-version','1'],directory,{env}));
  const packages=changedPackages(metadata,files,directory);const selection=packages.flatMap(name=>['-p',name]);
  if(packages.length) {
    check(cargo,['build','--locked',...selection],directory,{inherit:true,env});
    check(cargo,['test','--locked','--all-targets',...selection],directory,{inherit:true,env});
  }
  const affected=affectedPackages(metadata,packages);
  if(affected.length)check(cargo,['check','--locked','--all-targets',...affected.flatMap(pkg=>['-p',pkg.name])],directory,{inherit:true,env});
  check(cargo,['check','--locked','--workspace'],directory,{inherit:true,env});
  const wasm=affected.filter(pkg=>pkg.name.includes('web')||pkg.name.includes('wasm')||(pkg.dependencies??[]).some(dep=>['wasm-bindgen','web-sys'].includes(dep.name)));
  if(wasm.length)check(cargo,['check','--locked','--target','wasm32-unknown-unknown','--lib',...wasm.flatMap(pkg=>['-p',pkg.name])],directory,{inherit:true,env});
  // Tests may refresh tracked result reports. The branch was clean before the
  // run, so restore only these generated outputs, never source or user edits.
  const reports=git(directory,'diff','--name-only','-z','--','results/').split('\0').filter(Boolean);
  if(reports.length)git(directory,'restore','--worktree','--',...reports);
  cleanTracked(directory);cleanNewFiles(directory);cleanTracked(repo.primary);
  if(git(directory,'rev-parse',repo.main)!==base)throw new Error('Main advanced during your parallel build; rerun merge to rebase and retest');
  if(git(directory,'rev-parse','HEAD')!==head)throw new Error('Agent branch changed during tests; rerun merge and test the new commit');
  if(git(repo.primary,'branch','--show-current')!==repo.main)throw new Error('Integration worktree switched branches during tests; merge refused');
  git(repo.primary,'merge','--ff-only',head);
  await deletePrivateTarget(directory);
  return {branch,main:repo.main,commit:head,packages};
}
