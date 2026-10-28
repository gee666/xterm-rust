import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm,symlink} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {snapshot,snapshotPaths} from './quality-snapshot.mjs';
const root=fileURLToPath(new URL('../',import.meta.url));
async function fixture(run){
    const base=path.join(root,'tmp/quality-snapshot-scoped');await mkdir(base,{recursive:true});
    const dir=await mkdtemp(path.join(base,'run-'));
    try{await run(dir);}finally{await rm(dir,{recursive:true,force:true});}
}
test('scoped snapshots include exact fixture bytes and deduplicate overlapping inputs',()=>fixture(async root=>{
    await mkdir(path.join(root,'src'));await writeFile(path.join(root,'src/a.rs'),'// unsafe candidate\n');await writeFile(path.join(root,'src/table.txt'),'table-one');
    const first=await snapshotPaths(root,['src','src/a.rs']);assert.equal(first.files,2);assert.equal(first.unsafeCandidates.length,1);
    assert.deepEqual(first.scope,['src','src/a.rs']);
    const again=await snapshotPaths(root,['src/a.rs','src']);assert.equal(first.sha256,again.sha256);
    await writeFile(path.join(root,'src/table.txt'),'table-two');assert.notEqual(first.sha256,(await snapshotPaths(root,['src'])).sha256);
}));
test('unselected paths do not become implied evidence',()=>fixture(async root=>{
    await writeFile(path.join(root,'chosen.txt'),'chosen');await writeFile(path.join(root,'other.txt'),'one');
    const before=await snapshotPaths(root,['chosen.txt']);await writeFile(path.join(root,'other.txt'),'two');
    assert.equal(before.sha256,(await snapshotPaths(root,['chosen.txt'])).sha256);assert.deepEqual(before.scope,['chosen.txt']);
}));
test('missing, absolute and noncanonical input paths fail closed',()=>fixture(async root=>{
    await assert.rejects(snapshotPaths(root,[]),/required/);
    for(const name of ['','../out','a/../out','./file','a//b','a\\b','C:stream','/absolute'])await assert.rejects(snapshotPaths(root,[name]),/Invalid/);
    await assert.rejects(snapshotPaths(root,['missing']),/ENOENT/);
}));
test('linked selected components and linked descendants are rejected',()=>fixture(async root=>{
    await mkdir(path.join(root,'owned'));await writeFile(path.join(root,'owned/file.rs'),'data');
    await symlink(path.join(root,'owned'),path.join(root,'link'),process.platform==='win32'?'junction':'dir');
    await assert.rejects(snapshotPaths(root,['link/file.rs']),/Linked/);
    await mkdir(path.join(root,'container'));await symlink(path.join(root,'owned'),path.join(root,'container/link'),process.platform==='win32'?'junction':'dir');
    await assert.rejects(snapshotPaths(root,['container']),/symlinked/);
}));
test('default workspace snapshot format and hash remain unchanged',()=>fixture(async root=>{
    await writeFile(path.join(root,'Cargo.toml'),'manifest');await writeFile(path.join(root,'Cargo.lock'),'lock');await mkdir(path.join(root,'crates'));await writeFile(path.join(root,'crates/a.rs'),'fn main() {}');
    const expected=createHash('sha256').update('Cargo.lock\0lock\0Cargo.toml\0manifest\0crates/a.rs\0fn main() {}\0').digest('hex');
    assert.deepEqual(await snapshot(root),{sha256:expected,files:3,unsafeCandidates:[]});
}));
