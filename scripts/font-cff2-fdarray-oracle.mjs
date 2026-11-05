// Unchanged complete opentype.js constructors, never a production dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {cff2FdArrayInput,cff2FdArrayCases,missingFdArray} from '../crates/xterm-font-addons/tests/support/font_cff2_fdarray_input.mjs';
const require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.resolve('source_repo/package.json')),{parse}=require('opentype.js');
const hash=b=>createHash('sha256').update(b).digest('hex'),base=new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf'));
const v=new DataView(base.buffer),templates={};
for(let i=0;i<v.getUint16(4);i++){const at=12+i*16,tag=String.fromCharCode(...base.slice(at,at+4)),offset=v.getUint32(at+8);templates[tag]=[...deflateSync(base.slice(offset,offset+v.getUint32(at+12)))];}
const cases=[];
for(const c of cff2FdArrayCases()) {
  const encoded=c.woff?{[c.tag]:c.data?[...deflateSync(Uint8Array.from(c.data))]:templates['CFF ']}:undefined;
  const bytes=cff2FdArrayInput(base,c,{...templates,...encoded}),warnings=[],warn=console.warn,error=console.error;let outcome;
  try{console.warn=(...a)=>warnings.push(['warn',...a.map(String)]);console.error=(...a)=>warnings.push(['error',...a.map(String)]);parse(bytes.buffer);outcome={status:'ok'};}
  catch(e){outcome={status:'error',name:e.name,message:e.message};}finally{console.warn=warn;console.error=error;}
  const known=[missingFdArray,'Offset is outside the bounds of the DataView','Invalid b0 31','Invalid b0 255',"Cannot read properties of undefined (reading 'compression')"];
  if(warnings.length||outcome.status==='error'&&!/^Object \[object Object\] already has key \d+$/.test(outcome.message)&&!known.includes(outcome.message))throw Error(JSON.stringify({name:c.name,outcome,warnings}));
  if(c.presenceOnly&&outcome.message===missingFdArray)throw Error(`Truthy FDArray unexpectedly missing ${c.name}`);
  cases.push({...c,encoded,inputSha256:hash(bytes),inputFnv32:bytes.reduce((h,b)=>Math.imul(h^b,16777619)>>>0,2166136261),warnings,outcome});
}
const fixture={woffTemplates:{otf:templates},sourceSha256:hash(await fs.readFile(require.resolve('opentype.js'))),fontSha256:hash(base),cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-cff2-fdarray.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw Error('CFF2 FDArray source drift; inspect before generating');
const report={node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),seed:'0x519cff02',cases:cases.length,exactConstructors:cases.filter(c=>!c.presenceOnly).length,presenceOnly:cases.filter(c=>c.presenceOnly).length,outcomes:Object.fromEntries([...new Set(cases.map(c=>c.outcome.message??'ok'))].map(m=>[m,cases.filter(c=>(c.outcome.message??'ok')===m).length])),scope:'Missing FDArray rejection only. Truthy cases compare only this guard, not later INDEX equivalence. Full342/equivalence NOT PROVEN.'};
await fs.mkdir('results/font-cff2-fdarray',{recursive:true});await fs.writeFile('results/font-cff2-fdarray/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
