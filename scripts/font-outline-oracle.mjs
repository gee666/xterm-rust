// Source and Rust independently reconstruct the TrueType constructor inputs.
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
 const records=original.filter(record=>record.tag!==change.tag&&!change.remove?.includes(record.tag)).map(record=>{
 const data=Buffer.from(record.data);
 if(record.tag==='maxp'&&change.glyphCount!==undefined)data.writeUInt16BE(change.glyphCount,4);
 if(record.tag==='hhea'&&change.metrics!==undefined)data.writeUInt16BE(change.metrics,34);
 return {...record,data};
 });
 records.push(...(change.extraTables??[]).map(table=>({tag:table.tag,data:Buffer.from(table.data)})));
 records.push({tag:change.tag,data:change.fillLength===undefined?Buffer.from(change.data):Buffer.alloc(change.fillLength,change.fillValue)});
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

const table=tag=>original.find(t=>t.tag===tag).data;
const cases=[];
function add(name,tag,data,extra={}){cases.push({name,tag,data:[...data],...extra});}
for(const woff of [false,true]) {
 const suffix=woff?' WOFF':' SFNT';
 for(const glyphCount of [undefined,0,1,3,65535])for(const format of [0,1,2,65535]) {
  const head=Buffer.from(table('head'));head.writeUInt16BE(format,50);
  const required=(glyphCount??table('maxp').readUInt16BE(4))+1;
  for(const size of [0,1,2,3,4,6,8,required*2-1,required*2,required*4-1,required*4]) {
   add('loca EOF '+glyphCount+'/'+format+'/'+size+suffix,'loca',Buffer.alloc(size),{
    woff,...(glyphCount===undefined?{}:{glyphCount}),remove:['head'],
    extraTables:[{tag:'head',data:[...head]}]
   });
  }
 }
 for(const missing of ['head','maxp','hhea']) {
  for(const tag of ['loca','glyf'])add(tag+' missing '+missing+suffix,tag,[],{woff,remove:[missing]});
 }
 for(const tag of ['glyf','loca'])for(const position of ['first','last']) {
  add(tag+' invalid '+position+' duplicate'+suffix,tag,table(tag),{woff,duplicateTable:{tag,position}});
 }
 add('glyf empty lazy body'+suffix,'glyf',[],{woff});
 add('glyf arbitrary unread body'+suffix,'glyf',[255],{woff});
 add('loca arbitrary offsets remain lazy'+suffix,'loca',Buffer.alloc(table('loca').length,255),{woff});
 add('loca ignores declared length'+suffix,'loca',table('loca'),{woff,directoryLength:0});
 for(const missing of ['name','hmtx'])add('loca before missing '+missing+suffix,'loca',[],{woff,remove:[missing]});
 for(const tag of ['kern','GSUB','GDEF','meta'])add('loca before '+tag+suffix,'loca',[],{woff,remove:[tag],extraTables:[{tag,data:[0,2,0,0]}]});
}
let seed=0x10CA453;
const random=()=>{seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return seed>>>0;};
for(let i=0;i<128;i++){
 const count=random()%1024,short=random()%2===0,head=Buffer.from(table('head'));head.writeUInt16BE(short?0:1,50);
 const size=random()%8192,data=Buffer.alloc(size);for(let j=0;j<size;j++)data[j]=random()%256;
 add('loca seeded '+i,'loca',data,{woff:i%2===0,glyphCount:count,remove:['head'],extraTables:[{tag:'head',data:[...head]}]});
}
for(const change of cases) {
 const bytes=Buffer.from(Uint8Array.from(construct(change)).buffer);
 change.bytesSha256=hash(bytes);change.inputHash=inputHash(bytes);
 if(change.woff)change.woffPayloads=Object.fromEntries([...(change.extraTables??[]),{tag:change.tag,data:change.data}].map(({tag,data})=>[tag,[...deflateSync(Buffer.from(data))]]));
 try {parse(bytes.buffer);change.outcome={status:'ok'};}
 catch(error){change.outcome={status:'error',name:error.name,message:error.message};}
 if(change.data.length>64&&change.data.every(v=>v===change.data[0])) {
  change.fillLength=change.data.length;change.fillValue=change.data[0];delete change.data;
 }
}
const fixture={sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),seed:'0x10CA453',woffTemplate,cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-outline.json';
if(generate)await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw new Error('Outline corpus drift; inspect before --generate');
const report={node:process.version,scope:'TrueType constructor loca reads and glyf decompression, not full outline parity',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),seed:fixture.seed,cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length};
await fs.mkdir('results/font-outline',{recursive:true});await fs.writeFile('results/font-outline/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
export {construct,cases,base};
