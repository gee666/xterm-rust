// Source-derived inputs and comparisons only. Operation above is generated.
function sourceOperation(from, to, timing, actions) {
  const point = fields => ({scrollLeft:fields[0],scrollTop:fields[1],width:fields[2],height:fields[3]});
  let op;
  if(timing[0]){
    const now=Date.now;
    try { Date.now=()=>timing[1];op=SmoothScrollingOperation.start(point(from),point(to),timing[2]); }
    finally { Date.now=now; }
  } else op=new SmoothScrollingOperation(point(from),point(to),timing[1],timing[2]);
  const output=[[op.startTime,op.duration]];
  for(const [kind,value] of actions){
    if(kind==='hook'){value();continue;}
    let result;
    try {
      let update;
      if(kind==='accept')op.acceptScrollDimensions(new ScrollState(false,...value));
      else if(kind==='tick'){
        const tick=op._tick(value);update=[tick.scrollLeft,tick.scrollTop,tick.isDone];
      } else throw new Error('unknown test action');
      result=[true,update];
    } catch(error){result=[false,error];}
    output.push([result,op.to.scrollLeft,op.to.scrollTop,op.to.width,op.to.height]);
  }
  return output;
}
function fixture(from=[20,30,100,100],to=[800,700,100,100],timing=[false,0,100]) {
  return {args:[from,to,timing,[-10,0,10,32.999,33,50,99,100,NaN,Infinity].map(now=>['tick',now])],
    log:[],objects:[],token:{}};
}
function captured(run,fixture) {
  try {return [true,run(...fixture.args)];}
  catch(error){return [false,error];}
}
function normalized(value,fixture) {
  if(value===fixture.token)return ['token'];
  const identity=fixture.objects.indexOf(value);
  if(identity>=0)return ['identity',identity];
  if(Array.isArray(value))return value.map(item=>normalized(item,fixture));
  if(value===null)return ['null'];
  if(typeof value==='number'){
    if(Number.isNaN(value))return ['number','NaN'];
    if(Object.is(value,-0))return ['number','-0'];
    return ['number',String(value)];
  }
  if(typeof value==='symbol')return ['symbol',value.description];
  if(typeof value==='bigint')return ['bigint',String(value)];
  if(typeof value==='object')return ['error',value.name];
  return [typeof value,value];
}
function compare(run,build,label) {
  const a=build(),b=build();
  const expected=captured(sourceOperation,a),actual=captured(run,b);
  const x=JSON.stringify(normalized(expected,a)),y=JSON.stringify(normalized(actual,b));
  if(JSON.stringify(a.log)!==JSON.stringify(b.log))throw new Error(
    label+' coercions: '+JSON.stringify(b.log)+' != '+JSON.stringify(a.log));
  if(x!==y)throw new Error(label+' snapshots: '+y+' != '+x);
}
export function runAnimationPrimitives(run) {
  const values=[undefined,null,false,true,-0,NaN,Infinity,-Infinity,-4,2.5,'3','20','bad',2n,Symbol('bad'),'',0,100];
  let count=0;
  for(const start of [false,true])for(let field=0;field<8;field++){
    if(start&&field===6)continue; // Date.now itself always returns a number.
    for(const value of values){
      compare(run,()=>{
        const f=fixture();f.args[2][0]=start;
        if(field<2)f.args[0][field]=value;
        else if(field<6)f.args[1][field-2]=value;
        else f.args[2][field-5]=value;
        return f;
      },'primitive '+start+'/'+field+'/'+String(value));count++;
    }
  }
  return count;
}
function objectFixture(direction,throwsAt,mutates=false) {
  const from=direction===1?[800,700,100,100]:[20,30,100,100];
  const to=direction===1?[20,30,100,100]:direction===2?[40,50,100,100]:[800,700,100,100];
  const f=fixture(from,to);let calls=0,phase=0;
  for(const [group,values] of [[0,from],[1,to]]){
    for(let field=0;field<4;field++){
      const number=values[field];
      const object={
        [Symbol.toPrimitive](hint){
          f.log.push([phase,group,field,hint]);
          if(++calls===throwsAt)throw f.token;
          return mutates&&phase?String(number+5):number;
        }
      };
      values[field]=object;f.objects.push(object);
    }
  }
  if(mutates)f.args[3].splice(1,0,['hook',()=>{phase=1;}]);
  return f;
}
export function runAnimationObjects(run) {
  let count=0;
  for(let direction=0;direction<3;direction++){
    for(let throwsAt=0;throwsAt<=46;throwsAt++){
      compare(run,()=>objectFixture(direction,throwsAt),'object '+direction+'/'+throwsAt);count++;
    }
    compare(run,()=>objectFixture(direction,0,true),'mutating cached delta '+direction);count++;
  }
  compare(run,()=>{
    const f=fixture();f.args[2]=[true,0,{
      [Symbol.toPrimitive](hint){f.log.push(['duration',hint]);f.args[2][1]=1000;return '90';}
    }];return f;
  },'duration addition precedes clock sample');count++;
  return count;
}
export function runAnimationDimensions(run) {
  let count=0;
  for(let direction=0;direction<2;direction++)for(let throwsAt=0;throwsAt<=60;throwsAt++){
    compare(run,()=>{
      const f=objectFixture(direction,0);
      let calls=0,armed=false;
      const dims=[80,2000,0,90,2000,0].map((number,index)=>{
        const object={
          [Symbol.toPrimitive](hint){
            f.log.push(['accept',index,hint]);
            if(armed&&++calls===throwsAt)throw f.token;
            return number;
          }
        };
        f.objects.push(object);return object;
      });
      f.args[3]=[['tick',20],['hook',()=>{armed=true;}],['accept',dims],
        ['hook',()=>{armed=false;}],['tick',20],['tick',50],['tick',100]];
      return f;
    },'accept '+direction+'/'+throwsAt);count++;
  }
  for(const [left,top] of [[undefined,70],[80,undefined],[undefined,undefined]]){
    compare(run,()=>{
      const f=fixture([0,0,100,100],[left,top,100,100]);
      f.args[3]=[['accept',[100,1000,25,100,1000,35]],['tick',50],['tick',100]];
      return f;
    },'undefined target acceptance '+String(left)+'/'+String(top));count++;
  }
  return count;
}
