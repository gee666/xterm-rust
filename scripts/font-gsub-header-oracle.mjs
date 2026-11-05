// Independent source constructor baseline. Rust reconstructs these same recipes
// with the existing metadata helper, never with this oracle's parser or output.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {deflateSync} from 'node:zlib';
const generate=process.argv.includes('--generate');
if(process.argv.slice(2).some(arg=>arg!=='--generate'))throw new Error('Use --generate only');
const require=createRequire(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
const dependency=require.resolve('opentype.js'),{parse}=require('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputHash=bytes=>{let h=0x811c9dc5;for(const b of bytes)h=Math.imul(h^b,0x01000193);return h>>>0;};
export const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
const original=Array.from({length:base.readUInt16BE(4)},(_,i)=>{const at=12+i*16,start=base.readUInt32BE(at+8);return{tag:base.toString('latin1',at,at+4),data:base.subarray(start,start+base.readUInt32BE(at+12))};});
const woffTemplate=Object.fromEntries(original.map(({tag,data})=>[tag,[...deflateSync(data)]]));
export function construct(change){
 const records=original.filter(({tag})=>tag!=='GSUB'&&!change.remove?.includes(tag)).map(record=>({...record}));
 records.push(...(change.extraTables??[]).map(({tag,data})=>({tag,data:Buffer.from(data)})),{tag:'GSUB',data:Buffer.from(change.data)});
 const woff=change.woff,start=woff?44:12,width=woff?20:16,header=Buffer.alloc(start+records.length*width),pieces=[header];
 if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}else{base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
 let cursor=header.length;
 for(const [i,{tag,data}]of records.entries()){
  const encoded=woff?deflateSync(data):data,payload=encoded.length<data.length?encoded:data,at=start+i*width;
  header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+(woff?4:8));
  if(woff)header.writeUInt32BE(payload.length,at+8);
  header.writeUInt32BE(data.length,at+12);pieces.push(payload);cursor+=payload.length;
 }
 if(woff)header.writeUInt32BE(cursor,8);
 return Buffer.concat(pieces);
}
const words=values=>{const d=Buffer.alloc(2*values.length);values.forEach((v,i)=>d.writeUInt16BE(v,i*2));return [...d];};
const recipes=[];
for(const woff of [false,true]){
 const add=(name,data,extra={})=>recipes.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...extra});
 for(const [major,minor]of [[1,0],[1,1],[0,10],[0,11],[0,0],[1,2],[2,0],[65535,65535]]){
  const data=words([major,minor,0,0,0,0,0]);
  for(let length=0;length<=data.length;length++)add(`version ${major}.${minor} EOF${length}`,data.slice(0,length));
 }
 const empty=words([1,0,10,12,14,0,0,0]);
 for(const field of [4,6,8])for(const pointer of [0,10,12,14,65535]){
  const data=Buffer.from(empty);data.writeUInt16BE(pointer,field);add(`root pointer${field}/${pointer}`,[...data]);
 }
 for(const script of [0,8])for(const defaultLanguage of [0,10])for(const language of [0,10]){
  // ScriptList at10, script at18, language record at22, LangSys at28.
  const data=words([1,0,10,0,0,1]).concat([...Buffer.from('DFLT')],words([script,defaultLanguage,1]),[...Buffer.from('ENG ')],words([language,0,65535,0]));
  for(let length=0;length<=data.length;length++)add(`script${script}/default${defaultLanguage}/language${language} EOF${length}`,data.slice(0,length));
 }
 for(const major of [0,1,2])for(const minor of [0,1,65535])for(const count of [0,1,2,65535]){
  // Version1.1 follows only the variation header and fixed records, not targets.
  const data=words([1,1,0,0,0,0,14,major,minor,0,count,65535,65520,65535,65520,0,0,0,0]);
  for(const length of [14,17,18,21,22,29,30,37,38])add(`variation${major}.${minor}/count${count} EOF${length}`,data.slice(0,length));
 }
 for(const count of [0,1,2]){
  const data=words([1,1,0,0,0,0,14,1,0,0,count,...Array(4*count).fill(65535)]);
  add(`ignored variation targets${count}`,data);
  add(`variation${count} before invalid meta`,data,{extraTables:[{tag:'meta',data:[0,0,0,2]}]});
 }
 add('invalid GSUB before invalid META',words([0,0]),{extraTables:[{tag:'meta',data:[0,0,0,2]}]});
 for(const remove of ['name','hmtx'])add(`earlier missing ${remove}`,words([0,0]),{remove:[remove]});
}
export const cases=[];
for(const recipe of recipes){
 const bytes=construct(recipe);let outcome;
 try{parse(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length));outcome={status:'ok'};}
 catch(error){outcome={status:'error',name:error.name,message:error.message};}
 const woffPayloads=recipe.woff?Object.fromEntries([...(recipe.extraTables??[]),{tag:'GSUB',data:recipe.data}].map(({tag,data})=>[tag,[...deflateSync(Buffer.from(data))]])):undefined;
 cases.push({...recipe,...(woffPayloads?{woffPayloads}:{}),inputSha256:hash(bytes),inputHash:inputHash(bytes),outcome});
}
const fixture={sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),woffTemplate,cases};
const content=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-gsub-header.json';
if(generate)await fs.writeFile(file,content);else if(await fs.readFile(file,'utf8')!==content)throw new Error('GSUB header corpus drift');
const report={scope:'GSUB header, nullable root/script/language and retained variation records only',node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(content),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length};
await fs.mkdir('results/font-gsub-header',{recursive:true});await fs.writeFile('results/font-gsub-header/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
