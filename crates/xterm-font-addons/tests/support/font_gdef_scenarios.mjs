// Shared unchanged-source/Rust addon operations. Inputs are independently rebuilt
// by Node and Rust from the hash-bound constructor corpus.
export async function gdefLoadingScenario(create, inputs, good) {
  const owner=globalThis.window??{};
  const descriptors=[[globalThis,'window'],[globalThis,'navigator'],[owner,'queryLocalFonts'],[console,'warn'],[console,'debug']]
    .map(([object,key])=>[object,key,Object.getOwnPropertyDescriptor(object,key)]);
  if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
  Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
  const modes=['Warnings','Throw','Mutate','Reject','Compressed','LaterMeta'];
  let active;
  Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>modes.flatMap(mode=>
    [mode,mode+'Again',mode+'Good'].map(family=>({family,async blob(){return {async arrayBuffer(){
      active=(family.endsWith('Good')?good:inputs[mode]).slice();return active.buffer;
    }};}})))});
  function patch(tag,at,value){
    const view=new DataView(active.buffer),woff=view.getUint32(0)===0x774f4646;
    for(let i=0;i<view.getUint16(woff?12:4);i++){
      const record=(woff?44:12)+i*(woff?20:16);
      const actual=String.fromCharCode(...active.subarray(record,record+4));
      if(actual===tag){view.setUint16(view.getUint32(record+(woff?4:8))+at,value);return;}
    }
    throw new Error('Missing mutation table '+tag);
  }
  const results={};
  try {
    for(const mode of modes){
      let join,completed,family=mode,refreshes=0,reads=0,previousIdentity=false;
      const trace=[],thrown=Symbol('GDEF warning sentinel');
      const terminal={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){
        if(completed){const done=completed;completed=undefined;done();}return family;
      }},registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
      Object.defineProperty(console,'warn',{configurable:true,get(){trace.push('get');return function(message){
        reads++;trace.push(['call',this===console,message]);
        if(mode==='Throw')throw thrown;
        if(mode==='Mutate'&&reads===1)patch('GDEF',10,0);
        if(mode==='Compressed'&&reads===1)patch('GDEF',10,0);
        if(mode==='LaterMeta'&&reads===1)patch('meta',2,2);
      };}});
      Object.defineProperty(console,'debug',{configurable:true,value(previous){previousIdentity=previous===thrown;}});
      const addon=create({fallbackLigatures:['===']});
      try {
        addon.activate(terminal);
        const loading=join('===');await new Promise(resolve=>{completed=resolve;});
        const first={loading,ranges:join('==='),refreshes,trace:trace.slice()};
        const before=reads;join('===');join('===');
        const cached={reads:reads-before,refreshes};
        if(mode==='Throw'){
          family=mode+'Again';join('===');await new Promise(resolve=>{completed=resolve;});
        }
        const retainedIdentity=previousIdentity;
        family=mode+'Good';const retry=join('===');await new Promise(resolve=>{completed=resolve;});
        results[mode]={first,cached,retainedIdentity,retry,accepted:{ranges:join('==='),refreshes}};
      } finally {addon.dispose();}
    }
    return results;
  } finally {
    for(const [object,key,descriptor]of descriptors.reverse()){
      if(descriptor)Object.defineProperty(object,key,descriptor);else delete object[key];
    }
  }
}
