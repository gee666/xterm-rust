// A runtime mismatch is required. Compiler failures are not a negative control.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {runCommand} from './quality-process.mjs';
const file='crates/xterm-addons/src/ligatures/font_cff2_dict.rs',original=await fs.readFile(file),hash=b=>createHash('sha256').update(b).digest('hex');
const from='if !dictionary(bytes)? {',to='if !dictionary(bytes)? && std::hint::black_box(false) {';
if(original.toString().split(from).length!==2)throw Error('Unique FDArray presence guard required');
const record={originalSha256:hash(original),status:'running'};
await fs.mkdir('results/font-cff2-fdarray',{recursive:true});
async function run(name) {
  const log=`results/font-cff2-fdarray/${name}.log`;
  const result=await runCommand(process.env.CARGO??'cargo',['test','-j2','-p','xterm-addons','--test','font_cff2_fdarray','source_fdarray_guard_corpus','--','--exact'],{cwd:process.cwd(),env:process.env,log,timeoutMs:180000});
  const output=await fs.readFile(log,'utf8');
  if(result.error||result.timedOut||result.signal||result.terminationError)throw Error(JSON.stringify(result));
  return {result,output};
}
try {
  await fs.writeFile(file,original.toString().replace(from,to));
  const {result,output}=await run('negative');
  if(result.status==='passed'||!output.includes('running 1 test')||!output.includes('test source_fdarray_guard_corpus ... FAILED'))throw Error('Negative control did not execute the guard bypass');
  record.mismatches=(output.match(/expected Some\(Invalid\("This is a CFF2 font, but FDArray information is missing"\)\), got None/g)??[]).length;
  if(record.mismatches!==188)throw Error(`Expected188 executed missing-FDArray mismatches, got${record.mismatches}`);
  record.negative=result;
} finally {
  await fs.writeFile(file,original);
  record.restoredSha256=hash(await fs.readFile(file));
  if(record.restoredSha256!==record.originalSha256)throw Error('Source restoration failed');
  await fs.writeFile('results/font-cff2-fdarray/negative.json',JSON.stringify(record,null,2)+'\n');
}
const positive=await run('restored');
if(positive.result.status!=='passed')throw Error('Restored positive test failed');
record.status='passed';record.positive=positive.result;
await fs.writeFile('results/font-cff2-fdarray/negative.json',JSON.stringify(record,null,2)+'\n');
console.log(JSON.stringify(record));
