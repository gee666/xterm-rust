// Shared recipes and host interactions only. No JavaScript font parser runs in Rust tests.
export function colorPayload(tag,{version=0,palettes=0,colors=0,bases=0,layers=0,offset=14,layerOffset=14,length=14}={}) {
  const out=new Uint8Array(Math.max(14,length)),v=new DataView(out.buffer);
  v.setUint16(0,version);
  if(tag==='CPAL'){v.setUint16(2,65535);v.setUint16(4,palettes);v.setUint16(6,colors);v.setUint32(8,offset);}
  else {v.setUint16(2,bases);v.setUint32(4,offset);v.setUint32(8,layerOffset);v.setUint16(12,layers);}
  return out.slice(0,length);
}
export function colorCases() {
  const cases=[{name:'unchanged rebuilt font'}];
  const add=(tag,name,options={},extra={})=>cases.push({name:`${tag} ${name}`,tag,data:[...colorPayload(tag,options)],...extra});
  for(const tag of ['CPAL','COLR']) {
    for(const woff of [false,true]) {
      for(const version of [0,1,32768,65535])for(let length=0;length<=15;length++)add(tag,`v${version} EOF${length} WOFF${woff}`,{version,length},{woff});
      for(const offset of [0,1,12,14,15,4294967295])add(tag,`empty offset${offset} WOFF${woff}`,{offset,layerOffset:offset,length:14},{woff});
      for(const count of [1,2,32767,32768,65535])for(const length of [14,15,17,18,19,20,25,26]) {
        if(tag==='CPAL') {
          add(tag,`palettes${count} EOF${length} WOFF${woff}`,{palettes:count,length},{woff});
          add(tag,`colors${count} EOF${length} WOFF${woff}`,{colors:count,length},{woff});
        } else {
          add(tag,`bases${count} EOF${length} WOFF${woff}`,{bases:count,length},{woff});
          add(tag,`layers${count} EOF${length} WOFF${woff}`,{layers:count,length},{woff});
        }
      }
      add(tag,`ignore nominal directory length WOFF${woff}`,{length:14},{woff,...(!woff?{directoryLength:0}:{})});
      for(const position of ['first','last'])add(tag,`eager duplicate ${position} WOFF${woff}`,{length:14},{woff,duplicateTable:{tag,position}});
      for(const missing of ['name','hmtx'])add(tag,`warning precedes missing${missing} WOFF${woff}`,{version:1},{woff,remove:[missing]});
      add(tag,`warning precedes cmap assignment WOFF${woff}`,{version:1},{woff,glyphCount:0});
      for(const thrown of ['object','number','null','undefined'])add(tag,`throw${thrown} before laterEOF WOFF${woff}`,{version:1,length:2},{woff,throws:thrown});
      for(const field of ['payload','descriptor','next','cmap']) {
        const other=tag==='CPAL'?'COLR':'CPAL';
        const extraTables=field==='next'?[{tag:other,data:[...colorPayload(other)]}]:[];
        add(tag,`warning mutates${field} WOFF${woff}`,{version:1}, {woff,extraTables,...(field==='cmap'?{}:{frontTags:[tag]}),mutation:field,mutationTag:tag});
      }
    }
    // Move only descriptors, not payloads, to exercise true eager ordering.
    add(tag,'bad cmap before late warning',{version:1},{extraTables:[{tag:'cmap',data:[0,1]}]});
    add(tag,'early warning before bad cmap',{version:1},{frontTags:[tag],extraTables:[{tag:'cmap',data:[0,1]}]});
    add(tag,'early callback repairs later cmap',{version:1},{frontTags:[tag],extraTables:[{tag:'cmap',data:[0,1,0,0]}],mutation:'repairCmap',mutationTag:tag});
    add(tag,'early callback invalidates later head',{version:1},{frontTags:[tag],mutation:'head',mutationTag:tag});
    add(tag,'early descriptor mutation does not redirect later head',{version:1},{frontTags:[tag],mutation:'headDescriptor',mutationTag:tag});
    add(tag,'earlier cmap remains parsed after mutation',{version:1},{mutation:'cmap',mutationTag:tag});
    for(const mutation of ['cmap','checksum'])add(tag,`early WOFF callback changes cmap ${mutation}`,{version:1},{woff:true,frontTags:[tag],mutation,mutationTag:tag});
  }
  cases.push({name:'CPAL palette bounds precede negative color array',tag:'CPAL',data:[...colorPayload('CPAL',{palettes:2,colors:65535,length:12})]});
  for(const order of [['CPAL','COLR'],['COLR','CPAL']])cases.push({name:`warning order ${order.join('-')}`,extraTables:order.map(tag=>({tag,data:[...colorPayload(tag,{version:1})]}))});
  return cases;
}
export function reorderColor(bytes,change) {
  if(!change.frontTags)return bytes;
  const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),woff=v.getUint32(0)===0x774f4646,start=woff?44:12,width=woff?20:16,count=v.getUint16(woff?12:4);
  const records=Array.from({length:count},(_,i)=>Uint8Array.from(bytes.subarray(start+i*width,start+(i+1)*width)));
  const tag=record=>String.fromCharCode(...record.slice(0,4));
  records.sort((a,b)=>Number(change.frontTags.includes(tag(b)))-Number(change.frontTags.includes(tag(a))));
  records.forEach((record,i)=>bytes.set(record,start+i*width));return bytes;
}
export function mutateColor(bytes,change) {
  const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),woff=v.getUint32(0)===0x774f4646,start=woff?44:12,width=woff?20:16;
  const target=change.mutation==='payload'||change.mutation==='descriptor'?change.mutationTag:change.mutation==='next'?(change.mutationTag==='CPAL'?'COLR':'CPAL'):change.mutation.startsWith('head')?'head':'cmap';
  for(let i=0;i<v.getUint16(woff?12:4);i++) {
    const at=start+i*width;if(String.fromCharCode(...bytes.slice(at,at+4))!==target)continue;
    if(change.mutation==='descriptor'||change.mutation==='headDescriptor')v.setUint32(at+(woff?4:8),0xfffffff0);
    else {
      const offset=v.getUint32(at+(woff?4:8));
      if(change.mutation==='checksum'){bytes[offset+v.getUint32(at+8)-1]^=255;}
      else if(change.mutation==='payload')v.setUint16(offset+(target==='CPAL'?4:2),65535);
      else if(change.mutation==='repairCmap')v.setUint16(offset,0);
      else if(change.mutation==='head')v.setUint32(offset+12,0);
      else v.setUint16(offset,65535);
    }
    return;
  }
  throw new Error(`Missing mutation target ${target}`);
}
