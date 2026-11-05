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
  for(const site of ['clockGet','clock']) for(const throws of [false,true]) {
    const program=host=>dispatch=>{
      const act=(...args)=>{
        const [ok,value]=dispatch(args);
        host.events.push(['result',args[0],ok,value===host.token?'original-token':value]);
      };
      // Nested disposal during request lets two host timers coexist. Delivering
      // the second from the first timer's clock hook enters the SAME callback.
      host.setHook('request',()=>{act('dispose');act('refresh',4,6,10);});
      act('refresh',1,3,10);
      host.events.push(['pending',host.snapshot()]);
      host.setHook(site,()=>{host.fire();if(throws)throw host.token;});
      try{host.fire();}catch(error){host.events.push(['fire-error',error===host.token?'original-token':error]);}
      host.clear();host.time(250);act('refresh',0,2,8);
      host.events.push(['pending',host.snapshot()]);act('dispose');
      return host.result();
    };
    const source=harness(),actual=harness();
    try {
      const expected=sourceRun(source,program(source)),result=run(actual,program(actual));
      if(JSON.stringify(result)!==JSON.stringify(expected))
        throw new Error('timer nested delivery/'+site+'/'+throws+' expected '+JSON.stringify(expected)+' actual '+JSON.stringify(result));
    }finally{source.cleanup();actual.cleanup();}
    count++;
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
