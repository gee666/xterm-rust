// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Only observation and source test driving. Rust renders its own canvas.
let trace, undo, dprDescriptor, patched, throwNext, reenter, scheduled, dprHook;
const failure = {sentinel: 'canvas-layer-failure'};
export function beginLayerCapture() {
  trace=[]; undo=[]; throwNext=undefined; reenter=undefined; scheduled=0; dprHook=undefined;
  const scheduler='requestIdleCallback' in globalThis?'requestIdleCallback':'setTimeout';
  const originalScheduler=window[scheduler];
  window[scheduler]=function(...args){scheduled++;return originalScheduler.apply(this,args);};
  undo.push(()=>{window[scheduler]=originalScheduler;});
  dprDescriptor=Object.getOwnPropertyDescriptor(window,'devicePixelRatio');
  Object.defineProperty(window,'devicePixelRatio',{configurable:true,get(){if(dprHook){const callback=dprHook;dprHook=undefined;callback();}return 1.5;}});
  const contexts=new WeakSet();
  const prototype=HTMLCanvasElement.prototype, original=prototype.getContext;
  prototype.getContext=function(type, options) {
    const selected=this.classList.contains('xterm-link-layer')||this.classList.contains('xterm-test-layer');
    if(selected) {
      trace.push(['context',type,options]);
      if(throwNext==='context') {throwNext=undefined;throw failure;}
      if(throwNext==='null') {throwNext=undefined;return null;}
    }
    const context=original.call(this,type,options);
    if(selected && context && !contexts.has(context)) {
      contexts.add(context);
      for(const key of ['fillRect','clearRect','beginPath','rect','clip','fillText']) {
        const original=context[key];
        context[key]=function(...args) {
          trace.push([key,...args]);
          if(throwNext===key) {throwNext=undefined;throw failure;}
          if(reenter?.key===key) { const callback=reenter.callback;reenter=undefined;callback(); }
          return original.apply(this,args);
        };
      }
      for(const key of ['fillStyle','font','textBaseline']) {
        const descriptor=Object.getOwnPropertyDescriptor(CanvasRenderingContext2D.prototype,key);
        Object.defineProperty(context,key,{configurable:true,get(){return descriptor.get.call(this);},
          set(value){trace.push([key,value]);descriptor.set.call(this,value);}});
      }
    }
    return context;
  };
  undo.push(()=>{prototype.getContext=original;});
  patched=true;
}
export function endLayerCapture() {
  const result=JSON.stringify(trace);
  if(patched) {
    for(const callback of undo.reverse())callback();
    if(dprDescriptor)Object.defineProperty(window,'devicePixelRatio',dprDescriptor);
    else delete window.devicePixelRatio;
  }
  patched=false; trace=undefined;reenter=undefined;
  return result;
}
export function layerNote(value) { trace.push(['note',value]); }
export function layerScheduled() {layerNote('scheduled:'+scheduled);}
export function failLayer(key) { throwNext=key; }
export function layerFailure() {return failure;}
export function reenterLayer(key, callback) { reenter={key,callback}; }
export function reenterLayerDpr(callback) {dprHook=callback;}
export function layerSnapshot(canvas) {
  const pixels=canvas.getContext('2d').getImageData(0,0,canvas.width,canvas.height).data;
  let hash=2166136261;
  for(const byte of pixels)hash=Math.imul(hash^byte,16777619)>>>0;
  trace.push(['snapshot',canvas.width,canvas.height,canvas.style.width,canvas.style.height,hash]);
}
export function sourceLayer(name) {return globalThis.sourceCanvasLayer(name);}
