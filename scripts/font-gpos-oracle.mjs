// Independent JS/Rust font reconstruction shares recipes, not parser output.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
const args=process.argv.slice(2),generate=args.includes('--generate');
if(args.some(arg=>arg!=='--generate'))throw new Error('Use --generate only');
const require=createRequire(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
const {parse}=require('opentype.js'),dependency=require.resolve('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
const original=Array.from({length:base.readUInt16BE(4)},(_,i)=>{
 const at=12+i*16,start=base.readUInt32BE(at+8),length=base.readUInt32BE(at+12);
 return {tag:base.toString('latin1',at,at+4),data:base.subarray(start,start+length)};
});
const woffTemplate=Object.fromEntries(original.map(({tag,data})=>[tag,[...deflateSync(data)]]));
// Mirrors the retained metadata test format. Deliberately independent from the
// Rust byte builder so a shared malformed-input bug cannot make a false pass.
function construct(change){
 const records=original.filter(record=>record.tag!==change.tag&&!change.remove?.includes(record.tag)).map(record=>({...record}));
 records.push(...(change.extraTables??[]).map(({tag,data})=>({tag,data:Buffer.from(data)})));
 records.push({tag:change.tag,data:Buffer.from(change.data)});
 const extra=change.duplicateTable;
 if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:Buffer.alloc(0)});
 const woff=change.woff,start=woff?44:12,width=woff?20:16;
 const header=Buffer.alloc(start+records.length*width),pieces=[header];
 if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}
 else{base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
 let cursor=header.length;
 records.forEach(({tag,data},index)=>{
  const at=start+index*width,compressed=woff?deflateSync(data):data,payload=compressed.length<data.length?compressed:data;
  header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+(woff?4:8));
  if(woff)header.writeUInt32BE(payload.length,at+8);
  header.writeUInt32BE(index===records.length-1?(change.directoryLength??data.length):data.length,at+12);
  pieces.push(payload);cursor+=payload.length;
 });
 if(woff)header.writeUInt32BE(cursor,8);
 const bytes=Buffer.concat(pieces);
 if(extra){const at=start+(extra.position==='first'?0:records.length-1)*width;bytes.writeUInt32BE(0xfffffff0,at+(woff?4:8));bytes.writeUInt32BE(128,at+12);if(woff)bytes.writeUInt32BE(16,at+8);}
 return bytes;
}
const words=values=>{const bytes=Buffer.alloc(values.length*2);values.forEach((value,i)=>bytes.writeUInt16BE(value,i*2));return [...bytes];};
const empty=words([1,0,0,0,0]);
// Root lookup list10 -> lookup14 -> subtable22. Coverage is null unless changed.
const lookup=(kind,data,flags=0)=>words([1,0,0,0,10,1,4,kind,flags,1,8]).concat(data);
const single=lookup(1,words([1,0,15,1,2,3,4]));
const pair=lookup(2,words([2,0,0,0,16,18,1,1,3,4]));
// Default DFLT language selects feature0, whose kern record selects lookup0.
const layout=words([1,0,10,30,44,1]).concat([...Buffer.from('DFLT')],words([8,4,0,0,65535,1,0,1]),[...Buffer.from('kern')],words([8,0,1,0,1,4,3,0,0]));
const cases=[];
function add(name,data,extra={}){cases.push({name,tag:'GPOS',data,...extra});}
for(const woff of [false,true]){
 const suffix=woff?' WOFF':' SFNT',extra={woff};
 for(const [name,data]of [
  ['pair sets',lookup(2,words([1,0,15,240,2,14,0,2,7,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]))],
  ['class1',lookup(2,words([2,0,1,2,16,24,2,2,1,7,1,3,2,1,0,0,0,1,2,3,4,5,6,7,8]))],
  ['coverage1',lookup(1,words([2,14,1,2,3,4,5,1,1,7]))],
  ['coverage2',lookup(1,words([1,8,1,7,2,1,7,9,0]))],
  ['languages',words([1,0,10,0,0,1]).concat([...Buffer.from('latn')],words([8,0,1]),[...Buffer.from('ENG ')],words([10,0,65535,2,1,2]))],
 ])for(let length=0;length<=data.length;length++)add(`GPOS ${name} EOF${length}${suffix}`,data.slice(0,length),extra);
 for(const [major,minor]of [[1,0],[1,1],[0,10],[0,11],[1,2],[2,0],[65535,65535]]){
  const data=words([major,minor,0,0,0,0,0]);
  for(let length=0;length<=data.length;length++)add(`GPOS version ${major}.${minor} EOF${length}${suffix}`,data.slice(0,length),extra);
 }
 for(const [name,data]of [['single',single],['pair warnings',pair],['default layout',layout],['empty',empty]]){
  for(let length=0;length<=data.length;length++)add(`GPOS ${name} EOF${length}${suffix}`,data.slice(0,length),extra);
 }
 for(const kind of [0,1,2,3,4,5,6,7,8,9,10,65535])for(const format of [0,1,2,3]){
  add(`GPOS lookup${kind} format${format}${suffix}`,lookup(kind,words([format,0,0,0,0,0,0,0])),extra);
 }
 for(const kind of [1,2])for(const format of [1,2])for(const valueFormat of [0,1,2,4,8,16,32,64,128,255,256,65535]){
  const payload=words([format,0,valueFormat,valueFormat,0,0,0,0,...Array(20).fill(0)]);
  add(`GPOS values${kind}/${format}/${valueFormat}${suffix}`,lookup(kind,payload),extra);
 }
 for(const kind of [1,2])for(const coverage of [0,1,2,3])for(const count of [0,1,2,65535]){
  const payload=words([1,14,0,0,0,0,0,coverage,count,0,0,0]);
  add(`GPOS coverage${kind}/${coverage}/${count}${suffix}`,lookup(kind,payload),extra);
 }
 for(const pointer of [0,1,4,22,65535])for(const flags of [0,16,65535]){
  const bytes=Buffer.from(lookup(9,words([0]),flags));bytes.writeUInt16BE(pointer,20);
  add(`GPOS ignored payload${pointer}/flags${flags}${suffix}`,[...bytes],extra);
 }
 for(const field of [4,6,8])for(const pointer of [0,1,10,11,65535]){
  const bytes=Buffer.from([...empty,...Array(16).fill(0)]);bytes.writeUInt16BE(pointer,field);
  add(`GPOS root pointer${field}/${pointer}${suffix}`,[...bytes],extra);
 }
 for(const [major,minor]of [[1,0],[1,1],[0,0]])for(const count of [0,1,2,65535]){
  const data=words([1,1,0,0,0,0,14,major,minor,0,count,65535,65535,65535,65535]);
  add(`GPOS variations${major}/${minor}/${count}${suffix}`,data,extra);
 }
 for(const position of ['first','last'])add(`GPOS invalid ${position} descriptor${suffix}`,empty,{...extra,duplicateTable:{tag:'GPOS',position}});
 for(const missing of ['name','hmtx'])add(`GPOS earlier missing ${missing}${suffix}`,[0],{...extra,remove:[missing]});
 for(const [tag,data]of [['GDEF',words([2,0])],['GSUB',words([0,0])],['fvar',words([2,0])],['meta',words([0,2])]]){
  add(`GPOS versus invalid ${tag}${suffix}`,[0],{...extra,extraTables:[{tag,data}],remove:[tag]});
 }
 add('GPOS two class warnings'+suffix,pair,extra);
 add('GPOS class warning changes next class'+suffix,pair,{...extra,warningMutation:{tag:'GPOS',at:32,value:0}});
 add('GPOS warning breaks GSUB'+suffix,pair,{...extra,remove:['GSUB'],extraTables:[{tag:'GSUB',data:words([1,0,10,12,14,0,0,0])}],warningMutation:{tag:'GSUB',at:0,value:0}});
 for(const thrown of ['object','number','undefined','null'])add(`GPOS class warning throws ${thrown}${suffix}`,pair,{...extra,throws:thrown});
 add('GPOS compressed class warning'+suffix,[...pair,...Array(1024).fill(0)],{...extra,warningMutation:{tag:'GPOS',at:32,value:0}});
}
for(const woff of [false,true])for(const [field,values]of [[16,[0,1,8]],[18,[0,4]],[28,[0,1,65535]],[36,[0,8]],[42,[0,1,65535]],[46,[0,4]]])for(const value of values){
 const data=Buffer.from(layout);data.writeUInt16BE(value,field);add(`GPOS init ${field}/${value}/${woff}`,[...data],{woff});
}
add('GPOS ignored directory length',empty,{directoryLength:0});
let seed=0x4476a05;
const random=()=>{seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;};
for(let i=0;i<128;i++){
 const data=Buffer.from(lookup(1+random()%9,words(Array.from({length:24},()=>random()%20))));
 if(i%3===0)data.writeUInt16BE(1+random()%2,22);
 add('GPOS seeded'+i,[...data],{woff:i%2===0});
}
const descriptor=Object.getOwnPropertyDescriptor(console,'warn');
try{
 for(const change of cases){
  const bytes=Buffer.from(Uint8Array.from(construct(change)).buffer),warnings=[];
  change.bytesSha256=hash(bytes);change.inputHash=bytes.reduce((h,b)=>Math.imul(h^b,0x01000193)>>>0,0x811c9dc5);
  if(change.woff)change.woffPayloads=Object.fromEntries([...(change.extraTables??[]),{tag:change.tag,data:change.data}].map(({tag,data})=>[tag,[...deflateSync(Buffer.from(data))]]));
  Object.defineProperty(console,'warn',{configurable:true,value(message){
   warnings.push(message);
   if(change.throws)throw change.throws==='object'?{}:change.throws==='number'?17:change.throws==='null'?null:undefined;
   if(warnings.length===1&&change.warningMutation){const m=change.warningMutation,woff=bytes.toString('latin1',0,4)==='wOFF',start=woff?44:12,width=woff?20:16;for(let i=0;i<bytes.readUInt16BE(woff?12:4);i++){const at=start+i*width;if(bytes.toString('latin1',at,at+4)===m.tag){bytes.writeUInt16BE(m.value,bytes.readUInt32BE(at+(woff?4:8))+m.at);break;}}}
  }});
  try{parse(bytes.buffer);change.outcome={status:'ok'};}catch(error){change.outcome=change.throws&&warnings.length?{status:'warning'}:{status:'error',name:error.name,message:error.message};}
  change.warnings=warnings;
 }
}finally{if(descriptor)Object.defineProperty(console,'warn',descriptor);else delete console.warn;}
const fixture={sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),seed:'0x4476a05',woffTemplate,cases};
const content=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-gpos.json';
if(generate)await fs.writeFile(file,content);else if(await fs.readFile(file,'utf8')!==content)throw new Error('GPOS corpus drift; inspect before --generate');
const report={node:process.version,scope:'GPOS constructor only, not full font or Terminal parity',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(content),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,warningFailures:cases.filter(c=>c.outcome.status==='warning').length};
await fs.mkdir('results/font-gpos',{recursive:true});await fs.writeFile('results/font-gpos/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
export {construct,cases,base};
