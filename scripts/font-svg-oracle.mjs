// Independent SVG constructor inputs for unchanged opentype.js. The Rust tests
// reconstruct the same descriptors with the existing metadata corpus builder.
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
const original=Array.from({length:base.readUInt16BE(4)},(_,i)=>{
 const at=12+i*16,start=base.readUInt32BE(at+8),length=base.readUInt32BE(at+12);
 return {tag:base.toString('latin1',at,at+4),data:base.subarray(start,start+length)};
});
function construct(change) {
 const records=original.filter(record=>!change.remove?.includes(record.tag)).map(record=>({...record}));
 records.push({tag:'SVG ',data:Buffer.from(change.data)});
 const extra=change.duplicateTable;
 if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:Buffer.alloc(0)});
 const woff=change.woff,start=woff?44:12,width=woff?20:16;
 const header=Buffer.alloc(start+records.length*width),pieces=[header];
 if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}
 else {base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
 let cursor=header.length;
 records.forEach(({tag,data},index)=>{
  const at=start+index*width,compressed=woff?deflateSync(data):data;
  const payload=compressed.length<data.length?compressed:data;
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
function svg({version=0,list=10,records=[],size=64}={}) {
 const data=Buffer.alloc(size);data.writeUInt16BE(version);data.writeUInt32BE(list,2);
 if(list+2<=size){data.writeUInt16BE(records.length,list);records.forEach(([first,last,offset,length],i)=>{const at=list+2+i*12;data.writeUInt16BE(first,at);data.writeUInt16BE(last,at+2);data.writeUInt32BE(offset,at+4);data.writeUInt32BE(length,at+8);});}
 return [...data];
}
const cases=[];
function add(name,data,extra={}){cases.push({name,tag:'SVG ',data,...extra});}
for(const woff of [false,true]) {
 const suffix=woff?' WOFF':' SFNT',valid=svg({records:[[0,0,14,4]],size:28});
 for(let size=0;size<=28;size++)add('SVG EOF '+size+suffix,valid.slice(0,size),{woff});
 for(const version of [0,1,2,65535])for(const size of [0,1,2,3,5,6,10])add(`SVG version ${version} bytes ${size}${suffix}`,svg({version}).slice(0,size),{woff});
 for(const list of [0,2,4,6,8,10,63,64,65535,0xffffffff])add(`SVG list offset ${list}${suffix}`,svg({list}),{woff});
 for(const [offset,length] of [[14,0],[54,0],[55,0],[0xffffffff,0],[14,4],[14,5],[0,2],[14,0xffffffff]]) {
  add(`SVG doc offset ${offset} length ${length}${suffix}`,svg({records:[[0,0,offset,length]]}),{woff});
  add(`SVG reversed glyph doc offset ${offset} length ${length}${suffix}`,svg({records:[[2,1,offset,length]]}),{woff});
 }
 add('SVG cached offset ignores later length'+suffix,svg({records:[[0,1,26,4],[2,3,26,0xffffffff]],size:40}),{woff});
 add('SVG distinct offset reads later length'+suffix,svg({records:[[0,1,26,4],[2,3,27,0xffffffff]],size:40}),{woff});
 for(const position of ['first','last'])add(`SVG invalid ${position} descriptor${suffix}`,valid,{woff,duplicateTable:{tag:'SVG ',position}});
 add('SVG eager bounds precede missing name'+suffix,[0],{woff,remove:['name']});
 add('SVG eager bounds precede missing hmtx'+suffix,[0],{woff,remove:['hmtx']});
 add('SVG unsupported version reaches missing name'+suffix,[0,1],{woff,remove:['name']});
}
add('SVG ignored directory length',svg({records:[[0,1,14,4]],size:28}),{directoryLength:0});
// A real compressed table exercises decompression before version/read checks.
add('SVG compressed valid zero records',svg({size:1024}),{woff:true});
add('SVG compressed invalid document offset',svg({records:[[0,0,0xffffffff,4]],size:1024}),{woff:true});
for(const change of cases){try{const bytes=construct(change);parse(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length));change.outcome={status:'ok'};}catch(error){change.outcome={status:'error',name:error.name,message:error.message};}}
const fixture={source:dependency.replaceAll('\\','/').split('/node_modules/').at(-1),sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-svg.json';
if(generate)await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw new Error('SVG corpus drift; inspect before --generate');
const report={node:process.version,scope:'SVG constructor reads only, not full font parity',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length};
await fs.mkdir('results/font-svg',{recursive:true});await fs.writeFile('results/font-svg/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
