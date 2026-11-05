// Nested frame delivery retains the original callback array through replacement.
export function runRenderCallbackCases(run) {
  let count=0;
  const tokens=[{},undefined,null,false,0,'callback-error',Symbol('callback-error'),19n];
  const modes=['plain','appendAfter','disposeAfter','caughtInnerError','propagatedInnerError',
    'outerErrorAfter','appendDuringNested','renderDuringNested'];
  for(const token of tokens) for(const mode of modes) {
    const program=host=>dispatch=>{
      const act=(...args)=>{
        const [ok,value]=dispatch(args);
        host.events.push(['result',args[0],ok,!ok&&Object.is(value,token)?'original-token':value]);
        return [ok,value];
      };
      const deliver=()=>{
        try {host.frame();host.events.push(['delivered']);}
        catch(error){host.events.push(['delivery-error',Object.is(error,token)?'original-token':String(error)]);}
      };
      let nested=false,thrown=false,appended=false;
      const callbackA=()=>{
        if(nested)return;
        nested=true;
        act('callback','C');
        if(mode==='renderDuringNested')act('refresh',3,6,8);
        if(mode==='caughtInnerError') {
          try {host.frame();}
          catch(error){host.events.push(['caught-inner',Object.is(error,token)?'original-token':String(error)]);}
        } else host.frame();
        if(mode==='appendAfter'||mode==='outerErrorAfter')act('callback','D');
        if(mode==='disposeAfter')act('dispose');
        if(mode==='outerErrorAfter')throw token;
      };
      const callbackB=()=>{
        if(!thrown&&(mode==='caughtInnerError'||mode==='propagatedInnerError')){
          thrown=true;throw token;
        }
        if(!appended&&mode==='appendDuringNested'){
          appended=true;act('callback','E');
        }
      };
      act('callback','A',callbackA);
      act('callback','B',callbackB);
      deliver();
      host.events.push(['pending',host.snapshot()]);
      // Recovery also observes whether an outer throw retained the old list or
      // a nested successful replacement left the new list with only D.
      act('callback','tail');act('refresh',0,2,8);
      deliver();deliver();deliver();
      host.events.push(['pending',host.snapshot()]);
      act('dispose');
      return host.result();
    };
    const source=harness(token),actual=harness(token);
    try {
      const expected=sourceRun(source,program(source));
      if(mode==='plain') {
        const callbacks=expected.filter(row=>row[0]==='callback').map(row=>row[1]);
        const known=['A','A','B','C','B','C','tail'];
        if(JSON.stringify(callbacks)!==JSON.stringify(known))
          throw new Error('source nested callback assertion '+JSON.stringify(callbacks));
      }
      const result=run(actual,program(actual));
      if(JSON.stringify(result)!==JSON.stringify(expected))
        throw new Error('render callback/'+mode+'/'+typeof token+' expected '+JSON.stringify(expected)+' actual '+JSON.stringify(result));
    }finally{source.cleanup();actual.cleanup();}
    count++;
  }
  return count;
}
