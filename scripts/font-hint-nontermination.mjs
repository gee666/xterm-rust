// Bounded unchanged-source safety reproduction. This is not an equivalence pass.
import fs from 'node:fs/promises';
import path from 'node:path';
import {Worker} from 'node:worker_threads';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {buildHintFont} from '../crates/xterm-font-addons/tests/support/font_hint_tables.mjs';
const packageFile = process.env.SOURCE_PACKAGE_JSON ?? path.resolve('source_repo/package.json');
const require = createRequire(packageFile), dependency = require.resolve('opentype.js');
const fixture = JSON.parse(await fs.readFile('crates/xterm-addons/tests/fixtures/font-hint-tables.json','utf8'));
const base = new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf'));
const input = buildHintFont(base,fixture.nontermination);
const hash = b => createHash('sha256').update(b).digest('hex');
assert.equal(hash(input),fixture.nontermination.inputSha256);
assert.equal(hash(await fs.readFile(dependency)),fixture.sourceSha256);
const deadlineMs = 2000;
const worker = new Worker(`const {parentPort,workerData}=require('node:worker_threads');
const {parse}=require(workerData.dependency);
parentPort.postMessage({status:'parsing'});
try{parse(Uint8Array.from(workerData.input).buffer);parentPort.postMessage({status:'returned'});}
catch(error){parentPort.postMessage({status:'threw',name:error.name,message:error.message});}`,
  {eval:true,workerData:{dependency,input:[...input]}});
let startup, deadline, report;
try {
  report = await new Promise((resolve,reject) => {
    startup = setTimeout(()=>reject(new Error('Source worker did not reach parse')),10000);
    worker.on('error',reject);
    worker.on('exit',code=>{if(!report)reject(new Error('Source worker exited before observation: '+code));});
    worker.on('message',message=>{
      if(message.status==='parsing') {
        clearTimeout(startup);
        deadline = setTimeout(()=>resolve({status:'source-nontermination-observed',deadlineMs}),deadlineMs);
      } else resolve(message);
    });
  });
} finally {
  clearTimeout(startup); clearTimeout(deadline);
  await worker.terminate();
}
await fs.writeFile('results/font-hint-tables/source-nontermination.json',JSON.stringify({
  node:process.version,inputSha256:hash(input),sourceSha256:fixture.sourceSha256,
  case:fixture.nontermination,...report,
  limitation:'A bounded timeout observation is not a mathematical proof of nontermination; the source loop is also inspected. Rust returns Compression instead.'
},null,2)+'\n');
assert.equal(report.status,'source-nontermination-observed');
console.log('Separate source nontermination observed after parsing began; worker terminated.');
