// Test inputs and comparisons only. ScrollState above is generated unchanged.
const fields=['rawScrollLeft','rawScrollTop','width','scrollWidth','scrollLeft','height','scrollHeight','scrollTop'];
const changed=['widthChanged','scrollWidthChanged','scrollLeftChanged','heightChanged','scrollHeightChanged','scrollTopChanged'];
function sourceRun(integer, args, action, update, raw) {
  const previous=new ScrollState(integer,...args);
  const state=action==='position'?previous.withScrollPosition(update):
    action==='dimensions'?previous.withScrollDimensions(update,raw):previous;
  const event=state.createScrollEvent(previous,false);
  return [...fields.map(key=>state[key]),...changed.map(key=>event[key]),state.equals(state)];
}
function record(run,fixture) {
  try {return {values:run(...fixture.args),log:fixture.log};}
  catch(error){return {error,log:fixture.log};}
}
function compare(run,build,label) {
  const a=build(),b=build();
  const expected=record(sourceRun,a),actual=record(run,b);
  const fail=message=>{throw new Error(`${label}: ${message}`);};
  if(JSON.stringify(expected.log)!==JSON.stringify(actual.log))fail(`coercion/getter trace ${JSON.stringify(actual.log)} != ${JSON.stringify(expected.log)}`);
  if('error' in expected||'error' in actual){
    if(!('error' in expected)||!('error' in actual))fail('only one implementation threw');
    if(expected.error===a.token){if(actual.error!==b.token)fail('thrown identity changed');}
    else if(expected.error?.name!==actual.error?.name)fail('exception type changed');
    return;
  }
  if(expected.values.length!==actual.values.length)fail('snapshot field count');
  for(let i=0;i<expected.values.length;i++){
    const x=expected.values[i],y=actual.values[i];
    const identity=a.objects.indexOf(x);
    if(identity>=0){if(b.objects[identity]!==y)fail(`raw object identity at field ${i}`);}
    else if(!Object.is(x,y))fail(`field ${i}: ${String(y)} != ${String(x)}`);
  }
}
function plain(integer,args,action='none',update={},raw=false) {
  return {args:[integer,args,action,update,raw],log:[],objects:[],token:{}};
}
export function runRawStatePrimitives(run) {
  const values=[undefined,null,false,true,-0,NaN,Infinity,-Infinity,-4,2.5,'3','20','bad',2n,Symbol('bad')];
  let count=0;
  for(const integer of [false,true])for(let field=0;field<6;field++)for(const value of values){
    compare(run,()=>{const args=[10,100,20,10,100,20];args[field]=value;return plain(integer,args);},`primitive ${integer}/${field}/${String(value)}`);
    count++;
  }
  return count;
}
export function runRawStateObjects(run) {
  let count=0;
  for(const integer of [false,true])for(const throwsAt of [0,1,2,3,4,5,6,7,8,9,10,11,12]){
    compare(run,()=>{
      const fixture=plain(integer,[]);let calls=0;
      const args=[10,100,20,10,100,20].map((value,index)=>({
        [Symbol.toPrimitive](hint){fixture.log.push(`${index}:${hint}`);if(++calls===throwsAt)throw fixture.token;return value;}
      }));
      fixture.objects=args;fixture.args[1]=args;return fixture;
    },`objects ${integer}/${throwsAt}`);
    count++;
  }
  // String primitives from objects must concatenate before relational coercion.
  compare(run,()=>{
    const fixture=plain(false,[10,100,0,10,100,0]);
    const value={[Symbol.toPrimitive](hint){fixture.log.push(hint);return '2';}};
    fixture.args[1][2]=value;fixture.args[1][5]=value;fixture.objects=[value];return fixture;
  },'object concatenation');
  return count+1;
}
export function runRawStateUpdates(run) {
  let count=0;
  for(const action of ['position','dimensions'])for(const raw of [false,true])for(const integer of [false,true]){
    const keys=action==='position'?['scrollLeft','scrollTop']:['width','scrollWidth','height','scrollHeight'];
    for(const throwsAt of [0,1,2,3,4,5,6,7,8]){
      compare(run,()=>{
        const fixture=plain(integer,[10,100,'200',10,100,'200'],action,undefined,raw);let calls=0;
        const proto={};
        for(const key of keys)Object.defineProperty(proto,key,{get(){
          fixture.log.push(key);if(++calls===throwsAt)throw fixture.token;
          // The second getter may become undefined and must not fall back.
          return calls%2===0?undefined:30;
        }});
        fixture.args[3]=Object.create(proto);return fixture;
      },`getter ${action}/${raw}/${integer}/${throwsAt}`);
      count++;
    }
    compare(run,()=>plain(integer,[10,100,'200',10,100,'200'],action,{},raw),`absent ${action}/${raw}/${integer}`);
    count++;
  }
  return count;
}
