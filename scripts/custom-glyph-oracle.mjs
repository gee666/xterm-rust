// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
// Capture unchanged source drawing calls; the browser bundle is test-only.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const sourceRoot=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const require=createRequire(path.resolve(process.env.REFERENCE_ROOT??sourceRoot,'package.json'));
const {build}=require('esbuild');
const directory=path.join(sourceRoot,'addons/addon-webgl/src/customGlyphs');
const entry=`export * from ${JSON.stringify(path.join(directory,'CustomGlyphDefinitions.ts'))}; export * from ${JSON.stringify(path.join(directory,'CustomGlyphRasterizer.ts'))};`;
const options={stdin:{contents:entry,resolveDir:sourceRoot,loader:'ts'},bundle:true,write:false,metafile:true,
  alias:{browser:path.join(sourceRoot,'src/browser'),common:path.join(sourceRoot,'src/common')}};
const built=await build({...options,format:'cjs',platform:'node'});
const module={exports:{}};
class Matrix { constructor(){this.x=0;this.y=0;} translateSelf(x,y){this.x+=x;this.y+=y;return this;} }
class Image { constructor(width,height){this.width=width;this.height=height;this.data=new Uint8ClampedArray(width*height*4);} }
class Canvas { constructor(w,h){this.width=w;this.height=h;}getContext(){return {putImageData(){}};} }
class Path2D { rect(...rect){this.rectangle=rect;} }
vm.runInNewContext(built.outputFiles[0].text,{module,exports:module.exports,console,OffscreenCanvas:Canvas,ImageData:Image,DOMMatrix:Matrix,Path2D});
const source=module.exports;
function capture(chars,p){
  const trace=[],stack=[];let fill='#e06090',stroke='#000000',width=1,cap='butt';
  const ctx={
    save(){trace.push(['save']);stack.push({fill,stroke,width,cap});},
    restore(){trace.push(['restore']);({fill,stroke,width,cap}=stack.pop());},
    beginPath(){trace.push(['begin']);},closePath(){trace.push(['close']);},
    moveTo(...v){trace.push(['move',...v]);},lineTo(...v){trace.push(['line',...v]);},
    quadraticCurveTo(...v){trace.push(['quadratic',...v]);},bezierCurveTo(...v){trace.push(['cubic',...v]);},
    arc(...v){trace.push(['arc',...v]);},ellipse(...v){trace.push(['ellipse',...v]);},
    fill(){trace.push(['fill']);},stroke(){trace.push(['stroke']);},rect(...v){trace.push(['rect',...v]);},fillRect(...v){trace.push(['fillRect',...v]);},
    clip(path){trace.push(path?['clipRect',...path.rectangle]:['clip']);},
    createPattern(){return {variant:0,setTransform(matrix){this.variant=(-matrix.x<<1)|-matrix.y;}};},
    get fillStyle(){return fill;},set fillStyle(value){fill=value;trace.push(typeof value==='string'?['fillStyle',value]:['pattern',value.variant]);},
    get strokeStyle(){return stroke;},set strokeStyle(value){stroke=value;trace.push(['strokeStyle',value]);},
    get lineWidth(){return width;},set lineWidth(value){width=value;trace.push(['lineWidth',value]);},
    get lineCap(){return cap;},set lineCap(value){cap=value;trace.push(['lineCap',value]);}
  };
  const result=source.tryDrawCustomGlyph(ctx,String.fromCharCode(...chars),...p,{error(message){trace.push(['error',message]);}},'#112233',3);
  return {result,trace};
}
const cases=[];
const geometries=[[2,3,8,16,8,16,15,1],[1.25,2.5,9,17,7.1,13.3,13,1.5],[0,0,15.5,21.5,11.25,17.75,20,2]];
for(const c of Object.keys(source.customGlyphDefinitions))for(const p of geometries){const chars=Array.from({length:c.length},(_,i)=>c.charCodeAt(i));cases.push({chars,p,...capture(chars,p)});}
for(const chars of [[65],[0xd800],[0xdc00],[0x2500,0x301],[],[0xd800,65],...Object.getOwnPropertyNames(Object.prototype).map(s=>[...s].map(c=>c.charCodeAt(0)))])cases.push({chars,p:geometries[0],...capture(chars,geometries[0])});
for(const cp of [0x2500,0x2502,0x2550]){const chars=[cp],p=[0,0,9007199254740992,16,8,16,15,1];cases.push({chars,p,...capture(chars,p)});}
const inputs=[];
for(const input of Object.keys(built.metafile.inputs).filter(p=>p!=='<stdin>').sort())inputs.push({path:path.relative(sourceRoot,input).replaceAll('\\','/'),sha256:createHash('sha256').update(await readFile(input)).digest('hex')});
await writeFile('crates/xterm-web/tests/fixtures/custom-glyphs.json',JSON.stringify({inputs,glyphs:Object.keys(source.customGlyphDefinitions).length,patternCodepoints:[...source.blockPatternCodepoints],cases})+'\n');
await mkdir('tmp/custom-glyphs',{recursive:true});
const browser=await build({...options,format:'iife',globalName:'CustomGlyphSource',platform:'browser'});
await writeFile('tmp/custom-glyphs/source.js',browser.outputFiles[0].text);
await writeFile('tmp/custom-glyphs/source-dom.js','globalThis.OffscreenCanvas = undefined;\n'+browser.outputFiles[0].text);
console.log(cases.length+' source custom-glyph traces; '+Object.keys(source.customGlyphDefinitions).length+' glyphs; test-only browser prelude tmp/custom-glyphs/source.js');
// Run the unchanged two original factory assertions as a separate baseline.
const testFile=path.join(directory,'CustomGlyphRasterizer.test.ts');
const tests=await build({entryPoints:[testFile],bundle:true,write:false,format:'cjs',platform:'node',nodePaths:[path.join(path.resolve(process.env.REFERENCE_ROOT??sourceRoot),'node_modules')],alias:options.alias});
const Mocha=require('mocha');
const mocha=new Mocha({reporter:'spec',timeout:5000});
const context={module:{exports:{}},console,process,EventTarget,Event,setTimeout,clearTimeout};
mocha.suite.emit('pre-require',context,testFile,mocha);
vm.runInNewContext(tests.outputFiles[0].text,context,{filename:testFile});
let runner;
const failures=await new Promise(resolve=>{runner=mocha.run(resolve);});
const original={test:'addons/addon-webgl/src/customGlyphs/CustomGlyphRasterizer.test.ts',sha256:createHash('sha256').update(await readFile(testFile)).digest('hex'),discovered:runner.stats.tests,executed:runner.stats.tests-runner.stats.pending,passed:runner.stats.passes,failed:failures,skipped:runner.stats.pending,unrun:0,inputs};
await mkdir('results/custom-glyphs',{recursive:true});
await writeFile('results/custom-glyphs/original.json',JSON.stringify(original,null,2)+'\n');
if(failures)process.exitCode=1;
