// The unchanged installed opentype constructor is the oracle, never a runtime dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {fvarCases} from '../crates/xterm-font-addons/tests/support/font_fvar_scenarios.mjs';
const args=process.argv.slice(2),generate=args.includes('--generate');
const source=args.find(arg=>arg.startsWith('--source='))?.slice(9)??'source_repo';
if(args.some(arg=>arg!=='--generate'&&!arg.startsWith('--source=')))throw new Error('Use --source=PATH and/or --generate');
const require=createRequire(path.resolve(source,'package.json')), {parse}=require('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
// This independent encoder deliberately mirrors the shared Rust fixture decoder.
// Both must reconstruct descriptors from the reference, not encode expected outcomes.
function construct(change) {
  const records=[];
  for(let i=0;i<base.readUInt16BE(4);i++) {
    const at=12+i*16,tag=base.toString('latin1',at,at+4),start=base.readUInt32BE(at+8);
    if(tag===change.tag||change.remove?.includes(tag))continue;
    const data=Buffer.from(base.subarray(start,start+base.readUInt32BE(at+12)));
    if(tag==='maxp'&&change.glyphCount!==undefined)data.writeUInt16BE(change.glyphCount,4);
    records.push({tag,data});
  }
  if(change.data)records.push({tag:change.tag,data:Buffer.from(change.data)});
  const extra=change.duplicateTable;
  if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:extra.invalidCompression?Buffer.from(Array.from({length:16},(_,i)=>i+1)):Buffer.alloc(0)});
  const woff=change.woff,header=Buffer.alloc((woff?44:12)+records.length*(woff?20:16)),pieces=[header];
  if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}
  else {base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
  let cursor=header.length;
  records.forEach(({tag,data},index)=>{
    const compressed=woff?deflateSync(data):data,payload=woff&&compressed.length<data.length?compressed:data,at=(woff?44:12)+index*(woff?20:16);
    header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+(woff?4:8));
    if(woff)header.writeUInt32BE(payload.length,at+8);
    header.writeUInt32BE(index+1===records.length&&!woff&&change.directoryLength!==undefined?change.directoryLength:data.length,at+12);
    pieces.push(payload);cursor+=payload.length;
  });
  if(woff)header.writeUInt32BE(cursor,8);
  if(extra){const index=extra.position==='first'?0:records.length-1,at=(woff?44:12)+index*(woff?20:16);
    if(!extra.invalidCompression)header.writeUInt32BE(0xfffffff0,at+(woff?4:8));
    header.writeUInt32BE(128,at+12);if(woff)header.writeUInt32BE(16,at+8);
  }
  return Buffer.concat(pieces);
}
const cases=[];
for(const change of fvarCases()) {
  const bytes=construct(change);let outcome;
  try{parse(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length));outcome={status:'ok'};}
  catch(error){outcome={status:'error',name:error.name,message:error.message};}
  cases.push({...change,outcome});
}
const dependency=require.resolve('opentype.js');
const fixture={source:dependency.replaceAll('\\','/').split('/node_modules/').at(-1),sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),cases};
const content=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-fvar.json';
if(generate)await fs.writeFile(file,content);else if(await fs.readFile(file,'utf8')!==content)throw new Error('fvar fixture drift');
const report={node:process.version,scope:'fvar constructor acceptance and error order only, not full font parity',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(content),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status!=='ok').length};
await fs.mkdir('results/font-fvar',{recursive:true});await fs.writeFile('results/font-fvar/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
