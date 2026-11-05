// Invalid type1 formats must reject before reading their coverage pointer.
import {gsub} from './font-gsub-null-recipes.mjs';
const words=values=>values.flatMap(v=>[v>>>8,v&255]);
function lookup(kind,subtable,flags){
 const payload=kind===7?words([1,1,0,8]).concat(subtable):subtable;
 return words([kind,flags,1,flags&16?10:8,...(flags&16?[0]:[])]).concat(payload);
}
export function singleCases(){
 const cases=[];
 for(const woff of [false,true])for(const kind of [1,7])for(const selected of [false,true])for(const flags of [0,16])for(const meta of [false,true]){
  const add=(name,subtable,extra={})=>cases.push({name:`${name} type${kind} selected${selected} flags${flags} meta${meta} ${woff?'WOFF':'SFNT'}`,tag:'GSUB',data:gsub([{tag:'calt',indices:selected?[0]:[]}],[lookup(kind,subtable,flags)]),woff,...extra,...(meta?{extraTables:[{tag:'meta',data:[0,0,0,2]}]}:{})});
  for(const format of [0,3,65535])for(const [tail,bytes]of [
   ['no pointer',[]],['half pointer',[255]],['out of bounds',words([65535])],
   ['invalid coverage',words([6,0,3,0])],['valid coverage',words([6,0,1,0])]
  ])add(`invalid format${format} ${tail}`,words([format]).concat(bytes),{format});
  add('no format',[]);
  add('half format',[0]);
  add('valid format1',words([1,6,0,1,0]));
  add('valid format2',words([2,6,0,1,0]));
 }
 return cases;
}
