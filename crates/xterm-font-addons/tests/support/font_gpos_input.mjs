// Browser builder is independent of the Rust metadata recipe decoder. Stored
// zlib payloads are input data, never expected parser outputs or replacements.
export function buildGposFont(base, fixture, item) {
  const view=new DataView(base.buffer,base.byteOffset,base.byteLength),records=[];
  for(let i=0;i<view.getUint16(4);i++){
    const at=12+i*16,tag=String.fromCharCode(...base.subarray(at,at+4));
    if(tag===item.tag||item.remove?.includes(tag))continue;
    const start=view.getUint32(at+8);records.push({tag,data:base.slice(start,start+view.getUint32(at+12))});
  }
  for(const table of item.extraTables??[])records.push({tag:table.tag,data:new Uint8Array(table.data)});
  records.push({tag:item.tag,data:new Uint8Array(item.data)});
  const duplicate=item.duplicateTable;
  if(duplicate)records[duplicate.position==='first'?'unshift':'push']({tag:duplicate.tag,data:new Uint8Array()});
  const woff=item.woff,start=woff?44:12,width=woff?20:16;
  const payloads=records.map(record=>{
    const compressed=item.woffPayloads?.[record.tag]??fixture.woffTemplate[record.tag];
    return woff&&compressed&&compressed.length<record.data.length?new Uint8Array(compressed):record.data;
  });
  const header=start+width*records.length,size=header+payloads.reduce((n,bytes)=>n+bytes.length,0);
  const bytes=new Uint8Array(size),output=new DataView(bytes.buffer);
  if(woff){bytes.set([119,79,70,70]);bytes.set(base.subarray(0,4),4);output.setUint16(12,records.length);output.setUint32(8,size);}
  else{bytes.set(base.subarray(0,12));output.setUint16(4,records.length);}
  let cursor=header;
  for(const [index,record]of records.entries()){
    const at=start+index*width;
    bytes.set([...record.tag].map(c=>c.charCodeAt(0)),at);
    output.setUint32(at+(woff?4:8),cursor);
    if(woff)output.setUint32(at+8,payloads[index].length);
    output.setUint32(at+12,index===records.length-1?(item.directoryLength??record.data.length):record.data.length);
    bytes.set(payloads[index],cursor);cursor+=payloads[index].length;
  }
  if(duplicate){
    const at=start+(duplicate.position==='first'?0:records.length-1)*width;
    output.setUint32(at+(woff?4:8),0xfffffff0);output.setUint32(at+12,128);
    if(woff)output.setUint32(at+8,16);
  }
  return bytes;
}
