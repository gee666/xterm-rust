// Null records retain their list positions. This writer is independent of Rust.
const words=values=>values.flatMap(v=>[v>>>8,v&255]);
export function gsub(features,lookups){
 const records=words([features.length]),payload=[];
 for(const {tag,indices} of features){
  records.push(...[...tag].map(c=>c.charCodeAt(0)),...words([indices===null?0:2+features.length*6+payload.length]));
  if(indices!==null)payload.push(...words([0,indices.length,...indices]));
 }
 const featureList=records.concat(payload),list=words([lookups.length]),tables=[];
 for(const lookup of lookups){
  list.push(...words([lookup===null?0:2+lookups.length*2+tables.length]));
  if(lookup!==null)tables.push(...lookup);
 }
 return words([1,0,0,10,10+featureList.length]).concat(featureList,list,tables);
}
export function nullCases(){
 const cases=[],empty=words([1,0,0]),invalid=words([10]),missingFlags=words([1]);
 for(const woff of [false,true]){
  const add=(name,data,extra={})=>cases.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...extra});
  const feature=(tag,indices)=>({tag,indices});
  for(const tag of ['calt','liga'])for(const indices of [null,[],[0],[1],[0,1],[1,0],[0,1,0],[65535]]){
   for(const lookups of [[null],[null,null],[null,empty],[empty,null],[null,invalid],[null,missingFlags]]){
    const name=`${tag} ${JSON.stringify(indices)} lookups${lookups.map(l=>l===null?'null':l.length===2?'invalid':l.length===6?'empty':'cut').join(',')}`;
    const data=gsub([feature(tag,indices)],lookups);
    add(name,data);
    add(name+' before META',data,{extraTables:[{tag:'meta',data:[0,0,0,2]}]});
   }
  }
  // A null calt survives the OpenType parser. Later feature/lookup reads and
  // metadata still run before FontImpl reduces calt lookupListIndexes.
  for(const firstTag of ['calt','liga'])for(const secondTag of ['calt','liga'])for(const indices of [null,[],[0]]){
   const data=gsub([feature(firstTag,null),feature(secondTag,indices)],[null,empty]);
   add(`two features ${firstTag} null then ${secondTag} ${JSON.stringify(indices)}`,data);
   for(const cut of [0,1,2,3])add(`later feature EOF${cut} after ${firstTag} null ${secondTag}`,data.slice(0,24+cut));
  }
  // Offset reads must interleave with pointed-to lookup parsing. A malformed
  // first lookup wins over a missing second pointer, while a null first skips.
  for(const first of [0,2,4,65535])for(const cut of [14,15,16,17,18]){
   const data=words([1,0,0,0,10,2,first,10,0]);
   add(`lookup pointer${first} next pointer EOF${cut}`,data.slice(0,cut));
  }
  for(const range of [false,true])for(const selected of [[0],[1],[0,1],[]]){
   const chain=words([6,0,1,8,3,0,1,14,0,1,0,1,...(range?[2,1,1,2,0]:[1,1,1])]);
   add(`null referenced lookup ${range?'range':'scalar'} selected${selected.join(',')}`,gsub([feature('calt',selected)],[chain,null]));
  }
  // Index1 must still select the real type6 lookup after the null index0.
  const chain=words([6,0,1,8,3,0,1,14,0,0,1,1,1]);
  for(const selected of [[1],[0],[0,1],[],[65535]])add(`retain null slot selected${selected.join(',')}`,gsub([feature('calt',selected)],[null,chain]));
 }
 return cases;
}
