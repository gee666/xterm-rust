// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Independent real headless terminal/queue traces, not a mock terminal engine.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root=resolve(import.meta.dirname,'..');
const receipt=resolve(root,'results/native-queue-review/oracle.json'),startedAt=new Date().toISOString();
mkdirSync(resolve(root,'results/native-queue-review'),{recursive:true});
writeFileSync(receipt,JSON.stringify({status:'incomplete',startedAt,scope:'Native queue source traces',overallEquivalence:'NOT PROVEN'})+'\n');
process.on('uncaughtExceptionMonitor',error=>writeFileSync(receipt,JSON.stringify({status:'failed',startedAt,error:String(error?.stack??error),overallEquivalence:'NOT PROVEN'})+'\n'));
const {verifyOrGenerate}=await import('./oracle-fixture.mjs');
const require=createRequire(resolve(root,'source_repo/package.json'));
const {build}=require('esbuild');
const output=resolve(root,'tmp/native-queue-oracle/terminal.cjs');
mkdirSync(resolve(root,'tmp/native-queue-oracle'),{recursive:true});
const bundled=await build({absWorkingDir:root,entryPoints:['source_repo/src/headless/public/Terminal.ts'],outfile:output,bundle:true,platform:'node',format:'cjs',target:'node24',tsconfig:'source_repo/src/headless/tsconfig.json',metafile:true,logLevel:'silent'});
const sourceHashes=Object.fromEntries(Object.keys(bundled.metafile.inputs).sort().map(file=>[file,createHash('sha256').update(readFileSync(resolve(root,file))).digest('hex')]));
const {Terminal}=require(output);
const originalTimeout=globalThis.setTimeout,originalClear=globalThis.clearTimeout;
const performance=Object.getOwnPropertyDescriptor(globalThis,'performance');
const timers=new Map();let next=0;
globalThis.setTimeout=callback=>{const id=++next;timers.set(id,callback);return id;};
globalThis.clearTimeout=id=>timers.delete(id);
Object.defineProperty(globalThis,'performance',{configurable:true,value:{now:()=>100}});
function tick(id){const callback=timers.get(id);if(callback){timers.delete(id);callback();}}
const hex=text=>Array.from({length:text.length},(_,i)=>text.charCodeAt(i).toString(16)).join(',')||'-';
const ops=[
 [['text','foo'],['bytes',[98,97,114]],['writeln','文'],['tick']],
 [['text','abcdefghijk'],['resize',5,3],['writeln','文'],['tick']],
 [['text','abc'],['resize',10,3],['tick']],
 [['input','x'],['text','a'],['disable'],['input','y'],['text','b'],['tick']],
 [['bytes',[0xf0,0x9f]],['bytes',[0x98,0x80]],['text','\u0301'],['tick'],['text','\ud83d'],['text','\ude00'],['tick']],
];
const rows=[];
try {
 for(let scenario=0;scenario<ops.length;scenario++) {
  timers.clear();const log=[];
  const terminal=new Terminal({cols:10,rows:3,scrollback:10,allowProposedApi:true,logLevel:'off'});
  terminal.onWriteParsed(()=>log.push('parsed'));
  terminal.onResize(({cols,rows})=>log.push(`resize:${cols},${rows}`));
  terminal.onData(data=>log.push('data:'+hex(data)));
  for(let step=0;step<ops[scenario].length;step++) {
   const [op,value,third]=ops[scenario][step];const label=`${scenario},${step}`;
   const callback=()=>log.push('callback:'+label);
   if(op==='text')terminal.write(value,callback);
   if(op==='bytes')terminal.write(Uint8Array.from(value),callback);
   if(op==='writeln')terminal.writeln(value,callback);
   if(op==='resize')terminal.resize(value,third);
   if(op==='input')terminal.input(value,true);
   if(op==='disable')terminal.options.disableStdin=true;
   if(op==='tick')tick(terminal._core._writeBuffer._innerWriteTimer._token);
   const buffer=terminal.buffer.active,queue=terminal._core._writeBuffer;
   const lines=Array.from({length:buffer.length},(_,i)=>hex(buffer.getLine(i).translateToString(true))).join(';');
   const encoded=typeof value==='string'?hex(value):Array.isArray(value)?value.map(v=>v.toString(16)).join(',')||'-':value??'-';
   rows.push([scenario,step,op,encoded,third??'-',terminal.cols,terminal.rows,buffer.cursorX,buffer.cursorY,buffer.baseY,buffer.viewportY,buffer.length,lines,queue._pendingData,queue._bufferOffset,queue._writeBuffer.length,+(queue._innerWriteTimer._token!==-1),log.join('|')||'-'].join('\t'));
  }
  const writeTimer=terminal._core._writeBuffer._innerWriteTimer;
  terminal.dispose();assert.equal(writeTimer._token,-1,'terminal disposal must cancel retained queue timer');
 }
} finally {globalThis.setTimeout=originalTimeout;globalThis.clearTimeout=originalClear;Object.defineProperty(globalThis,'performance',performance);}
const fixture='crates/xterm-core/tests/fixtures/native-queue.tsv';
await verifyOrGenerate(resolve(root,fixture),rows.join('\n')+'\n',process.argv.includes('--generate'));
const report={status:'passed',startedAt,completedAt:new Date().toISOString(),scope:'Five real headless queue integration traces',overallEquivalence:'NOT PROVEN',node:process.version,platform:process.platform,esbuild:require('esbuild/package.json').version,sourceHashes,scenarios:ops.length,steps:rows.length,fixture,fixtureSha256:createHash('sha256').update(readFileSync(resolve(root,fixture))).digest('hex'),unrun:['Browser queue adapter','Native Linux/macOS runtime','Randomized full-terminal equivalence']};
mkdirSync(resolve(root,'results/native-queue-review'),{recursive:true});
writeFileSync(resolve(root,'results/native-queue-review/oracle.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
