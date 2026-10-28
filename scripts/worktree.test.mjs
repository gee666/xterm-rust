import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdir,writeFile,readFile,rm,access,lstat,symlink,unlink} from 'node:fs/promises';
import path from 'node:path';
import {randomUUID} from 'node:crypto';
import {run,createWorktree,mergeWorktree,agentName,changedPackages,worktrees} from './worktree-lib.mjs';

async function fixture(t) {
  const root=path.resolve('tmp',`worktree-test-${randomUUID()}`);await mkdir(path.join(root,'src'),{recursive:true});
  await writeFile(path.join(root,'Cargo.toml'),'[package]\nname="worktree_sample"\nversion="0.1.0"\nedition="2021"\n\n[workspace]\n');
  await writeFile(path.join(root,'Cargo.lock'),'version = 4\n\n[[package]]\nname = "worktree_sample"\nversion = "0.1.0"\n');
  await writeFile(path.join(root,'.gitignore'),'target/\ntmp/\nsource_repo\n');
  await writeFile(path.join(root,'src/lib.rs'),'pub fn value()->u8 {1}\n#[test] fn check(){assert_eq!(value(),1);}\n');
  run('git',['init','-b','main'],root);run('git',['config','user.email','worktree-test@example.invalid'],root);
  run('git',['config','user.name','Worktree test'],root);run('git',['add','.'],root);run('git',['commit','-m','base'],root);
  const tree=await createWorktree({cwd:root,agent:'TestAgent',base:path.join(root,'tmp','trees')});
  t.after(async()=>{try{run('git',['worktree','remove','--force',tree.directory],root);}finally{await rm(root,{recursive:true,force:true});}});
  return {root,tree};
}
function commit(cwd,message){run('git',['add','src/lib.rs'],cwd);run('git',['commit','-m',message],cwd);}

