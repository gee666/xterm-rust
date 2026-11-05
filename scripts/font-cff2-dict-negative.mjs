// A bounded owned-source mutation proves the new constructor checks are live.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const file='crates/xterm-addons/src/ligatures/font_cff_index.rs',original=await fs.readFile(file),hash=b=>createHash('sha256').update(b).digest('hex');
const from='dict::validate(&data.bytes(offset, length)?)?;',to='let _ = data.bytes(offset, length)?;';
if(original.toString().split(from).length!==2)throw Error('Unique operand hook required');
const record={originalSha256:hash(original),status:'running'};
await fs.mkdir('results/font-cff2-dict',{recursive:true});
async function run(name) {
  const result=spawnSync(process.env.CARGO??'cargo',['test','-j2','-p','xterm-addons','--test','font_cff2_dict','full_source_constructor_operand_errors_and_ordering','--','--exact'],{encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,env:process.env});
  await fs.writeFile(`results/font-cff2-dict/${name}.log`,`${result.stdout??''}${result.stderr??''}`);
  if(result.error||result.signal)throw result.error??Error(result.signal);
  return result;
}
try {
  await fs.writeFile(file,original.toString().replace(from,to));
  const negative=await run('negative');
  const output=`${negative.stdout??''}${negative.stderr??''}`;
  if(negative.status===0||!output.includes('expected Some(CffOperand(31)), got None')||!output.includes('expected Some(CffOperand(255)), got None')||!output.includes('expected Some(Bounds), got None'))throw Error('Negative control failed to detect lexical bypass');
  record.mismatches=(output.match(/expected Some\(/g)??[]).length;
  record.negativeExitCode=negative.status;
} finally {
  await fs.writeFile(file,original);
  record.restoredSha256=hash(await fs.readFile(file));
  if(record.restoredSha256!==record.originalSha256)throw Error('Source restoration failed');
  await fs.writeFile('results/font-cff2-dict/negative.json',JSON.stringify(record,null,2)+'\n');
}
const positive=await run('restored');
if(positive.status!==0)throw Error('Restored positive test failed');
record.status='passed';record.positiveExitCode=positive.status;
await fs.writeFile('results/font-cff2-dict/negative.json',JSON.stringify(record,null,2)+'\n');
console.log(JSON.stringify(record));
