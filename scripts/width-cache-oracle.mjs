// Copyright (c) 2023 The xterm.js authors. All rights reserved. MIT license.
// Run unchanged WidthCache assertions with their original injected canvases.
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url),ts=require('../source_repo/node_modules/typescript');
function load(file,imports={},globals={}){
  const source=fs.readFileSync(`source_repo/src/browser/renderer/${file}`,'utf8');
  const code=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
  const module={exports:{}};
  vm.runInNewContext(code,{module,exports:module.exports,Float32Array,Map,require:id=>{if(!(id in imports))throw Error(`Unmapped source import ${id}`);return imports[id];},...globals},{filename:file});
  return module.exports;
}
const utils=load('shared/RendererUtils.ts');
const source=load('dom/WidthCache.ts',{'../shared/RendererUtils':utils});
const suites=[],tests=[];let assertions=0;
const counted=new Proxy(assert,{get(target,key){const fn=target[key];return typeof fn==='function'?(...args)=>{assertions++;return Reflect.apply(fn,target,args);}:fn;}});
load('dom/WidthCache.test.ts',{'./WidthCache':source,assert:counted},{
  describe(name,callback){suites.push({name,before:[]});callback();suites.pop();},
  beforeEach(callback){suites.at(-1).before.push(callback);},
  it(name,callback){tests.push({name:[...suites.map(s=>s.name),name].join(' / '),before:suites.flatMap(s=>s.before),callback});}
});
for(const test of tests){for(const before of test.before)before();test.callback();}
function fixture(){
  const state={log:[],width:3,configure:null,measure:null};let index=0;
  const cache=new source.WidthCache(()=>{const variant=index++;return{
    setFont(family){state.log.push([family,variant]);state.configure?.(family,variant);},
    measure(){const width=state.width;state.measure?.();return width;}
  };});
  cache.setFont('old',15,'normal','bold');state.log.length=0;return{cache,state};
}
let directed=0;
{
 const{cache,state}=fixture();cache.get('a',false,false);state.configure=(_,variant)=>{if(variant===1)throw Error('font setter');};
 assert.throws(()=>cache.setFont('new',15,'normal','bold'),/font setter/);assert.equal(state.log.length,2);cache.setFont('new',15,'normal','bold');assert.equal(state.log.length,2);assert.equal(cache.get('a',false,false),3);directed++;
}
{
 const{cache,state}=fixture();state.configure=()=>cache.dispose();assert.throws(()=>cache.setFont('new',15,'normal','bold'),{name:'TypeError'});assert.equal(state.log.length,1);directed++;
}
{
 const{cache,state}=fixture();let entered=false;state.configure=(family,variant)=>{if(family==='outer'&&variant===0&&!entered){entered=true;cache.setFont('inner',15,'normal','bold');}};
 cache.setFont('outer',15,'normal','bold');assert.equal(cache._font,'inner');assert.deepEqual(state.log,[['outer',0],['inner',0],['inner',1],['inner',2],['inner',3],['outer',1],['outer',2],['outer',3]]);directed++;
}
{
 const{cache,state}=fixture();state.measure=()=>cache.clear();assert.equal(cache.get('wide',false,false),3);state.measure=null;state.width=8;assert.equal(cache.get('wide',false,false),3);directed++;
}
for(const text of ['a','wide']){
 const{cache,state}=fixture();state.measure=()=>cache.dispose();if(text==='a'){assert.equal(cache.get(text,false,false),3);assert.equal(cache.get(text,false,false),3);}else assert.throws(()=>cache.get(text,false,false),{name:'TypeError'});directed++;
}
console.log(`WidthCache: ${tests.length} original tests/${assertions} assertions and ${directed} callback-state cases passed.`);
