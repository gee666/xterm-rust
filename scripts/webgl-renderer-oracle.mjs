// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
// Invoke unchanged WebglRenderer methods, bypassing only DOM/GPU construction.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const root=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const file=path.join(root,'addons/addon-webgl/src/WebglRenderer.ts');
const require=createRequire(path.join(root,'package.json'));
const hash=b=>createHash('sha256').update(b).digest('hex');
const wrapper="\nexport {CellData} from 'common/buffer/CellData'; export {BufferLine} from 'common/buffer/BufferLine'; export {RenderModel} from './RenderModel'; export {CellColorResolver} from './CellColorResolver'; export {channels} from 'common/Color';\n";
const built=await require('esbuild').build({stdin:{contents:await readFile(file,'utf8')+wrapper,sourcefile:file,resolveDir:path.dirname(file),loader:'ts'},
  bundle:true,write:false,platform:'node',format:'cjs',metafile:true,alias:{browser:path.join(root,'src/browser'),common:path.join(root,'src/common')}});
const module={exports:{}};
vm.runInNewContext(built.outputFiles[0].text,{module,exports:module.exports,process,console},{filename:file,timeout:10000});
const {WebglRenderer,CellData,BufferLine,RenderModel,CellColorResolver,channels}=module.exports;
const inputs=[{path:path.relative(root,file).replaceAll('\\','/'),sha256:hash(await readFile(file))}];
for(const input of Object.keys(built.metafile.inputs).sort())if(input!=='<stdin>')inputs.push({path:path.relative(root,input).replaceAll('\\','/'),sha256:hash(await readFile(input))});
const utf16=s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i));
const rgba=v=>channels.toColor(v>>>24,v>>>16&255,v>>>8&255,v&255);
const colors={foreground:rgba(0xa1b2c3ff),background:rgba(0x112233ff),cursor:rgba(0x445566ff),cursorAccent:rgba(0x778899ff),
  selectionBackgroundOpaque:rgba(0x334455ff),selectionInactiveBackgroundOpaque:rgba(0x556677ff),selectionForeground:undefined,
  ansi:Array.from({length:256},(_,i)=>rgba(((i*97531)<<8|255)>>>0))};
const baseContext={cols:6,rows:3,viewport:2,cursorX:1,cursorY:3,focused:true,initialized:true,hidden:false,cursorVisible:true,
  style:'block',inactive:'outline',cursorWidth:2,dpr:1.25,textEnabled:true,textOn:true};
