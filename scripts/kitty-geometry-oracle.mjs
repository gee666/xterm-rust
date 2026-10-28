// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Execute unchanged arithmetic blocks from _decodeAndDisplay. Bitmap/canvas
// pixels and WebIDL conversion are separate browser integration tests.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import vm from 'node:vm';
import { eraseModuleBoundaries } from './parser-oracle-common.mjs';
const handlerPath = 'source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts';
const source = (await readFile(handlerPath, 'utf8')).replace(/\r\n/g, '\n');
const types = await readFile('source_repo/addons/addon-image/src/kitty/KittyGraphicsTypes.ts', 'utf8');
const storage = await readFile('source_repo/addons/addon-image/src/ImageStorage.ts', 'utf8');
function block(start, end, after = 0) {
  const first = source.indexOf(start, after); assert.ok(first >= 0, start);
  const last = source.indexOf(end, first + start.length); assert.ok(last > first, end);
  return source.slice(first, last);
}
const defaults = storage.match(/export const CELL_SIZE_DEFAULT: ICellSize = \{[\s\S]*?\};/)?.[0];
assert.ok(defaults, 'source cell defaults');
const context = vm.createContext({});
vm.runInContext(eraseModuleBoundaries(types.replace(/^import .*;\r?$/gm, '')) + '\nglobalThis.parse = parseKittyCommand;', context);
vm.runInContext(eraseModuleBoundaries(defaults), context);
const crop = block('const cropX =', '      if (cropX !==');
const cropRequired = source.match(/if \((cropX !==[\s\S]*?)\) \{/)?.[1]; assert.ok(cropRequired);
const layout = block('const cw = this._renderer.dimensions?.css.cell.width', '      // Save cursor position');
const layer = block('const wantsBottom =', '      if (w !== bitmap.width');
const offset = block('const xOffset =', '      if (xOffset !==');
const canvas = block('const canvasW =', '        const offsetCanvas =');
const afterOffset = source.indexOf('bitmap = offsetBitmap;');
const complete = block('w = bitmap.width;', '      }\n\n      const zIndex =', afterOffset);
const cursor = block('if (cmd.cursorMovement === 1)', '    } catch (e)', source.indexOf('// Kitty cursor movement'));
function compile(name, body) {
  vm.runInContext(eraseModuleBoundaries(`globalThis.${name} = function(input,cmd) { ${body} };`), context);
}
compile('crop', `const bitmap=input.bitmap; ${crop} return {x:cropX,y:cropY,width:finalCropW,height:finalCropH,required:${cropRequired}};`);
compile('layout', `const bitmap=input.bitmap; ${layout} ${layer} return {cw,ch,columns:imgCols,rows:imgRows,width:w,height:h,layer};`);
compile('offset', `const bitmap=input.bitmap; const {cw,ch,columns:imgCols,rows:imgRows}=input.layout; ${offset}
if(xOffset!==0||yOffset!==0){${canvas} return {x:xOffset,y:yOffset,width:canvasW,height:canvasH};} return null;`);
compile('complete', `const bitmap=input.bitmap; const {cw,ch}=input.layout; let {columns:imgCols,rows:imgRows,width:w,height:h}=input.layout; ${complete} return {cw,ch,columns:imgCols,rows:imgRows,width:w,height:h,layer:input.layout.layer};`);
compile('cursor', `const {savedX,savedY,savedYbase,imgCols}=input; const buffer={x:input.currentX,y:input.currentY,ybase:input.currentYbase}; ${cursor} return {x:buffer.x,y:buffer.y};`);
const encode = value => Number.isNaN(value) ? 'nan' : (() => { const data=Buffer.alloc(8);data.writeDoubleBE(value);return data.toString('hex'); })();
const decode = value => value === 'nan' ? NaN : Buffer.from(value,'hex').readDoubleBE();
function numbers(value) {
  if (value === null) return null;
  return Object.fromEntries(Object.entries(value).map(([key,value]) => [key, typeof value === 'number' ? encode(value) : value]));
}
const values = [0,-0,-1,0.5,1,3,7,14,NaN,Infinity,-Infinity];
const controls = ['', 'x=10,y=20,w=50,h=30', 'x=-10,y=-20,w=-1,h=0', 'x=999,y=999', 'c=3', 'r=2', 'c=0,r=0', 'c=-1,r=3', 'X=3,Y=4', 'c=4,X=100,Y=100', 'z=-1,C=1', 'c=1,r=1,z=2', 'x=,w=,h=', 'c=,r=2'];
const sizes = [{width:1,height:1},{width:200,height:100},{width:0,height:100},{width:200,height:0},{width:7,height:14}];
const rows=[];
function trace(kind, input, control, overrides={}) {
  const command=context.parse(control);
  for(const [key,value] of Object.entries(overrides)) command[key]=decode(value);
  const host={_renderer:{dimensions:{css:{cell:input.cell??{width:7,height:14}}}},_opts:{pixelLimit:input.limit??16777216},_coreTerminal:{cols:input.columns??80}};
  let actual,error;
  try{actual=context[kind].call(host,input,command);}catch(caught){error=caught.message;}
  const encodedInput=structuredClone(input);
  if(encodedInput.cell) encodedInput.cell=numbers(encodedInput.cell);
  if(encodedInput.limit!==undefined) encodedInput.limit=encode(encodedInput.limit);
  if(encodedInput.layout) encodedInput.layout=numbers(encodedInput.layout);
  for(const key of ['savedX','savedY','savedYbase','imgCols','currentX','currentY','currentYbase','columns']) if(encodedInput[key]!==undefined)encodedInput[key]=encode(encodedInput[key]);
  rows.push({kind,input:encodedInput,control,overrides,expected:error?{error}:{value:numbers(actual)}});
}
for(const control of controls)for(const bitmap of sizes)trace('crop',{bitmap},control);
for(const control of controls)for(const bitmap of sizes)for(const cell of [{width:7,height:14},{width:0.5,height:1.5},{width:0,height:NaN},{width:-1,height:Infinity}])for(const limit of [0,100,20000,NaN,Infinity])trace('layout',{bitmap,cell,limit},control);
for(const control of controls)for(const bitmap of sizes)for(const cell of [{width:7,height:14},{width:0.5,height:1.5},{width:-1,height:-2}]){
 const layout={cw:cell.width,ch:cell.height,columns:3,rows:2,width:21,height:28,layer:'bottom'};
 trace('offset',{bitmap,layout},control);
 trace('complete',{bitmap,layout,limit:20000},control);
}
for(const control of controls)for(const currentYbase of [0,5,20])for(const imgCols of [-1,0,3,NaN,Infinity])trace('cursor',{savedX:78,savedY:10,savedYbase:5,currentX:1,currentY:20,currentYbase,imgCols,columns:80},control);
const seed=0x6b676733;let state=seed;function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
const keys=['x','y','sourceWidth','sourceHeight','xOffset','yOffset','columns','rows','zIndex','cursorMovement'];
for(let index=0;index<512;index++){
 const overrides={};for(let count=0;count<4;count++)overrides[keys[random()%keys.length]]=encode(values[random()%values.length]);
 const bitmap=sizes[random()%sizes.length],cell={width:values[random()%values.length],height:values[random()%values.length]},limit=values[random()%values.length];
 const layout={cw:cell.width,ch:cell.height,columns:values[random()%values.length],rows:values[random()%values.length],width:21,height:28,layer:random()%2?'top':'bottom'};
 trace('crop',{bitmap},'',overrides);trace('layout',{bitmap,cell,limit},'',overrides);trace('offset',{bitmap,layout},'',overrides);trace('complete',{bitmap,layout,limit},'',overrides);
}
const text=JSON.stringify(rows)+'\n', fixture='crates/xterm-addons/tests/fixtures/kitty-geometry.json';
assert.ok(Buffer.byteLength(text)<20*1024*1024);
if(process.argv.includes('--generate'))await writeFile(fixture,text);else assert.equal((await readFile(fixture,'utf8')).replace(/\r\n/g,'\n'),text,'Kitty geometry source changed');
await mkdir('results/kitty-graphics',{recursive:true});
await writeFile('results/kitty-graphics/geometry-summary.json',JSON.stringify({seed,cases:rows.length,source:handlerPath,scope:'Source arithmetic stages only; no platform bitmap conversion or browser pixel claim'},null,2)+'\n');
console.log(`Kitty geometry: ${rows.length} source arithmetic cases, seed ${seed}.`);
