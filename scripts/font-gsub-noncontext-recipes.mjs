// Null coverage is retained until a referenced substitution reads table.format.
import {gsub} from './font-gsub-null-recipes.mjs';
const words=values=>values.flatMap(v=>[v>>>8,v&255]);
function lookup(kind,table,flags=0){
 return words([kind,flags,1,flags&16?10:8,...(flags&16?[0]:[])]).concat(table);
}
function noncontext(kind,coverage,sets){
 const set=sets?(kind===4?[1,4,1,1]:[1,1]):[];
 const header=[1,coverage?6+(sets?2:0)+set.length*2:0,sets?1:0,...(sets?[8]:[])];
 return words([...header,...set,...(coverage?[1,1,1]:[])]);
}
function chain(range,position){
 const coverage=range?[2,1,1,2,0]:[1,1,1];
 return words([3,0,1,16,0,1,position,1,...coverage]);
}
export function noncontextCases(){
 const cases=[];
 for(const woff of [false,true]){
  const add=(name,data,meta=false)=>cases.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...(meta?{extraTables:[{tag:'meta',data:[0,0,0,2]}]}:{})});
  for(const kind of [2,3,4])for(const coverage of [false,true])for(const sets of [false,true]){
   const table=noncontext(kind,coverage,sets);
   for(const wrapped of [false,true]){
    const target=lookup(wrapped?7:kind,wrapped?words([1,kind,0,8]).concat(table):table);
    for(const selected of [false,true])for(const meta of [false,true])
     add(`direct type${kind} coverage${coverage} sets${sets} wrapped${wrapped} selected${selected} meta${meta}`,gsub([{tag:'calt',indices:selected?[0]:[]}],[target]),meta);
    for(const range of [false,true])for(const position of [0,1])for(const selected of [false,true])for(const meta of [false,true])
     add(`referenced type${kind} coverage${coverage} sets${sets} wrapped${wrapped} range${range} position${position} selected${selected} meta${meta}`,gsub([{tag:'calt',indices:selected?[0]:[]}],[lookup(6,chain(range,position)),target]),meta);
   }
   if(!coverage)for(const length of [2,3,4,5,...(sets?[6,7,8,9]:[])])
    add(`type${kind} null coverage EOF${length} sets${sets}`,gsub([],[lookup(kind,table.slice(0,length))]));
   if(!coverage&&sets)for(const flags of [0,16])
    add(`type${kind} null coverage malformed set flags${flags}`,gsub([],[lookup(kind,words([1,0,1,65535]),flags)]));
  }
 }
 return cases;
}
