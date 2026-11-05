// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Execute unchanged CellColorResolver over typed service inputs, without a DOM.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const file=path.join(root,'addons/addon-webgl/src/CellColorResolver.ts');
const source=await readFile(file,'utf8');
const require=createRequire(path.join(root,'package.json'));
const wrapper="\nexport { AttributeData } from 'common/buffer/AttributeData'; export { channels } from 'common/Color';\n";
const built=await require('esbuild').build({stdin:{contents:source+wrapper,sourcefile:file,resolveDir:path.dirname(file),loader:'ts'},
  bundle:true,write:false,platform:'node',format:'cjs',metafile:true,alias:{browser:path.join(root,'src/browser'),common:path.join(root,'src/common')}});
const module={exports:{}};
vm.runInNewContext(built.outputFiles[0].text,{module,exports:module.exports,process,console},{filename:file,timeout:10000});
const {CellColorResolver,AttributeData,channels}=module.exports;
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputs=[{path:path.relative(root,file).replaceAll('\\','/'),sha256:hash(await readFile(file))}];
for(const input of Object.keys(built.metafile.inputs).sort()) {
  if(input==='<stdin>'||path.resolve(input)===file)continue;
  inputs.push({path:path.relative(root,input).replaceAll('\\','/'),sha256:hash(await readFile(input))});
}
const rgba=value=>channels.toColor(value>>>24,(value>>>16)&255,(value>>>8)&255,value&255);
const palette=Array.from({length:256},(_,i)=>(((i*943931)<<8)|(i^0xa3))>>>0);
const foreground=0xb1b2b360,background=0x22334480,selection=0x817253ff,inactive=0x273645ff;
const cases=[];
const numeric=v=>typeof v==='string'?({NaN:NaN,Infinity:Infinity,'-Infinity':-Infinity})[v]:v;
function run(name,overrides={}) {
  const c={name,bg:0,fg:0,ext:0,url:0,code:65,x:3,y:5,cw:9,ch:17,font:15,dpr:1,selected:false,focused:true,
    selectionForeground:null,paletteLength:256,decorations:[],...overrides};
  const colors={foreground:rgba(foreground),background:rgba(background),ansi:palette.slice(0,c.paletteLength).map(rgba),
    selectionBackgroundOpaque:rgba(selection),selectionInactiveBackgroundOpaque:rgba(inactive),
    selectionForeground:c.selectionForeground===null?undefined:rgba(c.selectionForeground)};
  const decorationService={forEachDecorationAtCell(x,y,layer,callback){
    for(const d of c.decorations)if(d.top===(layer==='top'))callback({backgroundColorRGB:d.bg===null?undefined:rgba(d.bg),foregroundColorRGB:d.fg===null?undefined:rgba(d.fg)});
  }};
  const resolver=new CellColorResolver({}, {rawOptions:{fontSize:numeric(c.font)}}, {isCellSelected:()=>c.selected},
    decorationService, {dpr:numeric(c.dpr),isFocused:c.focused}, {colors});
  const cell=new AttributeData();cell.bg=c.bg;cell.fg=c.fg;cell.extended.ext=c.ext;cell.extended.urlId=c.url;cell.getCode=()=>c.code;
  let error=null;
  try{resolver.resolve(cell,numeric(c.x),numeric(c.y),numeric(c.cw),numeric(c.ch));}catch(e){error=e.name;}
  cases.push({...c,expected:{bg:resolver.result.bg>>>0,fg:resolver.result.fg>>>0,ext:resolver.result.ext>>>0},numbers:[resolver.result.bg,resolver.result.fg,resolver.result.ext],error});
}
const deco=(bg,fg,top=false)=>({bg,fg,top});
const layers=[[],[deco(0x12345680,null)],[deco(null,0xabcdef80)],[deco(0x123456ff,0xabcdef01)],
  [deco(0x112233ff,null),deco(null,0x445566ff,true),deco(0x778899ff,null,true)],
  [deco(null,0),deco(0,null,true),deco(null,null,true)]];
for(const bg of [0,0x1000003,0x2000034,0x3001278])for(const fg of [0,0x1000002,0x2000065,0x300abcd])
for(const inverse of [0,0x4000000])for(const selected of [false,true])for(const focused of [false,true])
for(const code of [65,0xe0b0,0x2588])for(const decorations of layers)
  run('matrix-'+cases.length,{bg:bg|0x8000000,fg:fg|inverse|0x8000000,selected,focused,code,decorations,selectionForeground:selected&&focused?0x10203080:null});
for(const code of [0,65,0x2591,0x2592,0x2593,0x1fb95,0xe0b0])for(const ext of [0,0x10000000,0xf0000000])
for(const [cw,ch,font,dpr]of [[9,17,15,1],[8.25,16.5,32,1.25],[0,0,1,0],[-1.5,-3,15,2],[1e20,1e20,15,1]])
  run('phase-'+cases.length,{code,bg:0x10000000,ext,cw,ch,font,dpr});
for(const [bg,fg]of [[0x20000ff,0],[0,0x60000ff],[0x20000ff,0x4000000],[0,0x20000ff]])
for(const selected of [false,true])for(const code of [65,0xe0b0])
  run('missing-'+cases.length,{bg,fg,selected,code,paletteLength:1});
run('link-dotted-becomes-dashed',{bg:0x10000000,ext:0xf0000000,url:1});
for(const key of ['x','y','cw','ch','font','dpr'])for(const value of ['NaN','Infinity','-Infinity'])
  run('nonfinite-'+key+'-'+value,{bg:0x10000000,ext:0xf0000000,code:0x2591,[key]:value});
let state=0x7a12c431;
function random(){state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;}
for(let i=0;i<512;i++) {
  const mode=()=>[0,0x1000000,0x2000000,0x3000000][random()%4];
  const word=()=>{const m=mode();return (m|(random()&(m===0x3000000?0xffffff:255))|((random()%32)<<26))>>>0;};
  run('seeded-'+i,{bg:word(),fg:word(),ext:random(),code:[0,65,0xe0b0,0x2591,0x1fb95][random()%5],
    x:random()%80,y:random()%300,cw:(random()%65)/4,ch:(random()%81)/4,font:random()%100,dpr:(random()%13)/4,
    selected:!!(random()%2),focused:!!(random()%2),selectionForeground:random()%2?random():null,
    decorations:Array.from({length:random()%6},()=>deco(random()%2?random():null,random()%2?random():null,!!(random()%2)))});
}
const output=path.resolve(process.env.CELL_COLOR_ORACLE_OUTPUT??'crates/xterm-web/tests/fixtures/webgl-cell-colors.json');
await mkdir(path.dirname(output),{recursive:true});
await writeFile(output,JSON.stringify({seed:'0x7a12c431',inputs,wrapper,bundleSha256:hash(built.outputFiles[0].contents),
  generatorSha256:hash(await readFile(new URL(import.meta.url))),foreground,background,selection,inactive,palette,cases})+'\n');
console.log(`${cases.length} unchanged-source cell color cases -> ${output}`);
