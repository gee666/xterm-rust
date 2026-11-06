// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Reuse the existing browser runner; finally restores exact production bytes.
import {readFile,writeFile,copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {runCommand} from './quality-process.mjs';
const directory='results/canvas-layer',phase=process.argv.includes('--phase');
const names=phase?['canvas_layer','canvas_layer_text']:['link_layer'];
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const files=[];
for(const name of names){
 const source=`crates/xterm-web/src/webgl/${name}.rs`,original=await readFile(source);
 let mutant;
 if(phase)mutant=await readFile(`${directory}/before-phase-fix-${name}.rs.txt`);
 else{
  const text=original.toString('utf8'),before='if middle > 0.0 {';
  assert.equal(text.split(before).length,2);
  mutant=Buffer.from(text.replace(before,'if middle < 0.0 {'));
  await writeFile(directory+'/negative-mutant.rs.txt',mutant);
 }
 assert.notEqual(hash(original),hash(mutant));files.push({source,original,mutant});
}
const proof={mutation:phase?'Restore the two pre-fix DPR/theme and text/dimension snapshots.':'Clear middle rows only for a negative span, suppressing actual multiline clearing.',
 files:files.map(f=>({path:f.source,originalSha256:hash(f.original),mutantSha256:hash(f.mutant)})),restored:false,expectedFailureObserved:false};
if(!phase)Object.assign(proof,{originalSha256:proof.files[0].originalSha256,mutantSha256:proof.files[0].mutantSha256});
const prefix=phase?'phase-negative':'negative';
try{
 for(const file of files)await writeFile(file.source,file.mutant);
 const record=await runCommand(process.execPath,['scripts/browser-host-check.mjs'],{cwd:process.cwd(),env:{...process.env,BROWSERS:'chromium'},timeoutMs:240000,log:directory+'/'+prefix+'.log'});
 const reportPath='results/browser-host/wasm_webgl_canvas_layer-results.json';
 const report=JSON.parse(await readFile(reportPath,'utf8'));
 await copyFile(reportPath,directory+'/browser-'+prefix+'.json');
 assert.equal(record.status,'failed');assert.equal(report.status,'failed');
 const browser=report.browsers[0];assert.equal(browser.name,'chromium');
 assert.equal(browser.counts.ignored,0);assert.equal(browser.counts.filtered,0);
 assert.ok(browser.counts.failed>0,'actual Rust assertions must reject production mutation');
 if(phase){assert.equal(browser.counts.passed,10);assert.equal(browser.counts.failed,2);}
 proof.counts=browser.counts;proof.expectedFailureObserved=true;
}finally{
 for(const [index,file] of files.entries()){
  await writeFile(file.source,file.original);
  proof.files[index].restoredSha256=hash(await readFile(file.source));
 }
 proof.restored=proof.files.every(f=>f.originalSha256===f.restoredSha256);
 if(!phase)proof.restoredSha256=proof.files[0].restoredSha256;
 await writeFile(directory+'/'+prefix+'.json',JSON.stringify(proof,null,2)+'\n');
}
assert.equal(proof.restored,true);
console.log('Production '+prefix+' rejected; exact original bytes restored');
