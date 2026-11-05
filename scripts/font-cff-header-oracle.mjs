// Unchanged opentype constructor baseline, not a product dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {cffHeaderInput,cffHeaderCases,mutateCffHeader} from '../crates/xterm-font-addons/tests/support/font_cff_header_input.mjs';
const source=process.argv.find(x=>x.startsWith('--source='))?.slice(9)??'source_repo';
const require=createRequire(path.resolve(source,'package.json')),{parse}=require('opentype.js');
const hash=x=>createHash('sha256').update(x).digest('hex');
const fonts={};for(const [key,name]of [['otf','FiraCode-Regular.otf'],['ttf','UbuntuMono-Regular.ttf']])fonts[key]=new Uint8Array(await fs.readFile(`crates/xterm-addons/tests/fixtures/ligature-fonts/${name}`));
const woffTemplates={};
for(const [key,base]of Object.entries(fonts)) {
  const v=new DataView(base.buffer,base.byteOffset,base.byteLength),encoded={};
  for(let i=0;i<v.getUint16(4);i++) {const at=12+i*16,tag=String.fromCharCode(...base.slice(at,at+4)),off=v.getUint32(at+8);encoded[tag]=[...deflateSync(base.slice(off,off+v.getUint32(at+12)))];}
  woffTemplates[key]=encoded;
}
const cases=[];
for(const c of cffHeaderCases()) {
  const base=fonts[c.base??'otf'],v=new DataView(base.buffer,base.byteOffset,base.byteLength),encoded={};
  const original=Array.from({length:v.getUint16(4)},(_,i)=>12+i*16).find(at=>String.fromCharCode(...base.slice(at,at+4))==='CFF ');
  const payload=c.data?Uint8Array.from(c.data):base.slice(v.getUint32(original+8),v.getUint32(original+8)+v.getUint32(original+12));
  if(c.header)payload.set(c.header);encoded[c.tag]=c.compressedPayload??[...deflateSync(c.inflated?Uint8Array.from(c.inflated):payload,c.compression)];
  if(c.other)encoded[c.other.tag]=[...deflateSync(Uint8Array.from(c.other.data))];
  const bytes=cffHeaderInput(base,c,{...woffTemplates[c.base??'otf'],...encoded}),initial=bytes.slice(),warnings=[];
  const sentinel={object:{},number:42,null:null,undefined:undefined}[c.throws];
  const warn=console.warn,error=console.error;
  let outcome;
  try {console.warn=(...args)=>{warnings.push(['warn',...args.map(String)]);if(c.mutation)mutateCffHeader(bytes,c);if(c.throws)throw sentinel;};console.error=(...args)=>warnings.push(['error',...args.map(String)]);parse(bytes.buffer);outcome={status:'ok'};}
  catch(e){outcome=c.throws&&Object.is(e,sentinel)?{status:'warning'}:{status:'error',name:e.name,message:e.message};}
  finally {console.warn=warn;console.error=error;}
  cases.push({...c,encoded:c.woff?encoded:undefined,inputSha256:hash(initial),inputFnv32:[...initial].reduce((h,b)=>Math.imul(h^b,16777619)>>>0,2166136261),warnings,outcome});
}
const fixture={woffTemplates,sourceSha256:hash(await fs.readFile(require.resolve('opentype.js'))),fontHashes:Object.fromEntries(Object.entries(fonts).map(([k,v])=>[k,hash(v)])),seed:'0x457cff01',cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-cff-header.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw new Error('CFF header source drift; inspect before generating');
const report={node:process.version,sourceSha256:fixture.sourceSha256,fontHashes:fixture.fontHashes,fixtureSha256:hash(text),seed:fixture.seed,cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,warningAborts:cases.filter(c=>c.outcome.status==='warning').length,errors:Object.fromEntries([...new Set(cases.filter(c=>c.outcome.status==='error').map(c=>c.outcome.message??'warning abort'))].map(m=>[m,cases.filter(c=>(c.outcome.message??'warning abort')===m).length])),scope:'Selected CFF/CFF2 loading and header errors only; full342 and equivalence NOT PROVEN'};
await fs.mkdir('results/font-cff-header',{recursive:true});await fs.writeFile('results/font-cff-header/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
