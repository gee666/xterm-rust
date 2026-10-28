// Unchanged upstream constructor and methods in real browsers. Rust replays
// these exact inputs from the retained report, without executing this bundle.
import {createRequire} from 'node:module';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {withArtifactServer} from './browser-artifact-server.mjs';

const root=resolve(import.meta.dirname,'..');
const require=createRequire(import.meta.url);
const esbuild=require(process.env.ESBUILD_MODULE??resolve(root,'source_repo/node_modules/esbuild'));
const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??resolve(root,'source_repo/node_modules/playwright/index.mjs')));
const work=resolve(root,'tmp/scroll-element-source');await mkdir(work,{recursive:true});
const source=resolve(root,'source_repo/src/browser/scrollable/scrollableElement.ts');
const bundle=esbuild.buildSync({stdin:{contents:`export {SmoothScrollableElement} from ${JSON.stringify(source)}; export {Scrollable} from ${JSON.stringify(resolve(root,'source_repo/src/browser/scrollable/scrollable.ts'))};`,resolveDir:root},bundle:true,write:false,format:'esm',platform:'browser',tsconfig:resolve(root,'source_repo/src/browser/tsconfig.json')});
await writeFile(resolve(work,'source.js'),bundle.outputFiles[0].contents);
const wheel=(values={})=>({kind:'wheel',values});
const scenario=(name,options={},actions=[wheel()],extra={})=>({name,options:{mouseWheelSmoothScroll:false,...options},actions,...extra});
const cases=[
 scenario('native pixel'),scenario('line',{},[wheel({deltaMode:1,deltaY:3})]),
 scenario('page',{},[wheel({deltaMode:2,deltaY:1})]),
 scenario('modern pixel',{},[wheel({modern:true})]),
 scenario('modern line',{},[wheel({modern:true,deltaMode:1,deltaY:3})]),
 scenario('modern page',{},[wheel({modern:true,deltaMode:2,deltaY:1})]),
 scenario('legacy',{},[wheel({legacyY:-120,legacyX:0})]),
 scenario('legacy up',{},[wheel({legacyY:120,legacyX:0})],{top:150}),
 scenario('diagonal vertical',{},[wheel({modern:true,deltaX:20,deltaY:40})]),
 scenario('diagonal horizontal',{},[wheel({modern:true,deltaX:40,deltaY:20})]),
 scenario('diagonal tie',{},[wheel({modern:true,deltaX:40,deltaY:40})]),
 scenario('both axes',{scrollPredominantAxis:false},[wheel({modern:true,deltaX:20,deltaY:40})]),
 scenario('flip axes',{flipAxes:true},[wheel({modern:true})]),
 scenario('convert vertical',{scrollYToX:true},[wheel({modern:true})]),
 scenario('cancel converted diagonal',{scrollYToX:true},[wheel({modern:true,deltaX:40,deltaY:-40})]),
 scenario('shift',{},[wheel({modern:true,shiftKey:true})]),
 scenario('fast alt',{},[wheel({modern:true,altKey:true})]),
 scenario('fractional sensitivity',{mouseWheelScrollSensitivity:0.01},[wheel({modern:true})]),
 scenario('inverted sensitivity',{mouseWheelScrollSensitivity:-1},[wheel({modern:true})],{top:150}),
 scenario('zero sensitivity',{mouseWheelScrollSensitivity:0},[wheel({modern:true})]),
 scenario('fast sensitivity',{fastScrollSensitivity:2},[wheel({modern:true,altKey:true})]),
 scenario('already prevented',{},[wheel({modern:true,prevented:true})]),
 scenario('at bottom',{},[wheel({modern:true})],{top:360}),
 scenario('always consume',{alwaysConsumeMouseWheel:true},[wheel({modern:true})],{top:360}),
 scenario('needed consume',{consumeMouseWheelIfScrollbarIsNeeded:true},[wheel({modern:true})],{top:360}),
 scenario('not needed',{consumeMouseWheelIfScrollbarIsNeeded:true},[wheel({modern:true})],{scrollHeight:40,scrollWidth:100}),
 scenario('toggle wheel',{},[wheel({modern:true}),{kind:'update',values:{handleMouseWheel:false}},wheel({modern:true}),{kind:'update',values:{handleMouseWheel:true}},wheel({modern:true})]),
 scenario('update sensitivity',{},[{kind:'update',values:{mouseWheelScrollSensitivity:2,fastScrollSensitivity:3,scrollPredominantAxis:false}},wheel({modern:true,deltaX:20,altKey:true})]),
 scenario('external listener',{},[wheel({modern:true})],{external:true}),
 scenario('no shadows',{useShadows:false},[wheel({modern:true})]),
 scenario('lazy render',{lazyRender:true},[wheel({modern:true}),{kind:'render'}]),
 scenario('class update',{},[{kind:'class',value:'custom'},{kind:'class',value:'again'}]),
 scenario('change geometry',{},[{kind:'update',values:{horizontal:2,vertical:2,horizontalScrollbarSize:12,verticalScrollbarSize:14,horizontalHasArrows:true,verticalHasArrows:true,scrollByPage:true}},wheel({modern:true})]),
 scenario('hidden bars',{horizontal:2,vertical:2},[wheel({modern:true})]),
 scenario('visible bars',{horizontal:3,vertical:3},[wheel({modern:true})]),
 scenario('zero delta',{},[wheel({modern:true,deltaY:0})]),
 scenario('bounds fractional',{mouseWheelScrollSensitivity:0.005},[wheel({modern:true}),wheel({modern:true,deltaY:-40})]),
 scenario('dispose listener',{},[{kind:'dispose'},wheel({modern:true})])
];
const report={kind:'unchanged-scroll-element-source',sourceSha256:createHash('sha256').update(await readFile(source)).digest('hex'),cases,browsers:[]};
await mkdir(resolve(root,'results/scroll-element'),{recursive:true});
const output=resolve(root,'results/scroll-element/source.json');
report.status='running';await writeFile(output,JSON.stringify(report,null,2)+'\n');
try {
await withArtifactServer({directory:work,css:await readFile(resolve(root,'crates/xterm-web/assets/xterm.css')),html:'<!doctype html><link rel="stylesheet" href="/xterm.css"><body></body>'},async url=>{
 for(const name of ['chromium','firefox','webkit']){
  const browser=await playwright[name].launch({headless:true,timeout:30000});
  try{
   const page=await browser.newPage();const errors=[];page.on('pageerror',error=>errors.push(error.message));page.setDefaultTimeout(30000);await page.goto(url);
   const results=await page.evaluate(async cases=>{
    const {SmoothScrollableElement,Scrollable}=await import('/source.js');
    return cases.map(test=>{
     const parent=document.createElement('div');parent.className='xterm';document.body.appendChild(parent);
     const external=document.createElement('div');parent.appendChild(external);
     const model=new Scrollable({forceIntegerValues:false,smoothScrollDuration:0,scheduleAtNextAnimationFrame:callback=>{const id=requestAnimationFrame(callback);return{dispose(){cancelAnimationFrame(id);}};}});
     const element=new SmoothScrollableElement(document.createElement('div'),{...test.options,...test.external?{listenOnDomNode:external}:{}},model);
     const node=element.getDomNode();parent.appendChild(node);
     element.setScrollDimensions({width:100,height:40,scrollWidth:test.scrollWidth??400,scrollHeight:test.scrollHeight??400});
     element.setScrollPosition({scrollTop:test.top??0});
     const steps=[];
     try{for(const action of test.actions){
      let prevented=null;
      if(action.kind==='wheel'){
       const {modern,legacyY,legacyX,prevented:pre,...init}=action.values;
       const event=new WheelEvent('wheel',{view:window,deltaY:40,deltaMode:0,bubbles:true,cancelable:true,...init});
       if(modern)Object.defineProperties(event,{wheelDeltaY:{value:undefined},wheelDeltaX:{value:undefined},wheelDelta:{value:undefined}});
       if(legacyY!==undefined)Object.defineProperty(event,'wheelDeltaY',{value:legacyY});
       if(legacyX!==undefined)Object.defineProperty(event,'wheelDeltaX',{value:legacyX});
       if(pre)event.preventDefault();(test.external?external:node).dispatchEvent(event);prevented=event.defaultPrevented;
      }else if(action.kind==='update')element.updateOptions(action.values);
      else if(action.kind==='render')element.renderNow();
      else if(action.kind==='class')element.updateClassName(action.value);
      else if(action.kind==='dispose')element.dispose();
      const p=element.getScrollPosition();steps.push({top:p.scrollTop,left:p.scrollLeft,prevented,className:node.className,shadows:[...node.querySelectorAll('.xterm-shadow')].map(n=>n.className)});
     }}finally{element.dispose();model.dispose();parent.remove();}
     return{name:test.name,steps};
    });
   },cases);
   if(errors.length)throw new Error(errors.join('\n'));report.browsers.push({name,version:browser.version(),results});console.log(`${name}: ${results.length} source scenarios`);
  }finally{await browser.close();}
 }
});
report.status='passed';
} catch(error) { report.status='failed';report.error=error.stack??String(error);throw error; }
finally { await writeFile(output,JSON.stringify(report,null,2)+'\n'); }
