// The real browser adapter resolves its queue before scheduling. The source
// injects the same callback at its scheduler boundary. No animation is delivered
// during these synchronous programs, so they do not depend on wall-clock timing.
function sourceQueueRun(setup) {
  let hook;
  const model=new Scrollable({forceIntegerValues:false,smoothScrollDuration:90,
    scheduleAtNextAnimationFrame() {if(hook)hook();return {dispose(){}};}});
  const dispatch=a=>{
    try {
      let result;
      switch(a[0]) {
        case 'hook': hook=a[1];break;
        case 'dimensions': model.setScrollDimensions({width:a[1],scrollWidth:a[2],height:a[3],scrollHeight:a[4]},false);break;
        case 'now': model.setScrollPositionNow({scrollTop:a[1]});break;
        case 'smooth': model.setScrollPositionSmooth({scrollTop:a[1]},a[2]);break;
        case 'dispose': model.dispose();break;
        case 'snapshot': {
          const s=model.getScrollDimensions(),p=model.getFutureScrollPosition();
          result=[s.width,s.scrollWidth,s.height,s.scrollHeight,s.scrollTop,p.scrollTop,model.hasPendingScrollAnimation()];break;
        }
        default: throw new Error('unknown queue action');
      }
      return [true,result];
    }catch(error){return [false,error];}
  };
  try{return setup(dispatch);}finally{hook=undefined;model.dispose();}
}
export function runBrowserSchedulerCases(run) {
  let count=0;
  for(const phase of ['initial','replace','reuse'])
    for(const reaction of ['now','smooth','dimensions','dispose','throw','smoothThrow']) {
      const program=dispatch=>{
        const output=[],token={};
        const act=(...a)=>{const result=dispatch(a);output.push(result);return result;};
        act('dimensions',100,1000,100,1000);
        if(phase!=='initial')act('smooth',100,false);
        act('hook',()=>{
          act('hook',undefined);
          if(reaction==='now')act('now',7);
          if(reaction==='smooth'||reaction==='smoothThrow')act('smooth',300,false);
          if(reaction==='dimensions')act('dimensions',120,1400,80,1300);
          if(reaction==='dispose')act('dispose');
          if(reaction==='throw'||reaction==='smoothThrow')throw token;
        });
        act('smooth',200,phase==='reuse');act('snapshot');
        act('hook',undefined);act('now',9);act('snapshot');act('dispose');
        return output.map(([ok,value])=>[ok,value===token?'thrown-identity':value]);
      };
      const expected=sourceQueueRun(program),actual=run(program);
      if(JSON.stringify(actual)!==JSON.stringify(expected))
        throw new Error('browser queue/'+phase+'/'+reaction+' expected '+JSON.stringify(expected)+' actual '+JSON.stringify(actual));
      count++;
    }
  return count;
}
