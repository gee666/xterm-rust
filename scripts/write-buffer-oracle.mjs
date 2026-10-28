// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root=resolve(import.meta.dirname,'..');
const temp=resolve(root,'tmp/write-buffer-oracle'); mkdirSync(temp,{recursive:true});
const sourceHashes={};
function read(path) {
  const text=readFileSync(resolve(root,path),'utf8'); sourceHashes[path]=createHash('sha256').update(text).digest('hex'); return text;
}
function emit(name,source) {
  const path=resolve(temp,name+'.mjs'); writeFileSync(path,stripTypeScriptTypes(source,{mode:'transform'})); return pathToFileURL(path).href;
}
const base='source_repo/src/common/';
const lifecycle=emit('lifecycle',read(base+'Lifecycle.ts'));
const event=emit('event',read(base+'Event.ts').replace('IDisposable, ', '').replace("'./Lifecycle'",JSON.stringify(lifecycle)));
const async=emit('async',read(base+'Async.ts').replace(', IDisposable', '').replace("'./Lifecycle'",JSON.stringify(lifecycle)));
const buffer=emit('write-buffer',read(base+'input/WriteBuffer.ts')
 .replace("'../Lifecycle'",JSON.stringify(lifecycle)).replace("'../Event'",JSON.stringify(event)).replace("'../Async'",JSON.stringify(async)));
const tests=[]; const suites=[]; const hooks=[];
globalThis.describe=(name,callback)=>{suites.push(name); hooks.push([]); try{callback();}finally{suites.pop(); hooks.pop();}};
globalThis.beforeEach=callback=>hooks.at(-1).push(callback);
globalThis.it=(name,callback)=>tests.push({name:[...suites,name].join(' / '),callback,hooks:hooks.flat()});
await import(emit('upstream-tests',read(base+'input/WriteBuffer.test.ts').replace("'./WriteBuffer'",JSON.stringify(buffer))
 .replace("import { assert } from 'chai';","import assert from 'node:assert/strict';")));
