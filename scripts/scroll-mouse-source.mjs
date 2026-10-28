// Direct source constructor cases, including legacy metadata without a DOM
// WheelEvent constructor's browser-specific synthetic legacy-field defaults.
import {createRequire} from 'node:module';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {withArtifactServer} from './browser-artifact-server.mjs';
const root=resolve(import.meta.dirname,'..'),require=createRequire(import.meta.url);
const esbuild=require(process.env.ESBUILD_MODULE??resolve(root,'source_repo/node_modules/esbuild'));
const playwright=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE??resolve(root,'source_repo/node_modules/playwright/index.mjs')));
const work=resolve(root,'tmp/scroll-mouse-source');await mkdir(work,{recursive:true});
const source=resolve(root,'source_repo/src/browser/scrollable/mouseEvent.ts');
const bundle=esbuild.buildSync({entryPoints:[source],bundle:true,write:false,format:'esm',platform:'browser',tsconfig:resolve(root,'source_repo/src/browser/tsconfig.json')});
await writeFile(resolve(work,'source.js'),bundle.outputFiles[0].contents);
const wheelCases=[
 {},{wheelDeltaY:-120,wheelDeltaX:240},{wheelDeltaY:0,wheelDeltaX:0,wheelDelta:120},
 {VERTICAL_AXIS:2,axis:2,detail:6},{HORIZONTAL_AXIS:1,axis:1,detail:6},
 {type:'wheel',deltaMode:0,DOM_DELTA_LINE:1,deltaY:40,deltaX:20},
 {type:'wheel',deltaMode:1,DOM_DELTA_LINE:1,deltaY:3,deltaX:-6},
 {type:'wheel',deltaMode:2,DOM_DELTA_LINE:1,deltaY:1,deltaX:2},
 {wheelDeltaY:'-60',wheelDeltaX:'30'},{wheelDeltaY:null,wheelDeltaX:false},
 {targetNode:'target',wheelDeltaY:120},{srcElement:'target',wheelDeltaY:-120},
 {target:'target',targetNode:'unused',srcElement:'unused',wheelDelta:60}
];
const mouseCases=[
 {},{button:1,buttons:2,detail:0,ctrlKey:true,shiftKey:true},
 {button:2,buttons:3,type:'dblclick',detail:9,altKey:true,metaKey:true},
 {button:'0',detail:null,pageX:12.5,pageY:-3.5},
 {pageX:null,clientX:3.5,clientY:4.5},
 {pageX:'4',clientX:'2',clientY:'3'},
 {pageX:0,pageY:0,detail:false,buttons:'raw'},
 {button:3,buttons:0,pageX:-2.5,pageY:7.25}
];
const report={kind:'unchanged-standard-scroll-mouse-source',sourceSha256:createHash('sha256').update(await readFile(source)).digest('hex'),wheelCases,mouseCases,browsers:[]};
await mkdir(resolve(root,'results/scroll-element'),{recursive:true});
const output=resolve(root,'results/scroll-element/mouse-source.json');
report.status='running';await writeFile(output,JSON.stringify(report,null,2)+'\n');
try {
await withArtifactServer({directory:work,css:'',html:'<!doctype html><body></body>'},async url=>{
 for(const name of ['chromium','firefox','webkit']){
  const browser=await playwright[name].launch({headless:true,timeout:30000});
  try{
   const page=await browser.newPage();const errors=[];page.on('pageerror',error=>errors.push(error.message));page.setDefaultTimeout(30000);await page.goto(url);
   const results=await page.evaluate(async({wheelCases,mouseCases})=>{
    const {StandardWheelEvent,StandardMouseEvent}=await import('/source.js');
    const wheels=wheelCases.map(input=>{const e=new StandardWheelEvent({...input,view:window});return{deltaX:e.deltaX,deltaY:e.deltaY,target:e.target};});
    const mice=mouseCases.map(input=>{
     const raw={button:0,buttons:1,detail:1,type:'mousedown',pageX:10,pageY:20,ctrlKey:false,shiftKey:false,altKey:false,metaKey:false,...input,target:document.body,view:window};
     const e=new StandardMouseEvent(window,raw);
     return{left:e.leftButton,middle:e.middleButton,right:e.rightButton,buttons:e.buttons,detail:e.detail,x:e.posx,y:e.posy,ctrl:e.ctrlKey,shift:e.shiftKey,alt:e.altKey,meta:e.metaKey};
    });return{wheels,mice};
   },{wheelCases,mouseCases});
   if(errors.length)throw new Error(errors.join('\n'));report.browsers.push({name,version:browser.version(),...results});console.log(`${name}: ${wheelCases.length} wheel and ${mouseCases.length} mouse source cases`);
  }finally{await browser.close();}
 }
});
report.status='passed';
} catch(error) { report.status='failed';report.error=error.stack??String(error);throw error; }
finally { await writeFile(output,JSON.stringify(report,null,2)+'\n'); }

