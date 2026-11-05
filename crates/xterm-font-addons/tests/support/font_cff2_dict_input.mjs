// Test recipes reconstruct complete fonts; no parser is used by the Rust addon.
import {cffHeaderInput} from './font_cff_header_input.mjs';
export const cff2DictInput=cffHeaderInput;
export function cff2DictCases() {
  const cases=[],add=c=>cases.push({name:`dict${cases.length}`,tag:'CFF2',...c});
  for(const woff of [false,true])for(const tag of ['CFF ','CFF2']) {
    add({woff,tag}); // Unchanged CFF1 control, independent of directory tag.
    const make=(dict,extra={})=>{
      const size=extra.size??5,data=Array(size).fill(0);data.splice(0,5,2,0,size,dict.length>>>8,dict.length&255);
      data.push(...dict,...(extra.badGlobal?[255,255,255,255,255]:[0,0,0,0]));
      add({woff,tag,data,...extra});
    };
    for(const dict of [[31],[255],[12],[28],[28,31],[29],[29,255],[29,255,31],[29,255,31,255],
      [30],[30,0x12],[30,0x12,0xdd],[247],[250],[251],[254],[12,255,31],[30,0x12,0x3f,255],
      [139,0,139,0,31],[139,0,139,0,28]])make(dict);
    for(const byte of [0x0f,0xf0,0xff,0xdf,0xfd])make([30,byte,31]);
    for(let byte=0;byte<=255;byte++) {
      const dict=[byte];
      if(byte===12)dict.push(255);
      else if(byte===28)dict.push(31,255);
      else if(byte===29)dict.push(31,255,31,255);
      else if(byte===30)dict.push(0xff);
      else if(byte>=247&&byte<=254)dict.push(255);
      dict.push(31);make(dict);
    }
    for(const size of [5,8,255])for(const dict of [[31],[255],[28],[30,0x12]])make(dict,{size});
    for(const dict of [[31],[255],[28],[30]])make(dict,{badGlobal:true});
    // Reads cross the inflater prefix, but only the ushort-sized dictionary is
    // retained. Source lexical errors win over deferred duplicate-key errors.
    if(woff)for(const length of [32767,32768,32769,65535])make([...Array(length-1).fill(0),255]);
    make([31],{remove:['hmtx']});make([31],{remove:['name']});
  }
  return cases;
}
