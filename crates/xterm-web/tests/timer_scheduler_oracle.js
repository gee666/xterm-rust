// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Generated test-only intact source oracle. SHA-256: 1219c93fea754ff30a17cae65256c17d7555907e620efe6c413194949f15515e
const sourceModel=new Function("host","with(host){\"use strict\";\nvar Source = (() => {\n  var __defProp = Object.defineProperty;\n  var __getOwnPropDesc = Object.getOwnPropertyDescriptor;\n  var __getOwnPropNames = Object.getOwnPropertyNames;\n  var __hasOwnProp = Object.prototype.hasOwnProperty;\n  var __export = (target, all) => {\n    for (var name in all)\n      __defProp(target, name, { get: all[name], enumerable: true });\n  };\n  var __copyProps = (to, from, except, desc) => {\n    if (from && typeof from === \"object\" || typeof from === \"function\") {\n      for (let key of __getOwnPropNames(from))\n        if (!__hasOwnProp.call(to, key) && key !== except)\n          __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });\n    }\n    return to;\n  };\n  var __toCommonJS = (mod) => __copyProps(__defProp({}, \"__esModule\", { value: true }), mod);\n\n  // source_repo/src/browser/TimeBasedDebouncer.ts\n  var TimeBasedDebouncer_exports = {};\n  __export(TimeBasedDebouncer_exports, {\n    TimeBasedDebouncer: () => TimeBasedDebouncer\n  });\n  var RENDER_DEBOUNCE_THRESHOLD_MS = 1e3;\n  var TimeBasedDebouncer = class {\n    constructor(_renderCallback, _debounceThresholdMS = RENDER_DEBOUNCE_THRESHOLD_MS) {\n      this._renderCallback = _renderCallback;\n      this._debounceThresholdMS = _debounceThresholdMS;\n      // The last moment that the Terminal was refreshed at\n      this._lastRefreshMs = 0;\n      // Whether a trailing refresh should be triggered due to a refresh request that was throttled\n      this._additionalRefreshRequested = false;\n    }\n    dispose() {\n      if (this._refreshTimeoutID) {\n        clearTimeout(this._refreshTimeoutID);\n        this._refreshTimeoutID = void 0;\n      }\n      this._additionalRefreshRequested = false;\n    }\n    refresh(rowStart, rowEnd, rowCount) {\n      this._rowCount = rowCount;\n      rowStart = rowStart ?? 0;\n      rowEnd = rowEnd ?? this._rowCount - 1;\n      this._rowStart = this._rowStart !== void 0 ? Math.min(this._rowStart, rowStart) : rowStart;\n      this._rowEnd = this._rowEnd !== void 0 ? Math.max(this._rowEnd, rowEnd) : rowEnd;\n      const refreshRequestTime = performance.now();\n      if (refreshRequestTime - this._lastRefreshMs >= this._debounceThresholdMS) {\n        if (this._refreshTimeoutID !== void 0) {\n          clearTimeout(this._refreshTimeoutID);\n          this._refreshTimeoutID = void 0;\n          this._additionalRefreshRequested = false;\n        }\n        this._lastRefreshMs = refreshRequestTime;\n        this._innerRefresh();\n      } else if (!this._additionalRefreshRequested) {\n        const elapsed = refreshRequestTime - this._lastRefreshMs;\n        const waitPeriodBeforeTrailingRefresh = this._debounceThresholdMS - elapsed;\n        this._additionalRefreshRequested = true;\n        this._refreshTimeoutID = window.setTimeout(() => {\n          this._lastRefreshMs = performance.now();\n          this._innerRefresh();\n          this._additionalRefreshRequested = false;\n          this._refreshTimeoutID = void 0;\n        }, waitPeriodBeforeTrailingRefresh);\n      }\n    }\n    _innerRefresh() {\n      if (this._rowStart === void 0 || this._rowEnd === void 0 || this._rowCount === void 0) {\n        return;\n      }\n      const start = Math.max(this._rowStart, 0);\n      const end = Math.min(this._rowEnd, this._rowCount - 1);\n      this._rowStart = void 0;\n      this._rowEnd = void 0;\n      this._renderCallback(start, end);\n    }\n  };\n  return __toCommonJS(TimeBasedDebouncer_exports);\n})();\n/**\n * Copyright (c) 2018 The xterm.js authors. All rights reserved.\n * @license MIT\n */\n;return Source.TimeBasedDebouncer;}");
// Synchronous host reentry, with explicit timeout delivery and no timed waits.
function harness(...tokens) {
  const token=tokens.length?tokens[0]:{};
  const events=[], timers=new Map();
  let clock=0,next=0,hook;
  const trigger=site=>{if(hook?.site===site){const saved=hook;hook=undefined;saved.action();}};
  const performance={get now(){
    events.push(['get-clock']);trigger('clockGet');
    return function(){events.push(['clock',this===performance]);trigger('clock');return clock;};
  }};
  const window={performance};
  window.window=window;
  Object.defineProperties(window,{
    setTimeout:{get(){
      events.push(['get-request']);trigger('requestGet');
      return function(callback,delay){
        events.push(['request',delay,this===window]);
        const id=++next;trigger('request');timers.set(id,callback);return id;
      };
    }},
    clearTimeout:{get(){
      events.push(['get-cancel']);trigger('cancelGet');
      return function(id){events.push(['cancel',id,this===window]);trigger('cancel');timers.delete(id);};
    }}
  });
  return {events,token,window,
    time(value){clock=value;},
    setHook(site,action){hook={site,action};},clear(){hook=undefined;},
    fire(){const entry=timers.entries().next().value;if(entry){timers.delete(entry[0]);events.push(['fire',entry[0]]);entry[1]();}},
    snapshot(){return [...timers.keys()];},
    result(){return events.map(row=>row.map(value=>value===token?'original-token':value));},
    cleanup(){hook=undefined;timers.clear();}
  };
}
export function sourceRun(host,program) {
  const Model=sourceModel(host.window);
  const model=new Model((start,end)=>host.events.push(['render',start,end]),100);
  const dispatch=args=>{
    try {
      if(args[0]==='refresh')model.refresh(args[1],args[2],args[3]);
      else if(args[0]==='dispose')model.dispose();
      else throw new Error('unknown timer action');
      return [true,undefined];
    }catch(error){return [false,error];}
  };
  try{return program(dispatch);}finally{model.dispose();}
}
export function runTimerSchedulerCases(run) {
  let count=0;
  for(const operation of ['refresh','immediate','dispose','fire']) {
    const sites=operation==='refresh'?['clockGet','clock','requestGet','request']:
      operation==='fire'?['clockGet','clock']:operation==='dispose'?['cancelGet','cancel']:
      ['clockGet','clock','cancelGet','cancel'];
    for(const site of sites) for(const reaction of ['refresh','dispose','fire','fireRefresh','throw','refreshThrow']) {
      const program=host=>dispatch=>{
        const act=(...args)=>{
          const [ok,value]=dispatch(args);
          host.events.push(['result',args[0],ok,value===host.token?'original-token':value]);
        };
        if(operation!=='refresh')act('refresh',1,3,10);
        if(operation==='immediate')host.time(101);
        host.setHook(site,()=>{
          host.events.push(['hook',site,reaction]);
          if(reaction==='refresh'||reaction==='refreshThrow')act('refresh',4,15,12);
          if(reaction==='dispose')act('dispose');
          if(reaction==='fire'||reaction==='fireRefresh')host.fire();
          if(reaction==='fireRefresh')act('refresh',6,11,14);
          if(reaction==='throw'||reaction==='refreshThrow')throw host.token;
        });
        if(operation==='refresh'||operation==='immediate')act('refresh',2,5,10);
        if(operation==='dispose')act('dispose');
        if(operation==='fire')try{host.fire();}catch(error){host.events.push(['fire-error',error===host.token?'original-token':error]);}
        host.clear();host.events.push(['pending',host.snapshot()]);
        act('refresh',0,2,8);host.fire();host.fire();
        host.time(250);act('refresh',2,3,8);
        host.events.push(['pending',host.snapshot()]);act('dispose');
        return host.result();
      };
      const source=harness(),actual=harness();
      try {
        const expected=sourceRun(source,program(source)),result=run(actual,program(actual));
        if(JSON.stringify(result)!==JSON.stringify(expected))
          throw new Error('timer scheduler/'+operation+'/'+site+'/'+reaction+' expected '+JSON.stringify(expected)+' actual '+JSON.stringify(result));
      }finally{source.cleanup();actual.cleanup();}
      count++;
    }
  }
  return count;
}
export function runRawTimerSchedulerCases(run) {
  let count=0;
  for(const token of [undefined,null,false,0,'thrown-string',Symbol('thrown-symbol'),17n]) {
    for(const site of ['clockGet','clock','requestGet','request','cancelGet','cancel']) {
      const program=host=>dispatch=>{
        const act=(...args)=>{
          const [ok,value]=dispatch(args);
          host.events.push(['result',args[0],ok,!ok&&Object.is(value,token)?'original-token':value]);
        };
        if(site.startsWith('cancel'))act('refresh',1,3,10);
        host.setHook(site,()=>{throw token;});
        if(site.startsWith('cancel'))act('dispose');else act('refresh',2,5,10);
        host.clear();host.events.push(['pending',host.snapshot()]);
        host.time(250);act('refresh',0,2,8);host.fire();act('dispose');
        return host.result();
      };
      const source=harness(token),actual=harness(token);
      try {
        const expected=sourceRun(source,program(source)),result=run(actual,program(actual));
        if(JSON.stringify(result)!==JSON.stringify(expected))
          throw new Error('timer raw error/'+typeof token+'/'+site+' expected '+JSON.stringify(expected)+' actual '+JSON.stringify(result));
      }finally{source.cleanup();actual.cleanup();}
      count++;
    }
  }
  return count;
}
