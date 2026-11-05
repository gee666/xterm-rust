// Bounded diagnostic using the already-built, hash-verified Rust test artifacts.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {withArtifactServer} from '../../scripts/browser-artifact-server.mjs';
const name=process.argv[2];
const timeout=name==='firefox'?420000:120000;
assert.ok(['chromium','firefox','webkit'].includes(name));
const original=JSON.parse(await fs.readFile('results/font-gsub-subtables/initial-rust-browser.json','utf8'));
for(const record of Object.values(original.artifacts).filter(r=>r&&typeof r==='object'&&r.path)){
 const bytes=await fs.readFile(record.path);assert.equal(bytes.length,record.bytes);assert.equal(createHash('sha256').update(bytes).digest('hex'),record.sha256);
}
const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE));
const record={name,status:'starting',errors:[],console:[],checkpoints:[],shutdown:'close owned client, then owned BrowserServer.close; forced kill only on failure'};
const output=`results/font-gsub-subtables/replay-${name}.json`;
async function checkpoint(stage){record.checkpoints.push({stage,time:new Date().toISOString()});await fs.writeFile(output,JSON.stringify(record,null,2)+'\n');console.log(name,stage);}
await checkpoint('start');
const css=await fs.readFile('crates/xterm-web/assets/xterm.css');
let server;
try{
 await withArtifactServer({directory:path.dirname(original.artifacts.bindings.path),css,html:'<!doctype html><pre id="output"></pre>'},async url=>{
  server=await playwright[name].launchServer({headless:true,timeout:30000});record.processId=server.process().pid;
  const browser=await playwright[name].connect(server.wsEndpoint(),{timeout:30000});record.version=browser.version();
  const page=await browser.newPage();page.setDefaultTimeout(timeout);
  page.on('pageerror',error=>record.errors.push(String(error.stack??error)));page.on('console',event=>record.console.push({type:event.type(),text:event.text()}));
  await page.goto(url,{timeout:30000});await checkpoint('page loaded');
  await page.evaluate(()=>{
   globalThis.__wbg_test_invoke=callback=>callback();
   globalThis.testRun=(async()=>{
    const bindings=await import('/suite.js');globalThis.stage='bindings loaded';
    const exports=await bindings.default();globalThis.stage='wasm loaded';
    const names=Object.keys(exports).filter(name=>name.startsWith('__wbgt_')).sort();globalThis.testNames=names;
    if(!names.length)throw Error('No Rust assertions');
    const context=new bindings.WasmBindgenTestContext();
    try{return{names,passed:await context.run(names.map(name=>exports[name])),output:document.querySelector('#output').textContent};}
    finally{context.free();}
   })();
   globalThis.testRun.then(value=>{globalThis.testResult=value;},error=>{globalThis.testResult={error:String(error.stack??error),stage:globalThis.stage};});
  });await checkpoint('test scheduled');
  await page.waitForFunction(()=>globalThis.testResult!==undefined,{},{timeout});
  const result=await page.evaluate(()=>globalThis.testResult);record.tests=result.names??[];record.output=result.output;
  if(result.error)record.errors.push(result.error);
  const summary=result.output?.match(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) filtered out/);
  if(summary)record.counts=Object.fromEntries(['passed','failed','ignored','filtered'].map((field,i)=>[field,Number(summary[i+1])]));
  assert.deepEqual(record.counts,{passed:2,failed:0,ignored:0,filtered:0});assert.equal(result.passed,true);assert.deepEqual(record.errors,[]);
  record.status='passed';await fs.writeFile(`results/font-gsub-subtables/font_gsub_subtables_browser-${name}.log`,result.output);await checkpoint('assertions passed');
  // Disconnect the remote client before closing its owned server. Closing
  // the server with that client still attached stalled the earlier diagnostic.
  await browser.close();await server.close();server=undefined;await checkpoint('owned browser stopped');
 });
}catch(error){record.status='failed';record.errors.push(String(error.stack??error));process.exitCode=1;}
finally{if(server)await server.kill();await checkpoint('finished');}
