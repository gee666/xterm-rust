// Real addon loading and diagnostic callbacks, without a source parser in WASM.
export async function colorAddonScenarios(create,base,insert,payload) {
  const owner=globalThis.window??{};
  const saved=[[globalThis,'window'],[globalThis,'navigator'],[owner,'queryLocalFonts'],[console,'warn'],[console,'debug']].map(([o,k])=>[o,k,Object.getOwnPropertyDescriptor(o,k)]);
  if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
  Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
  const modes=['CPALInvalid','COLRInvalid','CPALWarning','COLRWarning','CPALMutable','COLRMutable','CPALDescriptor','COLRDescriptor','ObjectThrow','NumberThrow','NullThrow','UndefinedThrow'];
  let family,join,complete,reads,refreshes,warnings,previousIdentity,inWarning=false,mutable;
  let thrown;
  const dataFor=mode=>{
    if(mode==='Good')return base.slice();
    const tag=mode.startsWith('COLR')?'COLR':'CPAL',bad=mode.endsWith('Invalid');
    const data=insert(base,payload(tag,{version:bad?0:1,palettes:bad?65535:0,bases:bad?1:0,length:14}));
    const view=new DataView(data.buffer),record=12+(view.getUint16(4)-1)*16;
    data.set([...tag].map(c=>c.charCodeAt(0)),record);
    return data;
  };
  Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>[...modes,'AfterThrow','Good'].map(name=>({family:name,async blob(){reads++;return {async arrayBuffer(){mutable=dataFor(name);return mutable.buffer;}};}}))});
  const term={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){if(complete&&!inWarning){const done=complete;complete=undefined;done();}return family;}},registerCharacterJoiner(fn){join=fn;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
  const results={};
  Object.defineProperty(console,'debug',{configurable:true,value(previous){previousIdentity=Object.is(previous,thrown);}});
  Object.defineProperty(console,'warn',{configurable:true,value(message){
    warnings.push(message);
    if(family.endsWith('Throw'))throw thrown;
    const view=new DataView(mutable.buffer),record=12+(view.getUint16(4)-1)*16,offset=view.getUint32(record+8);
    if(family.endsWith('Mutable'))view.setUint16(offset+(family.startsWith('COLR')?2:4),65535);
    if(family.endsWith('Descriptor'))view.setUint32(record+8,0xfffffff0);
    if(family.endsWith('Warning')){inWarning=true;try{join('===');}finally{inWarning=false;}}
  }});
  try {
    for(const mode of modes) {
      family=mode;reads=0;refreshes=0;warnings=[];previousIdentity=false;
      thrown=mode==='NumberThrow'?42:mode==='NullThrow'?null:mode==='UndefinedThrow'?undefined:{};
      const addon=create({fallbackLigatures:['===']});
      async function load(next) {family=next;const loading=join('===');await new Promise(resolve=>{complete=resolve;});return {loading,ranges:join('==='),reads,refreshes};}
      try {
        addon.activate(term);
        const first=await load(mode);join('===');join('===');const cached={reads,refreshes};
        if(mode.endsWith('Throw'))await load('AfterThrow');
        const recovery=await load('Good');
        results[mode]={first,cached,warnings:[...warnings],previousIdentity,recovery};
      } finally {addon.dispose();}
    }
    return results;
  } finally {
    for(const [object,key,descriptor]of saved.reverse())if(descriptor)Object.defineProperty(object,key,descriptor);else delete object[key];
  }
}
