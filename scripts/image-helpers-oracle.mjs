// Execute unchanged image helper source/tests, then replay their state transitions in Rust.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
const require=createRequire(import.meta.url);
const ts=require('../source_repo/node_modules/typescript');
const chai=require('../source_repo/node_modules/chai');
const root=path.resolve(import.meta.dirname,'..');
const addon=path.join(root,'source_repo/addons/addon-image');
const results=path.join(root,'results/image-helpers');
fs.mkdirSync(results,{recursive:true});
const hashes={};
const digest=value=>createHash('sha256').update(value).digest('hex');
function load(file,imports={},browser=false,transform=source=>source){
  const original=fs.readFileSync(path.join(addon,'src',file),'utf8');hashes[file]=digest(original);
  const source=transform(original);
  const code=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
  const module={exports:{}};
  const localRequire=id=>{if(!(id in imports))throw Error(`unmapped ${file} import ${id}`);return imports[id];};
  const globals={module,exports:module.exports,require:localRequire,Uint32Array,Uint8Array,TextDecoder,atob,Buffer:browser?undefined:Buffer};
  vm.runInNewContext(code,globals,{filename:file});return module.exports;
}
const native=load('IIPHeaderParser.ts');
const browser=load('IIPHeaderParser.ts',{},true);
const metrics=load('IIPMetrics.ts');
const units=text=>Array.from({length:text.length},(_,i)=>text.charCodeAt(i));
const number=value=>{const b=Buffer.alloc(8);b.writeDoubleBE(value);return b.toString('hex');};
function field(value){
  if(value===null)return['null'];if(value===undefined)return['u'];
  if(typeof value==='number')return['n',number(value)];
  if(typeof value==='string')return['s',units(value)];
  if(typeof value==='boolean')return['b',value];
  if(value instanceof Uint32Array)return['a',Array.from(value)];
  return['o'];
}
function snapshot(parser,result){
  const buffer=Array.from(parser._buffer);while(buffer.at(-1)===0)buffer.pop();
  return {result,state:parser.state,position:parser._position,key:units(parser._key),buffer,nullPrototype:Object.getPrototypeOf(parser.fields)===null,fields:Object.keys(parser.fields).map(key=>[units(key),field(parser.fields[key])])};
}
const operations=[],expected=[],original=[];
let currentTest='';let collectingOriginal=true;
function record(operation,output){
  // VM realms have different Object prototypes. The replay protocol compares
  // explicit typed values, not the harness realm owning a metrics object.
  const transported=JSON.parse(JSON.stringify(output));
  operations.push(operation);expected.push(transported);
  if(collectingOriginal)original.push({test:currentTest,operation,expected:transported});
}
class TracedHeader extends native.HeaderParser{
  constructor(){super();record({op:'new',policy:'node'},snapshot(this,0));}
  reset(){super.reset();record({op:'reset'},snapshot(this,0));}
  parse(data,start,end){const result=super.parse(data,start,end);record({op:'parse',data:Array.from(data),start,end},snapshot(this,result));return result;}
  end(){const result=super.end();record({op:'end'},snapshot(this,result));return result;}
}
const fixtures=new Map();const fixtureHashes={};
const fixtureFs={readFileSync(file){const data=fs.readFileSync(path.join(root,'source_repo',file));const name=path.basename(file);fixtures.set(data,name);fixtureHashes[name]={bytes:data.length,sha256:digest(data)};return data;}};
function tracedMetrics(data){const result=metrics.imageType(data);record(fixtures.has(data)?{op:'metrics',file:fixtures.get(data)}:{op:'metrics',data:Array.from(data)},result);return result;}
let assertions=0;const tests=[];
const counted=new Proxy(chai.assert,{get(target,key){const value=target[key];return typeof value==='function'?(...args)=>{assertions++;return value(...args);}:value;}});
const testGlobals={chai:{assert:counted},'./IIPHeaderParser':{...native,HeaderParser:TracedHeader},'./IIPMetrics':{...metrics,imageType:tracedMetrics},fs:fixtureFs};
// The test declarations stay byte-for-byte unchanged. Only module imports resolve to instrumented source classes.
function executeTestFile(file){
 const source=fs.readFileSync(path.join(addon,'src',file),'utf8');hashes[file]=digest(source);
 const code=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
 const module={exports:{}};
 vm.runInNewContext(code,{module,exports:module.exports,require:id=>{if(!(id in testGlobals))throw Error(`unmapped test ${id}`);return testGlobals[id];},Uint32Array,describe:(_name,callback)=>callback(),it:(name,callback)=>{currentTest=`${file}: ${name}`;tests.push(currentTest);callback();}},{filename:file});
}
executeTestFile('IIPHeaderParser.test.ts');executeTestFile('IIPMetrics.test.ts');
collectingOriginal=false;
fs.mkdirSync(path.join(root,'crates/xterm-addons/tests/fixtures'),{recursive:true});
fs.writeFileSync(path.join(root,'crates/xterm-addons/tests/fixtures/image-helpers-upstream.json'),JSON.stringify(original)+'\n');
let parser=new native.HeaderParser();
function operation(op){
 let output;
 if(op.op==='new'){parser=new (op.policy==='browser'?browser:native).HeaderParser();output=snapshot(parser,0);}
 else if(op.op==='reset'){parser.reset();output=snapshot(parser,0);}
 else if(op.op==='parse')output=snapshot(parser,parser.parse(Uint32Array.from(op.data),op.start,op.end));
 else if(op.op==='end')output=snapshot(parser,parser.end());
 else if(op.op==='metrics')output=metrics.imageType(Uint8Array.from(op.data));
 else throw Error(`bad operation ${op.op}`);
 record(op,output);
}
function parse(text){const data=typeof text==='string'?units(text):text;operation({op:'parse',data,start:0,end:data.length});}
const headers=['File=inline=1:sizeignored','File=size=;name=:','MultipartFile=inline=1;name=77u/dGVzdA==','FilePart=abcd','FileEnd','ReportCellSize',':',';','=bad','Filegarbage=inline=1:','MultipartFileEXTRA=inline=1:','File=x=1;2=2;01=3;0=4;4294967295=5:',...['__proto__','constructor','toString','toLocaleString','valueOf','hasOwnProperty','propertyIsEnumerable','isPrototypeOf','__lookupGetter__','__lookupSetter__','__defineGetter__','__defineSetter__'].map(key=>`File=${key}=123:`),...['auto','10px','20%','0','00','-1','AUTO','0.5',''].map(size=>`File=width=${size}:`),...['','0','9'.repeat(309),'9007199254740991','9007199254740992','123x'].map(size=>`File=size=${size}:`),...['A','A=','AA==','AAAA===','AA A\n','_w','-w','77u/YQ==','abc!def','😀','\ud800'].map(name=>`File=name=${name}:`),...['x','size'].flatMap(key=>[1023,1024,1025].map(length=>`File=${key}=${'1'.repeat(length)}:`))];
for(const policy of ['node','browser'])for(const header of headers){
 for(const split of [0,1,4,Math.floor(header.length/2),header.length]){
  operation({op:'new',policy});parse(header.slice(0,split));parse(header.slice(split));operation({op:'end'});parse('ignored');operation({op:'reset'});parse('File=inline=1:');
 }
}
for(const text of headers.filter(text=>text.length<120))for(let split=0;split<=text.length;split++){
 operation({op:'new',policy:'node'});parse(text.slice(0,split));parse(text.slice(split));operation({op:'end'});
}
operation({op:'new',policy:'node'});parse('123456789012345');parse('=');operation({op:'end'});
operation({op:'new',policy:'node'});parse([70,105,108,101,61,0x10061,61,0xffffffff,58]);
const seed=0x1a6e2026;let state=seed;
function random(n){state^=state<<13;state^=state>>>17;state^=state<<5;return(state>>>0)%n;}
for(let i=0;i<1000;i++){
 const policy=random(2)?'node':'browser';operation({op:'new',policy});
 const starter=['File=','MultipartFile=','FilePart=','FileEnd','ReportCellSize','bad='][random(6)];
 const data=units(starter+['size','name','width','x','toString','type'][random(6)]+'=');
 for(let j=0,n=random(40);j<n;j++)data.push([0,48+random(10),59,58,61,65+random(60),0x10000+random(0x10000),random(0xffffffff)][random(8)]);
 if(random(2))data.push(58);
 for(let start=0;start<data.length;){const count=1+random(11);const chunk=[999,...data.slice(start,start+count),998];operation({op:'parse',data:chunk,start:1,end:chunk.length-1});start+=count;}
 operation({op:'end'});
}
for(let length=0;length<=32;length++)operation({op:'metrics',data:new Array(length).fill(0)});
for(let i=0;i<1000;i++){
 const data=new Uint8Array(32+random(1100));for(let j=0;j<data.length;j++)data[j]=random(256);
 const kind=random(9);
 if(kind===0){data.set([137,80,78,71,13,10,26,10]);data.set(units('IHDR'),12);}
 if(kind===1)data.set([255,216,255]);
 if(kind===2)data.set(units('GIF89a'));
 if(kind===3)data.set(units('qoif'));
 if(kind>=4&&kind<=6){data.set(units('RIFF'));data.set(units('WEBPVP8'+['X','L',' '][kind-4]),8);if(kind===5)data[20]=47;if(kind===6)data.set([157,1,42],23);}
 if(kind===7){data.set(units('ftypavif'),4);const pos=Math.min(data.length-16,8+random(1040));data.set(units('ispe'),pos);}
 operation({op:'metrics',data:Array.from(data)});
}
fs.writeFileSync(path.join(results,'operations.ndjson'),operations.map(op=>JSON.stringify(op)).join('\n')+'\n');
const build=spawnSync('cargo',['build','-p','xterm-addons','--example','image_helpers_trace'],{cwd:root,encoding:'utf8',timeout:120000});fs.writeFileSync(path.join(results,'build.log'),(build.stdout??'')+(build.stderr??''));assert.equal(build.status,0,build.stderr||build.error?.message);
const executable=path.join(root,'target/debug/examples/image_helpers_trace'+(process.platform==='win32'?'.exe':''));
const run=spawnSync(executable,[],{cwd:root,input:operations.map(op=>JSON.stringify(op)).join('\n')+'\n',encoding:'utf8',timeout:120000,maxBuffer:128*1024*1024});assert.equal(run.status,0,run.stderr||run.error?.message);
const actual=run.stdout.replace(/\r?\n$/,'').split(/\r?\n/).map(line=>JSON.parse(line));
const negativeOperations=[],negativeExpected=[],controlEnds=[];
function control(name,file,from,to,commands){
  const mutant=load(file,{},false,source=>{assert.ok(source.includes(from),`missing mutant site ${name}`);return source.replace(from,to);});
  const hp=file==='IIPHeaderParser.ts'?new mutant.HeaderParser():undefined;
  for(const op of commands){
    negativeOperations.push(op);
    if(op.op==='new')negativeExpected.push(snapshot(hp,0));
    else if(op.op==='parse')negativeExpected.push(snapshot(hp,hp.parse(Uint32Array.from(op.data),op.start,op.end)));
    else if(op.op==='end')negativeExpected.push(snapshot(hp,hp.end()));
    else negativeExpected.push(JSON.parse(JSON.stringify(mutant.imageType(Uint8Array.from(op.data)))));
  }
  controlEnds.push({name,index:negativeExpected.length-1});
}
const parseOperation=text=>({op:'parse',data:units(text),start:0,end:text.length});
control('header capacity 1024 to1023','IIPHeaderParser.ts','MAX_FIELDCHARS = 1024','MAX_FIELDCHARS = 1023',[{op:'new',policy:'node'},parseOperation(`File=x=${'1'.repeat(1024)}:`)]);
control('decimal radix10 to9','IIPHeaderParser.ts','v = v * 10','v = v * 9',[{op:'new',policy:'node'},parseOperation('File=size=12:')]);
control('end commits abort instead of end','IIPHeaderParser.ts','this.state = HeaderState.END','this.state = HeaderState.ABORT',[{op:'new',policy:'node'},parseOperation('FileEnd'),{op:'end'}]);
const png=new Array(32).fill(0);png.splice(0,8,137,80,78,71,13,10,26,10);png.splice(12,4,...units('IHDR'));png[16]=1;
control('PNG high byte shift24 to16','IIPMetrics.ts','d[16] << 24','d[16] << 16',[{op:'metrics',data:png}]);
const negativeRun=spawnSync(executable,[],{cwd:root,input:negativeOperations.map(op=>JSON.stringify(op)).join('\n')+'\n',encoding:'utf8',timeout:30000,maxBuffer:1024*1024});
assert.equal(negativeRun.status,0,negativeRun.stderr);
const negativeActual=negativeRun.stdout.trimEnd().split(/\r?\n/).map(line=>JSON.parse(line));
const negativeControls=controlEnds.map(({name,index})=>{let rejected=false;try{assert.deepEqual(negativeActual[index],negativeExpected[index]);}catch{rejected=true;}return{name,rejected};});
assert.ok(negativeControls.every(control=>control.rejected),'oracle accepted a deliberately incorrect source mutant');
const failures=[];
for(let i=0;i<Math.max(actual.length,expected.length);i++){try{assert.deepEqual(actual[i],expected[i]);}catch{failures.push({index:i,operation:operations[i],expected:expected[i],actual:actual[i]});if(failures.length===10)break;}}
if(failures.length)fs.writeFileSync(path.join(results,'counterexample.json'),JSON.stringify({seed,failures},null,2)+'\n');
const implementation=['crates/xterm-addons/src/image/header.rs','crates/xterm-addons/src/image/header_codec.rs','crates/xterm-addons/src/image/metrics.rs','crates/xterm-addons/tests/image_helper_support/mod.rs','scripts/image-helpers-oracle.mjs'].map(file=>({path:file,sha256:digest(fs.readFileSync(path.join(root,file)))}));
const summary={scope:'Native image header/metrics only; required live WASM/3browser image acceptance NOT PROVEN',status:failures.length?'failed':'passed',node:process.version,typescript:ts.version,platform:process.platform,originalTests:tests.length,originalAssertions:assertions,originalReplayOperations:original.length,tests,seed,randomHeaderCases:1000,randomMetricsCases:1000,comparisons:expected.length,mismatches:failures.length,negativeControls,sourceHashes:hashes,fixtureHashes,implementation,operationsSha256:digest(JSON.stringify(operations)),outputsSha256:digest(JSON.stringify(expected)),failures};
fs.writeFileSync(path.join(results,'summary.json'),JSON.stringify(summary,null,2)+'\n');console.log(JSON.stringify({...summary,fixtureHashes:`${Object.keys(fixtureHashes).length} original fixtures`,failures:failures.map(f=>({index:f.index,operation:f.operation}))},null,2));if(failures.length)process.exitCode=1;
