// Shared test recipes expand existing font bytes before container reconstruction.
import {cffHeaderInput} from './font_cff_header_input.mjs';
export function cffIndexCase(base,c) {
  const view=new DataView(base.buffer,base.byteOffset,base.byteLength);
  const at=Array.from({length:view.getUint16(4)},(_,i)=>12+i*16).find(at=>String.fromCharCode(...base.slice(at,at+4))==='CFF ');
  const original=base.slice(view.getUint32(at+8),view.getUint32(at+8)+view.getUint32(at+12));
  let data=c.data?Uint8Array.from(c.data):original;
  for(const patch of c.patches??[])data.set(patch.bytes,patch.at);
  if(c.truncate!==undefined)data=data.slice(0,c.truncate);
  return {...c,data:[...data]};
}
export function cffIndexInput(base,c,encoded={}) {return cffHeaderInput(base,cffIndexCase(base,c),encoded);}
export function cffIndexCases() {
  const cases=[],add=c=>cases.push({name:`index${cases.length}`,tag:'CFF ',...c});
  for(const woff of [false,true])for(const tag of ['CFF ','CFF2']) {
    add({woff,tag});
    // Fixed source fixture index starts and first payload boundaries, not parser
    // output. Truncations hit both offset tables and forward object byte reads.
    for(const [start,width,count]of [[4,1,1],[25,1,1],[74,2,1404],[15351,2,448]]) {
      for(const delta of [0,1,2,3,4,5,8,15,32,63,127,255])add({woff,tag,truncate:start+delta});
      for(const truncate of [start+3+(count+1)*width-1,start+3+(count+1)*width,start+3+(count+1)*width+1])add({woff,tag,truncate});
    }
    for(const bytes of [[0,0],[0,1,0],[0,4,0],[0,1,1,1,1],[0,1,1,2,1],[0,1,1,255,1],[0,1,1,0,1],
      [0,1,4,255,255,255,255,0,0,0,0],[0,1,4,127,255,255,255,0,0,0,0],
      [0,1,4,128,0,0,0,0,0,0,0],[0,1,4,0,0,0,1,127,255,255,255]])
      add({woff,tag,patches:[{at:15351,bytes}]});
    for(const width of [5,7,31,255]) {
      const bytes=[0,1,width,...Array(width*2).fill(0)];bytes[bytes.length-1]=1;
      add({woff,tag,patches:[{at:15351,bytes}]});
    }
    // The high count offset table must fail before reading any object data.
    add({woff,tag,patches:[{at:15351,bytes:[255,255,255]}]});
  }
  for(const tag of ['CFF ','CFF2'])for(const woff of [false,true]) {
    for(const size of [0,1,5,8,255])for(const length of (size<5?[255,65535]:[0,1,255,65535])) {
      const data=[2,0,size,length>>>8,length&255,0,0,0,1];
      add({tag,woff,data});
    }
  }
  let seed=0x472cff01;
  for(let i=0;i<96;i++){seed=(Math.imul(seed,1664525)+1013904223)>>>0;add({woff:!!(seed&1),truncate:4+(seed%15345),seed});}
  return cases;
}
