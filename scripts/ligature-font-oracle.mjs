// Run the original three fontLigatures suites without changing assertions.
// Production Rust never reads these expected graphs or delegates to this code.
import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
const root=process.cwd(), temporary=path.join(root,'tmp/ligature-font-oracle');
const resultDir=path.join(root,'results/font-addons');
const fixtureDir=path.join(root,'crates/xterm-addons/tests/fixtures');
const sourceDir=path.join(root,'source_repo/addons/addon-ligatures/src/fontLigatures');
const fontDir=path.join(root,'source_repo/addons/addon-ligatures/fonts');
const require=createRequire(path.join(root,'source_repo/package.json'));
const generate=process.argv.slice(2).includes('--generate');
if(process.argv.slice(2).some(arg=>arg!=='--generate'))throw new Error('Only --generate is supported');
await Promise.all([temporary,resultDir,fixtureDir].map(dir=>fs.mkdir(dir,{recursive:true})));
const hashes={};
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
async function copy(dir,relative='') {
 for(const entry of await fs.readdir(dir,{withFileTypes:true})) {
  const name=path.join(relative,entry.name),file=path.join(dir,entry.name);
  if(entry.isDirectory()){await copy(file,name);continue;}
  if(!name.endsWith('.ts'))continue;
  const bytes=await fs.readFile(file);hashes[path.relative(root,file)]=digest(bytes);
  let text=bytes.toString('utf8');
  text=text.replace(/import\s+\{([^}]+)\}\s+from\s+(['"])([^'"]*(?:types|tables))\2/g,'import type {$1} from $2$3$2');
  text=text.replace(/from\s+(['"])(\.[^'"]+)\1/g,(_,quote,spec)=>`from ${quote}${spec.endsWith('.ts')?spec:spec+'.ts'}${quote}`);
  text=text.replace(', IEntryMeta } from', ' } from');
  text=text.replace(/from (['"])(?:opentype\.js|lru-cache)\1/g,"from './dependencies.ts'");
  text=text.replace(/from (['"])chai\1/g,"from './assertions.ts'");
  if(name==='index.ts')text=text.replace('export { IFont as Font, ILigatureData as LigatureData, IOptions as Options };','export type { IFont as Font, ILigatureData as LigatureData, IOptions as Options };');
  if(name==='index.test.ts')text='const __dirname='+JSON.stringify(sourceDir)+';\n'+text;
  if(name==='merge.test.ts')text=text.replace("from './merge.ts'","from './merge-observer.ts'");
  if(name==='mergeRange.test.ts')text=text.replace("from './mergeRange.ts'","from './merge-range-observer.ts'");
  const target=path.join(temporary,name==='index.ts'?'reference-index.ts':name);
  await fs.mkdir(path.dirname(target),{recursive:true});await fs.writeFile(target,text);
 }
}
await copy(sourceDir);
await fs.writeFile(path.join(temporary,'dependencies.ts'),`import {createRequire} from 'node:module';const require=createRequire(${JSON.stringify(path.join(root,'source_repo/package.json'))});export const parse=require('opentype.js').parse;export const LRUCache=require('lru-cache').LRUCache;\n`);
await fs.writeFile(path.join(temporary,'assertions.ts'),`import {createRequire} from 'node:module';const require=createRequire(${JSON.stringify(path.join(root,'source_repo/package.json'))});const native=require('chai').assert;export const assert=new Proxy(native,{get(target,key){const value=Reflect.get(target,key);if(typeof value!=='function')return value;return (...args)=>{globalThis.ligatureAssertions.executed++;try{const result=Reflect.apply(value,target,args);globalThis.ligatureAssertions.passed++;return result}catch(error){globalThis.ligatureAssertions.failed++;throw error}}}});\n`);
await fs.writeFile(path.join(temporary,'index.ts'),`import {loadBuffer as reference} from './reference-index.ts';export function loadBuffer(bytes,options){const font=reference(bytes,options);const id=globalThis.ligatureTrace.newFont(bytes,options);return {findLigatures(text){return globalThis.ligatureTrace.call(id,'full',text,()=>font.findLigatures(text))},findLigatureRanges(text){return globalThis.ligatureTrace.call(id,'ranges',text,()=>font.findLigatureRanges(text))}}}\n`);
await fs.writeFile(path.join(temporary,'merge-observer.ts'),`import merge from './merge.ts';export default function(trees){const before=globalThis.ligatureTrace.graph(trees);const result=merge(trees);globalThis.ligatureTrace.merges.push({before,after:globalThis.ligatureTrace.graph([...trees,result])});return result}\n`);
await fs.writeFile(path.join(temporary,'merge-range-observer.ts'),`import merge from './mergeRange.ts';export default function(ranges,start,end){const before=ranges.map(pair=>[...pair]);const result=merge(ranges,start,end);globalThis.ligatureTrace.rangeMerges.push({before,start,end,after:ranges.map(pair=>[...pair]),sameObject:result===ranges});return result}\n`);
const files=['FiraCode-Regular.otf','iosevka-regular.ttf','Monoid-Regular.ttf','UbuntuMono-Regular.ttf'];
const fonts=await Promise.all(files.map(async file=>{const bytes=await fs.readFile(path.join(fontDir,file));return{file,sha256:digest(bytes),bytes}}));
const instances=[],calls=[],merges=[],rangeMerges=[],handleIDs=new WeakMap();let nextHandle=0;
function handle(value){let id=handleIDs.get(value);if(id===undefined){id=nextHandle++;handleIDs.set(value,id)}return id}
function number(value){return value===undefined?'undefined':Number.isNaN(value)?'NaN':Object.is(value,-0)?'-0':value}
function graph(roots) {
 const trees=[],entries=[],lookups=[],tm=new WeakMap(),em=new WeakMap(),lm=new WeakMap(),work=[];
 function add(value,map,array,kind){if(!value)return null;let id=map.get(value);if(id===undefined){id=array.length;map.set(value,id);array.push(null);work.push([kind,id,value])}return id}
 const tree=value=>add(value,tm,trees,'tree'),entry=value=>add(value,em,entries,'entry'),lookup=value=>add(value,lm,lookups,'lookup');
 const rootIDs=roots.map(tree);
 for(let cursor=0;cursor<work.length;cursor++) {
  const[kind,id,value]=work[cursor];
  if(kind==='tree')trees[id]={individual:Object.entries(value.individual).map(([key,node])=>[key,entry(node)]),range:value.range.map(part=>({range:[...part.range],entry:entry(part.entry)}))};
  else if(kind==='entry')entries[id]={lookup:lookup(value.lookup),forward:tree(value.forward),reverse:tree(value.reverse)};
  else lookups[id]={substitutions:value.substitutions.map(number),length:value.length,index:value.index,subIndex:value.subIndex,contextRange:[...value.contextRange]};
 }
 const result={roots:rootIDs,trees,entries,lookups};Object.defineProperty(result,'lookupIDs',{value:lm});return result;
}
globalThis.ligatureAssertions={executed:0,passed:0,failed:0};
globalThis.ligatureTrace={merges,rangeMerges,graph,
 newFont(bytes,options){const hash=digest(Buffer.from(bytes));const font=fonts.find(font=>font.sha256===hash);if(!font)throw new Error('Unknown oracle font bytes');const id=instances.length;instances.push({id,font:font.file,cacheSize:options?.cacheSize??0});return id},
 call(instance,method,input,callback){
  try{const value=callback();const record={instance,method,input:Array.from({length:input.length},(_,index)=>input.charCodeAt(index))};
   if(method==='full'){record.inputGlyphs=value.inputGlyphs.map(number);record.outputGlyphs=value.outputGlyphs.map(number);record.ranges=value.contextRanges.map(pair=>[...pair]);record.handles=[handle(value),handle(value.inputGlyphs),handle(value.outputGlyphs),handle(value.contextRanges)]}
   else{record.ranges=value.map(pair=>[...pair]);record.handles=[handle(value)]}
   calls.push(record);return value;
  }catch(error){calls.push({instance,method,input:Array.from({length:input.length},(_,index)=>input.charCodeAt(index)),error:{name:error.name,message:error.message}});throw error}
 }
};
const Mocha=require('mocha');const mocha=new Mocha({reporter:'spec'});mocha.suite.emit('pre-require',globalThis,'original-font-ligature-tests',mocha);
for(const file of ['index.test.ts','merge.test.ts','mergeRange.test.ts'])await import(pathToFileURL(path.join(temporary,file)).href);
const outcomes=[];let runner;
await new Promise(done=>{runner=mocha.run(done);runner.on('pass',test=>outcomes.push({name:test.fullTitle(),status:'passed'}));runner.on('fail',(test,error)=>outcomes.push({name:test.fullTitle(),status:'failed',error:String(error.stack)}));runner.on('pending',test=>outcomes.push({name:test.fullTitle(),status:'skipped'}));});
if(runner.stats.failures||runner.stats.pending)throw new Error('Original suite failed/skipped; expected fixtures were not rewritten');
// Use the same source implementation and observer for seeded cross-font traces.
const load=(await import(pathToFileURL(path.join(temporary,'index.ts')).href)).loadBuffer;
let seed=0x6C1747E5;
function random(){seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0}
const alphabet=[32,33,35,38,42,43,45,46,47,58,60,61,62,63,64,91,92,93,94,95,96,97,98,99,102,124,126,0,0x301,0x754c,0xD800,0xDC00,0xD83D,0xDE00];
for(const font of fonts){
 const source=load(font.bytes.buffer.slice(font.bytes.byteOffset,font.bytes.byteOffset+font.bytes.byteLength),{cacheSize:64});
 for(let index=0;index<512;index++){
  const input=String.fromCharCode(...Array.from({length:1+random()%32},()=>alphabet[random()%alphabet.length]));
  if(index%3)source.findLigatures(input);else source.findLigatureRanges(input);
 }
 for(const input of ['','==','in --> out','😀==','\uD800==','==\uDC00']){
  try{source.findLigatures(input)}catch(error){if(input!==''||error.name!=='TypeError')throw error}
  try{source.findLigatureRanges(input)}catch(error){if(input!==''||error.name!=='TypeError')throw error}
 }
}
function flatTree(root){const trees=new WeakMap(),entries=new WeakMap();function tree(value){let result=trees.get(value);if(result)return result;result={individual:{},range:[]};trees.set(value,result);for(const[key,child]of Object.entries(value))result.individual[key]=entry(child);return result}function entry(value){let result=entries.get(value);if(result)return result;result={};entries.set(value,result);if(value.lookup)result.lookup=value.lookup;if(value.forward)result.forward=tree(value.forward);if(value.reverse)result.reverse=tree(value.reverse);return result}return tree(root)}
const directed=[];
const merge=(await import(pathToFileURL(path.join(temporary,'merge.ts')).href)).default;
const flatten=(await import(pathToFileURL(path.join(temporary,'flatten.ts')).href)).default;
const walk=(await import(pathToFileURL(path.join(temporary,'walk.ts')).href)).default;
function data(glyph,index=0,subIndex=0){return {substitutions:[glyph],length:1,index,subIndex,contextRange:[0,1]}}
function mergeCase(trees){const before=graph(trees),result=merge(trees);directed.push({operation:'merge',before,after:graph([...trees,result])})}
for(let start=0;start<4;start++)for(let end=start;end<5;end++)for(let glyph=-1;glyph<6;glyph++)mergeCase([{individual:{},range:[{range:[start,end],entry:{lookup:data(1,1)}}]},{individual:{[glyph]:{lookup:data(2,0)}},range:[]}]);
for(let a=0;a<4;a++)for(let b=a+1;b<5;b++)for(let c=0;c<4;c++)for(let d=c+1;d<5;d++)mergeCase([{individual:{},range:[{range:[a,b],entry:{lookup:data(1)}}]},{individual:{},range:[{range:[c,d],entry:{lookup:data(2,1)}}]}]);
const shared={lookup:data(11)},shadow={lookup:data(12)};const cyclic={individual:{1:shared,3:shadow},range:[{range:[2,4],entry:shared}]};shared.forward=cyclic;
const before=graph([cyclic]);const flat=flatten(cyclic);directed.push({operation:'flatten',before,after:graph([cyclic,flatTree(flat)])});
for(const forward of [false,true])for(const index of [0,1,2])for(const subIndex of [0,1,2]){
 const tree={1:{lookup:data(1,1,2),reverse:{6:{lookup:data(2,index,subIndex),reverse:{5:{lookup:data(3,index,0)}}}}}};
 if(forward)tree[1].forward={2:{lookup:data(4,1,1)}};
 const wrapper={individual:tree,range:[]};
 function unflatten(node){for(const entry of Object.values(node.individual)){for(const key of ['forward','reverse'])if(entry[key]){entry[key]={individual:entry[key],range:[]};unflatten(entry[key])}}}
 unflatten(wrapper);const flattened=flatten(wrapper);const raw=graph([wrapper,flatTree(flattened)]);const result=walk(flattened,[5,6,1,2],2,2);directed.push({operation:'walk',before:raw,sequence:[5,6,1,2],start:2,root:1,result:result?raw.lookupIDs.get(result):null});
}
async function fixture(name,value){const file=path.join(fixtureDir,name),text=JSON.stringify(value)+'\n';if(generate)await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text){await fs.writeFile(path.join(temporary,name+'.actual'),text);throw new Error('Fixture drift '+name+'; inspect temporary actual before --generate')}return digest(text)}
const fontTrace={fonts:fonts.map(({file,sha256})=>({file,sha256})),instances,calls};
const fixtureHashes={fonts:await fixture('ligature-font-trace.json',fontTrace),merges:await fixture('ligature-merge-trace.json',merges),ranges:await fixture('ligature-range-trace.json',rangeMerges),directed:await fixture('ligature-directed-tree.json',directed)};
const report={node:process.version,scope:'Original JS baseline and expected traces, not Rust evidence',seed:'0x6C1747E5',hashes,upstream:{discovered:runner.total,executed:runner.stats.tests-runner.stats.pending,passed:runner.stats.passes,failed:runner.stats.failures,skipped:runner.stats.pending,unrun:runner.total-runner.stats.tests,outcomes},assertions:globalThis.ligatureAssertions,fontInstances:instances.length,fontCalls:calls.length,seededCalls:2048,mergeCalls:merges.length,rangeMergeCalls:rangeMerges.length,directedGraphs:directed.length,fixtureHashes,mode:generate?'generate':'verify'};
await fs.writeFile(path.join(resultDir,'ligature-oracle.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({...report,hashes:undefined,upstream:{...report.upstream,outcomes:undefined}}));