const cell=(code=65,fg=0,bg=0,width=1,ext=0,chars=[])=>({content:((width<<22)|code)>>>0,fg:fg>>>0,bg:bg>>>0,ext:ext>>>0,chars});
const initial=()=>[Array.from({length:6},(_,x)=>cell(65+x)),[cell(0x1f680,0,0,2),cell(0,0,0,0),cell(0x200000,0x80000000,0x10000000,1,0x10000000,[65,769]),cell(0),cell(0),cell(0)],null];
const cases=[];
function run(name,changes={},operations=[{},{}]) {
  const context={...baseContext,...changes}, renderer=Object.create(WebglRenderer.prototype);
  renderer._model=new RenderModel();renderer._model.resize(context.cols,context.rows);
  renderer._workCell=new CellData();renderer._rowHasBlinkingCells=Array(context.rows).fill(false);renderer._rowHasBlinkingCellsCount=0;
  let trace=[],fail=null,failCount=0;
  const record=(name,...args)=>{trace.push([name,...args]);if(fail?.name===name&&++failCount===fail.at)throw Error('injected');};
  const terminal={cols:context.cols,rows:context.rows,options:{},buffer:{ydisp:0,lines:{}},buffers:{active:{ydisp:0}},screenElement:{isConnected:true}};
  const publicTerminal={cols:context.cols,rows:context.rows,buffer:{active:{baseY:0,cursorY:0,cursorX:0,viewportY:0}}};
  renderer._core=terminal;renderer._terminal=publicTerminal;
  renderer._coreService={decPrivateModes:{},isCursorInitialized:true,isCursorHidden:false};
  renderer._coreBrowserService={isFocused:true,dpr:1.25};renderer._devicePixelRatio=context.dpr;
  renderer.dimensions={device:{cell:{width:8.25,height:17.5}}};renderer._themeService={colors};
  renderer._textBlinkStateManager={isEnabled:true,isBlinkOn:true,setNeedsBlinkInViewport:v=>record('blink',v)};
  renderer._cursorBlinkStateManager={value:{isCursorVisible:true}};
  renderer._optionsService={rawOptions:{fontSize:15}};
  const resolver=new CellColorResolver(publicTerminal,renderer._optionsService,renderer._model.selection,
    {forEachDecorationAtCell(){}},renderer._coreBrowserService,renderer._themeService);
  renderer._cellColorResolver=resolver;
  const resolve=resolver.resolve.bind(resolver);
  resolver.resolve=(c,x,y,w,h)=>{record('resolve',x,y,c.getCode()??null);resolve(c,x,y,w,h);};
  renderer._rectangleRenderer={value:{updateBackgrounds:()=>record('backgrounds'),updateCursor:()=>record('cursor'),renderBackgrounds:()=>record('draw-backgrounds'),renderCursor:()=>record('draw-cursor')}};
  let frames=[];
  renderer._glyphRenderer={value:{updateCell:(x,y,code,bg,fg,ext,chars,width,lastBg)=>record('glyph',x,y,code??null,bg,fg,ext,utf16(chars),width,lastBg),
    beginFrame:()=>{const value=frames.shift()??false;record('begin',value);return value;},clear:()=>record('clear'),invalidateAtlasTextures:()=>record('invalidate'),render:()=>record('draw-glyphs')}};
  renderer._renderLayers=[{handleGridChanged:(_t,start,end)=>record('grid',start,end)}];
  const steps=[];
  let raw=initial().slice(0,context.rows).map(r=>r?.slice(0,context.cols)??null),joins=Array.from({length:context.rows},()=>[]),selection=null;
  for(const operation of operations) {
    Object.assign(context,operation.context??{});
    raw=operation.lines??raw;joins=operation.joins??joins;selection=operation.selection===undefined?selection:operation.selection;
    trace=[];fail=operation.fail??null;failCount=0;
    const lines=raw.map(row=>row&&(()=>{const l=new BufferLine(context.cols);row.forEach((v,x)=>{const c=new CellData();c.content=v.content;c.fg=v.fg;c.bg=v.bg;c.extended.ext=v.ext;c.combinedData=String.fromCharCode(...v.chars);l.setCell(x,c);});
      const load=l.loadCell.bind(l);l.loadCell=(x,c)=>{record('load',x);return load(x,c);};
      const translate=l.translateToString.bind(l);l.translateToString=(trim,start,end)=>{record('translate',start,end);return translate(trim,start,end);};return l;})());
    terminal.buffer.ydisp=context.viewport;terminal.buffers.active.ydisp=context.viewport;
    terminal.buffer.lines.get=row=>{record('line',row);return lines[row-context.viewport];};
    terminal.options={cursorStyle:context.style,cursorInactiveStyle:context.inactive,cursorWidth:context.cursorWidth};
    Object.assign(publicTerminal.buffer.active,{cursorX:context.cursorX,cursorY:context.cursorY,viewportY:context.viewport});
    renderer._coreService.isCursorInitialized=context.initialized;renderer._coreService.isCursorHidden=context.hidden;
    renderer._coreBrowserService.isFocused=context.focused;renderer._devicePixelRatio=context.dpr;
    renderer._cursorBlinkStateManager.value.isCursorVisible=context.cursorVisible;
    renderer._textBlinkStateManager.isEnabled=context.textEnabled;renderer._textBlinkStateManager.isBlinkOn=context.textOn;
    renderer._characterJoinerService={getJoinedCharacters:row=>{record('joins',row);return structuredClone(joins[row-context.viewport]??[]);}};
    renderer._model.selection.update(terminal,selection?.start,selection?.end,selection?.column??false);
    const start=operation.start??0,end=operation.end??context.rows-1;
    frames=[...(operation.frames??[])];renderer._charAtlas=operation.atlas===false?undefined:{};
    renderer._isAttached=operation.attached!==false;terminal.screenElement.isConnected=renderer._isAttached;renderer._charSizeService={width:0,height:0};
    const savedRect=renderer._rectangleRenderer.value;
    if(operation.ready===false)renderer._rectangleRenderer.value=undefined;
    let error=null;
    try { if(operation.frame)renderer.renderRows(start,end);else renderer._updateModel(start,end); }catch(e){error=e.message;}
    renderer._rectangleRenderer.value=savedRect;
    const r=resolver.result;
    steps.push({context:{...context},lines:raw,joins,selection,start,end,frame:operation.frame??false,frames:operation.frames??[],
      atlas:operation.atlas!==false,attached:operation.attached!==false,ready:operation.ready!==false,fail,
      expected:{cells:[...renderer._model.cells],lengths:[...renderer._model.lineLengths],cursor:renderer._model.cursor??null,
        blinking:[...renderer._rowHasBlinkingCells],count:renderer._rowHasBlinkingCellsCount,colors:[r.bg>>>0,r.fg>>>0,r.ext>>>0],trace,error}});
  }
  cases.push({name,steps});
}
run('stable-incremental');
for(const [cols,rows] of [[1,1],[2,2],[9,4]])run('grid-'+cols+'-'+rows,{cols,rows});
run('empty-combined',{},[{lines:[Array(6).fill(cell(0x200000,0,0,1,0,[])),null,null]},{}]);
run('signed-cache-and-variant', {cursorY:100},[{lines:[Array(6).fill(cell(0x2591,0x80000000,0x10000000,1,0x10000000)),null,null],selection:{start:[0,2],end:[6,2],column:false}},{},{context:{textOn:false}}]);
run('unchanged-joined-cell-stays-loaded', {cursorY:100},[{lines:[Array(6).fill(cell(0)),null,null],joins:[[[0,2],[3,5]],[],[]]},{},{}]);
run('wide-block-cursor', {cursorX:0,cursorY:3});
run('cursor-wrapped-x', {cursorX:6,cursorY:2});
for(const style of ['block','bar','underline','outline','none'])for(const focused of [true,false])run('style-'+style+'-'+focused,{style,inactive:style,focused});
for(const flag of ['hidden','initialized','cursorVisible'])run(flag,{[flag]:flag==='hidden'});
run('blink-transitions',{},[{},{context:{textOn:false},start:1,end:1},{lines:[initial()[0],Array(6).fill(cell(90)),null],start:1,end:1},{start:2,end:2}]);
run('clamped-reversed',{},[{start:-10,end:100},{start:2,end:0},{start:100,end:100}]);
run('large-positive-rows',{},[{start:4294967296,end:4294967296},{start:9007199254740991,end:9007199254740991},{start:-4294967296,end:-4294967296}]);
run('large-negative-to-positive',{},[{start:-4294967296,end:4294967296}]);
run('joined', {cursorY:100},[{joins:[[[0,3],[3,6]],[[2,4]],[]]},{},{selection:{start:[1,2],end:[4,2],column:false}},{context:{cursorY:2,cursorX:1}}]);
run('column-selection', {cursorY:100},[{joins:[[[0,3]],[],[]],selection:{start:[4,2],end:[1,3],column:true}},{context:{textOn:false}}]);
run('glyph-error',{},[{fail:{name:'glyph',at:2}},{},{fail:{name:'cursor',at:1}},{fail:{name:'blink',at:1}},{fail:{name:'backgrounds',at:1},context:{textOn:false}}]);
for(const frames of [[],[true,false],[false,true,false],[false,...Array(40).fill(true)]])run('frame-'+frames.length,{},[{frame:true,frames}]);
run('frame-detached',{},[{frame:true,attached:false}]);run('frame-not-ready',{},[{frame:true,ready:false}]);run('frame-no-atlas',{},[{frame:true,atlas:false,frames:[false,true]}]);
let state=0x54b7d329;
const random=()=>{state^=state<<13;state^=state>>>17;state^=state<<5;return state>>>0;};
for(let n=0;n<128;n++) {
  const lines=Array.from({length:3},()=>random()%5?Array.from({length:6},()=>{const code=[0,65,0x2591,0x1f680,0x200000][random()%5];
    return cell(code,((random()%4)<<24)|((random()%2)<<31),((random()%4)<<24)|0x10000000,random()%3,random(),code===0x200000?[65,769]:[]);}):null);
  const selection={start:[random()%6,2],end:[random()%6,4],column:!!(random()%2)};
  run('seed-'+n,{cursorX:random()%8,cursorY:random()%7,style:['block','bar','underline'][random()%3],focused:!!(random()%2)},
    [{lines,selection,joins:[[[0,2],[3,5]],[[1,4]],[]]},{start:random()%3,end:random()%3,context:{textOn:false}},{frame:true,frames:[false,true,false]}]);
}
const output=path.resolve(process.env.WEBGL_RENDERER_ORACLE_OUTPUT??'crates/xterm-web/tests/fixtures/webgl-renderer.json');
await mkdir(path.dirname(output),{recursive:true});
await writeFile(output,JSON.stringify({seed:'0x54b7d329',inputs,wrapper,bundleSha256:hash(built.outputFiles[0].contents),generatorSha256:hash(await readFile(new URL(import.meta.url))),cases})+'\n');
console.log(`${cases.length} cases, ${cases.reduce((n,c)=>n+c.steps.length,0)} steps -> ${output}`);
