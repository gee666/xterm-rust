// Browser input reconstruction uses stored zlib inputs, never parser results.
// Native/WASM tests hash these bytes against independently generated source data.
export function buildHeaderFont(base,fixture,item){
 const view=new DataView(base.buffer,base.byteOffset,base.byteLength),records=[];
 for(let i=0;i<view.getUint16(4);i++){
  const at=12+i*16,tag=String.fromCharCode(...base.subarray(at,at+4));
  if(tag==='GSUB'||item.remove?.includes(tag))continue;
  const start=view.getUint32(at+8);records.push({tag,data:base.slice(start,start+view.getUint32(at+12))});
 }
 records.push(...(item.extraTables??[]).map(({tag,data})=>({tag,data:new Uint8Array(data)})),{tag:'GSUB',data:new Uint8Array(item.data)});
 const woff=item.woff,start=woff?44:12,width=woff?20:16;
 const payloads=records.map(({tag,data})=>{const encoded=item.woffPayloads?.[tag]??fixture.woffTemplate[tag];return woff&&encoded&&encoded.length<data.length?new Uint8Array(encoded):data;});
 const size=start+records.length*width+payloads.reduce((n,d)=>n+d.length,0),bytes=new Uint8Array(size),out=new DataView(bytes.buffer);
 if(woff){bytes.set([119,79,70,70]);bytes.set(base.subarray(0,4),4);out.setUint16(12,records.length);out.setUint32(8,size);}
 else{bytes.set(base.subarray(0,12));out.setUint16(4,records.length);}
 let cursor=start+records.length*width;
 for(const [i,{tag,data}]of records.entries()){
  const at=start+i*width;bytes.set([...tag].map(c=>c.charCodeAt(0)),at);out.setUint32(at+(woff?4:8),cursor);
  if(woff)out.setUint32(at+8,payloads[i].length);
  out.setUint32(at+12,data.length);bytes.set(payloads[i],cursor);cursor+=payloads[i].length;
 }
 return bytes;
}
// Real addon calls, shared unchanged between the source and Rust constructors.
export async function headerAddonCases(create,input,count){
 const owner=globalThis.window??{},saved=[[globalThis,'window'],[globalThis,'navigator'],[owner,'queryLocalFonts'],[console,'debug']].map(([o,k])=>[o,k,Object.getOwnPropertyDescriptor(o,k)]);
 let active,reads=0;
 if(!globalThis.window)Object.defineProperty(globalThis,'window',{configurable:true,value:owner});
 Object.defineProperty(globalThis,'navigator',{configurable:true,value:{}});
 Object.defineProperty(console,'debug',{configurable:true,value(){}});
 Object.defineProperty(owner,'queryLocalFonts',{configurable:true,value:async()=>[{family:'Header',async blob(){reads++;return{async arrayBuffer(){return input(active).buffer;}};}}]});
 const results=[];
 try{
  for(active=0;active<count;active++){
   let join,complete,refreshes=0;reads=0;
   const terminal={rows:5,element:{style:{}},options:{logLevel:'debug',get fontFamily(){if(complete){const done=complete;complete=undefined;done();}return 'Header';}},registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
   const addon=create({fallbackLigatures:['===']});
   try{
    addon.activate(terminal);const loading=join('===');await new Promise(resolve=>{complete=resolve;});
    const ranges=join('==='),before=reads;join('===');join('===');
    results.push({loading,ranges,reads,cacheReads:reads-before,refreshes});
   }finally{addon.dispose();}
  }
  return results;
 }finally{for(const [o,k,d]of saved.reverse())if(d)Object.defineProperty(o,k,d);else delete o[k];}
}
