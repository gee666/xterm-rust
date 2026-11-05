// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Exercise the unchanged TextureAtlas color routines without creating a canvas.
import {readFile,writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const file=path.join(root,'addons/addon-webgl/src/TextureAtlas.ts');
const source=await readFile(file,'utf8');
const require=createRequire(path.join(root,'package.json'));
const wrapper="\nexport { AttributeData, channels }; export {ColorContrastCache} from 'browser/ColorContrastCache';\n";
const built=await require('esbuild').build({stdin:{contents:source+wrapper,sourcefile:file,resolveDir:path.dirname(file),loader:'ts'},
  bundle:true,write:false,platform:'node',format:'cjs',metafile:true,alias:{browser:path.join(root,'src/browser'),common:path.join(root,'src/common')}});
const module={exports:{}};
vm.runInNewContext(built.outputFiles[0].text,{module,exports:module.exports,process,console},{filename:file});
const {TextureAtlas,AttributeData,channels,ColorContrastCache}=module.exports;
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputs=[{path:path.relative(root,file).replaceAll('\\','/'),sha256:hash(await readFile(file))}];
for(const input of Object.keys(built.metafile.inputs).sort()) {
  if(input==='<stdin>'||path.resolve(input)===file)continue;
  inputs.push({path:path.relative(root,input).replaceAll('\\','/'),sha256:hash(await readFile(input))});
}
const rgba=value=>channels.toColor(value>>>24,(value>>>16)&255,(value>>>8)&255,value&255);
const palette=Array.from({length:256},(_,i)=>(((i*943931)<<8)|(i^0xa3))>>>0);
const foreground=0xb1b2b360,background=0x22334480;
const cases=[];
function run(name,bg,fg,opts={}) {
  const {ratio=4.5,dim=false,bold=false,inverse=false,allowTransparency=false,bright=true,excluded=false,paletteLength=256,cacheNull=false}=opts;
  bg=(bg|(dim?0x08000000:0))>>>0;
  fg=(fg|(bold?0x08000000:0)|(inverse?0x04000000:0))>>>0;
  const colors={foreground:rgba(foreground),background:rgba(background),ansi:palette.slice(0,paletteLength).map(rgba),
    contrastCache:new ColorContrastCache(),halfContrastCache:new ColorContrastCache()};
  const config={colors,minimumContrastRatio:ratio,allowTransparency,drawBoldTextInBrightColors:bright};
  const atlas=Object.create(TextureAtlas.prototype);atlas._config=config;
  const a=new AttributeData();a.bg=bg;a.fg=fg;
  let bgMode=a.getBgColorMode(),fgMode=a.getFgColorMode(),bgColor=a.getBgColor(),fgColor=a.getFgColor();
  if(inverse){[bgMode,fgMode]=[fgMode,bgMode];[bgColor,fgColor]=[fgColor,bgColor];}
  const cache=dim?colors.halfContrastCache:colors.contrastCache;
  const original=opts.original??[bg,fg];
  if(cacheNull)cache.setColor(...original,null);
  if(opts.seedColor!==undefined)cache.setColor(...original,rgba(opts.seedColor));
  let expected,error;
  try {
    const backgrounds=[],foregrounds=[];
    for(let i=0;i<2;i++) {
      backgrounds.push(atlas._getBackgroundColor(bgMode,bgColor,inverse,dim));
      foregrounds.push(atlas._getForegroundColor(original[0],bgMode,bgColor,original[1],fgMode,fgColor,inverse,dim,bold,excluded));
    }
    const cached=cache.getColor(...original);
    expected={background:backgrounds[0],foreground:foregrounds[0],sameBackground:backgrounds[0]===backgrounds[1],sameForeground:foregrounds[0]===foregrounds[1],
      cache:cached===undefined?'missing':cached===null?'null':cached};
  }catch(e){error=e.message;}
  cases.push({name,bg,fg,ratio,allowTransparency,bright,excluded,paletteLength,cacheNull,
    ...(opts.original?{original,seedColor:opts.seedColor??null}:{}),expected:expected??null,error:error??null});
}
for(const allowTransparency of [false,true])for(const dim of [false,true])for(const inverse of [false,true])for(const bold of [false,true])
  for(const ratio of [1,4.5,21])for(const [bg,fg]of [[0,0],[0x1000003,0x1000002],[0x2000034,0x2000065],[0x3007777,0x3007766]])
    run('direct-'+cases.length,bg,fg,{allowTransparency,dim,inverse,bold,ratio});
run('bg-palette-missing',0x20000ff,0,{paletteLength:1});
run('transparent-no-contrast-does-not-read-bg-palette',0x20000ff,0,{paletteLength:1,allowTransparency:true,ratio:1});
run('transparent-cached-null-does-not-read-bg-palette',0x20000ff,0,{paletteLength:1,allowTransparency:true,cacheNull:true});
run('fg-palette-missing',0,0x20000ff,{paletteLength:1});
let state=0x3acb128f;
function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
for(let i=0;i<256;i++) {
  const mode=()=>[0,0x1000000,0x2000000,0x3000000][random()%4];
  const bgMode=mode(),fgMode=mode();
  const value=m=>m|(random()&(m===0x3000000?0xffffff:255));
  run('seeded-'+i,value(bgMode),value(fgMode),{allowTransparency:!!(random()%2),dim:!!(random()%2),bold:!!(random()%2),inverse:!!(random()%2),
    ratio:[1,2,4.5,7,21][random()%5],bright:!!(random()%2),excluded:!!(random()%2)});
}
for(const dim of [false,true])for(const cacheNull of [false,true,null])
  for(const signedBg of [false,true])for(const signedFg of [false,true]) {
  const bg=0x80000000|(dim?0x08000000:0),fg=0x80000000;
  run(`signed-${dim}-${cacheNull}-${signedBg}-${signedFg}`,bg,fg,{dim,cacheNull,ratio:21,
    original:[signedBg?bg:bg>>>0,signedFg?fg|0:fg>>>0],...(cacheNull===false?{seedColor:0xff3456ff}:{})});
}
const output=path.resolve('crates/xterm-web/tests/fixtures/webgl-atlas-colors.json');
await writeFile(output,JSON.stringify({seed:'0x3acb128f',inputs,wrapper,foreground,background,palette,cases})+'\n');
console.log(cases.length+' unchanged-source atlas color cases -> '+output);
