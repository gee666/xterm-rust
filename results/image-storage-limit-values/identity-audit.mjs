import {readFile,writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {createHash} from 'node:crypto';
const hash=b=>createHash('sha256').update(b).digest('hex');
const out='results/image-storage-limit-values';
const final=JSON.parse(await readFile(`${out}/final-verification.json`,'utf8'));
const audit={rust:[],original:[],control:[]};
const original='P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo';
for(const pin of final.rustInputs){
  const actual=hash(await readFile(pin.path));
  if(actual!==pin.sha256)throw Error(`Rust pin mismatch ${pin.path}`);
  audit.rust.push({path:pin.path,sha256:actual});
}
for(const pin of final.sourceInputs){
  const local=path.resolve(pin.path);const relative=path.relative(path.resolve('source_repo'),local);
  if(relative.startsWith('..')||path.isAbsolute(relative))continue;
  const actual=hash(await readFile(local)),reference=hash(await readFile(path.join(original,relative)));
  if(actual!==pin.sha256||actual!==reference)throw Error(`Original mismatch ${pin.path}`);
  audit.original.push({path:relative.split(path.sep).join('/'),sha256:actual});
}
const negative=JSON.parse(await readFile(`${out}/negative-audit.json`,'utf8'));
for(const pin of negative.files){
  const baseline=spawnSync('git',['show',`${negative.base}:${pin.file}`],{timeout:30000,maxBuffer:1024*1024});
  if(baseline.error||baseline.status!==0||hash(baseline.stdout)!==pin.baselineSha256)throw Error(`Git control baseline mismatch ${pin.file}`);
  const actual=hash(await readFile(pin.file));if(actual!==pin.fixedSha256||actual!==pin.restoredSha256)throw Error(`Fixed restore mismatch ${pin.file}`);
  audit.control.push({...pin,actualSha256:actual});
}
const report=JSON.parse(await readFile(`${out}/negative-browser.json`,'utf8'));
audit.negative=report.browsers.map(({name,counts,tests})=>({name,counts,failedTests:tests.filter(t=>t.status==='failed')}));
audit.counts={rust:audit.rust.length,original:audit.original.length,control:audit.control.length};
await writeFile(`${out}/identity-audit.json`,JSON.stringify(audit,null,2)+'\n');
console.log(audit.counts);
