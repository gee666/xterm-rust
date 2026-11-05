// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Generated test-only intact source oracle. SHA-256: 2600736f0e7b63f05c3e8dd78e1a9a968a74e5cac8a24077aa232bd74bfa1c7c
// source_repo/src/browser/RenderDebouncer.ts
var RenderDebouncer = class {
  constructor(_renderCallback, _coreBrowserService) {
    this._renderCallback = _renderCallback;
    this._coreBrowserService = _coreBrowserService;
    this._refreshCallbacks = [];
  }
  dispose() {
    if (this._animationFrame !== void 0) {
      this._coreBrowserService.window.cancelAnimationFrame(this._animationFrame);
      this._animationFrame = void 0;
    }
  }
  addRefreshCallback(callback) {
    this._refreshCallbacks.push(callback);
    this._animationFrame ??= this._coreBrowserService.window.requestAnimationFrame(() => this._innerRefresh());
    return this._animationFrame;
  }
  refresh(rowStart, rowEnd, rowCount) {
    this._rowCount = rowCount;
    rowStart = rowStart ?? 0;
    rowEnd = rowEnd ?? this._rowCount - 1;
    this._rowStart = this._rowStart !== void 0 ? Math.min(this._rowStart, rowStart) : rowStart;
    this._rowEnd = this._rowEnd !== void 0 ? Math.max(this._rowEnd, rowEnd) : rowEnd;
    if (this._animationFrame !== void 0) {
      return;
    }
    this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => this._innerRefresh());
  }
  _innerRefresh() {
    this._animationFrame = void 0;
    if (this._rowStart === void 0 || this._rowEnd === void 0 || this._rowCount === void 0) {
      this._runRefreshCallbacks();
      return;
    }
    const start = Math.max(this._rowStart, 0);
    const end = Math.min(this._rowEnd, this._rowCount - 1);
    this._rowStart = void 0;
    this._rowEnd = void 0;
    this._renderCallback(start, end);
    this._runRefreshCallbacks();
  }
  _runRefreshCallbacks() {
    for (const callback of this._refreshCallbacks) {
      callback(0);
    }
    this._refreshCallbacks = [];
  }
};
export {
  RenderDebouncer
};
/**
 * Copyright (c) 2018 The xterm.js authors. All rights reserved.
 * @license MIT
 */

// Deterministic synchronous host calls. No real animation is delivered or timed.
function harness() {
  const events=[], frames=new Map(), token={}, windows=[];
  let current=0, next=0, hook;
  const normalize=value=>value===token?'original-token':value;
  const trigger=site=>{if(hook?.site===site){const saved=hook;hook=undefined;saved.action();}};
  for(let index=0;index<2;index++) {
    const owner={};
    Object.defineProperties(owner,{
      requestAnimationFrame:{get(){
        events.push(['get-request',index]);trigger('requestGet');
        return function(callback){
          events.push(['request',index,this===owner]);
          const id=++next;trigger('request');
          frames.set(id,callback);return id;
        };
      }},
      cancelAnimationFrame:{get(){
        events.push(['get-cancel',index]);trigger('cancelGet');
        return function(id){
          events.push(['cancel',index,id,this===owner]);trigger('cancel');frames.delete(id);
        };
      }}
    });
    windows.push(owner);
  }
  return {events,token,windows,
    owner(){events.push(['source']);trigger('source');return windows[current];},
    setHook(site,action){hook={site,action};},
    clear(){hook=undefined;},
    switchWindow(){current=1-current;},
    frame(){const entry=frames.entries().next().value;if(entry){frames.delete(entry[0]);events.push(['frame',entry[0]]);entry[1](91);}},
    snapshot(){return [...frames.keys()];},
    result(){return events.map(row=>row.map(normalize));},
    cleanup(){hook=undefined;frames.clear();}
  };
}
export function sourceRun(host,program) {
  const model=new RenderDebouncer((start,end)=>host.events.push(['render',start,end]),{get window(){return host.owner();}});
  const dispatch=args=>{
    try {
      let result;
      if(args[0]==='refresh') model.refresh(args[1],args[2],args[3]);
      else if(args[0]==='callback') result=model.addRefreshCallback(time=>host.events.push(['callback',args[1],time]));
      else if(args[0]==='dispose') model.dispose();
      else throw new Error('unknown render action');
      return [true,result];
    }catch(error){return [false,error];}
  };
  try{return program(dispatch);}finally{model.dispose();}
}
export function runRenderSchedulerCases(run) {
  let count=0;
  for(const operation of ['refresh','callback','dispose']) {
    const sites=operation==='dispose'?['source','cancelGet','cancel']:['source','requestGet','request'];
    for(const site of sites)
      for(const reaction of ['refresh','callback','dispose','frame','frameRefresh','window','throw','refreshThrow']) {
        // WindowSource is a typed infallible native contract. Throw identity is
        // tested at both actual WebIDL method lookups/calls, not hidden there.
        if(site==='source'&&(reaction==='throw'||reaction==='refreshThrow'))continue;
        const program=host=>dispatch=>{
          const act=(...args)=>{
            const [ok,value]=dispatch(args);
            host.events.push(['result',args[0],ok,value===host.token?'original-token':value]);
          };
          if(operation==='dispose')act('refresh',1,3,10);
          host.setHook(site,()=>{
            host.events.push(['hook',site,reaction]);
            if(reaction==='refresh'||reaction==='refreshThrow')act('refresh',4,15,12);
            if(reaction==='callback')act('callback','nested');
            if(reaction==='dispose')act('dispose');
            if(reaction==='frame'||reaction==='frameRefresh')host.frame();
            if(reaction==='frameRefresh')act('refresh',6,11,14);
            if(reaction==='window')host.switchWindow();
            if(reaction==='throw'||reaction==='refreshThrow')throw host.token;
          });
          if(operation==='refresh')act('refresh',2,5,10);
          if(operation==='callback')act('callback','outer');
          if(operation==='dispose')act('dispose');
          host.clear();
          host.events.push(['pending',host.snapshot()]);
          act('callback','tail');act('refresh',0,2,8);
          host.frame();host.frame();host.frame();
          host.events.push(['pending',host.snapshot()]);
          act('dispose');
          return host.result();
        };
        const source=harness(),actual=harness();
        try {
          const expected=sourceRun(source,program(source));
          const result=run(actual,program(actual));
          if(JSON.stringify(result)!==JSON.stringify(expected))
            throw new Error('render scheduler/'+operation+'/'+site+'/'+reaction+' expected '+JSON.stringify(expected)+' actual '+JSON.stringify(result));
        } finally {source.cleanup();actual.cleanup();}
        count++;
      }
  }
  return count;
}
