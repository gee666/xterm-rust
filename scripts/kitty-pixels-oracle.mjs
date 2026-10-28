// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Run the original raw-pixel method with real Chromium ImageData/ImageBitmap.
// Capture ImageData before platform premultiplication; rendered pixels are a
// separate live-addon/browser acceptance gate.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { eraseModuleBoundaries } from './parser-oracle-common.mjs';
const require = createRequire(import.meta.url);
const { chromium } = require('../source_repo/node_modules/playwright');
const source = (await readFile('source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts','utf8')).replace(/\r\n/g,'\n');
const start=source.indexOf('private async _createBitmap('),end=source.indexOf('  private async _decompressZlib',start);
assert.ok(start>=0&&end>start);
const types=await readFile('source_repo/addons/addon-image/src/kitty/KittyGraphicsTypes.ts','utf8');
const method=eraseModuleBoundaries(types.replace(/^import .*;\r?$/gm,'')+`\nclass Probe { ${source.slice(start,end)} }\nglobalThis.Probe=Probe;`);
const encode=value=>{const bytes=Buffer.alloc(8);bytes.writeDoubleBE(value);return bytes.toString('hex');};
const cases=[];
for(const format of [24,32,0,NaN])for(let pixels=1;pixels<=20;pixels++){
 const channels=format===32?4:3;
 const bytes=Array.from({length:pixels*channels+2},(_,i)=>(i*37+pixels)&255);
 cases.push({format:encode(format),width:encode(pixels),height:encode(1),bytes});
 cases.push({format:encode(format),width:encode(pixels),height:encode(1),bytes:bytes.slice(0,pixels*channels-1)});
}
for(const format of [24,32])for(const [width,height,length] of [[0,1,4],[1,0,4],[-1,1,4],[-1,-1,4],[NaN,1,4],[Infinity,1,4],[1.01,1,5],[1.1,1.1,5],[1.25,1,5],[2.01,1,9],[0.99,1,4]]){
 cases.push({format:encode(format),width:encode(width),height:encode(height),bytes:Array.from({length},(_,i)=>i+1)});
}
const directory=path.resolve('tmp/kitty-pixel-browser');
await mkdir('tmp',{recursive:true});
let browser;
try{
 browser=await chromium.launchPersistentContext(directory,{channel:'chrome',headless:true});
 const page=await browser.newPage();
 await page.addScriptTag({content:method});
 const rows=await page.evaluate(async cases=>{
  const decode=text=>new DataView(Uint8Array.from(text.match(/../g),byte=>parseInt(byte,16)).buffer).getFloat64(0);
  const original=globalThis.createImageBitmap;
  const output=[];
  let captured;
  globalThis.createImageBitmap=(...args)=>{
   if(args[0] instanceof ImageData)captured={width:args[0].width,height:args[0].height,data:Array.from(args[0].data)};
   return original(...args);
  };
  try{
   for(const input of cases){
    captured=undefined;let error;
    try{
     const bitmap=await new Probe()._createBitmap({data:new Blob([Uint8Array.from(input.bytes)]),width:decode(input.width),height:decode(input.height),format:decode(input.format),compression:''});
     bitmap.close();
    }catch(caught){error={name:caught.name,message:caught.message};}
    output.push({...input,accepted:!error,pixels:captured??null,error:error??null});
   }
  }finally{globalThis.createImageBitmap=original;}
  return output;
 },cases);
 const fixture='crates/xterm-addons/tests/fixtures/kitty-pixels.json',text=JSON.stringify(rows)+'\n';
 if(process.argv.includes('--generate'))await writeFile(fixture,text);else assert.equal((await readFile(fixture,'utf8')).replace(/\r\n/g,'\n'),text,'Kitty pixel oracle drift');
 await mkdir('results/kitty-graphics',{recursive:true});
 await writeFile('results/kitty-graphics/pixels-summary.json',JSON.stringify({browser:browser.browser()?.version(),cases:rows.length,accepted:rows.filter(row=>row.accepted).length,scope:'Original pre-bitmap raw ImageData pixels; not live addon rendering'},null,2)+'\n');
 console.log(`Kitty raw pixels: ${rows.length} Chromium source cases.`);
}finally{if(browser)await browser.close();await rm(directory,{recursive:true,force:true});}
