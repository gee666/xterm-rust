// Browser fixture only: real DOM delivery, controlled clocks/RAF and observations.
function createBrowserTouchHost() {
  const host = { log:[], labels:new WeakMap(), meta:new WeakMap(), time:1000, tapTime:undefined,
    recording:false, frames:new Map(), next:0, listeners:[] };
  const w = window, doc = document;
  const original = { Date:w.Date, create:doc.createEvent, raf:w.requestAnimationFrame,
    cancel:w.cancelAnimationFrame, warn:console.warn, error:console.error,
    touch:Object.getOwnPropertyDescriptor(w,'ontouchstart') };
  const node = label => { const value=doc.createElement('div'); host.labels.set(value,label); return value; };
  host.document=doc; host.labels.set(doc,'document'); host.labels.set(doc.body,'body'); host.labels.set(doc.documentElement,'html');
  host.root=node('root'); host.child=node('child');
  host.leaf=node('leaf'); host.outside=node('outside');
  host.root.appendChild(host.child); host.child.appendChild(host.leaf);
  doc.body.appendChild(host.root); doc.body.appendChild(host.outside);
  Object.defineProperty(w,'ontouchstart',{configurable:true,writable:true,value:null});
  const clock = (kind,hook,value) => {
    if(host.recording)host.log.push('clock:'+kind+':enter');
    if(host[hook])host[hook]();
    const result=value();
    if(host.recording)host.log.push('clock:'+kind+':'+host.encode(result));
    return result;
  };
  class Date extends original.Date {
    constructor(...args){super(...(args.length?args:[host.time]));if(!args.length&&host.recording)host.log.push('clock:new');}
    static now(){return clock('now','nowHook',()=>host.time);}
    getTime(){return clock('tap','tapHook',()=>host.tapTime??host.time);}
  }
  w.Date=Date;
  w.requestAnimationFrame=callback=>{const id=++host.next;host.frames.set(id,callback);return id;};
  w.cancelAnimationFrame=id=>host.frames.delete(id);
  const report=value=>{
    if(!host.recording)return;
    const tag=host.labels.get(value);
    host.log.push('error:'+(tag?'@'+tag:value?.name??typeof value));
  };
  const errors=event=>{if(host.recording){report(event.error);event.preventDefault();event.stopImmediatePropagation();}};
  // Negative cases assert the full error ledger against source. Isolate those
  // deliberate listener failures from the generic WASM-test global error hook.
  w.addEventListener('error',errors,true);
  console.warn=(text,touch)=>{if(host.recording)host.log.push('warn:'+text+':'+host.encode(touch));else original.warn.call(console,text,touch);};
  console.error=value=>{if(host.recording)report(value);else original.error.call(console,value);};
  host.installCreate=factory=>{doc.createEvent=function(type){const event=original.create.call(this,type);return type==='CustomEvent'?factory(event):event;};};
  host.input=kind=>{const event=original.create.call(doc,'Event');event.initEvent(kind,false,true);return event;};
  host.dispatch=event=>doc.dispatchEvent(event);
  host.listen=(target,type,callback)=>{target.addEventListener(type,callback);host.listeners.push([target,type,callback]);};
  host.flush=()=>{const frames=[...host.frames.values()];host.frames.clear();for(const callback of frames)callback(0);};
  host.pending=()=>host.frames.size;
  host.restore=()=>{
    host.recording=false;
    for(const [target,type,callback]of host.listeners)target.removeEventListener(type,callback);
    host.listeners.length=0;w.removeEventListener('error',errors,true);
    doc.createEvent=original.create;w.Date=original.Date;
    w.requestAnimationFrame=original.raf;w.cancelAnimationFrame=original.cancel;
    console.warn=original.warn;console.error=original.error;
    if(original.touch)Object.defineProperty(w,'ontouchstart',original.touch);else delete w.ontouchstart;
    host.frames.clear();host.root.remove();host.outside.remove();
  };
  return host;
}
