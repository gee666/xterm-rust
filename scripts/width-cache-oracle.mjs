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
console.log(`WidthCache: ${tests.length} original tests, ${assertions} assertions passed.`);
