import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const files=['font.rs','font_build_helpers.rs','font_gsub.rs','font_tables.rs'].map(f=>'crates/xterm-addons/src/ligatures/'+f);
const saved=await Promise.all(files.map(f=>fs.readFile(f)));
const hash=b=>createHash('sha256').update(b).digest('hex');
try{
 for(const file of files){
  const p=spawnSync('git',['show','86996eb4:'+file],{timeout:30000,maxBuffer:1048576});
  assert.equal(p.status,0,p.stderr.toString());await fs.writeFile(file,p.stdout);
 }
 const p=spawnSync('cargo',['test','--locked','-j2','-p','xterm-addons','--test','font_gsub_type9'],{timeout:180000,encoding:'utf8',maxBuffer:10485760});
 await fs.writeFile('results/font-gsub-type9/native-before.log',p.stdout+p.stderr);
 assert.equal(p.error,undefined);assert.notEqual(p.status,0);
 assert.match(p.stdout,/mismatches:/);
}finally{
 for(const [i,file]of files.entries()){await fs.writeFile(file,saved[i]);assert.equal(hash(await fs.readFile(file)),hash(saved[i]));}
}
const p=spawnSync('cargo',['test','--locked','-j2','-p','xterm-addons','--test','font_gsub_type9'],{timeout:180000,encoding:'utf8',maxBuffer:10485760});
await fs.writeFile('results/font-gsub-type9/native-restored.log',p.stdout+p.stderr);
assert.equal(p.error,undefined);assert.equal(p.status,0,p.stdout+p.stderr);
console.log('Old product failed; exact restoration and final corpus passed');
