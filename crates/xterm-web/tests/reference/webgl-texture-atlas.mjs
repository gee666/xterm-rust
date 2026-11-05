// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Appended only to an in-memory bundle of the unchanged TextureAtlas.ts.
import {ColorContrastCache} from 'browser/ColorContrastCache';
import {UnicodeService} from 'common/services/UnicodeService';
import {UnicodeV6} from 'common/input/UnicodeV6';
const makeColor = value => channels.toColor(value >>> 24, value >>> 16 & 255, value >>> 8 & 255, value & 255);
function hashPixels(canvas) {
  const data = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
  let hash = 2166136261, nonzero = 0;
  for (let i = 0; i < data.length; i++) { hash = Math.imul(hash ^ data[i], 16777619) >>> 0; if ((i & 3) === 3 && data[i]) nonzero++; }
  return {hash, nonzero};
}
function glyphSnapshot(g) {
  return {page:g.texturePage,position:[g.texturePosition.x,g.texturePosition.y],
    texture:[g.texturePositionClipSpace.x,g.texturePositionClipSpace.y],
    size:[g.size.x,g.size.y],offset:[g.offset.x,g.offset.y],textureSize:[g.sizeClipSpace.x,g.sizeClipSpace.y]};
}
export function createSourceAtlas(spec) {
  AtlasPage.nextVersion=0;
  TextureAtlas.maxAtlasPages=spec.maxPages;TextureAtlas.maxTextureSize=spec.maxTexture;
  const palette=Array.from({length:256},(_,i)=>makeColor(((i*65793)<<8|255)>>>0));
  const colors={foreground:makeColor(0xeeeeeeff),background:makeColor(0x102030ff),ansi:palette,
    contrastCache:new ColorContrastCache(),halfContrastCache:new ColorContrastCache()};
  for (const seed of spec.contrastSeeds??[]) {
    const cache=seed.half?colors.halfContrastCache:colors.contrastCache;
    cache.setColor(seed.bg,seed.fg,seed.rgba===null?null:makeColor(seed.rgba));
  }
  const config={devicePixelRatio:spec.dpr,deviceMaxTextureSize:spec.maxTexture,
    deviceCellWidth:spec.cellWidth,deviceCellHeight:spec.cellHeight,deviceCharWidth:spec.charWidth,deviceCharHeight:spec.charHeight,
    fontSize:spec.fontSize,fontFamily:spec.fontFamily,fontWeight:400,fontWeightBold:700,letterSpacing:0,lineHeight:spec.lineHeight,
    allowTransparency:spec.transparency,drawBoldTextInBrightColors:true,minimumContrastRatio:spec.ratio,customGlyphs:spec.custom,colors};
  const unicode=new UnicodeService();unicode.register(new UnicodeV6());
  const log={debug(){},warn(){},error(){}};
  return new TextureAtlas(document,config,unicode,log);
}
globalThis.createSourceAtlas=createSourceAtlas;
globalThis.atlasGlyphSnapshot=glyphSnapshot;
export function atlasScenario(name) {
  const spec={dpr:1,cellWidth:9,cellHeight:18,charWidth:8,charHeight:16,fontSize:15,
    lineHeight:1,fontFamily:'monospace',transparency:false,custom:true,ratio:1,maxPages:8,maxTexture:2048,steps:[]};
  const glyph=(value,bg=0,fg=0,ext=0,restrict=false)=>spec.steps.push({value:typeof value==='string'?Array.from({length:value.length},(_,i)=>value.charCodeAt(i)):value,bg:bg>>>0,fg:fg>>>0,ext:ext>>>0,restrict});
  if (name==='signedContrast'||name==='signedHalfContrast'||name==='signedGpuContrast') {
    spec.ratio=7;
    const half=name==='signedHalfContrast',dim=half?0x08000000:0;
    spec.contrastSeeds=[];
    for(const [bg,fg,rgba] of [
      [-2147483648+dim,-2147483648,0xff0000ff],
      [2147483648+dim,-2147483648,0x00ff00ff],
      [-2147483648+dim,2147483648,0x0000ffff],
      [2147483648+dim,2147483648,null]]) {
      spec.contrastSeeds.push({half,bg,fg,rgba});
      spec.steps.push({value:77,bg,fg,ext:0,restrict:false});
    }
    // Different glyphs exercise the shared color cache rather than glyph hits.
    if(name!=='signedGpuContrast') {
      for(const seed of spec.contrastSeeds)spec.steps.push({value:[103,103],bg:seed.bg,fg:seed.fg,ext:0,restrict:false});
      spec.steps.push({value:65,bg:-2147483647+dim,fg:0x83080808|0,ext:0,restrict:false});
      spec.steps.push({value:65,bg:2147483649+dim,fg:0x83080808,ext:0,restrict:false});
    }
  } else if (name.startsWith('styles')) {
    spec.dpr=name==='styles2'?2:1;
    if (spec.dpr===2) {spec.cellWidth=19;spec.cellHeight=38;spec.charWidth=18;spec.charHeight=32;spec.lineHeight=1.25;}
    spec.transparency=name==='stylesTransparent';
    for (const code of [65,32,95,103,0,0xd800,0xe0b0,0xe0a4,0x2588,0x2500,0x2592,0x2801]) glyph(code);
    for (const text of ['fi','=>','a\u0301','😀','\ud800X','']) glyph(text);
    for (const flags of [0x08000000,0x04000000,0x10000000,0x40000000,0x80000000]) glyph(103,0,flags);
    for (const bg of [0x04000000,0x08000000,0x40000000,0x0c000000]) glyph(103,bg);
    for (let style=1;style<=5;style++) for (const restrict of [false,true]) {
      glyph('gg',0x10000000,0x10000000,(style<<26)|0x0300ff00,restrict);
      glyph(0x2588,0x10000000,0x10000000,(style<<26)|(3<<29),restrict);
    }
    glyph(103,0x40000000,0x90000000,(4<<26));
    glyph(65,0x02000002,0x0a000003);
    glyph(65,0x03123456,0x03abcdef);
    // First draw wins despite different restriction and parent container.
    glyph(103,0,0x10000000,0,true);glyph(103,0,0x10000000,0,false);
  } else if (name==='signedCache'||name==='signedCombinedCache') {
    const value=name==='signedCache'?103:'gg';
    const raw=(bg,fg,ext,restrict=false)=>spec.steps.push({value:typeof value==='string'?Array.from(value,c=>c.charCodeAt(0)):value,bg,fg,ext,restrict});
    const base=[0,0,0],high=0x80000000,signed=-0x80000000;
    for(let field=0;field<3;field++) {
      const words=base.slice();words[field]=high;raw(...words);
      words[field]=signed;raw(...words);
      words[field]=high;raw(...words,true);
      words[field]=signed;raw(...words,true);
    }
    raw(0,0,0);raw(-0,0,0,true);raw(0,-0,0);raw(0,0,-0);
    // Clear invalidates the old aliases; repeated fresh keys must be reused.
    spec.steps.push({clear:true});
    raw(high,0,0);raw(signed,0,0);raw(high,0,0,true);raw(signed,0,0,true);
  } else if (name==='noCustom') {
    spec.custom=false;
    for (const code of [0x2588,0x2500,0xe0b0,0x2801,103,95]) glyph(code,0,0x10000000);
  } else if (name==='contrast') {
    spec.ratio=7;
    for (const bg of [0,0x08000000,0x03121212]) for (const fg of [0x03808080,0x0b202020,0x04000000]) glyph(77,bg,fg);
  } else if (name==='merge'||name==='evict'||name==='reentry'||name==='nested') {
    if(name==='reentry')spec.reenter=true;
    if(name==='nested')spec.nested=true;
    Object.assign(spec,{cellWidth:128,cellHeight:128,charWidth:128,charHeight:128,fontSize:120,maxPages:4});
    if(name==='evict')spec.maxTexture=512;
    for(let i=0;i<48;i++)glyph(0x2588,0,0x03000000|(i+1)*1037);
  } else if(name==='overflow') {
    for(const text of ['W'.repeat(100),'='.repeat(100),'W'.repeat(100)])glyph(text);
    glyph(65);glyph('ffi');
  } else if(name==='seeded') {
    spec.seed='0x31e0a4';
    let seed=0x31e0a4;
    const random=()=>{seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;};
    const codes=[65,103,95,0x2588,0x2592,0x2801,0xe0b0,0x1f680,0xd800];
    const text=['fi','=>','a\u0301','😀'];
    for(let i=0;i<192;i++) {
      const value=i%7===0?text[random()%text.length]:codes[random()%codes.length];
      const bg=(random()&0xffffff)|0x03000000|[0,0x04000000,0x08000000,0x40000000][random()%4];
      const fg=(random()&0xffffff)|0x03000000|[0,0x08000000,0x10000000,0x80000000][random()%4];
      glyph(value,bg,fg,((random()%6)<<26)|((random()%8)<<29),!!(random()%2));
    }
  } else if(name==='warmup') {
    spec.warmup=true;
    for(let code=33;code<126;code++)glyph(code);
  } else if(name==='clear') {
    glyph(65);glyph(95);spec.steps.push({clear:true});glyph(65);glyph(95);
    spec.steps.push({clear:true});spec.steps.push({clear:true});glyph(103);
  } else throw new Error(`Unknown atlas scenario ${name}`);
  const atlas=createSourceAtlas(spec);
  const element=document.createElement('div');document.body.append(element);
  let cleared=false,nested=false,nestedGlyph=null;
  const events=[];atlas.onAddTextureAtlasCanvas(c=>{
    events.push(['add',c.width]);
    if(spec.reenter&&!cleared){cleared=true;atlas.clearTexture();}
    if(spec.nested&&!nested){nested=true;nestedGlyph=atlas.getRasterizedGlyph(103,0,0,0,false,element);}
  });
  atlas.onRemoveTextureAtlasCanvas(c=>events.push(['remove',c.width]));
  const refs=[],initial=[],layouts=[];
  let tracing=false;
  try {
    if(spec.nested){beginAtlasUploads();tracing=true;}
    const scheduled=spec.warmup?captureAtlasCallbacks(()=>{atlas.warmUp();atlas.warmUp();}):null;
    for(const step of spec.steps) {
      if(step.clear){atlas.clearTexture();layouts.push(atlas.pageLayoutVersion);continue;}
      const method=typeof step.value==='number'?'getRasterizedGlyph':'getRasterizedGlyphCombinedChar';
      const value=Array.isArray(step.value)?String.fromCharCode(...step.value):step.value;
      const g=atlas[method](value,step.bg,step.fg,step.ext,step.restrict,element);
      refs.push(g);initial.push(glyphSnapshot(g));layouts.push(atlas.pageLayoutVersion);
    }
    const uploads=tracing?endAtlasUploads():[];tracing=false;
    const pages=atlas.pages.map(p=>({width:p.canvas.width,height:p.canvas.height,version:p.version,
      row:{...p.currentRow},fixed:p.fixedRows.map(r=>({...r})),used:p.percentageUsed,
      glyphs:p.glyphs.length,...hashPixels(p.canvas)}));
    return JSON.stringify({spec,expected:{initial,final:refs.map(glyphSnapshot),events,layouts,pages,
      aliases:refs.map(g=>refs.indexOf(g)),scheduled,uploads,nested:nestedGlyph?glyphSnapshot(nestedGlyph):null,attached:atlas._tmpCanvas.parentElement===element}});
  } finally {if(tracing)endAtlasUploads();atlas.dispose();element.remove();}
}
globalThis.sourceAtlasScenario=atlasScenario;
let uploadTrace;
export function beginAtlasUploads(){
  if(uploadTrace)throw new Error('Upload trace already active');
  const proto=CanvasRenderingContext2D.prototype;
  const descriptor=Object.getOwnPropertyDescriptor(proto,'putImageData'),calls=[];
  uploadTrace={descriptor,calls};
  Object.defineProperty(proto,'putImageData',{configurable:true,writable:true,value:function(...args){calls.push(args.slice(1));return descriptor.value.apply(this,args);}});
}
export function endAtlasUploads(){
  const {descriptor,calls}=uploadTrace;
  Object.defineProperty(CanvasRenderingContext2D.prototype,'putImageData',descriptor);uploadTrace=undefined;
  return calls;
}
globalThis.beginAtlasUploads=beginAtlasUploads;
globalThis.endAtlasUploads=()=>JSON.stringify(endAtlasUploads());
export function captureAtlasCallbacks(invoke) {
  const names=['requestIdleCallback','setTimeout','cancelIdleCallback','clearTimeout'];
  const descriptors=names.map(name=>Object.getOwnPropertyDescriptor(globalThis,name));
  const now=Object.getOwnPropertyDescriptor(performance,'now');
  const callbacks=[];let scheduled=0;
  const request=callback=>{callbacks.push(callback);return ++scheduled;};
  for(const name of names)Object.defineProperty(globalThis,name,{configurable:true,writable:true,value:name.includes('cancel')||name==='clearTimeout'?()=>{}:request});
  // Priority fallback uses wall-time deadlines. A shared virtual clock makes the
  // scheduling assertion deterministic instead of comparing debug WASM speed.
  Object.defineProperty(performance,'now',{configurable:true,value:()=>0});
  try {
    invoke();
    for(let i=0;i<callbacks.length;i++) {
      if(i>=100)throw new Error('Warmup failed to drain within 100 host callbacks');
      callbacks[i]({timeRemaining:()=>10000});
    }
    return scheduled;
  } finally {
    names.forEach((name,i)=>{if(descriptors[i])Object.defineProperty(globalThis,name,descriptors[i]);else delete globalThis[name];});
    if(now)Object.defineProperty(performance,'now',now);else delete performance.now;
  }
}
globalThis.captureAtlasCallbacks=captureAtlasCallbacks;
