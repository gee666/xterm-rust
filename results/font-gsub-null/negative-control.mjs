// Run only in the owned worktree with a compiler allocation. Restore exact bytes.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const prefix='crates/xterm-addons/src/ligatures/';
const files=['font.rs','font_build.rs','font_build_helpers.rs','font_gsub.rs','font_tables.rs'].map(name=>prefix+name);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const originals=new Map(await Promise.all(files.map(async file=>[file,await fs.readFile(file)])));
const receipt={baseline:'58921f10',before:Object.fromEntries([...originals].map(([file,bytes])=>[file,hash(bytes)]))};
const cargo=process.env.CARGO??'cargo',git=process.env.GIT??'git';
const args=['test','--locked','-j2','-p','xterm-addons','--test','font_gsub_null'];
function test(log){
 const result=spawnSync(cargo,args,{encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024});
 if(result.error)throw result.error;
 return fs.writeFile(`results/font-gsub-null/${log}`,result.stdout+result.stderr).then(()=>result);
}
try{
 for(const file of files){
  const result=spawnSync(git,['show',`${receipt.baseline}:${file}`],{encoding:null,timeout:30000});
  if(result.error)throw result.error;assert.equal(result.status,0);await fs.writeFile(file,result.stdout);
 }
 const old=await test('negative-old.log');assert.equal(old.status,101);
 assert.match(old.stdout,/272 mismatches:/);receipt.oldMismatches=272;
}finally{
 for(const [file,bytes]of originals)await fs.writeFile(file,bytes);
 receipt.restored=Object.fromEntries(await Promise.all(files.map(async file=>[file,hash(await fs.readFile(file))])));
 assert.deepEqual(receipt.restored,receipt.before);
 await fs.writeFile('results/font-gsub-null/negative-control.json',JSON.stringify(receipt,null,2)+'\n');
}
const fixed=await test('negative-restored.log');assert.equal(fixed.status,0);
const target='target/debug/deps';
const exes=(await fs.readdir(target)).filter(name=>/^font_gsub_null-.*\.exe$/.test(name));assert.equal(exes.length,1);
const executable=path.join(target,exes[0]),bytes=await fs.readFile(executable);
await fs.writeFile('results/font-gsub-null/native-artifact.json',JSON.stringify({path:executable,sha256:hash(bytes),bytes:bytes.length,args},null,2)+'\n');
console.log('Old product: 272 mismatches. Five exact source hashes restored. Fixed test: PASS.');
