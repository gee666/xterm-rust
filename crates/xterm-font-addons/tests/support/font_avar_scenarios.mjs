// Constructor recipes and actual-addon host probes. No font parser runs here.
export function avarPayload({major=1,minor=0,axes=0,positions=[],length}={}) {
  const data=new Uint8Array(length??8+2*axes+positions.reduce((n,p)=>n+4*p,0)),view=new DataView(data.buffer);
  const set=(at,value)=>{if(at+2<=data.length)view.setUint16(at,value);};
  set(0,major);set(2,minor);set(4,65535);set(6,axes);
  let at=8;for(let i=0;i<axes;i++){const count=positions[i]??0;set(at,count);at+=2+4*count;}
  return data;
}
export function avarCases() {
  const cases=[],fvar=axes=>{const d=new Uint8Array(16+20*axes),v=new DataView(d.buffer);v.setUint32(0,65536);v.setUint16(4,16);v.setUint16(8,axes);v.setUint16(10,20);v.setUint16(14,4+4*axes);return [...d];};
  const add=(name,options={},extra={})=>cases.push({name,tag:'avar',data:[...avarPayload(options)],extraTables:[{tag:'fvar',data:fvar(options.fvarAxes??options.axes??0)}],...extra});
  for(const woff of [false,true]) {
    const suffix=woff?' WOFF':' SFNT';
    for(const major of [0,1,2,65535])for(let length=0;length<=12;length++)
      add(`version ${major} EOF ${length}${suffix}`,{major,length},{woff});
    for(const axes of [0,1,2,4])for(const fvarAxes of [0,1,2,4]) {
      for(const positions of [[],[1,2,0,3],[5,0,2,1]]) {
        const full=avarPayload({axes,positions});
        for(const tail of [-1,0,1])add(`axis ${axes}/${fvarAxes} maps ${positions} tail ${tail}${suffix}`,{axes,fvarAxes,positions,length:full.length+tail},{woff});
      }
    }
    for(const major of [0,1,2])for(const length of [0,3,4,6,7,8,10])
      add(`missing fvar version ${major} EOF ${length}${suffix}`,{major,length},{woff,extraTables:[]});
    for(const position of ['first','last'])add(`copied ${position} avar descriptor${suffix}`,{}, {woff,duplicateTable:{tag:'avar',position}});
    for(const remove of ['name','hmtx'])add(`earlier missing ${remove}${suffix}`,{major:2},{woff,remove:[remove]});
    add('invalid fvar precedes avar'+suffix,{major:2},{woff,extraTables:[{tag:'fvar',data:[0,2,0,0]}]});
    for(const axes of [0,1])add(`avar mismatch ${axes} before invalid META${suffix}`,{axes,fvarAxes:0},{woff,extraTables:[{tag:'fvar',data:fvar(0)},{tag:'meta',data:[0,0,0,2]}]});
    for(const throws of ['object','undefined','null','number'])add(`version warning throws ${throws}${suffix}`,{major:2},{woff,throws});
    add('missing fvar warning throws'+suffix,{}, {woff,extraTables:[],throws:'object'});
    add('missing fvar warning repairs avar version before read'+suffix,{major:0},{woff,extraTables:[],mutation:{tag:'avar',at:0,value:1}});
    add('missing fvar warning retains avar descriptor'+suffix,{}, {woff,extraTables:[],mutation:{tag:'avar',descriptor:true}});
    for(const mutation of [
      {tag:'avar',at:6,value:1},{tag:'avar',at:0,value:1},{tag:'avar',descriptor:true},
      {tag:'meta',at:2,value:2},{tag:'fvar',at:8,value:1}
    ])add(`warning mutation ${JSON.stringify(mutation)}${suffix}`,{major:2},{woff,extraTables:[{tag:'fvar',data:fvar(0)},{tag:'meta',data:[0,0,0,1,...Array(12).fill(0)]}],mutation});
    add('compressed warning keeps inflated bytes'+suffix,{major:2,length:1024},{woff,mutation:{tag:'avar',at:6,value:1}});
    add('unconstrained minor and coordinates'+suffix,{minor:65535,axes:1,positions:[2]},{woff});
  }
  add('zero nominal directory length',{}, {directoryLength:0});
  return cases;
}
function append(bytes,tag,payload,insert) {
  const result=insert(bytes,payload),view=new DataView(result.buffer);
  view.setUint32(12+(view.getUint16(4)-1)*16,tag);
  return result;
}
export async function avarAddonScenarios(create,bytes,insert) {
  const fvar=new Uint8Array(16),fv=new DataView(fvar.buffer);fv.setUint32(0,65536);fv.setUint16(4,16);fv.setUint16(10,20);fv.setUint16(14,4);
  const base=append(bytes,0x66766172,fvar,insert);
  const good=append(base,0x61766172,avarPayload(),insert);
  const bad=append(base,0x61766172,avarPayload({axes:1}),insert);
  const missing=append(bytes,0x61766172,avarPayload(),insert);
  const warned=append(base,0x61766172,avarPayload({major:2}),insert);
  const mutable=warned.slice(),descriptor=warned.slice();
  const records=data=>{const v=new DataView(data.buffer);for(let i=0;i<v.getUint16(4);i++)if(v.getUint32(12+i*16)===0x61766172)return 12+i*16;};
  const globals=['navigator','window'].map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]);
  const owner=globalThis.window??{},query=Object.getOwnPropertyDescriptor(owner,'queryLocalFonts'),warn=Object.getOwnPropertyDescriptor(console,'warn'),debug=Object.getOwnPropertyDescriptor(console,'debug');
  let reads=0,refreshes=0,join,complete,family,warnings=0,previousIdentity=false;
  const sentinel={};
  const term={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){if(complete){const done=complete;complete=undefined;done();}return family;}},
    registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
  if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
  Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
  Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>[
    ['BadAvar',bad],['GoodAvar',good],['MissingFvar',missing],['WarnAvar',warned],['ThrowAvar',warned],['AfterThrowAvar',warned],['MutableAvar',mutable],['DescriptorAvar',descriptor]
  ].map(([family,data])=>({family,async blob(){reads++;return {async arrayBuffer(){return data.buffer;}};}}))});
  Object.defineProperty(console,'debug',{configurable:true,value(previous){if(family==='AfterThrowAvar')previousIdentity=previous===sentinel;}});
  Object.defineProperty(console,'warn',{configurable:true,value(){warnings++;
    if(family==='ThrowAvar'||family==='AfterThrowAvar')throw sentinel;
    if(family==='MutableAvar'){const v=new DataView(mutable.buffer);v.setUint16(v.getUint32(records(mutable)+8)+6,1);}
    if(family==='DescriptorAvar')new DataView(descriptor.buffer).setUint32(records(descriptor)+8,0xfffffff0);
  }});
  const addon=create({fallbackLigatures:['===']});
  async function load(next){family=next;const loading=join('===');await new Promise(resolve=>{complete=resolve;});return {loading,ranges:join('==='),reads,refreshes,warnings};}
  try {
    family='BadAvar';addon.activate(term);
    const rejected=await load(family);join('===');join('===');
    const cached={reads,refreshes,warnings};
    return {rejected,cached,accepted:await load('GoodAvar'),missing:await load('MissingFvar'),warned:await load('WarnAvar'),thrown:await load('ThrowAvar'),afterThrown:await load('AfterThrowAvar'),get previousIdentity(){return previousIdentity;},mutated:await load('MutableAvar'),descriptor:await load('DescriptorAvar')};
  } finally {
    addon.dispose();
    if(warn)Object.defineProperty(console,'warn',warn);else delete console.warn;
    if(debug)Object.defineProperty(console,'debug',debug);else delete console.debug;
    if(query)Object.defineProperty(owner,'queryLocalFonts',query);else delete owner.queryLocalFonts;
    for(const [key,value]of globals)if(value)Object.defineProperty(globalThis,key,value);else delete globalThis[key];
  }
}
