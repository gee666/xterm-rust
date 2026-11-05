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
function duplicate(change) {
 return change.duplicateTable??(change.duplicateName?{tag:'name',position:change.duplicateName}:undefined);
}
function construct(change) {
 // Every case starts from the same original descriptors. Mutating a selected
 // hhea/maxp descriptor must not contaminate the next independent source case.
 const records=original.filter(record=>record.tag!==change.tag&&!change.remove?.includes(record.tag)).map(record=>({...record}));
 if(change.metaMutation) {
  const data=Buffer.alloc(18);data.writeUInt16BE(1);data.writeUInt32BE(2,4);
  records.push({tag:'kern',data});
 }
 if(change.data)records.push({tag:change.tag,data:Buffer.from(change.data)});
 if(change.metrics!==undefined) {
  const hhea=records.find(record=>record.tag==='hhea');
  hhea.data=Buffer.from(hhea.data);hhea.data.writeUInt16BE(change.metrics,34);
 }
 if(change.glyphCount!==undefined) {
  const maxp=records.find(record=>record.tag==='maxp');
  maxp.data=Buffer.from(maxp.data);maxp.data.writeUInt16BE(change.glyphCount,4);
 }
 const extra=duplicate(change);
 if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:extra.invalidCompression?Buffer.from(Array.from({length:16},(_,i)=>i+1)):Buffer.alloc(0)});
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
// Empty script/feature/lookup lists form a valid GSUB with no substitutions.
const emptyGsub=[0,1,0,0,0,10,0,12,0,14,0,0,0,0,0,0];
for(const compressed of [false,true]) {
 changes.push({name:`valid empty GSUB ${compressed?'WOFF':'SFNT'}`,tag:'GSUB',data:emptyGsub,woff:compressed});
 for(const position of ['first','last'])changes.push({name:`invalid ${position} GSUB descriptor ${compressed?'WOFF':'SFNT'}`,tag:'GSUB',data:emptyGsub,woff:compressed,duplicateTable:{tag:'GSUB',position}});
}
changes.push({name:'ignored first GSUB invalid compressed data',tag:'GSUB',data:emptyGsub,woff:true,duplicateTable:{tag:'GSUB',position:'first',invalidCompression:true}});
for(const compressed of [false,true]) {
 for(const metrics of [0,1,2,65535])for(const length of [0,1,3,4,7,8,5184])changes.push({name:`hmtx metrics ${metrics} bytes ${length} ${compressed?'WOFF':'SFNT'}`,tag:'hmtx',data:Array(length).fill(0),metrics,woff:compressed});
 for(const position of ['first','last'])changes.push({name:`invalid ${position} hmtx descriptor ${compressed?'WOFF':'SFNT'}`,tag:'hmtx',data:[0,0,0,0],metrics:1,woff:compressed,duplicateTable:{tag:'hmtx',position}});
 changes.push({name:`zero-read hmtx invalid offset ${compressed?'WOFF':'SFNT'}`,tag:'hmtx',data:[],metrics:0,woff:compressed,invalidHmtxOffset:true});
 changes.push({name:`hmtx ignored directory length ${compressed?'WOFF':'SFNT'}`,tag:'hmtx',data:[0,0,0,0],metrics:1,woff:compressed,hmtxDirectoryLength:0});
}
changes.push({name:'zero-read hmtx compressed invalid offset',tag:'hmtx',data:[],metrics:0,woff:true,invalidHmtxOffset:true,compressedHmtx:true});
// Kern is deferred until after outlines, metrics and cmap glyph-name assignment.
for(const compressed of [false,true]) {
 for(const version of [0,1,2,65535]) {
  const data=Buffer.alloc(26);data.writeUInt16BE(version);
  for(let size=0;size<=26;size++)changes.push({name:`kern version ${version} bytes ${size} ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:[...data.subarray(0,size)],woff:compressed});
 }
 for(const mac of [false,true])for(const format of [0,1,255,256])for(const pairs of [0,1,2])for(const available of [0,1,5,6,11,12]) {
  const header=mac?24:18,data=Buffer.alloc(header+available);
  data.writeUInt16BE(mac?1:0);
  if(mac){data.writeUInt32BE(1,4);data.writeUInt16BE(format,12);data.writeUInt16BE(pairs,16);}
  else {data.writeUInt16BE(1,2);data.writeUInt16BE(format,4);data.writeUInt16BE(pairs,10);}
  changes.push({name:`kern ${mac?'Mac':'Windows'} format ${format} pairs ${pairs} payload ${available} ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:[...data],woff:compressed});
 }
 const validKern=Array(18).fill(0);
 for(const position of ['first','last'])changes.push({name:`invalid ${position} kern descriptor ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:validKern,woff:compressed,duplicateTable:{tag:'kern',position}});
 changes.push({name:`kern cmap rejection precedes invalid version ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:[255,255],glyphCount:0,woff:compressed});
 changes.push({name:`kern cmap rejection precedes invalid descriptor ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:validKern,glyphCount:0,woff:compressed,duplicateTable:{tag:'kern',position:'last'}});
 changes.push({name:`kern version rejection precedes invalid GSUB descriptor ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:[0,2],woff:compressed,duplicateTable:{tag:'GSUB',position:'last'}});
 for(const tag of ['name','hmtx'])changes.push({name:`kern ${tag} missing precedes invalid version ${compressed?'WOFF':'SFNT'}`,tag:'kern',data:[255,255],remove:[tag],woff:compressed});
}
changes.push({name:'ignored first kern invalid compressed data',tag:'kern',data:Array(18).fill(0),woff:true,duplicateTable:{tag:'kern',position:'first',invalidCompression:true}});
changes.push({name:'kern ignored directory length',tag:'kern',data:Array(18).fill(0),directoryLength:0});
changes.push({name:'deferred GSUB ignored directory length',tag:'GSUB',data:emptyGsub,directoryLength:0});
// META is the last deferred constructor table. Its strings are byte reads,
// not UTF8 validation, and appl/bild payloads are deliberately never decoded.
const emptyMeta=Buffer.alloc(16);emptyMeta.writeUInt32BE(1);
for(const compressed of [false,true]) {
 for(const version of [0,1,2,0xffffffff])for(let size=0;size<=17;size++) {
  const data=Buffer.alloc(17);data.writeUInt32BE(version);
  changes.push({name:`meta version ${version} bytes ${size} ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[...data.subarray(0,size)],woff:compressed});
 }
 for(const tag of ['dlng','slng','appl','bild','____'])for(const offset of [0,28,29,0xfffffff0])for(const length of [0,1,2,3]) {
  const data=Buffer.alloc(30,0xff);emptyMeta.copy(data);data.writeUInt32BE(1,12);data.write(tag,16,4,'latin1');data.writeUInt32BE(offset,20);data.writeUInt32BE(length,24);
  changes.push({name:`meta map ${tag} offset ${offset} length ${length} ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[...data],woff:compressed});
 }
 for(const count of [1,2,0xffffffff])for(const length of [16,17,27,28,39,40]) {
  const data=Buffer.alloc(40);emptyMeta.copy(data);data.writeUInt32BE(count,12);
  changes.push({name:`meta maps ${count} bytes ${length} ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[...data.subarray(0,length)],woff:compressed});
 }
 for(const position of ['first','last'])changes.push({name:`invalid ${position} meta descriptor ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[...emptyMeta],woff:compressed,duplicateTable:{tag:'meta',position}});
 for(const tag of ['name','hmtx'])changes.push({name:`meta ${tag} missing precedes invalid version ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[0,0,0,2],remove:[tag],woff:compressed});
 changes.push({name:`meta cmap rejection precedes invalid version ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[0,0,0,2],glyphCount:0,woff:compressed});
 for(const tag of ['kern','GSUB'])changes.push({name:`meta ${tag} invalid descriptor precedes invalid version ${compressed?'WOFF':'SFNT'}`,tag:'meta',data:[0,0,0,2],woff:compressed,duplicateTable:{tag,position:'last'}});
}
changes.push({name:'ignored first meta invalid compressed data',tag:'meta',data:[...emptyMeta],woff:true,duplicateTable:{tag:'meta',position:'first',invalidCompression:true}});
changes.push({name:'selected meta invalid compressed data',tag:'meta',data:[...emptyMeta],woff:true,duplicateTable:{tag:'meta',position:'last',invalidCompression:true}});
changes.push({name:'meta ignored directory length',tag:'meta',data:[...emptyMeta],directoryLength:0});
for(const [initial,final]of [[1,2],[2,1]]) {
 const data=Buffer.from(emptyMeta);data.writeUInt32BE(initial);
 changes.push({name:`meta warning changes version ${initial} to ${final}`,tag:'meta',data:[...data],metaMutation:final});
}
const cases=[];
for(const change of changes) {
 let bytes=construct(change);
 if(change.woff)bytes=woff(bytes);
 const extra=duplicate(change);
 if(extra) {
  const index=extra.position==='first'?0:bytes.readUInt16BE(change.woff?12:4)-1,at=(change.woff?44:12)+index*(change.woff?20:16);
  if(!extra.invalidCompression)bytes.writeUInt32BE(0xfffffff0,at+(change.woff?4:8));
  bytes.writeUInt32BE(128,at+12);
  if(change.woff)bytes.writeUInt32BE(16,at+8);
 }
 if(change.hmtxDirectoryLength!==undefined) {
  const at=(change.woff?44:12)+(bytes.readUInt16BE(change.woff?12:4)-1)*(change.woff?20:16);
  bytes.writeUInt32BE(change.hmtxDirectoryLength,at+12);
 }
 if(change.invalidHmtxOffset) {
  const at=(change.woff?44:12)+(bytes.readUInt16BE(change.woff?12:4)-1)*(change.woff?20:16);
  bytes.writeUInt32BE(0xfffffff0,at+(change.woff?4:8));
  if(change.compressedHmtx){bytes.writeUInt32BE(16,at+8);bytes.writeUInt32BE(128,at+12);}
 }
 if(change.directoryLength!==undefined)bytes.writeUInt32BE(change.directoryLength,12+(bytes.readUInt16BE(4)-1)*16+12);
 let outcome;
 const input=bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length);
 const warn=Object.getOwnPropertyDescriptor(console,'warn');
 if(change.metaMutation)Object.defineProperty(console,'warn',{configurable:true,value(){
  const view=new DataView(input);
  for(let i=0;i<view.getUint16(4);i++)if(view.getUint32(12+i*16)===0x6d657461)view.setUint32(view.getUint32(20+i*16),change.metaMutation);
 }});
 try {parse(input);outcome={status:'ok'};}
 catch(error){outcome={status:'error',name:error.name,message:error.message};}
 finally {if(change.metaMutation){if(warn)Object.defineProperty(console,'warn',warn);else delete console.warn;}}
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
