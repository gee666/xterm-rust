// Constructor recipes share existing browser/native byte builders, not parsers.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {buildHeaderFont,headerAddonCases} from '../crates/xterm-font-addons/tests/support/font_gsub_header_scenarios.mjs';
const corpus=process.env.GSUB_CORPUS??'lookup';
if(!['lookup','type9','null','refs','subtables','single'].includes(corpus))throw new Error('Unknown GSUB_CORPUS');
const prefix=`font-gsub-${corpus}`;
const generate=process.argv.includes('--generate');
if(process.argv.slice(2).some(arg=>arg!=='--generate'))throw new Error('Use --generate only');
const require=createRequire(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
const dependency=require.resolve('opentype.js'),{parse}=require('opentype.js');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const base=await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf');
const original=Array.from({length:base.readUInt16BE(4)},(_,i)=>{const at=12+i*16,start=base.readUInt32BE(at+8);return{tag:base.toString('latin1',at,at+4),data:base.subarray(start,start+base.readUInt32BE(at+12))};});
const woffTemplate=Object.fromEntries(original.map(({tag,data})=>[tag,[...deflateSync(data)]]));
// Independent Node container writer checks the reused browser and Rust builders.
function construct(change){
 const records=original.filter(({tag})=>tag!=='GSUB').concat((change.extraTables??[]).map(({tag,data})=>({tag,data:Buffer.from(data)})),{tag:'GSUB',data:Buffer.from(change.data)});
 const woff=change.woff,start=woff?44:12,width=woff?20:16,header=Buffer.alloc(start+records.length*width),pieces=[header];
 if(woff){header.write('wOFF');base.copy(header,4,0,4);header.writeUInt16BE(records.length,12);}else{base.copy(header,0,0,12);header.writeUInt16BE(records.length,4);}
 let cursor=header.length;
 for(const [i,{tag,data}]of records.entries()){
  const encoded=woff?deflateSync(data):data,payload=encoded.length<data.length?encoded:data,at=start+i*width;
  header.write(tag,at,4,'latin1');header.writeUInt32BE(cursor,at+(woff?4:8));if(woff)header.writeUInt32BE(payload.length,at+8);
  header.writeUInt32BE(data.length,at+12);pieces.push(payload);cursor+=payload.length;
 }
 if(woff)header.writeUInt32BE(cursor,8);return Buffer.concat(pieces);
}
const words=values=>{const d=Buffer.alloc(2*values.length);values.forEach((v,i)=>d.writeUInt16BE(v,i*2));return [...d];};
const cases=[];
if(corpus==='subtables')cases.push(...(await import('./font-gsub-subtables-recipes.mjs')).subtablesCases());
else if(corpus==='single')cases.push(...(await import('./font-gsub-single-recipes.mjs')).singleCases());
else if(corpus==='refs')cases.push(...(await import('./font-gsub-refs-recipes.mjs')).refsCases());
else if(corpus==='null')cases.push(...(await import('./font-gsub-null-recipes.mjs')).nullCases());
else if(corpus==='type9')cases.push(...(await import('./font-gsub-type9-recipes.mjs')).type9Cases());
else for(const woff of [false,true]){
 const add=(name,data,extra={})=>cases.push({name:name+(woff?' WOFF':' SFNT'),tag:'GSUB',data,woff,...extra});
 for(const kind of [0,10,11,65535])for(const flags of [0,16,65535])for(const count of [0,1,65535]){
  const data=words([1,0,0,0,10,1,4,kind,flags,count,2,0]);
  for(const length of [14,15,16,17,18,19,20,21,22,24])add(`invalid type${kind} flags${flags} count${count} EOF${length}`,data.slice(0,length));
 }
 for(const kind of [1,2,3,4,5,6,7,8])for(const flags of [0,16,17,65535]){
  const data=words([1,0,0,0,10,1,4,kind,flags,0,65535]);
  for(const length of [16,17,18,19,20,21,22])add(`empty type${kind} flags${flags} EOF${length}`,data.slice(0,length));
 }
 // Offset2 points at the snapshotted flags. Format16 rejects without reading
 // any later word, so missing markFilteringSet must not replace that error.
 for(const kind of [2,3,4,5,6,7,8])for(const flags of [16,17,65535]){
  const data=words([1,0,0,0,10,1,4,kind,flags,1,2,7]);
  for(const length of [20,21,22,23,24])add(`subtable before filter type${kind} flags${flags} EOF${length}`,data.slice(0,length));
 }
 for(const kind of [2,7]){
  const data=words([1,0,0,0,10,1,4,kind,16,1,2]);
  add(`subtable before later META type${kind}`,data,{extraTables:[{tag:'meta',data:[0,0,0,2]}]});
 }
 add('valid empty then invalid META',words([1,0,0,0,10,1,4,2,16,0,0]),{extraTables:[{tag:'meta',data:[0,0,0,2]}]});
}
for(const item of cases){
 const bytes=construct(item);item.inputSha256=hash(bytes);item.inputHash=bytes.reduce((h,b)=>Math.imul(h^b,0x01000193)>>>0,0x811c9dc5);
 if(item.woff)item.woffPayloads=Object.fromEntries([...(item.extraTables??[]),{tag:'GSUB',data:item.data}].map(({tag,data})=>[tag,[...deflateSync(Buffer.from(data))]]));
 assert.deepEqual(Buffer.from(buildHeaderFont(base,{woffTemplate},item)),bytes,item.name);
 try{parse(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length));item.outcome={status:'ok'};}
 catch(error){item.outcome={status:'error',name:error.name,message:error.message};}
}
const fixture={sourceSha256:hash(await fs.readFile(dependency)),fontSha256:hash(base),woffTemplate,cases};
async function check(file,value){const content=JSON.stringify(value)+'\n';if(generate)await fs.writeFile(file,content);else assert.equal(await fs.readFile(file,'utf8'),content,file+' drift');return hash(content);}
const baseline=spawnSync(process.execPath,['scripts/ligature-addon-oracle.mjs'],{encoding:'utf8',timeout:120000,env:process.env});
if(baseline.error||baseline.status!==0)throw new Error(baseline.error?.message??baseline.stderr);
const {LigaturesAddon}=await import(pathToFileURL(path.resolve('tmp/ligature-addon-oracle/LigaturesAddon.js')));
if(corpus!=='lookup'){
 const {loadBuffer}=await import(pathToFileURL(path.resolve('tmp/ligature-addon-oracle/fontLigatures/index.js')));
 for(const item of cases){const bytes=construct(item);try{loadBuffer(bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length));item.loadOutcome={status:'ok'};}catch(error){item.loadOutcome={status:'error',name:error.name,message:error.message};}}
}
const fixtureSha256=await check(`crates/xterm-addons/tests/fixtures/${prefix}.json`,fixture);
const addon=await headerAddonCases(o=>new LigaturesAddon(o),i=>new Uint8Array(construct(cases[i])),cases.length);
for(const [i,record]of addon.entries()){const accepted=(cases[i].loadOutcome??cases[i].outcome).status==='ok';assert.deepEqual(record,{loading:[[0,3]],ranges:accepted?[]:[[0,3]],reads:1,cacheReads:0,refreshes:accepted?1:0},cases[i].name);}
const addonSha256=await check(`crates/xterm-font-addons/tests/fixtures/${prefix}-addon.json`,addon);
const report={scope:`GSUB ${corpus} constructor reads only; full parity NOT PROVEN`,node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256,addonSha256,cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,addonCases:addon.length,...(corpus!=='lookup'?{loadAccepted:cases.filter(c=>c.loadOutcome.status==='ok').length,loadRejected:cases.filter(c=>c.loadOutcome.status==='error').length}:{}),errors:[...new Set(cases.filter(c=>c.outcome.status==='error').map(c=>c.outcome.name+': '+c.outcome.message))]};
await fs.mkdir(`results/${prefix}`,{recursive:true});await fs.writeFile(`results/${prefix}/oracle.json`,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