delete globalThis.describe; delete globalThis.beforeEach; delete globalThis.it;
const outcomes=[];
for(const test of tests) {
  let timer,listener;
  try {
    for(const hook of test.hooks) hook();
    await new Promise((resolve,reject)=>{
      timer=setTimeout(()=>reject(new Error('original test exceeded 2000ms deadline')),2000);
      listener=error=>reject(error); process.once('uncaughtException',listener);
      const done=error=>error?reject(error):resolve();
      try { if(test.callback.length) test.callback(done); else {test.callback(); resolve();} } catch(error){reject(error);}
    });
    outcomes.push({name:test.name,status:'passed'});
  } catch(error) { outcomes.push({name:test.name,status:'failed',error:String(error.stack)}); }
  finally {clearTimeout(timer); process.removeListener('uncaughtException',listener);}
}
const {WriteBuffer}=await import(buffer);
const rejectionOrder=[];
{
 const originalError=new Error('retained rejection'); let first=true;
 const onError=error=>{assert.equal(error,originalError);rejectionOrder.push('error');};
 process.once('uncaughtException',onError);
 try {
  await new Promise(resolve=>{
   const wb=new WriteBuffer((data,result)=>{rejectionOrder.push('action:'+result);if(first){first=false;return Promise.reject(originalError);}});
   wb.onWriteParsed(()=>rejectionOrder.push('parsed'));
   wb.write('a',()=>{rejectionOrder.push('callback');resolve();});
  });
 } finally {process.removeListener('uncaughtException',onError);}
 assert.deepEqual(rejectionOrder,['action:true','error','action:false','callback','parsed']);
}
const originalTimeout=globalThis.setTimeout,originalClear=globalThis.clearTimeout;
const performanceDescriptor=Object.getOwnPropertyDescriptor(globalThis,'performance');
let now=100; let next=0; const timers=new Map();
globalThis.setTimeout=callback=>{const id=++next;timers.set(id,callback);return id;};
globalThis.clearTimeout=id=>timers.delete(id);
Object.defineProperty(globalThis,'performance',{configurable:true,value:{now:()=>now}});
let state=0x5717eb0f;
function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
function units(s){return Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16)).join(',')||'-';}
const rows=[]; const operations=['write','sync','flush','input','tick']; const counts={};
const externalViews={};
function tick(){const first=timers.entries().next().value;if(first){timers.delete(first[0]);first[1]();}}
try {
 for(let trace=0;trace<128;trace++) {
  timers.clear(); now=100; const log=[]; const cost=[0,3,6,12,13][trace%5];
  const wb=new WriteBuffer((data,result)=>{log.push('action:'+ (typeof data==='string'?'t:'+units(data):'b:'+Array.from(data).map(c=>c.toString(16)).join(','))+':'+String(result));now+=cost;});
  wb.onWriteParsed(()=>log.push('parsed'));
  for(let step=0;step<64;step++) {
   const op=step===63?'dispose':operations[random()%operations.length]; counts[op]=(counts[op]??0)+1;
   const text=['','a','😀','e\u0301','\ud800'][random()%5]; const bytes=!!(random()%2); const value=bytes?Uint8Array.from([random()%256,random()%256].slice(0,random()%3)):text;
   const label=trace+','+step;
   if(op==='write')wb.write(value,()=>log.push('callback:'+label));
   if(op==='sync')wb.writeSync(value);
   if(op==='flush')wb.flushSync();
   if(op==='input')wb.handleUserInput();
   if(op==='dispose')wb.dispose();
   if(op==='tick') {const first=timers.entries().next().value;if(first){timers.delete(first[0]);first[1]();}}
   const encoded=typeof value==='string'?units(value):(Array.from(value).map(c=>c.toString(16)).join(',')||'-');
   rows.push([trace,step,op,bytes?'b':'t',encoded,cost,wb._pendingData,wb._bufferOffset,wb._writeBuffer.length,+wb._store.isDisposed,timers.size,log.join('|')||'-'].join('\t'));
  }
 }
 for(const scenario of ['mutated','detached','resized']) {
  timers.clear();now=100;const seen=[];
  const wb=new WriteBuffer(data=>{seen.push(Array.from({length:data.length},(_,i)=>data[i]));now+=12;});
  const backing=new ArrayBuffer(2,{maxByteLength:4});const view=new Uint8Array(backing);view.set([65,66]);
  wb.write(view);
  if(scenario==='mutated')view[0]=67;
  if(scenario==='detached')structuredClone(backing,{transfer:[backing]});
  if(scenario==='resized'){backing.resize(4);view.set([65,66,67,68]);}
  if(scenario!=='mutated')wb.write(new Uint8Array([88]));
  tick();externalViews[scenario]={seen,pending:wb._pendingData,offset:wb._bufferOffset,chunks:wb._writeBuffer.length};
  wb.dispose();
 }
 assert.deepEqual(externalViews.mutated.seen,[[67,66]]);
 assert.deepEqual(externalViews.detached,{seen:[[]],pending:3,offset:1,chunks:2});
 assert.deepEqual(externalViews.resized,{seen:[[65,66,67,68]],pending:-1,offset:1,chunks:2});
} finally {globalThis.setTimeout=originalTimeout;globalThis.clearTimeout=originalClear;Object.defineProperty(globalThis,'performance',performanceDescriptor);}
const fixture='crates/xterm-core/tests/fixtures/write-buffer-traces.tsv';
writeFileSync(resolve(root,fixture),rows.join('\n')+'\n');
const report={scope:'WriteBuffer only',overallEquivalence:'NOT PROVEN',node:process.version,platform:process.platform,sourceHashes,
 seed:'0x5717eb0f',traces:128,steps:rows.length,counts,rejectionOrder,externalViews,fixture,fixtureSha256:createHash('sha256').update(readFileSync(resolve(root,fixture))).digest('hex'),
 baseline:{discovered:tests.length,executed:outcomes.length,passed:outcomes.filter(t=>t.status==='passed').length,failed:outcomes.filter(t=>t.status==='failed').length,outcomes},
 unrun:['Native Linux/macOS runtime','Integrated terminal/browser scheduling','Async future differential traces, covered by controlled Rust tests only']};
mkdirSync(resolve(root,'results/write-buffer'),{recursive:true});
writeFileSync(resolve(root,'results/write-buffer/oracle.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2)); if(report.baseline.failed)process.exitCode=1;
