// Identical operations run against the unchanged source addon and Rust/WASM.
export async function outlineLoadingScenario(create, inputs, good) {
 const owner=globalThis.window??{};
 const descriptors=[[globalThis,'window'],[globalThis,'navigator'],[owner,'queryLocalFonts'],[console,'debug']]
  .map(([object,key])=>[object,key,Object.getOwnPropertyDescriptor(object,key)]);
 if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
 Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
 const modes=['Reject','CompressedReject','LazyOffsets','EmptyGlyf'];
 const fetches={};let active;
 Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>modes.flatMap(mode=>
  [mode,mode+'Good'].map(family=>({family,async blob(){fetches[family]=(fetches[family]??0)+1;return {async arrayBuffer(){
   active=(family.endsWith('Good')?good:inputs[mode]).slice();return active.buffer;
  }};}})))});
 const results={};
 try {
  for(const mode of modes){
   let join,completed,family=mode,refreshes=0;const errors=[];
   const terminal={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){
    if(completed){const done=completed;completed=undefined;done();}return family;
   }},registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
   Object.defineProperty(console,'debug',{configurable:true,value(previous,error){errors.push({previousUndefined:previous===undefined,name:error.name,message:error.message});}});
   const addon=create({fallbackLigatures:['===']});
   try {
    addon.activate(terminal);
    const loading=join('===');await new Promise(resolve=>{completed=resolve;});
    const first={loading,ranges:join('==='),refreshes,errors:errors.slice(),fetches:fetches[mode]};
    join('===');join('===');
    const cached={fetches:fetches[mode],errors:errors.length,refreshes};
    family=mode+'Good';const retry=join('===');await new Promise(resolve=>{completed=resolve;});
    results[mode]={first,cached,retry,accepted:{ranges:join('==='),refreshes,fetches:fetches[family]}};
   }finally{addon.dispose();}
  }
  return results;
 }finally{
  for(const [object,key,descriptor]of descriptors.reverse()){
   if(descriptor)Object.defineProperty(object,key,descriptor);else delete object[key];
  }
 }
}
