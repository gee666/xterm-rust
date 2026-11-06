// Complete source constructors for the FDArray INDEX phase only.
export {cffHeaderInput as cff2IndexInput} from './font_cff_header_input.mjs';
export function cff2IndexCases() {
  const cases=[],add=c=>cases.push({name:`index${cases.length}`,tag:'CFF2',...c});
  const integer=n=>[29,...[24,16,8,0].map(shift=>(n>>shift)&255)];
  const make=(operand,tail=[],extra={})=>{
    const dict=[...operand,12,36,...(extra.lexical??[])];
    const data=[2,0,5,dict.length>>>8,dict.length&255,...dict,0,0,0,0];
    if(extra.pad)data.push(...Array(extra.pad).fill(0));
    data.push(...tail);
    add({data,...extra});
  };
  for(const tag of ['CFF ','CFF2'])for(const woff of [false,true]) {
    const extra={tag,woff};
    add(extra); // Known-good CFF1 control.
    for(const n of [-2147483648,-65536,-1,1,4,5,12,16,255,65535,2147483647])
      make(integer(n),[],extra);
    for(const operand of [
      [],[30,0x1b,0x99,0x99,0xff],[30,0xe1,0xb9,0x99,0x9f],
      [30,0x1a,0x5f],[30,0xe0,0xa5,0xff],[30,0xff],
      [139],[28,255,255]
    ])make(operand,[],extra);
    // Fixed-width operand keeps the tail at offset16.
    for(const tail of [
      [],[0],[0,0,0],[0,0,0,0],[0,0,0,1],[0,0,0,1,0],
      [0,0,0,1,1],[0,0,0,1,1,1],[0,0,0,1,1,1,2],
      [0,0,0,1,1,1,2,0],[0,0,0,1,1,2,1],
      [0,0,0,1,1,0,0],[0,0,0,1,1,0,1],
      [0,0,0,2,1,1,1,1],[0,0,0,2,1,1,2,3,0,0]
    ]) {
      make(integer(16),tail,extra);
      make(integer(16),tail,{...extra,remove:['hmtx']});
      make(integer(16),tail,{...extra,remove:['name']});
    }
    for(const width of [2,3,4,5,8,255])for(const offsets of [[0,0],[1,1],[2,1],[1,2],[0xffffffff,0]]) {
      const bytes=n=>Array.from({length:width},(_,i)=>i<width-4?0:(n>>>((width-i-1)*8))&255);
      make(integer(16),[0,0,0,1,width,...offsets.flatMap(bytes),0,0],extra);
    }
    for(const lexical of [[31],[0,0],[0,0,255]])
      make(integer(2147483647),[],{...extra,lexical});
  }
  // Compressed reads beyond the retained DEFLATE prefix.
  for(const size of [32767,32768,32769,65520])for(const tail of [[0,0,0],[0,0,0,1,1,1,2,0]])
    make(integer(size),tail,{woff:true,pad:size-16});
  let seed=0x533cff02;
  for(let i=0;i<64;i++) {
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    make(integer(16),[0,0,0,1,1,seed&255,(seed>>>8)&255,...Array(16).fill(0)],{woff:!!(seed&1),seed});
  }
  return cases;
}
