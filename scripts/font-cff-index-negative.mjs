// Deliberately bypass only the new INDEX phase, then restore exact source bytes.
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const file='crates/xterm-addons/src/ligatures/font_cff_header.rs',before=await fs.readFile(file),hash=b=>createHash('sha256').update(b).digest('hex');
let mutant=before.toString();
for(const hook of ['index::compressed(input, limit, &prefix[..size])','index::validate(reader, offset)']) {
  if(mutant.split(hook).length!==2)throw new Error(`Expected one hook: ${hook}`);
  mutant=mutant.replace(hook,'Ok(())');
}
const run=()=>spawnSync(process.env.CARGO??'cargo',['test','--locked','-j2','-p','xterm-addons','--test','font_cff_index','source_constructor_index_corpus_matches_native'],{encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024});
const report={originalSha256:hash(before),status:'running'};
await fs.mkdir('results/font-cff-index',{recursive:true});
try {
  await fs.writeFile(file,mutant);
  const result=run(),log=(result.stdout??'')+(result.stderr??'');
  await fs.writeFile('results/font-cff-index/negative-index.log',log);
  if(result.error||result.signal||result.status===0)throw new Error(`Invalid negative control ${JSON.stringify({error:result.error?.message,status:result.status,signal:result.signal})}`);
  report.mismatches=(log.match(/expected Some\(Bounds\), got None/g)??[]).length;
  if(report.mismatches!==420)throw new Error(`Expected420 INDEX mismatches, got${report.mismatches}`);
  report.status='passed';
} finally {
  await fs.writeFile(file,before);
  report.restoredSha256=hash(await fs.readFile(file));
  if(report.restoredSha256!==report.originalSha256)throw new Error('Source restoration mismatch');
  await fs.writeFile('results/font-cff-index/negative.json',JSON.stringify(report,null,2)+'\n');
}
const restored=run();await fs.writeFile('results/font-cff-index/restored-index.log',(restored.stdout??'')+(restored.stderr??''));
if(restored.error||restored.status!==0)throw new Error('Restored INDEX test failed');
console.log(JSON.stringify(report));
