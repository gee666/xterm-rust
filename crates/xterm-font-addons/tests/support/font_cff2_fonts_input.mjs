// Test-only recipes. Containers are independently reconstructed in Rust.
export {cffHeaderInput as cff2FontsInput} from './font_cff_header_input.mjs';
export const integer=n=>[29,...[24,16,8,0].map(shift=>(n>>shift)&255)];
const privateAt=256,charsAt=1024;
function index(objects) {
  const length=objects.reduce((n,b)=>n+b.length,0),width=length<255?1:2;
  let offset=1;
  const offsets=[offset,...objects.map(b=>(offset+=b.length))];
  return [...[24,16,8,0].map(s=>(objects.length>>>s)&255),width,...offsets.flatMap(n=>width===1?[n]:[n>>>8,n&255]),...objects.flat()];
}
export function fontTable(objects,privateBytes=[],extra={}) {
  const dict=[...integer(32),12,36,...integer(charsAt),17];
  const data=Array(extra.tableLength??charsAt+5).fill(0);
  data.splice(0,5+dict.length,...[2,0,5,0,dict.length,...dict]);
  const fd=extra.fdIndex??index(objects);
  if(32+fd.length>privateAt)throw Error('Font dictionaries overlap private DICT');
  data.splice(32,fd.length,...fd);
  data.splice(extra.privateLocation??privateAt,privateBytes.length,...privateBytes);
  if(extra.subrs)data.splice(extra.subrsAt??512,extra.subrs.length,...extra.subrs);
  for(const [at,value] of extra.patches??[])data[at]=value;
  // Match unchanged FiraCode maxp1629. Empty charstrings are lazily parsed.
  data.splice(charsAt,5,0,0,6,93,0);
  if(extra.truncate!==undefined)data.length=extra.truncate;
  return data;
}
export function cff2FontsCases() {
  const cases=[],add=(objects,privateBytes=[],extra={})=>cases.push({name:`fonts${cases.length}`,tag:'CFF2',data:fontTable(objects,privateBytes,extra),...extra});
  const field=n=>Array.isArray(n)?n:integer(n);
  const privateDict=(size,offset=privateAt)=>[...field(size),...field(offset),18];
  const fonts=[
    [],[31],[255],[28],[29],[30],[12],[0,0],[12,36,12,36],[0,0,31],
    [0,0,28],[139,140,0,0],[30,0xff,0,0],[30,0x1a,0x5f,0,0],
    [139,140,140,23,0,0],[18],[139,18],[139,140,141,18]
  ];
  for(const tag of ['CFF ','CFF2'])for(const woff of [false,true]) {
    cases.push({name:`fonts${cases.length}`,tag,woff,addonGood:true}); // Unchanged good-font control.
    for(const bytes of fonts)add([bytes],[],{tag,woff,...(bytes.length===2&&bytes.every(b=>b===0)?{addonBad:true}:{})});
    for(const bytes of [[],[31],[255],[28],[29],[30],[12],[0,0],[0,0,255],[30,0xff,0,0],[139,140,0,0]]) {
      add([privateDict(bytes.length)],bytes,{tag,woff});
      add([privateDict(bytes.length),[31]],bytes,{tag,woff});
    }
    // Private operands may read beyond declared size into the whole DataView.
    for(const size of [-1,0,1,2,3,4,5,6])
      add([privateDict(size)],[29,0,0,0,31],{tag,woff});
    for(const offset of [0,-1,privateAt,1028,1029,2147483647])
      add([privateDict(1,offset)],[31],{tag,woff});
    for(const subrs of [[19],[139,19],[140,19],[...integer(256),19],[30,0xff,19]])
      for(const tail of [[0,0,0,0],[0,0,0,1,1,1,2,31],[0,0,0,1,1]])
        add([privateDict(subrs.length)],subrs,{tag,woff,subrs:tail});
    // Fractional and NaN fields retain arithmetic until each DataView read.
    for(const size of [[30,0x0a,0x5f],[30,0x1a,0x5f],[30,0xe0,0xa5,0xff],[30,0xff]])
      for(const offset of [[30,0xff],[30,0xe0,0xa5,0xff],[30,0x25,0x6a,0x5f]])
        add([privateDict(size,offset)],[28,0,31,0,0],{tag,woff});
    // In the compressed/table-relative view, privateOffset+[]+base is "2560".
    // Adding base before [] would read256, whose bogus INDEX is deliberately
    // out of bounds. No hazardous wrong-order source parser is ever launched.
    add([privateDict(1)],[19,0,0,0,1],{tag,woff,tableLength:3000,stringOrder:true});
    // A later malformed object range must fail before the earlier byte31 DICT.
    add([[31]],[],{tag,woff,fdIndex:[0,0,0,2,2,0,1,0,2,255,255,31],indexOrder:true});
    for(const at of [32767,32768,32769])
      add([privateDict(2,at)],[0,0],{tag,woff,tableLength:at+4,privateLocation:at});
    for(const subrs of [[0,0,0,1,1,1,2,255],[0,0,0,1,255]])
      add([privateDict(6)],[...integer(32768-privateAt),19],{tag,woff,tableLength:32780,subrsAt:32768,subrs});
    add([[31],[0,0]],[],{tag,woff});
    add([[],[31]],[],{tag,woff});
    add([privateDict(2),[31]],[0,0],{tag,woff});
    add([[31]],[],{tag,woff,remove:['name']});
    add([[31]],[],{tag,woff,remove:['hmtx']});
  }
  let seed=0x546cff02;
  for(let i=0;i<32;i++) {
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    const bytes=Array.from({length:1+seed%8},(_,j)=>(seed>>>(j%4*8))&255);
    // Negative blend counts can hang unchanged source. Exclude opcode23 from
    // random recipes, but retain directed finite blend inputs above.
    for(let j=0;j<bytes.length;j++)if(bytes[j]===23)bytes[j]=22;
    add([bytes],[],{woff:!!(seed&1),seed});
  }
  return cases;
}
