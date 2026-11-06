// Source-derived FDSelect recipes. Rust independently reconstructs containers.
import {cffHeaderInput} from './font_cff_header_input.mjs';
export {cffHeaderInput as fdSelectInput};
const integer=n=>[29,...[24,16,8,0].map(s=>(n>>s)&255)];
const u16=n=>[n>>>8&255,n&255],u32=n=>[24,16,8,0].map(s=>n>>>s&255);
export function fdSelectTable(select,extra={}) {
  const at=extra.at??128,field=extra.field??integer(at);
  const dict=[...integer(64),12,36,...integer(32),17,...field,12,37,...(extra.dictTail??[])];
  const bytes=Array(Math.max(69,at+select.length)).fill(0);
  bytes.splice(0,5+dict.length,...[2,0,5,0,dict.length,...dict]);
  bytes.splice(32,5,0,0,6,93,0); // FiraCode maxp1629, lazy empty charstrings.
  if(extra.badFont)bytes.splice(64,8,0,0,0,1,1,1,2,31);
  else bytes.splice(64,5,...u32(extra.fonts??1),0);
  bytes.splice(at,select.length,...select);
  return bytes;
}
export function fdSelectCases(base) {
  const cases=[];
  const add=(select,extra={})=>cases.push({name:`select${cases.length}`,tag:'CFF2',data:fdSelectTable(select,extra),...extra});
  const range=(format,first=0,fd=0,end=1629,count=1)=>format===3?[3,...u16(count),...u16(first),fd,...u16(end)]:[4,...u32(count),...u32(first),...u16(fd),...u32(end)];
  for(const tag of ['CFF ','CFF2'])for(const woff of [false,true]) {
    const env={tag,woff};
    cases.push({name:`select${cases.length}`,...env,addonGood:true});
    for(const format of [1,2,5,7,127,255])add([format],{...env,...(format===7?{addonBad:true}:{})});
    add([0,...Array(1629).fill(0)],env);
    add([0,0,1],env);
    add([0,0],{...env,fonts:0});
    for(const data of [[],[0],[3],[3,0],[3,0,1],[3,0,1,0],[3,0,1,0,0],[3,0,1,0,0,9],[3,0,1,0,0,9,0]])add(data,env);
    for(const format of [3,4]) {
      add(range(format),env);
      add(range(format,1),env);
      add(range(format,0,1),env);
      add(range(format,0,0,1630),env);
      add(range(format,0,1,1630),env); // FD error precedes bad endpoint.
      add(range(format,0,0,1628),env);
      add(range(format,0,0,1629,0),env);
      add(range(format,0,0,1629,2).slice(0,format===3?8:15),env);
      const sequence=format===3?[3,0,3,0,0,0,...u16(100),0,...u16(50),0,...u16(1629)]:[4,...u32(3),...u32(0),...u16(0),...u32(100),...u16(0),...u32(50),...u16(0),...u32(1629)];
      add(sequence,env); // Source accepts descending ranges.
      add(range(format),{...env,at:32768});
      add(range(format),{...env,remove:['maxp']});
      add(range(format,0,0,1629,0),{...env,remove:['maxp']});
    }
    add([0,...Array(1629).fill(1)],{...env,fonts:2});
    add(range(3,0,1),{...env,fonts:2});
    add(range(4,0,256),{...env,fonts:257});
    add(range(4,65536),env);
    add(range(4,0,0,1629,65536),env); // Wide range count reaches the second missing range.
    add(range(4,0,0,16777217),env);
    add(range(4,0,65535),{...env,fonts:257});
    for(const length of [10,11,12,13,14])add(range(4).slice(0,length),env);
    add([0],{...env,remove:['maxp']});
    add([7],{...env,badFont:true});
    add([7],{...env,remove:['name']});
    add([7],{...env,remove:['hmtx']});
    for(const field of [integer(0),integer(-1),integer(2147483647),[30,0xff],[],[139,140],[30,0x12,0x8a,0x5f],[30,0xe0,0xa5,0xff]])add([7],{...env,field});
    add([7],{...env,dictTail:[12,37,31]});
    add([7],{...env,dictTail:[12,37]});
  }
  if(base) {
    // [] makes Parser.offset the decimal table base as a string. The initial
    // read is base+"0", subsequent relative1/3/5/6 reads must still concatenate.
    const probe=cffHeaderInput(base,{tag:'CFF2',data:[]}),view=new DataView(probe.buffer);
    const start=view.getUint32(12+(view.getUint16(4)-1)*16+8);
    add(range(3),{at:start*9,field:[],relativeString:true});
  }
  return cases;
}
