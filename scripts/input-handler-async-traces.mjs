// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Real source terminal pending-handler lifecycle, independent of the Rust engine.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'..'),require=createRequire(resolve(root,'source_repo/package.json'));
const {build}=require('esbuild');const output=resolve(root,'tmp/input-handler-async/terminal.cjs');
mkdirSync(resolve(root,'tmp/input-handler-async'),{recursive:true});
const bundle=await build({absWorkingDir:root,entryPoints:['source_repo/src/headless/public/Terminal.ts'],outfile:output,bundle:true,platform:'node',format:'cjs',target:'node24',tsconfig:'source_repo/src/headless/tsconfig.json',metafile:true,logLevel:'silent'});
const sourceHashes=Object.fromEntries(Object.keys(bundle.metafile.inputs).sort().map(file=>[file,createHash('sha256').update(readFileSync(resolve(root,file))).digest('hex')]));
const {Terminal}=require(output),originalTimeout=globalThis.setTimeout,originalClear=globalThis.clearTimeout;
const performance=Object.getOwnPropertyDescriptor(globalThis,'performance');
let next=0;const timers=new Map();
globalThis.setTimeout=callback=>{const id=++next;timers.set(id,callback);return id;};globalThis.clearTimeout=id=>timers.delete(id);
Object.defineProperty(globalThis,'performance',{configurable:true,value:{now:()=>100}});
const hex=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16)).join(',')||'-';
const rows=[],errors={};
function snapshot(name,phase,terminal,log){const queue=terminal._core._writeBuffer,disposed=queue._store.isDisposed;
 const text=disposed?'-':hex(terminal.buffer.active.getLine(0).translateToString(true));
 rows.push([name,phase,text,queue._pendingData,queue._bufferOffset,queue._writeBuffer.length,+disposed,log.join('|')||'-'].join('\t'));
}
try{
 for(const name of ['handled','bubble','rejected','reset','dispose','remove']){
  timers.clear();const log=[];errors[name]=[];const terminal=new Terminal({cols:80,rows:30,scrollback:1000,allowProposedApi:true,logLevel:'off'});
  const onError=error=>{errors[name].push({name:error.name,message:error.message});log.push('error');};process.on('uncaughtException',onError);
  try{
   terminal.onWriteParsed(()=>log.push('parsed'));
   terminal.parser.registerOscHandler(1000,data=>{log.push('older:'+hex(data));return true;});
   let resolveGate,rejectGate;const gate=new Promise((resolve,reject)=>{resolveGate=resolve;rejectGate=reject;});
   const handler=terminal.parser.registerOscHandler(1000,data=>{log.push('handler:'+hex(data));return gate;});
   let complete;const completed=new Promise(resolve=>{complete=resolve;});
   terminal.write('A\x1b]1000;some data\x07B',()=>log.push('callback:a'));
   terminal.write('C',()=>{log.push('callback:b');complete();});
   const id=terminal._core._writeBuffer._innerWriteTimer._token;const callback=timers.get(id);assert.equal(typeof callback,'function');timers.delete(id);callback();
   snapshot(name,'paused',terminal,log);
   if(name==='reset'){terminal.reset();log.push('control:reset');}
   if(name==='dispose'){terminal.dispose();log.push('control:dispose');}
   if(name==='remove'){handler.dispose();log.push('control:remove');}
   snapshot(name,'controlled',terminal,log);
   if(name==='rejected')rejectGate(new Error('retained rejection'));else resolveGate(name!=='bubble');
   if(name==='dispose'){
    // Bound the known promise chain without creating a sleep or host timer.
    for(let turn=0;turn<32;turn++)await Promise.resolve();
    assert(!log.includes('callback:a'));assert(!log.includes('callback:b'));
   }else await completed;
   snapshot(name,'complete',terminal,log);
  }finally{terminal.dispose();process.removeListener('uncaughtException',onError);}
 }
}finally{globalThis.setTimeout=originalTimeout;globalThis.clearTimeout=originalClear;Object.defineProperty(globalThis,'performance',performance);}
const fixture='crates/xterm-core/tests/fixtures/input-handler-async.tsv';writeFileSync(resolve(root,fixture),rows.join('\n')+'\n');
const report={scope:'Six real terminal OSC pause/lifecycle traces, three observations each',overallEquivalence:'NOT PROVEN',node:process.version,platform:process.platform,sourceHashes,scenarios:6,observations:rows.length,errors,fixture,fixtureSha256:createHash('sha256').update(readFileSync(resolve(root,fixture))).digest('hex'),unrun:['Rust replay until executed','Native Linux/macOS runtime','Other InputHandler and browser suites']};
mkdirSync(resolve(root,'results/input-handler-async'),{recursive:true});writeFileSync(resolve(root,'results/input-handler-async/traces.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
