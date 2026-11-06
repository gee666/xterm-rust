// Bypass only the FDArray INDEX call, require executed mismatches, restore exactly.
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const file='crates/xterm-addons/src/ligatures/font_cff_index.rs',before=await fs.readFile(file),text=before.toString(),needle='fd_array.validate(&data, start)?;';
assert.equal(text.split(needle).length,2);
const sha=b=>createHash('sha256').update(b).digest('hex'),out='results/font-cff2-index';
const report={beforeSha256:sha(before),status:'running'};
const run=name=>{
  const args=['test','-j2','-p','xterm-addons','--test','font_cff2_index','source_complete_fdarray_index_rejections'];
  const r=spawnSync(process.env.CARGO??'cargo',args,{encoding:'utf8',timeout:900000,maxBuffer:16*1024*1024,windowsHide:true});
  const raw=(r.stdout??'')+(r.stderr??'');return fs.writeFile(out+'/'+name+'.log',raw).then(()=>({r,raw,args}));
};
await fs.mkdir(out,{recursive:true});
try {
  await fs.writeFile(file,text.replace(needle,'let _ = fd_array;'));
  report.mutantSha256=sha(await fs.readFile(file));
  const {r,raw,args}=await run('negative');report.command=['cargo',...args];report.exitCode=r.status;
  assert.notEqual(r.status,0);assert.match(raw,/test result: FAILED\. 0 passed; 1 failed/);
  const cases=[...raw.matchAll(/"index(\d+)" expected Some\(Bounds\), got (None|Some\([^\n]+\))/g)];
  report.executedBoundsMismatches=cases.length;report.caseNames=cases.map(m=>'index'+m[1]);
  report.accepted=cases.filter(m=>m[2]==='None').length;
  report.laterHmtx=cases.filter(m=>m[2]!=='None').length;
  assert.deepEqual([cases.length,report.accepted,report.laterHmtx],[147,119,28],'Negative must execute exact147 relevant Bounds mismatches, not a compile failure');
  report.status='passed';
} catch(e) {report.status='failed';report.error=String(e.stack??e);throw e;}
finally {
  await fs.writeFile(file,before);report.restoredSha256=sha(await fs.readFile(file));assert.equal(report.restoredSha256,report.beforeSha256);
  const {r}=await run('restored');report.positiveExitCode=r.status;
  if(r.status!==0)report.status='failed';
  await fs.writeFile(out+'/negative.json',JSON.stringify(report,null,2)+'\n');
  assert.equal(r.status,0,'Positive replay after exact restoration must pass');
}
console.log(JSON.stringify(report));
