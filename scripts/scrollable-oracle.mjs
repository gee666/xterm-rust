// Run the supplied scrollable.ts implementation against the Rust numeric model.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {stripTypeScriptTypes} from 'node:module';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import vm from 'node:vm';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const erase=source=>stripTypeScriptTypes(source.replace(/^import .*;\r?$/gm,'').replace(/^export /gm,''),{mode:'transform'});
let clock=1000;
const context=vm.createContext({Date:{now:()=>clock},console});
const files=['src/common/Lifecycle.ts','src/common/Event.ts','src/browser/scrollable/scrollable.ts'];
vm.runInContext(files.map(file=>erase(readFileSync(path.join(root,'source_repo',file),'utf8'))).join('\n')+'\nglobalThis.Reference=Scrollable;',context);
const encode=value=>Object.is(value,-0)?'-0':String(value);
const input=value=>encode(value);
const state=value=>[value.width,value.scrollWidth,value.scrollLeft,value.height,value.scrollHeight,value.scrollTop,value.rawScrollLeft,value.rawScrollTop].map(encode);
let world;
function make(integer,duration,now) {
  clock=now;
  const next={queue:new Map(),requested:[],canceled:[],events:[],next:0,requestFail:false,cancelFail:false,eventFail:false,eventCancel:false};
  next.model=new context.Reference({forceIntegerValues:integer,smoothScrollDuration:duration,scheduleAtNextAnimationFrame(callback) {
    if(next.requestFail)throw new Error('request');
    const id=++next.next;next.queue.set(id,callback);next.requested.push(id);
    return {dispose(){if(next.cancelFail)throw new Error('cancel');next.queue.delete(id);next.canceled.push(id);}};
  }});
  next.model.onScroll(event=>{
    next.events.push([
      [event.oldWidth,event.oldScrollWidth,event.oldScrollLeft,event.oldHeight,event.oldScrollHeight,event.oldScrollTop].map(encode),
      [event.width,event.scrollWidth,event.scrollLeft,event.height,event.scrollHeight,event.scrollTop].map(encode),
      event.inSmoothScrolling,[event.widthChanged,event.scrollWidthChanged,event.scrollLeftChanged,event.heightChanged,event.scrollHeightChanged,event.scrollTopChanged]
    ]);
    if(next.eventFail)throw new Error('event');
    if(next.eventCancel&&event.inSmoothScrolling)next.model.setScrollPositionNow({scrollTop:8});
  });
  return next;
}
world=make(false,0,1000);
function update(keys,values) {return Object.fromEntries(keys.flatMap((key,index)=>values[index]===null?[]:[[key,Number(values[index])]]));}
function apply(row) {
  let error=null;
  try {
    const [kind,...args]=row;
    if(kind==='new') {world.cancelFail=false;world.model.dispose();world=make(args[0],Number(args[1]),Number(args[2]));}
    else if(kind==='dims')world.model.setScrollDimensions(update(['width','scrollWidth','height','scrollHeight'],args),args[4]);
    else if(kind==='now')world.model.setScrollPositionNow(update(['scrollLeft','scrollTop'],args));
    else if(kind==='smooth')world.model.setScrollPositionSmooth(update(['scrollLeft','scrollTop'],args),args[2]);
    else if(kind==='frame') {const first=world.queue.entries().next().value;if(!first)throw new Error('frame');world.queue.delete(first[0]);first[1]();}
    else if(kind==='time')clock=Number(args[0]);
    else if(kind==='duration')world.model.setSmoothScrollDuration(Number(args[0]));
    else if(['requestFail','cancelFail','eventFail','eventCancel'].includes(kind))world[kind]=args[0];
    else if(kind==='dispose')world.model.dispose();
    else throw new Error(`unknown command ${kind}`);
  } catch(thrown) {error=thrown.message;}
  const future=world.model.getFutureScrollPosition();
  return JSON.parse(JSON.stringify([state(world.model.getScrollDimensions()),[future.scrollLeft,future.scrollTop].map(encode),world.model.hasPendingScrollAnimation(),[...world.queue.keys()],world.requested,world.canceled,world.events,error]));
}
const rows=[],expected=[];
function add(...row) {rows.push(row);expected.push(apply(row));}
const values=[-Infinity,-4294967297,-100,-10,-1,-0,0,Number.MIN_VALUE,0.49,0.5,1,2,9.5,10,20,50,100,1000,2147483648,4294967296,Infinity,NaN];
const seed=0x5343524c;let randomState=seed;
function random(){randomState=(Math.imul(randomState,1664525)+1013904223)>>>0;return randomState;}
function sample(){return values[(random()>>>16)%values.length];}
function optional(){return random()&3?input(sample()):null;}
// Directed timing, clamping, cancellation, raw positions and error partial state.
for(const integer of [false,true]) {
 add('new',integer,'90','1000');add('dims','100','1000','100','1000',false);add('smooth',null,'100',false);
 add('time','1040');add('frame');add('smooth',null,'200',true);add('time','1090');add('frame');
 add('now',null,'2000.5');add('dims',null,null,null,'4000',true);add('eventCancel',true);add('smooth',null,'10',false);add('frame');
 add('requestFail',true);add('smooth',null,'300',false);add('requestFail',false);add('smooth',null,'400',false);
 add('cancelFail',true);add('now',null,'1');add('cancelFail',false);add('eventFail',true);add('now',null,'3');add('eventFail',false);add('dispose');
}
// Dense finite animations exercise interpolation, not only exceptional numbers.
for(let scenario=0;scenario<100;scenario++) {
 const duration=1+(random()%400);
 add('new',!!(scenario&1),input(duration),'1000');
 add('dims',input(20+(random()%1000)/10),'10000',input(20+(random()%1000)/10),'10000',false);
 add('now',input((random()%3000)/10),input((random()%3000)/10));
 add('smooth',input((random()%100000)/10),input((random()%100000)/10),false);
 for(let step=0;step<=20&&world.queue.size;step++) { add('time',input(1000+duration*step/20)); add('frame'); }
}
for(let scenario=0;scenario<500;scenario++) {
 add('new',!!(random()&1),input(sample()),input(sample()));
 for(let step=0;step<30;step++) {
  const choice=(random()>>>16)%12;
  if(choice===0)add('dims',optional(),optional(),optional(),optional(),!!(random()&1));
  else if(choice===1)add('now',optional(),optional());
  else if(choice===2)add('smooth',optional(),optional(),!!(random()&1));
  else if(choice===3&&world.queue.size)add('frame');
  else if(choice===4)add('time',input(sample()));
  else if(choice===5)add('duration',input(sample()));
  else if(choice===6)add('requestFail',!!(random()&1));
  else if(choice===7)add('cancelFail',!!(random()&1));
  else if(choice===8)add('eventFail',!!(random()&1));
  else if(choice===9)add('eventCancel',!!(random()&1));
  else if(choice===10)add('dispose');
  else add('dims','100','1000','100','1000',false);
 }
}
add('cancelFail',false);add('dispose');
function run(command,args,options={}) {const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:120000,maxBuffer:128*1024*1024,...options});if(result.error)throw result.error;assert.equal(result.status,0,result.stderr);return result.stdout;}
run(process.env.CARGO??'cargo',['build','-p','xterm-web','--example','scrollable_trace']);
const binary=path.join(process.env.CARGO_TARGET_DIR??path.join(root,'target'),'debug/examples',process.platform==='win32'?'scrollable_trace.exe':'scrollable_trace');
const actual=run(binary,[],{input:rows.map(row=>JSON.stringify(row)).join('\n')+'\n'}).trimEnd().split(/\r?\n/).map(line=>JSON.parse(line));
assert.equal(actual.length,expected.length);
function compare(a,b,location='') {
 if(Array.isArray(a)) {if(!Array.isArray(b)||a.length!==b.length)return location;for(let i=0;i<a.length;i++){const mismatch=compare(a[i],b[i],`${location}/${i}`);if(mismatch!==null)return mismatch;}return null;}
 if(typeof a==='string'&&typeof b==='string'&&/^(?:NaN|[-+]?Infinity|[-+]?(?:\d|\.))/.test(a)&&/^(?:NaN|[-+]?Infinity|[-+]?(?:\d|\.))/.test(b)) {
  return Object.is(Number(a),Number(b))?null:location;
 }
 return Object.is(a,b)?null:location;
}
for(let index=0;index<actual.length;index++) {
 const mismatch=compare(expected[index],actual[index]);
 if(mismatch!==null) {
  let start=index;while(start>0&&rows[start][0]!=='new')start--;
  mkdirSync(path.join(root,'tmp'),{recursive:true});
  writeFileSync(path.join(root,'tmp/scrollable-counterexample.json'),JSON.stringify({seed,index,mismatch,rows:rows.slice(start,index+1),expected:expected[index],actual:actual[index]},null,2));
  assert.fail(`scroll mismatch at ${index}${mismatch}; see tmp/scrollable-counterexample.json`);
 }
}
console.log(`Scrollable source differential: ${actual.length} snapshots passed, seed ${seed}`);
