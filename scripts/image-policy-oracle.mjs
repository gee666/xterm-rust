// Original ImageAddon control methods versus Rust policy. No DOM activation.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
const require=createRequire(import.meta.url),ts=require('../source_repo/node_modules/typescript');
const root=path.resolve(import.meta.dirname,'..');
const output=path.join(root,'results/image-policy');fs.mkdirSync(output,{recursive:true});
function load(file,imports){
 const module={exports:{}};const code=ts.transpileModule(fs.readFileSync(path.join(root,file),'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
 vm.runInNewContext(code,{module,exports:module.exports,Array,Map,require:key=>{if(!(key in imports))throw Error(`unmapped ${key}`);return imports[key];}},{filename:file});return module.exports;
}
const lifecycle=load('source_repo/src/common/Lifecycle.ts',{});
const event=load('source_repo/src/common/Event.ts',{'./Lifecycle':lifecycle});
const unused=class{constructor(){throw Error('DOM/protocol activation is outside this policy oracle');}};
const imports={'common/Event':event,'./ImageStorage':{ImageStorage:unused,CELL_SIZE_DEFAULT:{width:7,height:14}}};
for(const name of ['IIPHandler','ImageRenderer','KittyGraphicsHandler','KittyImageStorage','SixelHandler','SixelImageStorage','IIPImageStorage'])imports[(name.startsWith('Kitty')?'./kitty/':'./')+name]={[name]:unused};
const {ImageAddon}=load('source_repo/addons/addon-image/src/ImageAddon.ts',imports);
const bits=value=>{if(Number.isNaN(value))return'NaN';const bytes=Buffer.alloc(8);bytes.writeDoubleBE(value);return bytes.toString('hex');};
const inputBits=value=>{const bytes=Buffer.alloc(8);bytes.writeDoubleBE(value);return bytes.toString('hex');};
const value=hex=>Buffer.from(hex,'hex').readDoubleBE();
const flags=new Set(['enableSizeReports','showPlaceholder','sixelSupport','sixelScrolling','iipSupport','kittySupport']);
const keys=['enableSizeReports','pixelLimit','storageLimit','showPlaceholder','sixelSupport','sixelScrolling','sixelPaletteLimit','sixelSizeLimit','iipSupport','iipSizeLimit','kittySupport','kittySizeLimit'];
const options=data=>keys.map(key=>flags.has(key)?data[key]:bits(data[key]));
const decode=data=>Object.fromEntries(Object.entries(data??{}).map(([key,item])=>[key,flags.has(key)?item:value(item)]));
const decodeParams=params=>params.map(param=>param[0]==='u'?undefined:param[0]==='n'?value(param[1]):param[1].map(value));
const n=value=>['n',inputBits(value)],a=values=>['a',values.map(inputBits)];
const geometry=(width,height,columns=80,rows=24)=>({canvas:width===undefined?null:[inputBits(width),inputBits(height)],columns:inputBits(columns),rows:inputBits(rows)});
let addon,effects=[],current;
function attach(){
 addon._storage={reset(){effects.push(['clear',bits(addon._opts.sixelPaletteLimit),addon._opts.sixelScrolling]);}};
 addon._handlers=new Map([['recording',{reset(){effects.push(['reset',bits(addon._opts.sixelPaletteLimit),addon._opts.sixelScrolling]);}}]]);
 addon._terminal={cols:80,rows:24,_core:{input(text,user){effects.push(['reply',Array.from({length:text.length},(_,i)=>text.charCodeAt(i)),user]);Object.assign(addon._opts,decode(current.duringReply));if(current.clearHandlersDuringReply)addon._handlers.clear();}}};
 addon._renderer={dimensions:undefined};
}
const operations=[],expected=[];
function operation(op){
 current=op;effects=[];let handled=false;
 switch(op.op){
  case'new':addon=new ImageAddon(decode(op.options));attach();break;
  case'set':Object.assign(addon._opts,decode(op.options));break;
  case'reset':handled=addon.reset();break;
  case'decset':handled=addon._decset(decodeParams(op.params));break;
  case'decrst':handled=addon._decrst(decodeParams(op.params));break;
  case'da':handled=addon._da1(decodeParams(op.params));break;
  case'graphics':{
   addon._terminal.cols=value(op.geometry.columns);addon._terminal.rows=value(op.geometry.rows);
   addon._renderer.dimensions=op.geometry.canvas?{css:{canvas:{width:value(op.geometry.canvas[0]),height:value(op.geometry.canvas[1])}}}:undefined;
   handled=addon._xtermGraphicsAttributes(decodeParams(op.params));break;
  }
  default:throw Error('unknown operation');
 }
 operations.push(op);expected.push({handled,effects,options:options(addon._opts),defaults:options(addon._defaultOpts)});
}
operation({op:'new'});
for(const key of keys){operation({op:'new',options:{[key]:flags.has(key)?false:inputBits(16)}});operation({op:'reset'});}
operation({op:'new',options:{sixelPaletteLimit:inputBits(128),sixelScrolling:false}});
for(const params of [[],[n(0)],[n(-0)],[n(NaN)],[n(1)],[a([])],[a([0])],[['u']]])operation({op:'da',params});
for(const op of ['decset','decrst'])for(const params of [[n(80)],[a([80])],[n(0),n(80)]])operation({op,params});
for(const item of [n(1),n(2),n(3),a([1,2]),['u'],n(NaN)])for(const action of [n(1),n(2),n(3),n(4),n(99),a([1])])for(const third of [undefined,n(4096),n(4097),n(-1),n(1.5),n(NaN),n(-Infinity),a([1])])operation({op:'graphics',params:third?[item,action,third]:[item,action],geometry:geometry(20.5,10.5)});
for(const limit of [-1,0,100,Infinity,NaN]){operation({op:'set',options:{pixelLimit:inputBits(limit)}});for(const dims of [geometry(),geometry(10,10),geometry(11,11),geometry(0,100,0,NaN)])operation({op:'graphics',params:[n(2),n(1)],geometry:dims});}
operation({op:'graphics',params:[n(1),n(2)],geometry:geometry(),duringReply:{sixelPaletteLimit:inputBits(7)}});
operation({op:'graphics',params:[n(1),n(2)],geometry:geometry(),clearHandlersDuringReply:true});operation({op:'reset'});
const seed=0x1a66c51;let state=seed;const random=n=>{state^=state<<13;state^=state>>>17;state^=state<<5;return(state>>>0)%n;};
operation({op:'new'});
for(let index=0;index<2000;index++){
 const choice=random(10);
 if(choice<2)operation({op:choice?'decset':'decrst',params:[random(2)?n(80):a([80])]});
 else if(choice===2)operation({op:'reset'});
 else if(choice===3)operation({op:'da',params:[n(random(3))]});
 else if(choice===4)operation({op:'set',options:{pixelLimit:inputBits([0,100,16777216,NaN,Infinity][random(5)])}});
 else operation({op:'graphics',params:[n(1+random(4)),n(1+random(6)),n(random(8192)-100)],geometry:geometry(random(2000)/2,random(1000)/2,random(100),random(30))});
}
const input=operations.map(operation=>JSON.stringify(operation)).join('\n')+'\n';
fs.writeFileSync(path.join(output,'operations.ndjson'),input);
const build=spawnSync('cargo',['build','-p','xterm-addons','--example','image_policy_trace'],{cwd:root,encoding:'utf8',timeout:120000});assert.equal(build.status,0,build.stderr||build.error?.message);
const executable=path.join(root,'target/debug/examples/image_policy_trace'+(process.platform==='win32'?'.exe':''));
const run=spawnSync(executable,[],{cwd:root,input,encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});assert.equal(run.status,0,run.stderr||run.error?.message);
const actual=run.stdout.trimEnd().split(/\r?\n/).map(line=>JSON.parse(line));assert.equal(actual.length,expected.length);
for(let index=0;index<actual.length;index++)try{assert.deepEqual(actual[index],expected[index]);}catch{fs.writeFileSync(path.join(output,'counterexample.json'),JSON.stringify({seed,index,operation:operations[index],expected:expected[index],actual:actual[index]},null,2)+'\n');throw Error(`policy mismatch ${index}`);}
const summary={scope:'Native options and control policy; DOM activation and real graphics remain parent38',comparisons:actual.length,mismatches:0,seed,randomized:2000};fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2)+'\n');console.log(summary);
