// Actual source overview drawing, with a context call recorder rather than copied math.
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
const sourceRoot=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const require=createRequire(path.join(sourceRoot,'package.json'));
const {OverviewRulerRenderer}=require(path.join(sourceRoot,'out-esbuild/browser/decorations/OverviewRulerRenderer.js'));
const {ColorZoneStore}=require(path.join(sourceRoot,'out-esbuild/browser/decorations/ColorZoneStore.js'));
const cases=[];
for(const [width,dpr,height,length] of [[12,2,200,100],[10,1,100,10],[0,1.25,50,20],[7.5,1.5,75,25]]){
  const calls=[];const canvas={style:{},width:300,height:150};
  Object.defineProperty(canvas,'width',{get(){return this.w??300},set(v){this.w=v>>>0}});
  Object.defineProperty(canvas,'height',{get(){return this.h??150},set(v){this.h=v>>>0}});
  const context={clearRect(...args){calls.push(['clear',...args]);},fillRect(...args){calls.push(['rect',...args]);}};
  Object.defineProperty(context,'lineWidth',{set(v){calls.push(['line',v]);}});
  Object.defineProperty(context,'fillStyle',{set(v){calls.push(['style',Array.from({length:v.length},(_,i)=>v.charCodeAt(i))]);}});
  const normal={lines:{length}},active=normal;
  const options={scrollbar:{showScrollbar:true,width,overviewRuler:{showTopBorder:true,showBottomBorder:true}}};
  const decorations=[['red','left',0],['blue','center',Math.floor(length/2)],['green','right',length-1],['#ffff00','full',Math.floor(length/4)],['red','left',1]].map(([color,position,line])=>({marker:{line},options:{overviewRulerOptions:{color,position}}}));
  const source=Object.create(OverviewRulerRenderer.prototype);
  Object.assign(source,{_canvas:canvas,_ctx:context,_store:{isDisposed:false},_colorZoneStore:new ColorZoneStore(),_optionsService:{rawOptions:options},_coreBrowserService:{dpr},_bufferService:{buffer:active,buffers:{active,normal}},_renderService:{hasRenderer:()=>true,dimensions:{css:{canvas:{height:height/dpr}},device:{canvas:{height}}}},_themeService:{colors:{overviewRulerBorder:{css:'#ffffff'}}},_decorationService:{decorations},_shouldUpdateDimensions:true});
  source._refreshDecorations();
  cases.push({width,dpr,height,length,decorations:decorations.map(d=>({line:d.marker.line,...d.options.overviewRulerOptions})),canvas:{width:canvas.width,height:canvas.height,style:canvas.style},calls});
}
const file='crates/xterm-web/tests/fixtures/overview-ruler.json';const text=JSON.stringify(cases,null,2)+'\n';
if(process.argv.includes('--write-fixtures'))fs.writeFileSync(file,text);else if(fs.readFileSync(file,'utf8').replaceAll('\r\n','\n')!==text)throw new Error('Overview source fixture drift');
console.log(`${cases.length} source overview drawing cases passed`);
