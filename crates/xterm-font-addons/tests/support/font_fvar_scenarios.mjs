// Shared constructor recipes and actual-addon host probes. No parser runs here.
export function fvarPayload({axes=0, axisSize=20, instances=0, instanceSize=4+axes*4, offset=16, version=65536, length=offset+axes*axisSize+instances*instanceSize}={}) {
  const data=new Uint8Array(Math.max(16,length)), view=new DataView(data.buffer);
  view.setUint32(0,version);view.setUint16(4,offset);view.setUint16(6,65535);
  view.setUint16(8,axes);view.setUint16(10,axisSize);view.setUint16(12,instances);view.setUint16(14,instanceSize);
  return data.slice(0,length);
}
export function fvarCases() {
  const cases=[{name:'unchanged rebuilt font'}];
  const add=(name,options,extra={})=>cases.push({name,tag:'fvar',data:[...fvarPayload(options)],...extra});
  for(let length=0;length<=17;length++)add(`header EOF ${length}`,{length});
  for(const version of [0,1,65535,65537,131072,4294967295])add(`unsupported version ${version}`,{version});
  for(const axes of [1,2])for(const axisSize of [0,1,4,18,20,22])for(const tail of [-2,-1,0,1]) {
    const length=16+(axes-1)*axisSize+20+tail;
    add(`axes ${axes} stride ${axisSize} EOF ${length}`,{axes,axisSize,length});
  }
  for(const axes of [0,1,2])for(const instances of [1,2])for(const instanceSize of [0,1,2,4,5,6,8,9,10,12])for(const tail of [-1,0,1]) {
    const length=16+axes*20+(instances-1)*instanceSize+4+axes*4+tail;
    add(`instances axes ${axes} count ${instances} stride ${instanceSize} EOF ${length}`,{axes,instances,instanceSize,length});
  }
  for(const offset of [0,1,6,65535])add(`ignored empty data offset ${offset}`,{offset,length:16});
  add('unconstrained zero directory length',{axes:1},{directoryLength:0});
  for(const woff of [false,true]) {
    add(`valid empty ${woff?'WOFF':'SFNT'}`,{}, {woff});
    add(`valid axis instance ${woff?'WOFF':'SFNT'}`,{axes:1,instances:1},{woff});
    add(`optional PostScript name EOF ${woff?'WOFF':'SFNT'}`,{instances:1,instanceSize:6,length:21},{woff});
    add(`compressed repeated axes ${woff?'WOFF':'SFNT'}`,{axes:8},{woff});
    for(const position of ['first','last'])add(`invalid ${position} descriptor ${woff?'WOFF':'SFNT'}`,{}, {woff,duplicateTable:{tag:'fvar',position}});
    add(`GSUB error precedes fvar version ${woff?'WOFF':'SFNT'}`,{version:0},{woff,duplicateTable:{tag:'GSUB',position:'last'}});
    add(`cmap error precedes fvar version ${woff?'WOFF':'SFNT'}`,{version:0},{woff,glyphCount:0});
    for(const tag of ['name','hmtx'])add(`missing ${tag} precedes fvar version ${woff?'WOFF':'SFNT'}`,{version:0},{woff,remove:[tag]});
  }
  add('ignored earlier invalid compressed descriptor',{}, {woff:true,duplicateTable:{tag:'fvar',position:'first',invalidCompression:true}});
  return cases;
}
function withFvar(bytes,payload,insert) {
  const result=insert(bytes,payload), view=new DataView(result.buffer);
  view.setUint32(12+(view.getUint16(4)-1)*16,0x66766172);
  return result;
}
export async function fvarAddonScenarios(create,bytes,insert) {
  const payload=fvarPayload({axes:1,instances:1});
  const good=withFvar(bytes,payload,insert), bad=withFvar(bytes,fvarPayload({version:0}),insert);
  // A warning allows mutation after descriptors have been copied but before
  // fvar parsing. This is distinct from the metadata host's fixed snapshots.
  const warning=Uint8Array.of(0,1,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0);
  const mutable=insert(good,warning), descriptorOnly=mutable.slice(), mutableView=new DataView(mutable.buffer), descriptorView=new DataView(descriptorOnly.buffer);
  let fvarOffset, fvarRecord;
  for(let i=0;i<mutableView.getUint16(4);i++)if(mutableView.getUint32(12+i*16)===0x66766172){fvarRecord=12+i*16;fvarOffset=mutableView.getUint32(fvarRecord+8);}
  const descriptors=['navigator','window'].map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]);
  const owner=globalThis.window??{}, query=Object.getOwnPropertyDescriptor(owner,'queryLocalFonts'), warn=Object.getOwnPropertyDescriptor(console,'warn');
  let reads=0,refreshes=0,join,complete,family;
  const term={rows:5,element:{style:{}},options:{logLevel:'info',get fontFamily(){if(complete){const done=complete;complete=undefined;done();}return family;}},
    registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
  if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
  Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
  Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>[['BadFvar',bad],['GoodFvar',good],['MutableFvar',mutable],['DescriptorFvar',descriptorOnly]].map(([family,data])=>({family,async blob(){reads++;return {async arrayBuffer(){return data.buffer;}};}}))});
  Object.defineProperty(console,'warn',{configurable:true,value(){if(family==='MutableFvar')mutableView.setUint32(fvarOffset,0);else if(family==='DescriptorFvar')descriptorView.setUint32(fvarRecord+8,0xfffffff0);}});
  const addon=create({fallbackLigatures:['===']});
  async function load(next) {
    family=next;const loading=join('===');
    await new Promise(resolve=>{complete=resolve;});
    return {loading,ranges:join('==='),reads,refreshes};
  }
  try {
    family='BadFvar';addon.activate(term);
    const rejected=await load('BadFvar');join('===');join('===');
    const cached={reads,refreshes};
    return {rejected,cached,accepted:await load('GoodFvar'),mutated:await load('MutableFvar'),descriptor:await load('DescriptorFvar')};
  } finally {
    addon.dispose();
    if(warn)Object.defineProperty(console,'warn',warn);else delete console.warn;
    if(query)Object.defineProperty(owner,'queryLocalFonts',query);else delete owner.queryLocalFonts;
    for(const [key,descriptor]of descriptors)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];
  }
}
