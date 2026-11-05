// The unchanged installed opentype constructor is the behavioral oracle.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {avarCases} from '../crates/xterm-font-addons/tests/support/font_avar_scenarios.mjs';
const args=process.argv.slice(2),generate=args.includes('--generate');
const source=args.find(arg=>arg.startsWith('--source='))?.slice(9)??'source_repo';
if(args.some(arg=>arg!=='--generate'&&!arg.startsWith('--source=')))throw new Error('Use --source=PATH and/or --generate');
const require=createRequire(path.resolve(source,'package.json')),{parse}=require('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
function construct(change) {
  const records=[];
  for(let i=0;i<base.readUInt16BE(4);i++) {
    const at=12+i*16,tag=base.toString('latin1',at,at+4),start=base.readUInt32BE(at+8);
    if(tag===change.tag||change.remove?.includes(tag))continue;
    records.push({tag,data:Buffer.from(base.subarray(start,start+base.readUInt32BE(at+12)))});
  }
  records.push(...(change.extraTables??[]).map(({tag,data})=>({tag,data:Buffer.from(data)})));
  records.push({tag:change.tag,data:Buffer.from(change.data)});
  const extra=change.duplicateTable;
  if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:Buffer.alloc(0)});
  const woff=change.woff,start=woff?44:12,width=woff?20:16,header=Buffer.alloc(start+records.length*width),pieces=[header];
  if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}
  else{base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
  let cursor=header.length;
  records.forEach(({tag,data},index)=>{
    const at=start+index*width,compressed=woff?deflateSync(data):data,payload=compressed.length<data.length?compressed:data;
    header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+(woff?4:8));
    if(woff)header.writeUInt32BE(payload.length,at+8);
    header.writeUInt32BE(index+1===records.length?(change.directoryLength??data.length):data.length,at+12);
    pieces.push(payload);cursor+=payload.length;
  });
  if(woff)header.writeUInt32BE(cursor,8);
  const bytes=Buffer.concat(pieces);
  if(extra){const at=start+(extra.position==='first'?0:records.length-1)*width;bytes.writeUInt32BE(0xfffffff0,at+(woff?4:8));bytes.writeUInt32BE(128,at+12);if(woff)bytes.writeUInt32BE(16,at+8);}
  return bytes;
}
function mutate(bytes,item) {
  if(!item)return;
  const woff=bytes.toString('latin1',0,4)==='wOFF',start=woff?44:12,width=woff?20:16;
  for(let i=0;i<bytes.readUInt16BE(woff?12:4);i++){
    const at=start+i*width;if(bytes.toString('latin1',at,at+4)!==item.tag)continue;
    if(item.descriptor)bytes.writeUInt32BE(0xfffffff0,at+(woff?4:8));
    else bytes.writeUInt16BE(item.value,bytes.readUInt32BE(at+(woff?4:8))+item.at);
  }
}
const cases=[],descriptor=Object.getOwnPropertyDescriptor(console,'warn');
try {
  for(const change of avarCases()) {
    const input=construct(change),buffer=input.buffer.slice(input.byteOffset,input.byteOffset+input.length),bytes=Buffer.from(buffer),warnings=[],sentinel=change.throws==='undefined'?undefined:change.throws==='null'?null:change.throws==='number'?17:{},initialHash=hash(bytes);let outcome;
    Object.defineProperty(console,'warn',{configurable:true,value(message){warnings.push(message);mutate(bytes,change.mutation);if(change.throws)throw sentinel;}});
    try{parse(buffer);outcome={status:'ok'};}
    catch(error){outcome=error===sentinel?{status:'warning'}:{status:'error',name:error.name,message:error.message};}
    cases.push({...change,inputSha256:initialHash,warnings,outcome});
  }
} finally {if(descriptor)Object.defineProperty(console,'warn',descriptor);else delete console.warn;}
const dependency=require.resolve('opentype.js');
const fixture={sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),cases};
const content=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-avar.json';
if(generate)await fs.writeFile(file,content);else if(await fs.readFile(file,'utf8')!==content)throw new Error('avar fixture drift');
const report={node:process.version,scope:'avar constructor rejection and warning order only',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(content),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,warningAborts:cases.filter(c=>c.outcome.status==='warning').length};
await fs.mkdir('results/font-avar',{recursive:true});await fs.writeFile('results/font-avar/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
