// Source and Rust independently reconstruct these GDEF constructor inputs.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
const args=process.argv.slice(2),generate=args.includes('--generate');
if(args.some(arg=>arg!=='--generate'&&!arg.startsWith('--source=')))throw new Error('Use --generate and/or --source=PATH');
const source=args.find(arg=>arg.startsWith('--source='))?.slice(9)??'source_repo';
const require=createRequire(path.resolve(source,'package.json')),{parse}=require('opentype.js');
const dependency=require.resolve('opentype.js'),hash=value=>createHash('sha256').update(value).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
const inputHash=bytes=>{let value=0x811c9dc5;for(const byte of bytes)value=Math.imul(value^byte,0x01000193);return value>>>0;};
const original=Array.from({length:base.readUInt16BE(4)},(_,i)=>{
 const at=12+i*16,start=base.readUInt32BE(at+8),length=base.readUInt32BE(at+12);
 return {tag:base.toString('latin1',at,at+4),data:base.subarray(start,start+length)};
});
const woffTemplate=Object.fromEntries(original.map(({tag,data})=>[tag,[...deflateSync(data)]]));
function construct(change) {
 const records=original.filter(record=>record.tag!==change.tag&&!change.remove?.includes(record.tag)).map(record=>({...record}));
 records.push(...(change.extraTables??[]).map(table=>({tag:table.tag,data:Buffer.from(table.data)})));
 records.push({tag:change.tag,data:Buffer.from(change.data)});
 const extra=change.duplicateTable;
 if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:Buffer.alloc(0)});
 const woff=change.woff,start=woff?44:12,width=woff?20:16;
 const header=Buffer.alloc(start+records.length*width),pieces=[header];
 if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}
 else {base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
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
function words(values,size=values.length*2){const data=Buffer.alloc(size);values.forEach((value,i)=>data.writeUInt16BE(value,i*2));return [...data];}
const empty=words([1,0,0,0,0,0]),classes=words([1,0,14,0,0,16,0,3,4]);
const attach=words([1,0,0,16,0,0,0,0,8,1,16,0,1,1,7,0,2,4,8]);
const caret=words([1,0,0,0,16,0,0,0,0,1,8,0,1,4,1,42]);
const marks=words([1,2,0,0,0,0,16,0,37,1,8,0,1,1,7]);
const cases=[];
function add(name,data,extra={}){cases.push({name,tag:'GDEF',data,...extra});}
for(const woff of [false,true]) {
 const suffix=woff?' WOFF':' SFNT';
 for(const [major,minor] of [[1,0],[1,1],[1,2],[1,3],[1,4],[0,10],[0,12],[0,13],[2,0],[65535,65535]]) {
  const data=words([major,minor,0,0,0,0,0]);
  for(let size=0;size<=16;size++)add(`GDEF version ${major}.${minor} EOF ${size}${suffix}`,[...data,...[0,0]].slice(0,size),{woff});
 }
 for(const [label,data] of [['attach',attach],['caret',caret],['mark sets',marks]]) {
  for(let size=0;size<=data.length;size++)add(`GDEF ${label} EOF ${size}${suffix}`,data.slice(0,size),{woff});
 }
 for(const field of [4,6,8,10,12])for(const pointer of [0,1,2,12,13,14,15,16,17,65535]) {
  const data=Buffer.from(words([1,3,0,0,0,0,0],32));data.writeUInt16BE(pointer,field);
  add(`GDEF root pointer ${field}/${pointer}${suffix}`,[...data],{woff});
 }
 for(const format of [0,1,2,3,65535])for(const count of [0,1,2,65535]) {
  add(`GDEF class ${format} count ${count}${suffix}`,words([1,0,12,0,0,0,format,0,count,0,0,0]),{woff});
  const data=Buffer.from(attach);data.writeUInt16BE(format,24);data.writeUInt16BE(count,26);
  add(`GDEF coverage ${format} count ${count}${suffix}`,[...data],{woff});
 }
 for(const format of [0,1,2,3,4,65535]) {
  const data=Buffer.from(caret);data.writeUInt16BE(format,28);
  add(`GDEF caret format ${format}${suffix}`,[...data],{woff});
 }
 for(const where of [18,20,24,26])for(const value of [0,1,2,31,65535]) {
  const data=Buffer.from(caret);data.writeUInt16BE(value,where);
  add(`GDEF caret nested ${where}/${value}${suffix}`,[...data],{woff});
 }
 for(const position of ['first','last'])add(`GDEF invalid ${position} descriptor${suffix}`,empty,{woff,duplicateTable:{tag:'GDEF',position}});
 for(const missing of ['name','hmtx'])add(`GDEF earlier missing ${missing}${suffix}`,[0],{woff,remove:[missing]});
 add('GDEF invalid version'+suffix,words([2,0]),{woff});
 add('GDEF two class warnings'+suffix,classes,{woff});
 add('GDEF class warning skips next pointer'+suffix,classes,{woff,warningMutation:{tag:'GDEF',at:10,value:0}});
 add('GDEF class warning keeps parsed version'+suffix,classes,{woff,warningMutation:{tag:'GDEF',at:0,value:2}});
 for(const thrown of ['object','number','undefined','null'])add(`GDEF class warning throws ${thrown}${suffix}`,classes,{woff,throws:thrown});
 for(const tag of ['kern','GSUB','meta']) {
  const data=tag==='kern'?words([2,0]):tag==='meta'?words([0,2]):words([0,0]);
  add(`GDEF versus invalid ${tag}${suffix}`,[0],{woff,remove:[tag],extraTables:[{tag,data}]});
 }
 const kern=Buffer.from(words([1,0,0,2,0,0,1,0,0]));
 add('kern warning fixes GDEF'+suffix,words([2,0,0,0,0,0]),{woff,remove:['kern'],extraTables:[{tag:'kern',data:[...kern]}],warningMutation:{tag:'GDEF',at:0,value:1}});
 add('GDEF warning breaks later META'+suffix,classes,{woff,remove:['meta'],extraTables:[{tag:'meta',data:Array(16).fill(0).map((v,i)=>i===3?1:v)}],warningMutation:{tag:'meta',at:2,value:2}});
}
add('GDEF ignored nominal directory length',empty,{directoryLength:0});
for(const woff of [false,true]) {
 add(`GDEF large classes warning mutation ${woff?'compressed':'plain'}`,words([1,0,14,0,0,16,0,3,4],1024),{woff,warningMutation:{tag:'GDEF',at:10,value:0}});
 add(`GDEF large warning to GSUB ${woff?'compressed':'plain'}`,words([1,0,14,0,0,0,0,3],1024),{woff,remove:['GSUB'],extraTables:[{tag:'GSUB',data:words([0,0,10,12,14,0,0,0])}],warningMutation:{tag:'GSUB',at:0,value:1}});
}
let seed=0x6DEF428;
const random=()=>{seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;};
for(let i=0;i<256;i++) {
 const data=Buffer.alloc(16+random()%49);data.writeUInt16BE(1);data.writeUInt16BE([0,2,3][random()%3],2);
 for(let at=4;at+1<data.length;at+=2)data.writeUInt16BE(random()%40,at);
 add('GDEF seeded '+i,[...data],{woff:i%2===0});
}
const warn=Object.getOwnPropertyDescriptor(console,'warn');
try {
 for(const change of cases) {
  const bytes=Buffer.from(Uint8Array.from(construct(change)).buffer),warnings=[];let count=0;
  change.bytesSha256=hash(bytes);change.inputHash=inputHash(bytes);
  if(change.woff)change.woffPayloads=Object.fromEntries([...(change.extraTables??[]),{tag:change.tag,data:change.data}].map(({tag,data})=>[tag,[...deflateSync(Buffer.from(data))]]));
  Object.defineProperty(console,'warn',{configurable:true,value(message){
   warnings.push(message);count++;
   if(change.throws)throw change.throws==='object'?{}:change.throws==='number'?17:change.throws==='null'?null:undefined;
   if(count===1&&change.warningMutation){const m=change.warningMutation,woff=bytes.toString('latin1',0,4)==='wOFF',start=woff?44:12,width=woff?20:16;for(let i=0;i<bytes.readUInt16BE(woff?12:4);i++){const at=start+i*width;if(bytes.toString('latin1',at,at+4)===m.tag){bytes.writeUInt16BE(m.value,bytes.readUInt32BE(at+(woff?4:8))+m.at);break;}}}
  }});
  try {parse(bytes.buffer);change.outcome={status:'ok'};}
  catch(error){change.outcome=change.throws&&count?{status:'warning'}:{status:'error',name:error.name,message:error.message};}
  change.warnings=warnings;
 }
} finally {if(warn)Object.defineProperty(console,'warn',warn);else delete console.warn;}
const fixture={sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),seed:'0x6DEF428',woffTemplate,cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-gdef.json';
if(generate)await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw new Error('GDEF corpus drift; inspect before --generate');
const report={node:process.version,scope:'GDEF constructor reads and callbacks only, not full font parity',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,warningFailures:cases.filter(c=>c.outcome.status==='warning').length};
await fs.mkdir('results/font-gdef',{recursive:true});await fs.writeFile('results/font-gdef/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
export {construct,cases,base};
