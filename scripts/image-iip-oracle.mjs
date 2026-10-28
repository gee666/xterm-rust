// IIP protocol policy comparison. Recording bitmap/storage hosts do not prove pixels.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
const require=createRequire(import.meta.url),ts=require('../source_repo/node_modules/typescript');
const root=path.resolve(import.meta.dirname,'..'),sourceRoot=path.join(root,'source_repo/addons/addon-image/src'),output=path.join(root,'results/image-iip');
fs.mkdirSync(output,{recursive:true});
const sourceHashes={};const digest=value=>createHash('sha256').update(value).digest('hex');
let effects=[],lastMetrics,error=null;
function load(file,imports={}){
 const source=fs.readFileSync(path.join(sourceRoot,file),'utf8');sourceHashes[file]=digest(source);
 const code=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
 const module={exports:{}};
 vm.runInNewContext(code,{module,exports:module.exports,require:id=>{if(!(id in imports))throw Error(`unmapped ${file} import ${id}`);return imports[id];},Buffer,Uint32Array,Uint8Array,Uint8ClampedArray,Blob,TextDecoder,atob,console:{warn:message=>effects.push(['warning',message])},createImageBitmap:async(blob,options)=>{
  // Only the requested bytes/size are compared. No codec or canvas result is claimed.
  const bytes=Array.from(new Uint8Array(await blob.arrayBuffer()));const measured=lastMetrics??metrics.imageType(Uint8Array.from(bytes));
  effects.push(['request',bytes,blob.type,measured.width,measured.height,bits(options.resizeWidth),bits(options.resizeHeight)]);
  return {width:options.resizeWidth,height:options.resizeHeight};
 }},{filename:file});return module.exports;
}
const headers=load('IIPHeaderParser.ts'),metrics=load('IIPMetrics.ts');
const trackedMetrics={...metrics,imageType(data){lastMetrics=metrics.imageType(data);return lastMetrics;}};
const storeSource=fs.readFileSync(path.join(sourceRoot,'ImageStorage.ts'),'utf8');sourceHashes['ImageStorage.ts']=digest(storeSource);
const literal=storeSource.match(/export const CELL_SIZE_DEFAULT[^=]*=\s*(\{[^}]*\})/)[1];
const cellSize=vm.runInNewContext(`(${literal})`);
const decoderModule=require('../source_repo/node_modules/xterm-wasm-parts/lib/base64/Base64Decoder.wasm.js');
const qoiModule=require('../source_repo/node_modules/xterm-wasm-parts/lib/qoi/QoiDecoder.wasm.js');
const {IIPHandler}=load('IIPHandler.ts',{'./ImageRenderer':{ImageRenderer:{createCanvas(){throw Error('QOI canvas is outside this recording protocol harness');}}},'./ImageStorage':{CELL_SIZE_DEFAULT:cellSize},'./IIPHeaderParser':headers,'./IIPMetrics':trackedMetrics,'xterm-wasm-parts/lib/base64/Base64Decoder.wasm':decoderModule,'xterm-wasm-parts/lib/qoi/QoiDecoder.wasm':qoiModule});
const units=text=>Array.from({length:text.length},(_,i)=>text.charCodeAt(i));
const bits=value=>{if(Number.isNaN(value))return'NaN';const data=Buffer.alloc(8);data.writeDoubleBE(value);return data.toString('hex');};
const inputBits=value=>{const data=Buffer.alloc(8);data.writeDoubleBE(value);return data.toString('hex');};
const fromBits=value=>Buffer.from(value,'hex').readDoubleBE();
function field(value){if(value===null)return['null'];if(value===undefined)return['u'];if(typeof value==='number')return['n',bits(value)];if(typeof value==='string')return['s',units(value)];if(typeof value==='boolean')return['b',value];if(value instanceof Uint32Array)return['a',Array.from(value)];return['o'];}
const fields=value=>Object.keys(value).map(key=>[units(key),field(value[key])]);
const renderer={dimensions:undefined};const terminal={cols:80,rows:24,_core:{},input:(text,user)=>effects.push(['report',units(text),user])};
// The source policy calls the image host; the host records its request rather
// than substituting this source implementation for any Rust protocol logic.
const recordingStorage={addImage(_bitmap){}};
let handler=new IIPHandler({pixelLimit:16777216,iipSizeLimit:33554432},renderer,recordingStorage,terminal);
const operations=[],expected=[];
const geometry=(overrides={})=>({columns:inputBits(80),rows:inputBits(24),render:null,dpr:null,...overrides});
function applyGeometry(value){terminal.cols=fromBits(value.columns);terminal.rows=fromBits(value.rows);renderer.dimensions=value.render?{css:{cell:{width:fromBits(value.render.cell.width),height:fromBits(value.render.cell.height)},canvas:{width:fromBits(value.render.canvas.width),height:fromBits(value.render.canvas.height)}}}:undefined;terminal._core._coreBrowserService=value.dpr===null?undefined:{dpr:fromBits(value.dpr)};}
function snapshot(){return{aborted:handler._aborted,multipart:handler._isMultipart,abortMultipart:handler._abortMulti,parserState:handler._hp.state,fields:fields(handler._hp.fields),header:fields(handler._header),loadedBytes:handler._dec.loadedBytes,effects,error};}
async function operation(op){
 effects=[];error=null;let result;
 switch(op.op){
  case'new':handler=new IIPHandler({pixelLimit:fromBits(op.pixelLimit),iipSizeLimit:op.sizeLimit},renderer,recordingStorage,terminal);break;
  case'start':handler.start();break;
  case'reset':handler.reset();break;
  case'put':handler.put(Uint32Array.from(op.data),op.start,op.end);break;
  case'end':{
   applyGeometry(op.geometry);const original=Object.getOwnPropertyDescriptor(renderer,'dimensions');let fired=false;
   if(op.resetDuringGeometry||op.throwDuringGeometry)Object.defineProperty(renderer,'dimensions',{configurable:true,get(){if(!fired){fired=true;if(op.resetDuringGeometry)handler.reset();if(op.throwDuringGeometry)throw Error('geometry getter threw');}return original.value;}});
   try{await handler.end(op.success);}catch(cause){if(cause.message==='geometry getter threw')error=cause.message;else throw cause;}
   finally{Object.defineProperty(renderer,'dimensions',original);}
   break;
  }
  case'resize':applyGeometry(op.geometry);handler._header={width:String.fromCharCode(...op.width),height:String.fromCharCode(...op.height),preserveAspectRatio:fromBits(op.ratio)};result=handler._resize(fromBits(op.sourceWidth),fromBits(op.sourceHeight)).map(bits);break;
  case'fixed':result=fromBits(op.value).toFixed(3);break;
  default:throw Error(`invalid op ${op.op}`);
 }
 operations.push(op);expected.push(JSON.parse(JSON.stringify(result??snapshot())));
}
const fresh=async(pixelLimit=16777216,sizeLimit=33554432)=>operation({op:'new',pixelLimit:inputBits(pixelLimit),sizeLimit});
async function write(text,success=true,chunk=9999,metrics=geometry()){
 await operation({op:'start'});const data=typeof text==='string'?units(text):text;
 for(let index=0;index<data.length;index+=chunk){const part=[999,...data.slice(index,index+chunk),998];await operation({op:'put',data:part,start:1,end:part.length-1});}
 await operation({op:'end',success,geometry:metrics});
}
// A sniffable metadata header, deliberately not a valid decoded PNG. This
// corpus verifies requests; real format pixels are User img-23/browser work.
const png=Buffer.alloc(32);png.set([137,80,78,71,13,10,26,10]);png.set(Buffer.from('IHDR'),12);png.writeInt32BE(2,16);png.writeInt32BE(1,20);const payload=png.toString('base64');
for(const limit of[0,1,2,3,32,33,34])for(const text of[`File=inline=1:${payload}`,`File=inline=1:${payload.replace(/=+$/,'')}`]){await fresh(16777216,limit);await write(text);}
for(const limit of[-1,0,1,2,3,21,22,Infinity,NaN]){await fresh(limit);await write(`File=inline=1:${payload}`);await write(`File=inline=1;width=1:${payload}`);}
await fresh();
for(const text of[`File=inline=0:${payload}`,`File=size=999;inline=1:${payload}`,`File=inline=1:${payload}`,'File=inline=1:%%%', 'File=width=;inline=1:AAAA',':AAAA','File=inline=1:AA A','File=inline=1:AAAA====']){await write(text);await write(`File=inline=1:${payload}`,false);}
for(const chunk of[1,2,3,7,9999]){await fresh();await write('MultipartFile=inline=1',true,chunk);await write(`FilePart=${payload.slice(0,19)}`,false,chunk);await write(`FilePart=${payload.slice(19)}`,true,chunk);await write('FileEnd',true,chunk);await write('FileEnd',true,chunk);}
await fresh();await write('MultipartFile=inline=1');await write('FilePart=AAAA');await operation({op:'reset'});await write('FileEnd');
await fresh();await write('MultipartFile=inline=1');await write('FilePart=AAAA');await write(`File=inline=1:${payload}`);await write('FileEnd');
for(const text of[`File=inline=1:${payload}`,'ReportCellSize','File=inline=0:AAAA','File=inline=1:%%%'])for(const flag of['resetDuringGeometry','throwDuringGeometry']){
 await fresh();await operation({op:'start'});const data=units(text);await operation({op:'put',data,start:0,end:data.length});await operation({op:'end',success:true,geometry:geometry(),[flag]:true});
}
for(const dpr of[undefined,0,1,1.0625,NaN,Infinity]){await fresh();await write('ReportCellSize',false,1,geometry({dpr:dpr===undefined?null:inputBits(dpr),columns:inputBits(1),rows:inputBits(1),render:{cell:{width:inputBits(999),height:inputBits(999)},canvas:{width:inputBits(2.0625),height:inputBits(1.0625)}}}));}
const seed=0x13372026;let state=seed;function random(n){state^=state<<13;state^=state>>>17;state^=state<<5;return(state>>>0)%n;}
await fresh(10000,1000);
for(let i=0;i<1000;i++){
 const text=[`File=inline=${random(2)};width=${['auto','1','20%','320px'][random(4)]}:${payload}`,'MultipartFile=inline=1',`FilePart=${payload.slice(0,random(payload.length+1))}`,'FileEnd','ReportCellSize',`File=inline=1:${['A','AA','AAA','AAAA','%','AA='][random(6)]}`][random(6)];
 if(random(20)===0)await operation({op:'reset'});
 await write(text,random(4)!==0,1+random(40));
}
for(let i=0;i<1000;i++){
 const values=['auto','0','1','320px','50%','30%',' 12px','-5','9007199254740993px','9'.repeat(100)+'px'];
 const cell={width:inputBits([0,7,7.25,-1,NaN][random(5)]),height:inputBits([0,14,14.5,-1,NaN][random(5)])};
 await operation({op:'resize',geometry:geometry({render:random(2)?{cell,canvas:{width:inputBits(random(1000)),height:inputBits(random(600))}}:null}),sourceWidth:inputBits(random(2000)),sourceHeight:inputBits(random(1000)),width:units(values[random(values.length)]),height:units(values[random(values.length)]),ratio:inputBits(random(2))});
}
for(const value of[0,-0,1.0625,-1.0625,0.0625,1.0005,1e21,1e21-131072,Infinity,-Infinity,NaN,Number.MIN_VALUE,-Number.MIN_VALUE])await operation({op:'fixed',value:inputBits(value)});
for(let i=0;i<3000;i++){const buffer=Buffer.alloc(8);buffer.writeUInt32LE(random(0x100000000),0);buffer.writeUInt32LE(random(0x100000000),4);await operation({op:'fixed',value:buffer.toString('hex').match(/../g).reverse().join('')});}
const replay=operations.map(op=>JSON.stringify(op)).join('\n')+'\n';fs.writeFileSync(path.join(output,'operations.ndjson'),replay);
const implementationPaths=['crates/xterm-addons/src/image/iip/handler.rs','crates/xterm-addons/src/image/iip/geometry.rs','crates/xterm-addons/src/image/iip/reports.rs','crates/xterm-addons/src/image/numeric.rs','crates/xterm-addons/src/image/iip/types.rs','crates/xterm-addons/src/image/base64.rs','crates/xterm-addons/examples/image_iip_trace.rs','scripts/image-iip-oracle.mjs'];
const implementation=implementationPaths.map(file=>({path:file,sha256:digest(fs.readFileSync(path.join(root,file)))}));
const dependencyHashes=['base64/Base64Decoder.wasm.js','qoi/QoiDecoder.wasm.js'].map(file=>({path:`source_repo/node_modules/xterm-wasm-parts/lib/${file}`,sha256:digest(fs.readFileSync(path.join(root,'source_repo/node_modules/xterm-wasm-parts/lib',file)))}));
if(process.argv.includes('--reference-only')){
 const summary={status:'reference-only; Rust NOT RUN',scope:'Recording host policy probes, not pixels or integration',seed,referenceOperations:expected.length,sourceHashes,dependencyHashes,operationsSha256:digest(replay)};
 fs.writeFileSync(path.join(output,'reference-only.json'),JSON.stringify(summary,null,2)+'\n');console.log(JSON.stringify(summary,null,2));process.exit(0);
}
const build=spawnSync('cargo',['build','-p','xterm-addons','--example','image_iip_trace'],{cwd:root,encoding:'utf8',timeout:120000});fs.writeFileSync(path.join(output,'build.log'),(build.stdout??'')+(build.stderr??''));assert.equal(build.status,0,build.stderr||build.error?.message);
const binary=path.join(root,'target/debug/examples/image_iip_trace'+(process.platform==='win32'?'.exe':''));const run=spawnSync(binary,[],{cwd:root,input:replay,encoding:'utf8',timeout:60000,maxBuffer:64*1024*1024});assert.equal(run.status,0,run.stderr||run.error?.message);
const actual=run.stdout.replace(/\r?\n$/,'').split(/\r?\n/).map(line=>JSON.parse(line));const failures=[];
for(let index=0;index<Math.max(actual.length,expected.length);index++){try{assert.deepEqual(actual[index],expected[index]);}catch{failures.push({index,operation:operations[index],expected:expected[index],actual:actual[index]});if(failures.length===10)break;}}
if(failures.length)fs.writeFileSync(path.join(output,'counterexample.json'),JSON.stringify({seed,failures},null,2)+'\n');
for(const entry of implementation)assert.equal(digest(fs.readFileSync(path.join(root,entry.path))),entry.sha256,`implementation changed during build/run: ${entry.path}`);
const summary={implementation,dependencyHashes,scope:'Native IIP protocol policy and geometry only. Recording bitmap/storage hosts do not prove codec pixels, buffer tiles or live WASM/browser integration.',status:failures.length?'failed':'passed',seed,comparisons:expected.length,mismatches:failures.length,randomProtocolCases:1000,randomResizeCases:1000,randomFixedCases:3000,originalUpstreamCases:0,node:process.version,typescript:ts.version,platform:process.platform,sourceHashes,dependency:'xterm-wasm-parts0.4.1 original base64 WASM, not Rust execution delegation',operationsSha256:digest(replay),expectedSha256:digest(JSON.stringify(expected)),failures};
fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2)+'\n');console.log(JSON.stringify({...summary,failures:failures.map(f=>({index:f.index,operation:f.operation}))},null,2));if(failures.length)process.exitCode=1;
