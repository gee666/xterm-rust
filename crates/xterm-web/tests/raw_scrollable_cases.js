// Source and Rust receive independent inputs and the same action program.
function sourceRun(duration, setup) {
  let now=1000, next=0, failRequest, failCancel;
  const active=new Map(), trace=[], events=[];
  const oldNow=Date.now;
  Date.now=()=>now;
  const model=new Scrollable({forceIntegerValues:false, smoothScrollDuration:duration,
    scheduleAtNextAnimationFrame(callback) {
      if(failRequest!==undefined)throw failRequest;
      const id=++next;active.set(id,callback);trace.push(['request',id]);
      return {dispose() {if(failCancel!==undefined)throw failCancel;active.delete(id);trace.push(['cancel',id]);}};
    }});
  const fields=s=>[s.rawScrollLeft,s.rawScrollTop,s.width,s.scrollWidth,s.scrollLeft,s.height,s.scrollHeight,s.scrollTop];
  const listener=model.onScroll(e=>events.push([e.inSmoothScrolling,
    [undefined,undefined,e.oldWidth,e.oldScrollWidth,e.oldScrollLeft,e.oldHeight,e.oldScrollHeight,e.oldScrollTop],
    [undefined,undefined,e.width,e.scrollWidth,e.scrollLeft,e.height,e.scrollHeight,e.scrollTop],
    [e.widthChanged,e.scrollWidthChanged,e.scrollLeftChanged,e.heightChanged,e.scrollHeightChanged,e.scrollTopChanged]]));
  function dispatch(a) {
    try {
      let result;
      switch(a[0]) {
        case 'dimensions': result=model.setScrollDimensions({width:a[1],scrollWidth:a[2],height:a[3],scrollHeight:a[4]},a[5]);break;
        case 'now': result=model.setScrollPositionNow({scrollLeft:a[1],scrollTop:a[2]});break;
        case 'smooth': result=model.setScrollPositionSmooth({scrollLeft:a[1],scrollTop:a[2]},a[3]);break;
        case 'duration': result=model.setSmoothScrollDuration(a[1]);break;
        case 'clock': now=a[1];break;
        case 'frame': {const item=active.entries().next().value;if(item){active.delete(item[0]);item[1]();}break;}
        case 'failRequest': failRequest=a[1];break;
        case 'failCancel': failCancel=a[1];break;
        case 'validate': {const p=model.validateScrollPosition({scrollLeft:a[1],scrollTop:a[2]});result=[p.scrollLeft,p.scrollTop];break;}
        case 'dispose': result=model.dispose();break;
        case 'snapshot': {const p=model.getFutureScrollPosition();result=[fields(model.getScrollDimensions()),[p.scrollLeft,p.scrollTop],model.hasPendingScrollAnimation(),events.slice(),trace.slice(),[...active.keys()]];break;}
        default: throw new Error('unknown live action');
      }
      return [true,result];
    } catch(error) {return [false,error];}
  }
  try{return setup(dispatch);}finally {failCancel=undefined;listener.dispose();model.dispose();Date.now=oldNow;}
}
function normalize(value, objects) {
  if(Array.isArray(value))return value.map(v=>normalize(v,objects));
  if(objects.has(value))return ['identity',objects.get(value)];
  if(value instanceof Error)return ['error',value.name];
  if(typeof value==='number')return Number.isNaN(value)?'NaN':Object.is(value,-0)?'-0':Number.isFinite(value)?value:String(value);
  if(typeof value==='undefined')return 'undefined';
  if(typeof value==='bigint')return ['bigint',String(value)];
  if(typeof value==='symbol')return ['symbol',value.description];
  if(value!==null&&typeof value==='object')throw new Error('unexpected unregistered object');
  return value;
}
function scenario(program) {
  return dispatch=>{
    const output=[], coercions=[], objects=new Map();
    const act=(...args)=>{const result=dispatch(args);output.push(result);return result;};
    const object=(name, primitive, callback)=>{
      const value={[Symbol.toPrimitive](hint){coercions.push([name,hint]);if(callback)callback();return primitive;}};
      objects.set(value,name);return value;
    };
    const token={};objects.set(token,'thrown-token');
    act('dimensions',100,1000,100,1000,false);
    program({act,object,token});
    // Events expose dimensions/positions, not private raw requests. Strip the
    // latter from both event snapshots without altering public assertions.
    const snapshot=act('snapshot');
    if(snapshot[0])for(const event of snapshot[1][3])for(const state of [event[1],event[2]]){state[0]=undefined;state[1]=undefined;}
    const result=normalize([output,coercions],objects);
    dispatch(['failRequest',undefined]);dispatch(['failCancel',undefined]);dispatch(['dispose']);
    return result;
  };
}
export function runLiveCases(run) {
  const tests=[];
  const values=[0,-0,1,25.5,-3,'12',true,false,null,undefined,NaN,Infinity,-Infinity,2n,Symbol('s')];
  for(const value of values)for(const field of ['width','height','left','top','duration']) {
    tests.push([`${field}:${String(value)}`,({act})=>{
      if(field==='width')act('dimensions',value,1000,undefined,undefined,false);
      if(field==='height')act('dimensions',undefined,undefined,value,1000,false);
      if(field==='duration')act('duration',value);
      act('now',field==='left'?value:0,field==='top'?value:0);
      act('validate',field==='left'?value:10,field==='top'?value:20);
      act('smooth',field==='left'?value:100,field==='top'?value:200,false);
      act('clock',1040);act('frame');act('clock',1200);act('frame');
    }]);
  }
  for(const location of ['width','from','to'])for(const reaction of ['now','smooth','dimensions','throw'])for(let cut=1;cut<=12;cut++) {
    tests.push([`${location}/${reaction}/${cut}`,({act,object,token})=>{
      let armed=false, remaining=cut;
      const value=object(location,location==='width'?100:25,()=>{
        if(!armed||--remaining!==0)return;
        armed=false;
        if(reaction==='throw')throw token;
        if(reaction==='now')act('now',5,7);
        if(reaction==='smooth')act('smooth',300,350,false);
        if(reaction==='dimensions')act('dimensions',120,1400,80,1300,true);
      });
      if(location==='width')act('dimensions',value,1000,100,1000,false);
      if(location==='from')act('now',value,30);
      act('smooth',location==='to'?value:200,250,false);
      armed=true;
      if(location==='from'){act('clock',1040);act('frame');}
      else act('dimensions',undefined,1200,80,1300,true);
      act('clock',1060);act('frame');act('clock',1300);act('frame');
    }]);
  }
  for(const location of ['duration','target'])for(const replace of [false,true])for(const reuse of [false,true])
    for(const reaction of ['now','smooth','dimensions','throw'])for(let cut=1;cut<=10;cut++) {
      tests.push([`construct/${location}/${replace}/${reuse}/${reaction}/${cut}`,({act,object,token})=>{
        let armed=false,remaining=cut;
        const value=object(location,location==='duration'?90:250,()=>{
          if(!armed||--remaining!==0)return;
          armed=false;
          if(reaction==='throw')throw token;
          if(reaction==='now')act('now',5,7);
          if(reaction==='smooth')act('smooth',300,350,false);
          if(reaction==='dimensions')act('dimensions',120,1400,80,1300,true);
        });
        if(replace)act('smooth',100,200,false);
        if(location==='duration')act('duration',value);
        armed=true;
        act('smooth',location==='target'?value:200,250,reuse);
        act('clock',1040);act('frame');act('clock',1300);act('frame');
      }]);
    }
  for(const failure of ['failRequest','failCancel'])tests.push([failure,({act,token})=>{
    act('smooth',100,200,false);act(failure,token);
    act('smooth',200,300,true);act('now',20,30);act('clock',1040);act('frame');
    act(failure,undefined);act('clock',1300);act('frame');
  }]);
  // Seeded action programs keep failing inputs reproducible.
  let seed=0x426301;
  const random=()=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed;};
  for(let test=0;test<64;test++) {
    const input=[];
    for(let action=0;action<16;action++)input.push([(random()>>>16)%4,(random()%1400)-200,(random()%1400)-200,(random()>>>16)%2===0]);
    tests.push([`seed426301/${test}`,({act})=>{
      for(const [kind,left,top,reuse] of input) {
        if(kind===0)act('now',left,top);
        else if(kind===1)act('smooth',left,top,reuse);
        else if(kind===2)act('dimensions',Math.abs(left)%300,1400,Math.abs(top)%300,1400,reuse);
        else {act('clock',1000+Math.abs(top));act('frame');}
      }
    }]);
  }
  for(const [label,program] of tests) {
    const expected=sourceRun(90,scenario(program)), actual=run(90,scenario(program));
    if(JSON.stringify(actual)!==JSON.stringify(expected))throw new Error(`${label}\nexpected ${JSON.stringify(expected)}\nactual ${JSON.stringify(actual)}`);
  }
  return tests.length;
}
