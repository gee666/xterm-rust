// Same real-addon operations run against unchanged source and Rust/WASM.
export async function gposAddonCases(create, input, cases) {
  const owner=globalThis.window??{},saved=[[globalThis,'window'],[globalThis,'navigator'],[owner,'queryLocalFonts'],[console,'warn'],[console,'debug']]
    .map(([object,key])=>[object,key,Object.getOwnPropertyDescriptor(object,key)]);
  let active,reads=0;
  const families=cases.map((_,i)=>({family:'GPOS'+i,async blob(){reads++;return{async arrayBuffer(){active=input(i).slice();return active.buffer;}};}}));
  Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
  Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
  Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>families});
  const results=[];
  function mutate(m){
    const view=new DataView(active.buffer),woff=view.getUint32(0)===0x774f4646;
    for(let i=0;i<view.getUint16(woff?12:4);i++){
      const at=(woff?44:12)+i*(woff?20:16);
      if(String.fromCharCode(...active.subarray(at,at+4))===m.tag){view.setUint16(view.getUint32(at+(woff?4:8))+m.at,m.value);return;}
    }
    throw new Error('Missing mutation table '+m.tag);
  }
  try{
    for(const [index,item]of cases.entries()){
      let join,complete,refreshes=0;reads=0;
      const warnings=[];
      const terminal={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){if(complete){const done=complete;complete=undefined;done();}return 'GPOS'+index;}},registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
      const thrown=item.throws==='object'?{}:item.throws==='number'?17:item.throws==='null'?null:undefined;
      Object.defineProperty(console,'warn',{configurable:true,value(message){
        warnings.push(message);
        if(item.throws)throw thrown;
        if(warnings.length===1&&item.warningMutation)mutate(item.warningMutation);
      }});
      Object.defineProperty(console,'debug',{configurable:true,value(){}});
      const addon=create({fallbackLigatures:['===']});
      try{
        addon.activate(terminal);
        const loading=join('===');await new Promise(resolve=>{complete=resolve;});
        const ranges=join('==='),before=reads;
        join('===');join('===');
        results.push({loading,ranges,reads,cacheReads:reads-before,refreshes,warnings});
      }finally{addon.dispose();}
    }
    return results;
  }finally{
    for(const [object,key,descriptor]of saved.reverse()){
      if(descriptor)Object.defineProperty(object,key,descriptor);else delete object[key];
    }
  }
}
