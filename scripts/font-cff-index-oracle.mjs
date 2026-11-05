// Unchanged full opentype.js constructor baseline, not a product dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync,deflateRawSync,constants} from 'node:zlib';
import {cffIndexCase,cffIndexInput,cffIndexCases} from '../crates/xterm-font-addons/tests/support/font_cff_index_input.mjs';
const source=process.argv.find(x=>x.startsWith('--source='))?.slice(9)??'source_repo';
const require=createRequire(path.resolve(source,'package.json')),{parse}=require('opentype.js');
const hash=x=>createHash('sha256').update(x).digest('hex');
const base=new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf'));
const v=new DataView(base.buffer),templates={};
for(let i=0;i<v.getUint16(4);i++){const at=12+i*16,tag=String.fromCharCode(...base.slice(at,at+4)),off=v.getUint32(at+8);templates[tag]=[...deflateSync(base.slice(off,off+v.getUint32(at+12)))];}
const cases=[];
for(const c of cffIndexCases()) {
  const expanded=cffIndexCase(base,c),encoded=c.woff?{[c.tag]:[...deflateSync(Uint8Array.from(expanded.data))]}:undefined;
  const bytes=cffIndexInput(base,c,{...templates,...encoded}),warnings=[],warn=console.warn,error=console.error;
  let outcome;
  try{console.warn=(...args)=>warnings.push(['warn',...args.map(String)]);console.error=(...args)=>warnings.push(['error',...args.map(String)]);parse(bytes.buffer);outcome={status:'ok'};}
  catch(e){outcome={status:'error',name:e.name,message:e.message};}
  finally{console.warn=warn;console.error=error;}
  if(warnings.length||outcome.status==='error'&&(outcome.name!=='RangeError'||outcome.message!=='Offset is outside the bounds of the DataView'))throw new Error(`Unexpected non-INDEX outcome ${c.name}: ${JSON.stringify({outcome,warnings})}`);
  cases.push({...c,encoded,inputSha256:hash(bytes),inputFnv32:[...bytes].reduce((h,b)=>Math.imul(h^b,16777619)>>>0,2166136261),warnings,outcome});
}
const fixture={woffTemplates:{otf:templates},sourceSha256:hash(await fs.readFile(require.resolve('opentype.js'))),fontSha256:hash(base),seed:'0x472cff01',cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-cff-index.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw new Error('CFF INDEX source drift; inspect before generating');
const windowFixtures={};
const pattern=Buffer.alloc(70000);for(let i=0;i<pattern.length;i++)pattern[i]=i%256;
for(const [name,options]of [['stored',{level:0}],['dynamic',{level:9}],['fixed',{level:9,strategy:constants.Z_FIXED}]]) {
  const bytes=deflateRawSync(pattern,options),file=`crates/xterm-addons/tests/fixtures/cff-window-${name}.deflate`;
  if(process.argv.includes('--generate'))await fs.writeFile(file,bytes);else if(!(await fs.readFile(file)).equals(bytes))throw new Error(`Window fixture drift: ${file}`);
  windowFixtures[name]={bytes:bytes.length,sha256:hash(bytes)};
}
const report={node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),seed:fixture.seed,cases:cases.length,accepted:cases.filter(c=>c.outcome.status==='ok').length,rejected:cases.filter(c=>c.outcome.status==='error').length,windowFixtures,scope:'Initial CFF INDEX bounds only; full342 and overall equivalence NOT PROVEN'};
await fs.mkdir('results/font-cff-index',{recursive:true});await fs.writeFile('results/font-cff-index/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
