// Zero subtable pointers preserve indices and defer source FontImpl reads.
import {gsub} from './font-gsub-null-recipes.mjs';
const words=values=>values.flatMap(v=>[v>>>8,v&255]);
const single=words([1,6,0,1,1,1]);
const reverse=words([1,10,0,0,0,1,0]);
function lookup(kind,subtables,flags=0,filter=true){
 const offsets=[],payload=[];
 for(const subtable of subtables){
  offsets.push(subtable===null?0:6+subtables.length*2+(flags&16?2:0)+payload.length);
  if(subtable!==null)payload.push(...subtable);
 }
 return words([kind,flags,subtables.length,...offsets,...(flags&16&&filter?[0]:[])]).concat(payload);
}
function valid(kind){
 if(kind===1)return single;
 if(kind===6)return chain(false,[],true);
 if(kind===8)return reverse;
 if(kind===7)return words([1,1,0,8]).concat(single);
 return words([1,6,0,1,0]);
}
function chain(range,records,empty=false){
 const coverage=empty?[1,0]:range?[2,1,1,2,0]:[1,1,1];
 const offset=14+records.length*4;
 return words([3,0,2,offset,offset+coverage.length*2,0,records.length,...records.flat(),...coverage,...coverage]);
}
export function subtablesCases(){
 const cases=[];
 for(const woff of [false,true]){
  const add=(name,data,meta=false,extra={})=>cases.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...extra,...(meta?{extraTables:[{tag:'meta',data:[0,0,0,2]}]}:{})});
  for(let kind=1;kind<=8;kind++){
   for(const selected of [false,true])for(const meta of [false,true])for(const flags of [0,16]){
    for(const subtables of [[null],[null,null],[null,valid(kind)],[valid(kind),null]]){
     add(`type${kind} selected${selected} meta${meta} flags${flags} slots${subtables.map(s=>s===null?'null':'valid')}`,gsub([{tag:'calt',indices:selected?[0]:[]}],[lookup(kind,subtables,flags)]),meta);
    }
   }
   for(const flags of [0,16])for(const count of [1,2,3])for(const present of [0,1,2,3]){
    if(present>count)continue;
    const cut=words([kind,flags,count,...Array(present).fill(0)]);
    add(`type${kind} count${count} present${present} flags${flags} missing-filter`,gsub([{tag:'calt',indices:[0]}],[cut]));
   }
   // The first nonnull pointer targets the flags word. Its invalid format
   // wins before the missing second pointer and optional mark-filter word.
   if(kind!==1)for(const meta of [false,true]){
    add(`type${kind} first malformed before missing pointer meta${meta}`,gsub([],[words([kind,16,2,2])]),meta,{format:16});
   }
  }
  for(const range of [false,true])for(const kind of [1,2,3,4,5,6,7,8]){
   for(const position of [0,1,2])for(const selected of [false,true])for(const meta of [false,true]){
    add(`${range?'range':'scalar'} referenced type${kind} position${position} selected${selected} meta${meta}`,gsub([{tag:'calt',indices:selected?[0]:[]}],[lookup(6,[chain(range,[[position,1]])]),lookup(kind,[null])]),meta);
   }
  }
  // Source range substitution intentionally continues after all-nonnull
  // replacements. Scalar substitution returns before the later null slot.
  for(const range of [false,true])for(const subs of [[single,null],[null,single],[single,single,null]]){
   add(`${range?'range':'scalar'} ordered referenced ${subs.map(s=>s===null?'null':'single')}`,gsub([{tag:'calt',indices:[0]}],[lookup(6,[chain(range,[[0,1]])]),lookup(1,subs)]));
  }
 }
 return cases;
}
