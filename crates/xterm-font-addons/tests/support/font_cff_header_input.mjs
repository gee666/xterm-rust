// Test-only container reconstruction. No font parser runs in the Rust addon.
export function cffHeaderInput(base,c,encoded={}) {
  const view=new DataView(base.buffer,base.byteOffset,base.byteLength),records=[];
  for(let i=0;i<view.getUint16(4);i++) {
    const at=12+i*16,tag=String.fromCharCode(...base.slice(at,at+4));
    if(tag==='CFF '||tag==='CFF2'||c.remove?.includes(tag))continue;
    const offset=view.getUint32(at+8),data=base.slice(offset,offset+view.getUint32(at+12));
    records.push({tag,data});
  }
  const originalAt=Array.from({length:view.getUint16(4)},(_,i)=>12+i*16).find(at=>String.fromCharCode(...base.slice(at,at+4))==='CFF ');
  const original=originalAt===undefined?undefined:base.slice(view.getUint32(originalAt+8),view.getUint32(originalAt+8)+view.getUint32(originalAt+12));
  const payload=c.data?Uint8Array.from(c.data):original.slice();
  if(c.header)payload.set(c.header);
  if(c.mutation)records.unshift({tag:'CPAL',data:Uint8Array.from([0,1,...Array(12).fill(0)])});
  const table={tag:c.tag??'CFF ',data:payload};
  if(c.duplicate==='first')records.unshift({...table,badOffset:true});
  if(c.other)records.push({tag:c.other.tag,data:Uint8Array.from(c.other.data),badOffset:c.other.badOffset});
  records.push({...table,badOffset:c.badOffset});
  if(c.duplicate==='last')records.push({...table,badOffset:true});
  if(c.noCff)for(let i=records.length-1;i>=0;i--)if(records[i].tag==='CFF '||records[i].tag==='CFF2')records.splice(i,1);
  const start=c.woff?44:12,width=c.woff?20:16;
  const length=start+records.length*width+records.reduce((sum,r)=>sum+(c.woff&&encoded[r.tag]?.length<r.data.length?encoded[r.tag].length:r.data.length),0);
  const bytes=new Uint8Array(length),out=new DataView(bytes.buffer);
  bytes.set(c.woff?[119,79,70,70]:base.slice(0,4));
  if(c.woff){bytes.set(base.slice(0,4),4);out.setUint32(8,length);out.setUint16(12,records.length);}
  else out.setUint16(4,records.length);
  let offset=start+records.length*width;
  for(const [i,r]of records.entries()) {
    const at=start+i*width,payload=c.woff&&encoded[r.tag]?.length<r.data.length?Uint8Array.from(encoded[r.tag]):r.data;
    bytes.set([...r.tag].map(x=>x.charCodeAt(0)),at);
    out.setUint32(at+(c.woff?4:8),r.badOffset?0xfffffff0:offset);
    if(c.woff)out.setUint32(at+8,payload.length);
    out.setUint32(at+12,c.directoryLength!==undefined&&r===records.at(-1)?c.directoryLength:r.data.length);
    bytes.set(payload,offset);offset+=payload.length;
  }
  return bytes;
}
export function mutateCffHeader(bytes,c) {
  const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),start=c.woff?44:12,width=c.woff?20:16,count=v.getUint16(c.woff?12:4);
  const at=Array.from({length:count},(_,i)=>start+i*width).filter(at=>String.fromCharCode(...bytes.slice(at,at+4))===c.tag).at(-1),field=at+(c.woff?4:8),offset=v.getUint32(field);
  if(c.mutation==='descriptor')v.setUint32(field,0xfffffff0);
  else if(c.mutation==='major')bytes[offset]=3;
  else if(c.mutation==='repair')bytes[offset]=1;
  else if(c.mutation==='wrapper')bytes[offset]=255;
}
export function cffHeaderCases() {
  const cases=[];
  const add=c=>cases.push({name:`case${cases.length}`,tag:'CFF ',...c});
  for(const tag of ['CFF ','CFF2'])for(const woff of [false,true]) {
    for(const major of [0,1,2,3,4,127,255])for(let length=0;length<(major<2?4:major===2?5:3);length++)
      for(const minor of [0,13,255])add({tag,woff,data:[major,minor,0,0,0].slice(0,length)});
    if(woff)for(const header of [[1,0,4],[0,255,255]])add({tag,woff,header});
    else for(const major of [0,1])for(const minor of [0,17,255])for(const size of [0,4,255])
      add({tag,woff,header:[major,minor,size]});
    for(const duplicate of ['first','last'])add({tag,woff,duplicate});
    add({tag,woff,badOffset:true});
    add({tag,woff,header:[3,7]});
    if(woff)for(const compressedPayload of [[120,156,7,0],[120,156,7],[120],[]])add({tag,woff,compressedPayload});
    add({tag,woff,header:[1,255,255],directoryLength:0});
    add({tag,woff,data:[3,7],remove:['hmtx']});
    add({tag,woff,data:[3,7],remove:['name']});
    add({tag,woff,data:[3,7],remove:['head']});
    for(const data of [[3,9],[2,0,0,0],[1,0,4]])add({tag,woff,data,base:'ttf'});
  }
  for(const woff of [false,true]) {
    add({woff,data:[3,8],other:{tag:'CFF2',data:[4,9]}});
    add({tag:'CFF2',woff,data:[3,8],other:{tag:'CFF ',data:[4,9]}});
    add({woff,other:{tag:'CFF2',data:[4,9],badOffset:true}});
    add({woff,noCff:true});
    add({woff,data:[3,9],remove:['glyf'],base:'ttf'});
    add({woff,data:[3,9],remove:['loca'],base:'ttf'});
  }
  for(const tag of ['CFF ','CFF2']) {
    for(const mutation of ['major','repair','descriptor'])add({tag,mutation,header:mutation==='repair'?[3,7]:[1,0,4]});
    add({tag,woff:true,mutation:'wrapper'});
    for(const throws of ['object','number','null','undefined'])add({tag,mutation:'throw',throws,header:[3,7]});
  }
  for(const tag of ['CFF ','CFF2']) {
    for(const count of [1,2,3,8,100,8192])for(const compression of [{level:0},{level:6},{level:9,strategy:4}]) {
      const directoryLength=count===100?16:count===8192?128:128;
      // Stored streams may exceed the short declared table. Force compression
      // only when the encoded payload is actually smaller than this length.
      if(compression.level===0&&count>=100)continue;
      add({tag,woff:true,inflated:Array(count).fill(3),compression,directoryLength});
    }
    for(const compressedPayload of [[120,156,1,2,0,253,255,3],[120,156,1,2,0,253,255,3,7]])
      add({tag,woff:true,compressedPayload,directoryLength:128});
  }
  let seed=0x457cff01;
  for(let i=0;i<64;i++) {seed=(Math.imul(seed,1664525)+1013904223)>>>0;add({woff:!!(seed&1),tag:seed&2?'CFF ':'CFF2',data:[3+(seed%253),(seed>>>8)&255],seed});}
  return cases;
}
