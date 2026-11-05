// Recorded host for actual addon asynchronous construction, not a font engine.
export async function cffHeaderAddonScenarios(create,inputs) {
  const owner=globalThis.window??{};
  const saved=[[globalThis,'window'],[globalThis,'navigator'],[owner,'queryLocalFonts'],[console,'debug']].map(([o,k])=>[o,k,Object.getOwnPropertyDescriptor(o,k)]);
  if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
  Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
  let family,join,complete,reads=0,refreshes=0;
  const debug=[];
  Object.defineProperty(console,'debug',{configurable:true,value(previous){debug.push(previous===undefined?null:{name:previous.name,message:previous.message});}});
  Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>Object.entries(inputs).map(([name,bytes])=>({family:name,async blob(){reads++;return {async arrayBuffer(){return bytes.slice().buffer;}};}}))});
  const term={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){if(complete){const done=complete;complete=undefined;done();}return family;}},registerCharacterJoiner(fn){join=fn;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
  const addon=create({fallbackLigatures:['===']});
  async function load(name){family=name;const loading=join('a===b');await new Promise(resolve=>{complete=resolve;});return {loading,ranges:join('a===b'),reads,refreshes};}
  try {
    addon.activate(term);
    const first=await load('BadCff');join('a===b');join('a===b');const cached={reads,refreshes};
    const recovery=await load('GoodCff');
    const second=await load('BadCff2');
    const final=await load('GoodCff2');
    return {first,cached,recovery,second,final,debug};
  } finally {
    addon.dispose();
    for(const [o,k,d]of saved.reverse())if(d)Object.defineProperty(o,k,d);else delete o[k];
  }
}
