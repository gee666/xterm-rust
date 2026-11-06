// Full unchanged constructors in bounded children. In-memory exports are test-only.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import vm from 'node:vm';
import {fdSelectInput,fdSelectCases} from '../crates/xterm-font-addons/tests/support/font_cff_fdselect_input.mjs';
const require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.resolve('source_repo/package.json'));
const {parse}=require('opentype.js'),source=await fs.readFile(require.resolve('opentype.js'),'utf8');
const hash=b=>createHash('sha256').update(b).digest('hex');
if(hash(source)!=='737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484')throw Error('Reference drift');
const base=new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf'));
const v=new DataView(base.buffer),templates={};
for(let i=0;i<v.getUint16(4);i++){const at=12+i*16,tag=String.fromCharCode(...base.slice(at,at+4)),offset=v.getUint32(at+8);templates[tag]=[...deflateSync(base.slice(offset,offset+v.getUint32(at+12)))];}
const recipes=fdSelectCases(base);
function input(c) {
  const encoded=c.woff?{[c.tag]:c.data?[...deflateSync(Uint8Array.from(c.data))]:templates['CFF ']}:undefined;
  return {encoded,bytes:fdSelectInput(base,c,{...templates,...encoded})};
}
const worker=process.argv.indexOf('--worker');
if(worker>=0) {
  const c=recipes[Number(process.argv[worker+1])],{bytes}=input(c),warnings=[];
  console.warn=(...a)=>warnings.push(['warn',...a.map(String)]);console.error=(...a)=>warnings.push(['error',...a.map(String)]);
  const capture=fn=>{try{fn();return {status:'ok'};}catch(e){return {status:'error',name:e.name,message:e.message};}};
  const emit=(key,value)=>process.stdout.write(JSON.stringify({[key]:value})+'\n');
  if(c.data) {
    const anchor='return __toCommonJS(opentype_exports);';
    if(source.split(anchor).length!==2)throw Error('Source export anchor drift');
    const helpers=vm.runInNewContext(source.replace(anchor,'return {parseCFFIndex,parseCFFTopDict,gatherCFF2FontDicts,parseCFFFDSelect};')+';opentype');
    const phase=(view,start)=>{
      const size=view.getUint8(start+2),length=view.getUint16(start+3);
      helpers.parseCFFIndex(view,start+size+length,null,2);
      const dict=new DataView(Uint8Array.from(Array.from({length},(_,i)=>view.getUint8(start+size+i))).buffer);
      const top=helpers.parseCFFTopDict(dict,0,undefined,2);
      if(!top.fdArray)throw Error('This is a CFF2 font, but FDArray information is missing');
      const fd=helpers.parseCFFIndex(view,start+top.fdArray,null,2);
      helpers.gatherCFF2FontDicts(view,start,fd.objects);
      if(top.fdSelect)helpers.parseCFFFDSelect(view,start+top.fdSelect,c.remove?.includes('maxp')?undefined:1629,fd.objects.length,2);
    };
    emit('phaseTable',capture(()=>phase(new DataView(Uint8Array.from(c.data).buffer),0)));
  }
  emit('outcome',capture(()=>parse(bytes.buffer)));emit('warnings',warnings);process.exit(0);
}
const cases=[];
for(const [i,c] of recipes.entries()) {
  const {bytes,encoded}=input(c);process.stdout.write(`${i+1}/${recipes.length} ${c.name}\n`);
  const child=spawnSync(process.execPath,['--max-old-space-size=128',import.meta.filename,'--worker',String(i)],{encoding:'utf8',timeout:3000,maxBuffer:1024*1024,windowsHide:true});
  if(child.status!==0||child.error) {
    await fs.mkdir('results/font-cff-fdselect',{recursive:true});
    await fs.writeFile(`results/font-cff-fdselect/worker-failure-${c.name}.json`,JSON.stringify({recipe:c,inputSha256:hash(bytes),inputBase64:Buffer.from(bytes).toString('base64'),exitCode:child.status,signal:child.signal,error:child.error?.code,stdoutBase64:Buffer.from(child.stdout??'').toString('base64'),stderrBase64:Buffer.from(child.stderr??'').toString('base64')},null,2)+'\n');
    throw Error(`Unexpected source worker failure ${c.name}: ${child.error??''} ${child.stderr}`);
  }
  const raw=Object.assign({},...child.stdout.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line)));
  cases.push({...c,encoded,inputSha256:hash(bytes),inputFnv32:bytes.reduce((h,b)=>Math.imul(h^b,16777619)>>>0,2166136261),...raw});
}
const fixture={woffTemplates:{otf:templates},sourceSha256:hash(source),fontSha256:hash(base),cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-cff-fdselect.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw Error('FDSelect source drift; inspect before generating');
const report={node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),cases:cases.length,outcomes:Object.fromEntries([...new Set(cases.map(c=>c.outcome.message??c.outcome.status))].map(m=>[m,cases.filter(c=>(c.outcome.message??c.outcome.status)===m).length])),scope:'Eager CFF2 FDSelect only; later charstrings/CFF1/full342 and equivalence NOT PROVEN.'};
await fs.mkdir('results/font-cff-fdselect',{recursive:true});await fs.writeFile('results/font-cff-fdselect/oracle.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
