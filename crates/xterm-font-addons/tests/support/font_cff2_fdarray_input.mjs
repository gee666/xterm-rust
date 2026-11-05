// Source constructor recipes for the FDArray presence guard, not its later INDEX.
export {cffHeaderInput as cff2FdArrayInput} from './font_cff_header_input.mjs';
export const missingFdArray='This is a CFF2 font, but FDArray information is missing';
export function cff2FdArrayCases() {
  const cases=[],add=c=>cases.push({name:`fd${cases.length}`,tag:'CFF2',...c});
  const make=(dict,extra={})=>add({data:[2,0,5,dict.length>>>8,dict.length&255,...dict,...(extra.badGlobal?[255,255,255,255,255]:[0,0,0,0])],...extra});
  const falseValues=[[],[139,12,36],[28,0,0,12,36],[29,0,0,0,0,12,36],
    [30,0x0f,12,36],[30,0xe0,0xff,12,36],[30,0xff,12,36],[30,0xdf,12,36],
    [30,0xaf,12,36],[30,0xef,12,36],[30,0xbf,12,36],
    [30,0xff,12,36,139,12,36],[139,140,12,36,139,12,36],
    [139,140,140,23,12,36],[139,140,139,23,139,12,36]];
  for(const tag of ['CFF ','CFF2'])for(const woff of [false,true]) {
    add({tag,woff});
    for(const prefix of falseValues) {
      make(prefix,{tag,woff});
      for(const tail of [[0,0],[31],[255],[12],[28],[29],[30],[247],[0,0,31]])make([...prefix,...tail],{tag,woff});
      make(prefix,{tag,woff,badGlobal:true});
      make(prefix,{tag,woff,remove:['name']});
      make(prefix,{tag,woff,remove:['hmtx']});
    }
    // [] is truthy and reaches a later INDEX read. Preserve its complete source
    // outcome, but compare only presence in Rust until the later INDEX is ported.
    make([12,36],{tag,woff,presenceOnly:true});
    make([30,0x1b,0x99,0x99,0xff,12,36],{tag,woff,presenceOnly:true});
  }
  for(const length of [32767,32768,32769,65535])make([...Array(length-4).fill(139),26,139,12,36],{woff:true});
  let seed=0x519cff02;
  for(let i=0;i<64;i++) {
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    const operands=Array.from({length:seed%12},(_,i)=>139+((seed>>>(i%24))&31));
    make([...operands,26,...falseValues[seed%falseValues.length]],{woff:!!(seed&1),seed});
  }
  return cases;
}
