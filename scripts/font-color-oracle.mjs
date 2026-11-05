// Runs the unchanged installed opentype constructor, never a runtime dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {colorCases,reorderColor,mutateColor} from '../crates/xterm-font-addons/tests/support/font_color_scenarios.mjs';
const args=process.argv.slice(2),generate=args.includes('--generate');
if(args.some(x=>x!=='--generate'&&!x.startsWith('--source=')))throw new Error('Use --source=PATH and/or --generate');
const source=args.find(x=>x.startsWith('--source='))?.slice(9)??'source_repo';
const require=createRequire(path.resolve(source,'package.json')),{parse}=require('opentype.js');
const hash=x=>createHash('sha256').update(x).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
const originals=Array.from({length:base.readUInt16BE(4)},(_,i)=>{
  const at=12+i*16,offset=base.readUInt32BE(at+8);
  return {tag:base.toString('latin1',at,at+4),data:base.subarray(offset,offset+base.readUInt32BE(at+12))};
});
const woffTemplate=Object.fromEntries(originals.map(({tag,data})=>[tag,[...deflateSync(data)]]));
function construct(c) {
  const records=originals.filter(r=>r.tag!==c.tag&&!c.remove?.includes(r.tag)).map(r=>({...r,data:Buffer.from(r.data)}));
  if(c.glyphCount!==undefined)records.find(r=>r.tag==='maxp').data.writeUInt16BE(c.glyphCount,4);
  for(const r of c.extraTables??[])records.push({tag:r.tag,data:Buffer.from(r.data)});
  if(c.data)records.push({tag:c.tag,data:Buffer.from(c.data)});
  const extra=c.duplicateTable;
  if(extra)records[extra.position==='first'?'unshift':'push']({tag:extra.tag,data:Buffer.alloc(0)});
  const woff=c.woff,start=woff?44:12,width=woff?20:16,header=Buffer.alloc(start+records.length*width),pieces=[header];
  if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}
  else {base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
  let offset=header.length;
  records.forEach(({tag,data},i)=>{
    const compressed=woff?deflateSync(data):data,payload=compressed.length<data.length?compressed:data,at=start+i*width;
    header.write(tag,at,4,'latin1');header.writeUInt32BE(offset,at+(woff?4:8));
    if(woff)header.writeUInt32BE(payload.length,at+8);
    header.writeUInt32BE(!woff&&i===records.length-1&&c.directoryLength!==undefined?c.directoryLength:data.length,at+12);
    pieces.push(payload);offset+=payload.length;
    if(woff&&data.length&&!originals.some(r=>r.tag===tag&&r.data.equals(data))) (c.woffPayloads??={})[tag]=[...compressed];
  });
  if(woff)header.writeUInt32BE(offset,8);
  if(extra){const at=start+(extra.position==='first'?0:records.length-1)*width;header.writeUInt32BE(0xfffffff0,at+(woff?4:8));header.writeUInt32BE(128,at+12);if(woff)header.writeUInt32BE(16,at+8);}
  return reorderColor(Buffer.concat(pieces),c);
}
const cases=[],warn=Object.getOwnPropertyDescriptor(console,'warn');
try {
 for(const c of colorCases()) {
  const bytes=construct(c),input=bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length),warnings=[];
  const sentinel={object:{},number:42,null:null,undefined:undefined}[c.throws];
  let mutated=false,outcome;
  Object.defineProperty(console,'warn',{configurable:true,value(message){warnings.push(message);if(c.mutation&&!mutated){mutated=true;mutateColor(new Uint8Array(input),c);}if(c.throws)throw sentinel;}});
  try{parse(input);outcome={status:'ok'};}
  catch(error){outcome=c.throws&&Object.is(error,sentinel)?{status:'warning'}:{status:'error',name:error.name,message:error.message};}
  cases.push({...c,inputSha256:hash(bytes),inputFnv32:[...bytes].reduce((h,b)=>Math.imul(h^b,16777619)>>>0,2166136261),warnings,outcome});
 }
} finally {if(warn)Object.defineProperty(console,'warn',warn);else delete console.warn;}
const dependency=require.resolve('opentype.js'),fixture={source:dependency.replaceAll('\\','/').split('/node_modules/').at(-1),sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),woffTemplate,cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-color.json';
if(generate)await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw new Error('Color constructor corpus drift; inspect before generating');
const report={node:process.version,scope:'CPAL/COLR constructor reads, warnings and callback mutation only; full font parity NOT PROVEN',sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,warningAborts:cases.filter(c=>c.outcome.status==='warning').length};
await fs.mkdir('results/font-color',{recursive:true});await fs.writeFile('results/font-color/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
