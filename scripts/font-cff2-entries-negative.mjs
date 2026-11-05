// Reject compile-only failures and prove the new duplicate path executes.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const file='crates/xterm-addons/src/ligatures/font_cff2_dict.rs',original=await fs.readFile(file),hash=b=>createHash('sha256').update(b).digest('hex');
const from='duplicate.map_or(Ok(()), |key| Err(FontError::CffDuplicate(key)))',to='Ok(())';
if(original.toString().split(from).length!==2)throw Error('Unique duplicate result required');
const record={originalSha256:hash(original),status:'running'};
await fs.mkdir('results/font-cff2-entries',{recursive:true});
async function run(name) {
  const result=spawnSync(process.env.CARGO??'cargo',['test','-j2','-p','xterm-addons','--test','font_cff2_entries','directed_duplicate_regression','--','--exact'],{encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,env:process.env});
  await fs.writeFile(`results/font-cff2-entries/${name}.log`,`${result.stdout??''}${result.stderr??''}`);
  if(result.error||result.signal)throw result.error??Error(result.signal);
  return result;
}
try {
  await fs.writeFile(file,original.toString().replace(from,to));
  const negative=await run('negative'),output=`${negative.stdout??''}${negative.stderr??''}`;
  if(negative.status===0||!output.includes('running 1 test')||!output.includes('test directed_duplicate_regression ... FAILED')||!output.includes('expected Some(CffDuplicate(0)), got None'))throw Error('Negative control did not execute the duplicate bypass');
  record.mismatches=(output.match(/expected Some\(CffDuplicate\(/g)??[]).length;
  if(record.mismatches!==16)throw Error(`Expected16 duplicate mismatches, got${record.mismatches}`);
  record.negativeExitCode=negative.status;
} finally {
  await fs.writeFile(file,original);
  record.restoredSha256=hash(await fs.readFile(file));
  if(record.restoredSha256!==record.originalSha256)throw Error('Source restoration failed');
  await fs.writeFile('results/font-cff2-entries/negative.json',JSON.stringify(record,null,2)+'\n');
}
const positive=await run('restored');
if(positive.status!==0)throw Error('Restored positive test failed');
record.status='passed';record.positiveExitCode=positive.status;
await fs.writeFile('results/font-cff2-entries/negative.json',JSON.stringify(record,null,2)+'\n');
console.log(JSON.stringify(record));
