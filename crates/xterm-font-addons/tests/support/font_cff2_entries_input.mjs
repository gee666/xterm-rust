// Complete source constructor recipes. Shared container writer is test-only.
export {cffHeaderInput as cff2EntriesInput} from './font_cff_header_input.mjs';
export function cff2EntriesCases() {
  const cases=[],add=c=>cases.push({name:`entry${cases.length}`,tag:'CFF2',...c});
  const make=(dict,extra={})=>add({data:[2,0,5,dict.length>>>8,dict.length&255,...dict,...(extra.badGlobal?[255,255,255,255,255]:[0,0,0,0])],...extra});
  const keys=[...Array.from({length:28},(_,i)=>i).filter(i=>i!==12&&i!==23).map(i=>[i]),...Array.from({length:256},(_,i)=>[12,i])];
  for(const tag of ['CFF ','CFF2']) {
    add({tag});
    for(const key of keys)make([139,...key,140,...key],{tag});
  }
  const prefixes=[
    [],[139],[140],[28,255,255],[29,255,255,255,255],[247,255],[250,255],[251,255],[254,255],
    [139,140],[30,0xff],[30,0xdf],[30,0xaf],[30,0xef],[30,0xbf],[30,0xcf],
    [30,0x1d,0x2f],[30,0x1b,0xff],[30,0x1c,0xff],[30,0xa1,0xff],[30,0xe1,0xaf],
    [139,140,140,23],[139,140,139,23],[139,140,141,23],[139,140,142,23],
    [30,0xff,23],[139,30,0xff,23],[139,140,30,0x0a,0x5f,23],
    [139,140,30,0x1a,0x5f,23],[139,140,30,0x1b,0x99,0x99,0xff,23],
    [139,140,23,23], [23], [139,23], [139,140,23],
    [139,140,28,255,255,23],[139,140,29,0,0,0,1,23],[139,140,30,0xe0,0xff,23],
  ];
  for(let n=0;n<15;n++)prefixes.push([30,(n<<4)|15]);
  for(const woff of [false,true])for(const tag of ['CFF ','CFF2']) {
    add({woff,tag});
    // If key0 is replaceable, the later key26 duplicate must be reported.
    for(const prefix of prefixes)make([...prefix,0,0,26,26],{woff,tag});
    for(const tail of [[31],[255],[12],[28],[29],[30],[247]])make([0,0,...tail],{woff,tag});
    make([0,0],{woff,tag,badGlobal:true});
    make([0,0],{woff,tag,remove:['name']});
    make([0,0],{woff,tag,remove:['hmtx']});
    // Escaped23 is an ordinary operator, never the unescaped blend instruction.
    make([12,23,12,23],{woff,tag});
    make([1,1,0,0],{woff,tag});
  }
  // Dictionary copies stay ushort-bounded while the inflater crosses32/64KiB.
  for(const length of [32767,32768,32769,65535])make([...Array(length-4).fill(139),0,0,26,26],{woff:true});
  let seed=0x505cff02;
  for(let i=0;i<64;i++) {
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    const key=[12,(seed>>>8)&255],prefix=prefixes[seed%prefixes.length];
    make([...prefix,...key,...key,26,26],{woff:!!(seed&1),seed});
  }
  return cases;
}
