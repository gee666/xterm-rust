// Baseline font access/resolution only. GSUB parsing is explicitly outside this
// oracle, replaced with an observable loadBuffer boundary in a temporary copy.
import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
const generate = process.argv.slice(2).includes('--generate');
if (process.argv.slice(2).some(arg => arg !== '--generate')) throw new Error('Only --generate is supported');
const root = process.cwd();
const temporary = path.join(root, 'tmp/local-font-oracle');
const resultDir = path.join(root, 'results/font-addons');
await fs.mkdir(temporary, { recursive: true });
await fs.mkdir(resultDir, { recursive: true });
const sourcePath = 'source_repo/addons/addon-ligatures/src/font.ts';
const source = await fs.readFile(sourcePath, 'utf8');
await fs.writeFile(path.join(temporary, 'font.ts'), source.replace("from './fontLigatures/index';", "from './shaping.ts';").replace("from './parse';", "from './parse.ts';"));
await fs.copyFile('source_repo/addons/addon-ligatures/src/parse.ts', path.join(temporary, 'parse.ts'));
await fs.writeFile(path.join(temporary, 'shaping.ts'), `export class Font { constructor(public bytes: number[], public cacheSize: number) {} }
export function loadBuffer(buffer: ArrayBuffer, options: {cacheSize: number}): Font { return new Font(Array.from(new Uint8Array(buffer)), options.cacheSize); }
`.replace('constructor(public bytes: number[], public cacheSize: number) {}', 'bytes: number[]; cacheSize: number; constructor(bytes: number[], cacheSize: number) { this.bytes=bytes; this.cacheSize=cacheSize; }'));
const cases = [
 { name:'modern cache and first face', environment:"const env={navigator:{},queries:0,blobs:0};env.window={queryLocalFonts:async()=>{env.queries++;return [{family:'CodeFont',blob:async()=>{env.blobs++;return new Blob([new Uint8Array([1,2,3])])}},{family:'CodeFont',blob:async()=>new Blob([new Uint8Array([9])])}]}};return env;", calls:['CodeFont, monospace','CodeFont','monospace, CodeFont','"CodeFont"'] },
 { name:'legacy TypeError permission and priority', environment:"const env={permissions:0,legacy:0,modern:0};env.navigator={permissions:{request:async()=>{env.permissions++;throw new TypeError('unsupported')}},fonts:{query:async()=>{env.legacy++;return [{family:'Legacy',blob:async()=>new Blob([new Uint8Array([7])])}]}}};env.window={queryLocalFonts:async()=>{env.modern++;return []}};return env;", calls:['Legacy','Legacy'] },
 { name:'denial is retryable', environment:"const env={permissions:0,queries:0,window:{}};env.navigator={permissions:{request:async()=>{env.permissions++;return {state:'denied'}}},fonts:{query:async()=>{env.queries++;return []}}};return env;", calls:['CodeFont','CodeFont'] },
 { name:'query error caches empty', environment:"const env={navigator:{},queries:0};env.window={queryLocalFonts:async()=>{env.queries++;throw new Error('intentional query failure')}};return env;", calls:['CodeFont','CodeFont'] },
 { name:'missing blob stops and blob failure propagates', environment:"return {navigator:{},window:{queryLocalFonts:async()=>[{family:'NoBlob'},{family:'Broken',blob:async()=>{throw new Error('blob failure')}}]}};", calls:['NoBlob, Broken','Broken'] },
 { name:'no APIs', environment:"return {navigator:{},window:{}};", calls:['Missing, sans-serif'] },
 { name:'absent legacy permissions ignored', environment:"return {navigator:{fonts:{query:async()=>[]}},window:{}};", calls:['Missing'] },
 { name:'primitive permission rejection ignored', environment:"const env={queries:0,window:{}};env.navigator={permissions:{request:async()=>{throw 'permission unavailable'}},fonts:{query:async()=>{env.queries++;return [{family:'CodeFont',blob:async()=>new Blob([new Uint8Array([5])])}]}}};return env;", calls:['CodeFont'] },
 { name:'malformed CSS and codepoint errors', environment:"return {navigator:{},window:{}};", calls:['"unterminated','\\110000','\\D800','CodeFont'] },
 { name:'generic family boundaries and case', environment:"return {navigator:{},window:{queryLocalFonts:async()=>[{family:'CodeFont',blob:async()=>new Blob([new Uint8Array([8])])}]}};", calls:['serif, CodeFont','sans-serif, CodeFont','cursive, CodeFont','fantasy, CodeFont','monospace, CodeFont','system-ui, CodeFont','emoji, CodeFont','math, CodeFont','fangsong, CodeFont','SERIF, CodeFont','codefont'] }
];
const previous = ['navigator','window'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis,key)]);
const originalError = console.error;
let concurrencyResult;
const optionalPermission=[];
try {
 for (let index=0; index<cases.length; index++) {
  const item=cases[index];
  const env = new Function(item.environment)();
  for(const key of ['navigator','window']) Object.defineProperty(globalThis,key,{configurable:true,value:env[key]});
  const logs=[];
  console.error=(...args)=>logs.push(args.map(String).join(': '));
  const load=(await import(pathToFileURL(path.join(temporary,'font.ts')).href+'?case='+index)).default;
  item.results=[];
  for(const family of item.calls) {
   try { const font=await load(family,100000); item.results.push(font ? {status:'font',bytes:font.bytes,cacheSize:font.cacheSize} : {status:'none'}); }
   catch(error) { item.results.push({status:'error',name:error.name,message:error.message}); }
  }
  item.counters=Object.fromEntries(Object.entries(env).filter(([,value])=>typeof value==='number'));
  item.logs=logs;
 }
 const concurrentLogs=[];
 console.error=(...args)=>concurrentLogs.push(args.map(String).join(': '));
 const env = new Function("let begin,fail;const started=new Promise(r=>begin=r);const pending=new Promise((_,reject)=>fail=()=>reject(new Error('concurrent query failure')));const env={navigator:{},queries:0,started,fail};env.window={queryLocalFonts:()=>{env.queries++;if(env.queries===1){begin();return pending}return Promise.resolve([{family:'Concurrent',blob:async()=>new Blob([new Uint8Array([4,5])])}])}};return env;")();
 for(const key of ['navigator','window']) Object.defineProperty(globalThis,key,{configurable:true,value:env[key]});
 const load=(await import(pathToFileURL(path.join(temporary,'font.ts')).href+'?case=concurrent')).default;
 const first=load('Concurrent',100000);
 await env.started;
 const second=await load('Concurrent',100000);
 env.fail();
 const firstResult=await first;
 concurrencyResult={queries:env.queries,firstBytes:firstResult?.bytes,secondBytes:second?.bytes,logs:concurrentLogs};
 assert.deepEqual(concurrencyResult,{queries:2,firstBytes:[4,5],secondBytes:[4,5],logs:['Error: concurrent query failure']});
 for(const calls of [1,2]) {
  const trace=[]; const env={navigator:{permissions:{},fonts:{query(){trace.push('query');throw new Error('controlled synchronous query failure')}}},window:{}};
  for(const key of ['navigator','window'])Object.defineProperty(globalThis,key,{configurable:true,value:env[key]});
  const logs=[]; console.error=(...args)=>logs.push(args.map(String).join(': '));
  const load=(await import(pathToFileURL(path.join(temporary,'font.ts')).href+'?optional='+calls)).default;
  queueMicrotask(()=>trace.push('microtask'));
  await Promise.all(Array.from({length:calls},()=>load('Missing',100000)));
  assert.deepEqual(trace,['microtask',...Array(calls).fill('query')]);
  optionalPermission.push({calls,trace,logs});
 }
} finally {
 console.error=originalError;
 for(const [key,descriptor]of previous) {if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];}
}
const fixture=path.join(root,'crates/xterm-font-addons/tests/fixtures/local-font-cases.json');
const content=JSON.stringify(cases,null,2)+'\n';
if(generate) await fs.writeFile(fixture,content);
else {
 const expected=await fs.readFile(fixture,'utf8');
 if(expected!==content) {await fs.writeFile(path.join(temporary,'local-font-cases.actual.json'),content);assert.fail('Local font fixture drift. Inspect tmp/local-font-oracle/local-font-cases.actual.json before --generate.');}
}
const permissionFixture=path.join(root,'crates/xterm-font-addons/tests/fixtures/local-font-permission-trace.json');
const permissionText=JSON.stringify(optionalPermission)+'\n';
if(generate)await fs.writeFile(permissionFixture,permissionText);else assert.equal(await fs.readFile(permissionFixture,'utf8'),permissionText,'optional permission trace drift');
const report={node:process.version,scope:'Font metadata resolution, not GSUB',source:sourcePath,sha256:createHash('sha256').update(source).digest('hex'),cases:cases.length,calls:cases.reduce((sum,item)=>sum+item.calls.length,0),fixtureSha256:createHash('sha256').update(content).digest('hex'),mode:generate?'generate':'verify',concurrentQuery:concurrencyResult,optionalPermission};
await fs.writeFile(path.join(resultDir,'local-font-oracle.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