test('creates a private branch, builds/tests changed crate, fast-forwards and deletes only its target',async t=>{
  const {root,tree}=await fixture(t);
  assert.equal(run('git',['branch','--show-current'],tree.directory),'swarm/TestAgent');
  await writeFile(path.join(tree.directory,'src/lib.rs'),'pub fn value()->u8 {2}\n#[test] fn check(){assert_eq!(value(),2);}\n');
  commit(tree.directory,'agent change');
  const merged=await mergeWorktree({cwd:tree.directory});assert.deepEqual(merged.packages,['worktree_sample']);
  assert.equal(run('git',['rev-parse','HEAD'],root),merged.commit);
  assert.match(await readFile(path.join(root,'src/lib.rs'),'utf8'),/\{2\}/);
  await assert.rejects(access(tree.target),{code:'ENOENT'});
});
test('dirty main is never stashed, overwritten or merged',async t=>{
  const {root,tree}=await fixture(t);await writeFile(path.join(root,'src/lib.rs'),'user change\n');
  await assert.rejects(mergeWorktree({cwd:tree.directory}),/uncommitted tracked changes/);
  assert.equal(await readFile(path.join(root,'src/lib.rs'),'utf8'),'user change\n');
});
test('main advancing during parallel verification requires a new rebase/test',async t=>{
  const {root,tree}=await fixture(t);await writeFile(path.join(tree.directory,'src/lib.rs'),'pub fn value()->u8 {2}\n');commit(tree.directory,'agent');
  let advanced=false;
  const check=(command,args,cwd,options)=>{
    assert.equal(options.env.CARGO_TARGET_DIR,tree.target);
    if(args[0]==='metadata')return JSON.stringify({workspace_members:['sample'],packages:[{id:'sample',name:'worktree_sample',manifest_path:path.join(cwd,'Cargo.toml')}]});
    if(!advanced){advanced=true;run('git',['commit','--allow-empty','-m','another merged agent'],root);}
    return '';
  };
  await assert.rejects(mergeWorktree({cwd:tree.directory,check}),/Main advanced/);
  assert.equal(run('git',['log','-1','--format=%s'],root),'another merged agent');
});
test('creator carries exact final whitespace and refuses recursive secrets before registering a branch',async t=>{
  const {root,tree}=await fixture(t);const tabbed='pub fn value()->u8 {2}\t';
  await writeFile(path.join(root,'src/lib.rs'),tabbed);
  await createWorktree({cwd:root,agent:'TestAgent',base:path.dirname(tree.directory),copyOwned:['src/lib.rs']});
  assert.equal(await readFile(path.join(tree.directory,'src/lib.rs'),'utf8'),tabbed);
  assert.equal(await readFile(path.join(root,'src/lib.rs'),'utf8'),tabbed);
  await writeFile(path.join(root,'src/.env'),'private');
  await assert.rejects(createWorktree({cwd:root,agent:'Rejected',base:path.dirname(tree.directory),copyOwned:['src']}),/private/);
  assert(!worktrees(root).some(item=>item.branch==='refs/heads/swarm/Rejected'));
  await assert.rejects(createWorktree({cwd:root,agent:'Missing',base:path.dirname(tree.directory),copyOwned:['missing.rs']}),/No owned files/);
  assert(!worktrees(root).some(item=>item.branch==='refs/heads/swarm/Missing'));
});
test('carry preserves exact LF bytes even when Git autocrlf would convert the patch',async t=>{
  const {root,tree}=await fixture(t);
  run('git',['config','core.autocrlf','true'],root);
  const bytes=Buffer.from('pub fn value()->u8 {2}\n#[test] fn check(){assert_eq!(value(),2);}\n\n');
  await writeFile(path.join(root,'src/lib.rs'),bytes);
  await createWorktree({cwd:root,agent:'TestAgent',base:path.dirname(tree.directory),copyOwned:['src/lib.rs']});
  assert.deepEqual(await readFile(path.join(tree.directory,'src/lib.rs')),bytes);
  assert.deepEqual(await readFile(path.join(root,'src/lib.rs')),bytes);
});
test('reference is a physical private copy and Git removal cannot delete the original',async t=>{
  const {root,tree}=await fixture(t);await mkdir(path.join(root,'source_repo'));
  await writeFile(path.join(root,'source_repo','keep.txt'),'original');
  const other=await createWorktree({cwd:root,agent:'Reference',base:path.dirname(tree.directory)});
  try {
    assert(!(await lstat(path.join(other.directory,'source_repo'))).isSymbolicLink());
    await writeFile(path.join(other.directory,'source_repo','keep.txt'),'changed copy');
    assert.equal(await readFile(path.join(root,'source_repo','keep.txt'),'utf8'),'original');
  } finally {run('git',['worktree','remove','--force',other.directory],root);}
  assert.equal(await readFile(path.join(root,'source_repo','keep.txt'),'utf8'),'original');
});
test('creator rejects a linked target without deleting or following it',async t=>{
  const {root,tree}=await fixture(t);const outside=path.join(root,'outside-target');
  await mkdir(outside);await writeFile(path.join(outside,'keep.txt'),'keep');
  await rm(tree.target,{recursive:true,force:true});await symlink(outside,tree.target,process.platform==='win32'?'junction':'dir');
  try {await assert.rejects(createWorktree({cwd:root,agent:'TestAgent',base:path.dirname(tree.directory)}),/not private/);}
  finally {await unlink(tree.target);}
  assert.equal(await readFile(path.join(outside,'keep.txt'),'utf8'),'keep');
});

test('names and package selection reject unsafe paths and include manifest-wide changes',()=>{
  for(const name of ['../other','a/b','CON',''])assert.throws(()=>agentName(name));
  const cwd=path.resolve('tmp/example');const metadata={workspace_members:['a','b'],packages:[
    {id:'a',name:'core',manifest_path:path.join(cwd,'crates/core/Cargo.toml')},
    {id:'b',name:'web',manifest_path:path.join(cwd,'crates/web/Cargo.toml')}]};
  assert.deepEqual(changedPackages(metadata,['crates/core/src/lib.rs'],cwd),['core']);
  assert.deepEqual(changedPackages(metadata,['Cargo.lock'],cwd),['core','web']);
});
