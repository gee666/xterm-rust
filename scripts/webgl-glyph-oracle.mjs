// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Run unchanged GlyphRenderer.ts with an atlas provider and record its vertices.
import {readFile, writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const sourceRoot = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const require = createRequire(path.join(sourceRoot, 'package.json'));
const esbuild = require('esbuild');
const file = path.join(sourceRoot, 'addons/addon-webgl/src/GlyphRenderer.ts');
const built = await esbuild.build({entryPoints: [file], bundle: true, write: false, format: 'cjs',
  platform: 'node', metafile: true, alias: {browser: path.join(sourceRoot, 'src/browser'), common: path.join(sourceRoot, 'src/common')}});
const module = {exports: {}};
vm.runInNewContext(built.outputFiles[0].text, {module, exports: module.exports, Float32Array, Int32Array, Uint8Array, console, process}, {filename: file});
const {GlyphRenderer} = module.exports;
const inputs = [];
for (const input of Object.keys(built.metafile.inputs).sort()) {
  inputs.push({path: path.relative(sourceRoot, input).replaceAll('\\', '/'),
    sha256: createHash('sha256').update(await readFile(input)).digest('hex')});
}
function recorder() {
  const gl = {ARRAY_BUFFER:1, ELEMENT_ARRAY_BUFFER:2, STATIC_DRAW:3, STREAM_DRAW:4,
    FLOAT:5, TRIANGLE_STRIP:6, UNSIGNED_BYTE:7, VERTEX_SHADER:8, FRAGMENT_SHADER:9,
    COMPILE_STATUS:10, LINK_STATUS:11, MAX_TEXTURE_IMAGE_UNITS:12, MAX_TEXTURE_SIZE:13,
    createProgram:()=>({}), createShader:()=>({}), createVertexArray:()=>({}), createBuffer:()=>({}), createTexture:()=>({}),
    getProgramParameter:()=>true, getShaderParameter:()=>true, getUniformLocation:()=>({}),
    getParameter(key) { return key === this.MAX_TEXTURE_IMAGE_UNITS ? 8 : 4096; },
    bufferData(target, values) { if (target === this.ARRAY_BUFFER) this.last = [...values]; },
    drawElementsInstanced(_mode,_count,_type,_offset,count) { this.count = count; }};
  for (const name of ['attachShader','shaderSource','compileShader','linkProgram','deleteProgram',
    'deleteShader','deleteVertexArray','deleteBuffer','deleteTexture','bindVertexArray','bindBuffer','enableVertexAttribArray',
    'vertexAttribPointer','vertexAttribDivisor','useProgram','uniformMatrix4fv','uniform1iv','uniform2f','viewport',
    'activeTexture','bindTexture','texParameteri','texImage2D','enable','blendFunc']) gl[name]=()=>{};
  return gl;
}
const utf16 = text => Array.from({length:text.length}, (_,i) => text.charCodeAt(i));
const bits = values => [...new Uint32Array(new Float32Array(values).buffer)];
const cases=[];
const glyph=(size=[21,12], offset=[5,3], page=0) => ({offset:{x:offset[0],y:offset[1]},size:{x:size[0],y:size[1]},texturePage:page,
  texturePositionClipSpace:{x:0.125,y:0.25},sizeClipSpace:{x:size[0]/64,y:size[1]/64}});
const cell=(code=65, extra={}) => ({x:0,y:0,code,bg:7,fg:3,ext:9,chars:'A',width:1,lastBg:7,glyph:glyph(),...extra});
const dimensions=(cols,rows)=>({device:{cell:{width:8.25,height:16.5},canvas:{width:cols*8.25,height:rows*16.5},
  char:{width:7.1,height:15,left:0.25,top:1.5}}});
function run(name,cols,rows,updates,rescale=true,installed=true,lineLengths=Array(rows).fill(cols),resize=null) {
  const gl=recorder(), terminal={cols,rows,element:{}}, calls=[];
  let current;
  const atlas={pages:[{canvas:{width:64},version:1},{canvas:{width:128},version:2}],pageLayoutVersion:0,
    getRasterizedGlyph(...args) { calls.push(['single',...args.slice(0,5)]);return current; },
    getRasterizedGlyphCombinedChar(...args) { calls.push(['combined',...args.slice(0,5)]);return current; }};
  const d=dimensions(cols,rows);
  const renderer=new GlyphRenderer(terminal,gl,d,{rawOptions:{rescaleOverlappingGlyphs:rescale}},{error(m){throw Error(m);},warn(m){throw Error(m);}});
  if(installed) renderer.setAtlas(atlas);
  const frames=[renderer.beginFrame(),renderer.beginFrame()];
  atlas.pageLayoutVersion++;
  frames.push(renderer.beginFrame(),renderer.beginFrame());
  const snapshots=[];
  for (const c of updates) {
    current=c.glyph;
    renderer.updateCell(c.x,c.y,c.code === null ? undefined : c.code,c.bg,c.fg,c.ext,c.chars,c.width,c.lastBg);
    snapshots.push(bits(renderer._vertices.attributes));
  }
  if (resize) { terminal.cols=resize[0];terminal.rows=resize[1];renderer.handleResize(); }
  const attributes=bits(renderer._vertices.attributes);
  renderer.render({lineLengths});
  const upload=installed?bits(gl.last):null, count=installed?gl.count:null;
  renderer.clear();
  const cleared=bits(renderer._vertices.attributes);
  renderer.dispose();
  cases.push({name,cols,rows,dimensions:d,updates:updates.map(c=>({...c,chars:utf16(c.chars)})),rescale,installed,lineLengths,resize,frames,snapshots,attributes,upload,count,cleared,
    calls:calls.map(c=>c[0]==='combined'?[c[0],utf16(c[1]),...c.slice(2)]:c)});
}
run('null-keeps-texture-height',2,1,[cell(),cell(0),cell(null)]);
run('different-background-clips',2,1,[cell(256,{bg:8,chars:'Ā'})]);
run('no-atlas-null-clears',2,1,[cell(),cell(0)],true,false);
run('utf16-routing',4,1,[cell(0x1f680,{chars:'🚀'}),cell(-2147483583,{x:1,chars:'Á'}),cell(65,{x:2,chars:''}),cell(65,{x:3})]);
run('ucs2-surrogates',3,1,[cell(0xd800,{chars:'\ud800'}),cell(0xdc00,{x:1,chars:'\udc00'}),cell(-2147428288,{x:2,chars:'\ud800A'})]);
run('compact-prefixes',3,2,[cell(65,{x:2}),cell(65,{x:1,y:1})],false,true,[1,2]);
run('reshape-equal-count',3,2,[cell()],true,true,[2,2,2],[2,3]);
run('resize-different-count',3,2,[cell()],true,true,[1],[1,1]);
for(const code of [255,256,0x2500,0x2600,0x2700,0xe000,0xe0a4,0xf8ff,0x1f600,0x1fa00,-2147483583])
  run('rescale-'+code,2,1,[cell(code,{chars:'x'})]);
let seed=0x61ab9423;
function random(){seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;}
for(let i=0;i<128;i++){
  const cols=random()%8+1,rows=random()%4+1,updates=[];
  for(let n=0;n<20;n++){
    const page=random()%2;
    const g=glyph([random()%40,random()%32],[(random()%40-10)/2,(random()%40-10)/2],page);
    const c=cell([65,0,256,0x2500,0x1f600,-2147483583,null][random()%7],{
      x:random()%cols,y:random()%rows,bg:random()%3,lastBg:random()%3,width:random()%3,
      chars:['A','Á','🚀',''][random()%4],glyph:g});
    updates.push(c);
  }
  run('seeded-'+i,cols,rows,updates,!!(random()%2),true,Array.from({length:rows},()=>random()%(cols+1)));
}
const output=path.resolve('crates/xterm-web/tests/fixtures/webgl-glyphs.json');
await writeFile(output,JSON.stringify({seed:'0x61ab9423',inputs,cases})+'\n');
console.log(cases.length+' source GlyphRenderer cases -> '+output);
