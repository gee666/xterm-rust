// Type9 has no source subtable parser. Keep malformed recipes independent of Rust.
const words=values=>values.flatMap(v=>[v>>>8,v&255]);
export function type9Cases(){
 const cases=[];
 for(const woff of [false,true]){
  const add=(name,data,extra={})=>cases.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...extra});
  for(const flags of [0,1,16,17,65535])for(const offsets of [[],[0],[0,0],[1],[65535],[1,0],[0,1],[0,65535]]){
   const data=words([1,0,0,0,10,1,4,9,flags,offsets.length,...offsets,0]);
   // Directed cuts include a missing second offset after an earlier nonnull
   // pointer. Eagerly reading all offsets would mask its TypeError with bounds.
   for(let length=14;length<=data.length;length++)add(`type9 flags${flags} offsets${offsets.join(',')} EOF${length}`,data.slice(0,length));
   add(`type9 before META flags${flags} offsets${offsets.join(',')}`,data,{extraTables:[{tag:'meta',data:[0,0,0,2]}]});
  }
  // Selected type9 alone is ignored by FontImpl, but a type6 rule referencing
  // one of its null subtables dereferences coverage. Keep both scalar/range paths.
  for(const count of [0,1,2])for(const range of [false,true]){
   const feature=words([1,...[0x6361,0x6c74],8,0,1,0]);
   const chain=words([6,0,1,8,3,0,1,14,0,1,0,1,...(range?[2,1,1,2,0]:[1,1,1])]);
   const lookupList=words([2,6,6+chain.length]).concat(chain,words([9,0,count,...Array(count).fill(0)]));
   const data=words([1,0,0,10,10+feature.length]).concat(feature,lookupList);
   add(`referenced type9 count${count} ${range?'range':'scalar'}`,data);
  }
  for(const count of [0,1,2]){
   const feature=words([1,...[0x6361,0x6c74],8,0,1,0]);
   const data=words([1,0,0,10,10+feature.length]).concat(feature,words([1,4,9,0,count,...Array(count).fill(0)]));
   add(`selected type9 count${count}`,data);
  }
 }
 return cases;
}
