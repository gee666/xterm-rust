// Directed out-of-range lookup references, with independent JS font bytes.
import {gsub} from './font-gsub-null-recipes.mjs';
const words=values=>values.flatMap(v=>[v>>>8,v&255]);
const single=words([1,0,1,8,1,6,0,1,1,1]);
function chain(range,records,empty=false){
 const coverage=empty?[1,0]:range?[2,1,1,2,0]:[1,1,1];
 const offset=14+records.length*4;
 const sub=words([3,0,2,offset,offset+coverage.length*2,0,records.length,...records.flat(),...coverage,...coverage]);
 return words([6,0,1,8]).concat(sub);
}
export function refsCases(){
 const cases=[];
 for(const woff of [false,true]){
  const add=(name,data,meta=false)=>cases.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...(meta?{extraTables:[{tag:'meta',data:[0,0,0,2]}]}:{})});
  for(const range of [false,true]){
   for(const position of [0,1,2,65535])for(const reference of [1,2,65535])for(const selected of [true,false])for(const meta of [false,true]){
    const data=gsub([{tag:'calt',indices:selected?[0]:[]}],[chain(range,[[position,reference]]),single]);
    add(`${range?'range':'scalar'} position${position} reference${reference} selected${selected} meta${meta}`,data,meta);
   }
   for(const empty of [false,true])for(const records of [[],[[0,1],[0,2]],[[0,2],[0,1]],[[1,1],[1,65535]],[[1,65535],[1,1]],[[2,65535],[0,1]]]){
    add(`${range?'range':'scalar'} empty${empty} ordered${JSON.stringify(records)}`,gsub([{tag:'calt',indices:[0]}],[chain(range,records,empty),single]));
   }
  }
 }
 return cases;
}
