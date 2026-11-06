// Unchanged complete opentype.js constructors, never a production dependency.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {deflateSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import vm from 'node:vm';
import {cff2IndexInput,cff2IndexCases} from '../crates/xterm-font-addons/tests/support/font_cff2_index_input.mjs';
const require=createRequire(process.env.SOURCE_PACKAGE_JSON??path.resolve('source_repo/package.json')),{parse}=require('opentype.js');
const hash=b=>createHash('sha256').update(b).digest('hex'),base=new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf'));
const v=new DataView(base.buffer),templates={};
for(let i=0;i<v.getUint16(4);i++){const at=12+i*16,tag=String.fromCharCode(...base.slice(at,at+4)),offset=v.getUint32(at+8);templates[tag]=[...deflateSync(base.slice(offset,offset+v.getUint32(at+12)))];}
const cases=[],resources=[],recipes=cff2IndexCases(),worker=process.argv.indexOf('--worker');
if(worker>=0) {
  const c=recipes[Number(process.argv[worker+1])];
  const encoded=c.woff?{[c.tag]:c.data?[...deflateSync(Uint8Array.from(c.data))]:templates['CFF ']}:undefined;
  const bytes=cff2IndexInput(base,c,{...templates,...encoded}),warnings=[];
  console.warn=(...a)=>warnings.push(['warn',...a.map(String)]);console.error=(...a)=>warnings.push(['error',...a.map(String)]);
  const capture=fn=>{try{fn();return {status:'ok'};}catch(e){return {status:'error',name:e.name,message:e.message};}};
  const emit=(key,value)=>process.stdout.write(JSON.stringify({[key]:value})+'\n');
  if(c.data) {
    // In-memory test-only exports. Function bodies and original file stay intact.
    const source=await fs.readFile(require.resolve('opentype.js'),'utf8'),anchor='return __toCommonJS(opentype_exports);';
    if(source.split(anchor).length!==2)throw Error('Source export anchor drift');
    const helpers=vm.runInNewContext(source.replace(anchor,'return {parseCFFIndex,parseCFFTopDict};')+';opentype');
    const phase=(view,start)=>{
      const size=view.getUint8(start+2),length=view.getUint16(start+3);
      helpers.parseCFFIndex(view,start+size+length,null,2);
      const dict=new DataView(Uint8Array.from(Array.from({length},(_,i)=>view.getUint8(start+size+i))).buffer);
      const top=helpers.parseCFFTopDict(dict,0,undefined,2);
      if(!top.fdArray)throw Error('This is a CFF2 font, but FDArray information is missing');
      helpers.parseCFFIndex(view,start+top.fdArray,null,2);
    };
    const raw=Uint8Array.from(c.data),view=new DataView(bytes.buffer),woff=!!c.woff,count=view.getUint16(woff?12:4),first=woff?44:12,width=woff?20:16;
    const at=Array.from({length:count},(_,i)=>first+i*width).find(at=>String.fromCharCode(...bytes.slice(at,at+4))===c.tag);
    const compressed=woff&&view.getUint32(at+8)<view.getUint32(at+12);
    emit('phaseWhole',capture(()=>phase(compressed?new DataView(raw.buffer):view,compressed?0:view.getUint32(at+(woff?4:8)))));
    emit('outcome',capture(()=>parse(bytes.buffer)));
    emit('phaseTable',capture(()=>phase(new DataView(raw.buffer),0)));
  } else emit('outcome',capture(()=>parse(bytes.buffer)));
  emit('warnings',warnings);process.exit(0);
}
for(const [i,c] of recipes.entries()) {
  const encoded=c.woff?{[c.tag]:c.data?[...deflateSync(Uint8Array.from(c.data))]:templates['CFF ']}:undefined;
  const bytes=cff2IndexInput(base,c,{...templates,...encoded});
  const child=spawnSync(process.execPath,['--max-old-space-size=128',import.meta.filename,'--worker',String(i)],{encoding:'utf8',timeout:3000,maxBuffer:1024*1024,windowsHide:true});
  const raw=Object.assign({},...child.stdout.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line)));
  const resourceFailure=child.status===0?undefined:{status:'resourceFailure',kind:child.error?.code==='ETIMEDOUT'?'deadline':'heapLimit'};
  if(resourceFailure)resources.push({name:c.name,inputSha256:hash(bytes),exitCode:child.status,signal:child.signal,error:child.error?.code,stderrBase64:Buffer.from(child.stderr??'').toString('base64'),stderrSha256:hash(child.stderr??''),stdout:child.stdout});
  const outcome=raw.outcome??resourceFailure,warnings=raw.warnings??[];
  cases.push({...c,encoded,inputSha256:hash(bytes),inputFnv32:bytes.reduce((h,b)=>Math.imul(h^b,16777619)>>>0,2166136261),warnings,outcome,phaseWhole:raw.phaseWhole,phaseTable:raw.phaseTable,resourceFailure});
}
const fixture={woffTemplates:{otf:templates},sourceSha256:hash(await fs.readFile(require.resolve('opentype.js'))),fontSha256:hash(base),cases};
const text=JSON.stringify(fixture)+'\n',file='crates/xterm-addons/tests/fixtures/font-cff2-index.json';
if(process.argv.includes('--generate'))await fs.writeFile(file,text);else if(await fs.readFile(file,'utf8')!==text)throw Error('CFF2 INDEX source drift; inspect before generating');
const report={node:process.version,sourceSha256:fixture.sourceSha256,fontSha256:fixture.fontSha256,fixtureSha256:hash(text),seed:'0x533cff02',cases:cases.length,resourceFailures:cases.filter(c=>c.resourceFailure).length,phaseWholeComplete:cases.filter(c=>c.phaseWhole?.status==='error').length,phaseTableExecuted:cases.filter(c=>c.phaseTable).length,outcomes:Object.fromEntries([...new Set(cases.map(c=>c.outcome.message??c.outcome.status))].map(m=>[m,cases.filter(c=>(c.outcome.message??c.outcome.status)===m).length])),warnings:cases.filter(c=>c.warnings.length).length,scope:'FDArray INDEX only. Later font/private dictionaries, FDSelect and charstrings remain342. Full equivalence NOT PROVEN.'};
await fs.mkdir('results/font-cff2-index',{recursive:true});await fs.writeFile('results/font-cff2-index/oracle.json',JSON.stringify(report,null,2)+'\n');await fs.writeFile('results/font-cff2-index/resource-'+(process.argv.includes('--generate')?'generation':'replay')+'.json',JSON.stringify(resources,null,2)+'\n');console.log(JSON.stringify(report));
