// Constructor rejection corpus. The original opentype parser executes only here.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {deflateSync} from 'node:zlib';
const args=process.argv.slice(2), generate=args.includes('--generate');
const source=args.find(arg=>arg.startsWith('--source='))?.slice(9)??'source_repo';
if(args.some(arg=>arg!=='--generate'&&!arg.startsWith('--source=')))throw new Error('Use --generate and/or --source=PATH');
const require=createRequire(path.resolve(source,'package.json'));
const {parse}=require('opentype.js');
const dependency=require.resolve('opentype.js');
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
function entries(bytes) {
 return Array.from({length:bytes.readUInt16BE(4)},(_,i)=>{
  const at=12+i*16,offset=bytes.readUInt32BE(at+8),length=bytes.readUInt32BE(at+12);
  return {tag:bytes.toString('latin1',at,at+4),data:bytes.subarray(offset,offset+length)};
 });
}
const original=entries(base);
function construct(change) {
 const records=original.filter(record=>record.tag!==change.tag&&!change.remove?.includes(record.tag));
 if(change.data)records.push({tag:change.tag,data:Buffer.from(change.data)});
 if(change.duplicateName)records[change.duplicateName==='first'?'unshift':'push']({tag:'name',data:Buffer.alloc(0)});
 const header=Buffer.alloc(12+records.length*16);base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);
 let cursor=header.length;const pieces=[header];
 records.forEach(({tag,data},index)=>{
  const at=12+index*16;header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+8);header.writeUInt32BE(data.length,at+12);
  pieces.push(data);cursor+=data.length;
 });
 return Buffer.concat(pieces);
}
const changes=[{name:'unchanged rebuilt font'}];
for(const tag of ['name','hmtx','glyf','loca'])changes.push({name:'missing '+tag,remove:[tag]});
for(const tag of ['head','hhea','maxp','OS/2','post']) {
 const data=original.find(record=>record.tag===tag).data;
 const minimum={head:54,hhea:36,maxp:32,'OS/2':96,post:32}[tag];
 for(let length=0;length<=minimum+1;length++)changes.push({name:tag+' EOF '+length,tag,data:[...data.subarray(0,length)]});
 changes.push({name:tag+' ignored directory length',tag,data:[...data],directoryLength:0});
}
const head=[...original.find(record=>record.tag==='head').data];head[12]^=1;
changes.push({name:'head wrong magic',tag:'head',data:head});
function table(name,tag,length,writes) {
 const data=Buffer.alloc(Math.max(length,...writes.map(([at,,width=2])=>at+width)));
 for(const [at,value,width=2]of writes)data[width===4?'writeUInt32BE':'writeUInt16BE'](value,at);
 changes.push({name,tag,data:[...data.subarray(0,length)]});
}
for(const version of [0,1,2,5,0xffff])for(const size of [77,78,85,86,95,96])table('OS2 version '+version+' size '+size,'OS/2',size,[[0,version]]);
for(const version of [0,0x5000,0x10000,0x20000])for(const size of [5,6,31,32])table('maxp version '+version+' size '+size,'maxp',size,[[0,version,4],[4,1]]);
for(const version of [1,2])table('ltag version '+version,'ltag',12,[[0,version,4]]);
table('ltag out of bounds string','ltag',16,[[0,1,4],[8,1,4],[12,100],[14,1]]);
table('ltag zero length out of bounds string','ltag',16,[[0,1,4],[8,1,4],[12,65535]]);
for(const [platform,encoding,language]of [[0,0,65535],[0,0,0],[1,0,0],[1,1,0],[3,1,1033],[3,1,65535],[3,0,1033],[2,0,0]]) {
 for(const length of [0,1,2,3,4])table(`name ${platform}/${encoding}/${language} bytes ${length}`,'name',21,[[2,1],[4,18],[6,platform],[8,encoding],[10,language],[12,1],[14,length]]);
}
for(let language=0;language<=151;language++)for(const encoding of [0,1,7]) {
 table(`name Mac language ${language} encoding ${encoding}`,'name',18,[[2,1],[4,18],[6,1],[8,encoding],[10,language],[12,1],[14,1]]);
}
for(const format of [0,1,2])table('name format '+format+' empty','name',6,[[0,format]]);
for(const version of [0x10000,0x20000,0x25000,0x30000,0x40000])for(const size of [31,32,33,34,35,36,37]) {
 const data=Buffer.alloc(size);data.writeUInt32BE(version);if(size>=34)data.writeUInt16BE(1,32);if(size>=36)data.writeUInt16BE(258,34);
 changes.push({name:'post version '+version+' size '+size,tag:'post',data:[...data]});
}
for(const length of [0,1,2,127,128,255]) {
 const data=Buffer.alloc(37);data.writeUInt32BE(0x20000);data.writeUInt16BE(1,32);data.writeUInt16BE(258,34);data[36]=length;
 changes.push({name:'post signed string length '+length,tag:'post',data:[...data]});
}
for(const length of [127,128,200,250,255]) {
 const data=Buffer.alloc(39);data.writeUInt32BE(0x20000);data.writeUInt16BE(2,32);data.writeUInt16BE(258,34);data.writeUInt16BE(258,36);data[38]=length;
 changes.push({name:'post repeated signed string length '+length,tag:'post',data:[...data]});
}
for(const name of ['unchanged rebuilt font','head wrong magic','head EOF 53','head EOF 54','hhea EOF 35','name format 1 empty','ltag version 2','post signed string length 128']) {
 changes.push({...changes.find(change=>change.name===name),name:name+' compressed WOFF',woff:true});
}
function woff(bytes) {
 const records=entries(bytes),header=Buffer.alloc(44+records.length*20),pieces=[header];
 header.write('wOFF');bytes.copy(header,4,0,4);header.writeUInt16BE(records.length,12);
 let cursor=header.length;
 records.forEach(({tag,data},index)=>{
  const compressed=deflateSync(data),payload=compressed.length<data.length?compressed:data,at=44+index*20;
  header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+4);header.writeUInt32BE(payload.length,at+8);header.writeUInt32BE(data.length,at+12);
  pieces.push(payload);cursor+=payload.length;
 });
 header.writeUInt32BE(cursor,8);return Buffer.concat(pieces);
}
for(const position of ['first','last'])for(const compressed of [false,true])changes.push({name:`invalid ${position} name descriptor ${compressed?'WOFF':'SFNT'}`,duplicateName:position,woff:compressed});
const cases=[];
for(const change of changes) {
 let bytes=construct(change);
 if(change.woff)bytes=woff(bytes);
 if(change.duplicateName) {
  const index=change.duplicateName==='first'?0:bytes.readUInt16BE(change.woff?12:4)-1,at=(change.woff?44:12)+index*(change.woff?20:16);
  bytes.writeUInt32BE(0xfffffff0,at+(change.woff?4:8));
  bytes.writeUInt32BE(128,at+12);
  if(change.woff)bytes.writeUInt32BE(16,at+8);
 }
 if(change.directoryLength!==undefined)bytes.writeUInt32BE(change.directoryLength,12+(bytes.readUInt16BE(4)-1)*16+12);
 let outcome;
 try {parse(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length));outcome={status:'ok'};}
 catch(error){outcome={status:'error',name:error.name,message:error.message};}
 cases.push({...change,outcome});
}
const fixture={source:dependency.replaceAll('\\','/').split('/node_modules/').at(-1),sourceSha256:digest(await fs.readFile(dependency)),fontSha256:digest(base),cases};
const file='crates/xterm-addons/tests/fixtures/font-metadata.json',text=JSON.stringify(fixture)+'\n';
if(generate)await fs.writeFile(file,text);
else if(await fs.readFile(file,'utf8')!==text)throw new Error('Font metadata corpus drift; inspect before --generate');
await fs.mkdir('results/font-metadata',{recursive:true});
const report={node:process.version,scope:'Unchanged opentype.js metadata constructor baseline, not full font parity',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:digest(text),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length};
await fs.writeFile('results/font-metadata/oracle.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
