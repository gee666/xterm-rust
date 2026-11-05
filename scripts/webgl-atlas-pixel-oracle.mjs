// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Export private routines in a virtual entry only. Original source stays unchanged.
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import path from 'node:path';
import vm from 'node:vm';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const file=path.join(root,'addons/addon-webgl/src/TextureAtlas.ts');
const source=await readFile(file,'utf8');
const require=createRequire(path.join(root,'package.json'));
const {build}=require('esbuild');
const wrapper='\nexport { clearColor, checkCompletelyTransparent };\n';
const built=await build({stdin:{contents:source+wrapper,resolveDir:path.dirname(file),sourcefile:file,loader:'ts'},
  bundle:true,write:false,platform:'node',format:'cjs',metafile:true,
  alias:{browser:path.join(root,'src/browser'),common:path.join(root,'src/common')}});
const module={exports:{}};
vm.runInNewContext(built.outputFiles[0].text,{module,exports:module.exports,process,console,Uint8ClampedArray},{filename:file});
const {TextureAtlas,clearColor,checkCompletelyTransparent}=module.exports;
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputs=[{path:path.relative(root,file).replaceAll('\\','/'),sha256:sha(await readFile(file))}];
for(const input of Object.keys(built.metafile.inputs).sort()) {
  if(input==='<stdin>'||path.resolve(input)===file)continue;
  inputs.push({path:path.relative(root,input).replaceAll('\\','/'),sha256:sha(await readFile(input))});
}
const cases=[];
function run(name,width,height,pixels,scan,bg,fg,threshold){
  const input={data:new Uint8ClampedArray(pixels)};
  const ctx={_tmpCanvas:{width,height},_config:{deviceCellWidth:scan.cell[0],deviceCellHeight:scan.cell[1],
    deviceCharWidth:scan.character[0],deviceCharHeight:scan.character[1],lineHeight:scan.lineHeight}};
  const bounds=image=>{
    const box={};
    const glyph=TextureAtlas.prototype._findGlyphBoundingBox.call(ctx,image,box,scan.allowedWidth,scan.restricted,scan.custom,scan.padding);
    return {box,glyph};
  };
  const beforeTransparent=checkCompletelyTransparent(input),beforeBounds=bounds(input);
  const data=new Uint8ClampedArray(pixels);
  const empty=clearColor({data},{rgba:bg},{rgba:fg},threshold);
  cases.push({name,width,height,pixels,scan,bg,fg,threshold,beforeTransparent,beforeBounds,
    empty,cleared:[...data],afterTransparent:checkCompletelyTransparent({data}),afterBounds:bounds({data})});
}
const scan=(width,height,extra={})=>({allowedWidth:width,padding:0,restricted:false,custom:false,
  cell:[width,height],character:[width,height],lineHeight:1,...extra});
run('transparent-non-background-is-not-color-empty',1,1,[255,0,0,0],scan(1,1),0x000000ff,0xffffffff,false);
run('exact-rgb-retains-rgb-clears-alpha',1,1,[32,48,64,255],scan(1,1),0x20304000,0xffffffff,false);
for(const enabled of [false,true]) for(const difference of [62,63,64]) {
  run('strict-threshold-'+enabled+'-'+difference,1,1,[difference,0,0,255],scan(1,1),0x000000ff,0xffffffff,enabled);
}
for(const extra of [{},{restricted:true},{custom:true}]) {
  const pixels=Array(8*6*4).fill(0);
  for(const [x,y]of [[0,2],[6,3]])pixels[(y*8+x)*4+3]=255;
  run('asymmetric-'+JSON.stringify(extra),8,6,pixels,scan(5,6,{padding:2,cell:[5,4],character:[6,7],lineHeight:1.5,...extra}),0x000000ff,0xffffffff,false);
}
run('empty-default-inclusive-bounds',3,2,Array(24).fill(0),scan(2,2,{padding:1}),0x000000ff,0xffffffff,false);
run('padding-only-negative-width',3,1,[0,0,0,0,0,0,0,0,0,0,0,255],scan(0,1,{padding:3,cell:[0,1]}),0,0,true);
run('js-round-large-integer',1,1,[255,255,255,255],scan(1,1,{custom:true,cell:[1,9007199254740994],character:[1,0],lineHeight:1.5}),0x000000ff,0xffffffff,false);
run('grown-canvas-padded-row-crossing',4,2,[0,0,0,255,255,255,255,255,...Array(24).fill(0)],scan(4,2,{padding:4}),0x000000ff,0xffffffff,false);
run('zero-size-empty-bounds',0,0,[],scan(0,0),0,0,true);
let state=0x671ba92d;
function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
for(let i=0;i<512;i++) {
  const width=random()%12+1,height=random()%12+1,padding=random()%Math.min(3,width+1),allowedWidth=random()%(width-padding+1);
  const bg=random(),fg=random();
  const background=[bg>>>24,(bg>>>16)&255,(bg>>>8)&255];
  const pixels=[];
  for(let p=0;p<width*height;p++) {
    const kind=random()%5;
    const rgb=kind===0?background:kind===1?background.map(v=>Math.min(255,Math.max(0,v+(random()%15)-7))):[random()%256,random()%256,random()%256];
    pixels.push(...rgb,kind===4?0:random()%256);
  }
  const s=scan(allowedWidth,height,{padding,restricted:!!(random()%2),custom:!!(random()%2),
    cell:[random()%(width-padding+1),random()%(height+1)],character:[(random()%32)/2,(random()%32)/2],lineHeight:random()%2?1:1.25});
  run('seeded-'+i,width,height,pixels,s,bg,fg,!!(random()%2));
}
// Keep the original527 cases unchanged, then extend the stream for grown
// canvases where the source's padded horizontal scan exceeds its row width.
for(let i=0;i<128;i++) {
  const width=random()%12+1,height=random()%8+1,padding=random()%4+1;
  const pixels=Array.from({length:width*height*4},()=>random()%256);
  run('grown-seeded-'+i,width,height,pixels,scan(width,height,{padding,custom:!!(random()%2),lineHeight:1.25}),
      0x000000ff,0xffffffff,!!(random()%2));
}
const output=path.resolve('crates/xterm-web/tests/fixtures/webgl-atlas-pixels.json');
await writeFile(output,JSON.stringify({seed:'0x671ba92d',privateExportWrapper:wrapper,inputs,cases})+'\n');
console.log(cases.length+' unchanged-source atlas pixel cases -> '+output);
