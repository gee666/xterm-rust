// Use when reviewing528 before owned compiler/browser artifacts are removed.
// This audit starts no compiler, browser, server or source mutation.
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve,relative,isAbsolute} from 'node:path';
const root=process.cwd(),out='results/image-palette-values';
const original=process.env.ORIGINAL_REFERENCE;
if(!original)throw Error('Set ORIGINAL_REFERENCE to untouched source_repo');
const hash=b=>createHash('sha256').update(b).digest('hex');
const json=async p=>JSON.parse(await readFile(p,'utf8'));
const audit={status:'checking',source:[],comparator:[],artifacts:[],builds:[],originalCount:0,copiedOriginalCount:0};
async function check(file,expected){
 const bytes=await readFile(file);
 if(hash(bytes)!==expected)throw Error('Hash mismatch '+file);
 return bytes;
}
const control=await json(out+'/negative-audit.json');
if(control.status!=='passed-negative-control'||!control.restored||control.files.length!==11)throw Error('Incomplete control');
const old=new Map(control.files.map(e=>[e.file,e.baselineSha256]));
for(const e of control.files)await check(e.file,e.fixedSha256);
for(const phase of ['initial','negative','final','producer']){
 const report=await json(out+'/'+phase+'-browser.json');
 const verification=await json(out+'/'+phase+'-verification.json');
 const positive=phase!=='negative',expected=phase==='producer'?15:71;
 if(positive?(report.status!=='passed'||report.browsers.length!==3):
  (report.status!=='failed'||report.browsers.length!==1))throw Error('Incomplete '+phase);
 for(const b of report.browsers){
  if(b.tests.length!==expected||b.counts.passed!==(positive?expected:61)||
   b.counts.failed!==(positive?0:10)||b.counts.ignored||b.counts.filtered||b.errors.length)throw Error('Invalid '+phase+'/'+b.name);
 }
 for(const e of verification.rustInputs){
  const expected=phase==='negative'&&old.has(e.path)?old.get(e.path):e.sha256;
  if(expected!==e.sha256)throw Error('Wrong old production pin '+e.path);
  if(phase!=='negative'||!old.has(e.path))await check(e.path,e.sha256);
  audit.source.push({phase,...e});
 }
 for(const e of verification.sourceInputs){
  let checked=e.path;
  if(resolve(e.path)===resolve('tmp/image-canvas/image-comparators.ts')){
   // The shared runner overwrites its generated entry for each target. Retain
   // an exact reconstruction and require its historical pin before using it.
   const names=[...(phase==='producer'?[]:['ImageAddon']),'ImageRenderer','IIPHandler',
    'ImageStorage','IIPImageStorage',...(phase==='producer'?['SixelHandler','SixelImageStorage']:[])];
   const lines=names.map(name=>'export { '+name+' } from '+JSON.stringify(resolve('source_repo','addons/addon-image/src/'+name+'.ts'))+';');
   lines.push('export { Terminal } from '+JSON.stringify(resolve('source_repo','src/browser/public/Terminal.ts'))+';');
   checked=out+'/reconstructed-'+(phase==='producer'?'producer':'addon')+'-comparator.ts';
   await writeFile(checked,lines.join('\n'));
  }
  await check(checked,e.sha256);
  const local=relative(resolve('source_repo'),e.path);
  const copied=!isAbsolute(local)&&local!=='..'&&!local.startsWith('../')&&!local.startsWith('..\\');
  if(copied)await check(resolve(original,local),e.sha256);
  audit.comparator.push({phase,...e,checkedPath:checked,checkedAgainstOriginal:copied});
 }
 const test=report.testName,build=resolve(out,phase+'-build.log');
 const messages=(await readFile(build,'utf8')).split(/\r?\n/).filter(x=>x.startsWith('{')).map(x=>JSON.parse(x));
 const compiled=messages.find(x=>x.reason==='compiler-artifact'&&x.target.name===test&&x.profile.test);
 if(!compiled||compiled.fresh!==false)throw Error('No fresh compilation '+phase);
 audit.builds.push({phase,build,artifact:compiled});
 for(const kind of ['wasm','bindings']){
  const e=report.artifacts[kind];await check(e.path,e.sha256);audit.artifacts.push({phase,kind,...e});
 }
 const compiler=report.artifacts.compiler;
 await check(resolve(root,compiler.path),compiler.sha256);
 audit.artifacts.push({phase,kind:'compiler-pin',...compiler});
}
for(const e of await json('docs/baseline/reference-manifest.json')){
 await check(resolve(original,e.path),e.sha256);audit.originalCount++;
 await check(resolve('source_repo',e.path),e.sha256);audit.copiedOriginalCount++;
}
const final=await json(out+'/final-browser.json');
await check(final.artifacts.compiler.originalPath,final.artifacts.compiler.sha256);
const nativeLog=await readFile(out+'/native-sixel.log','utf8');
const nativeMatch=nativeLog.match(/Running unittests.*\((.*\.exe)\)/);
if(!nativeMatch)throw Error('No retained native executable');
const native=resolve(nativeMatch[1]),nativeBytes=await readFile(native);
audit.native={path:native,sha256:hash(nativeBytes),bytes:nativeBytes.length};
for(const name of ['native','format']){
 for(const c of await json(out+'/'+name+'-commands.json')){
  if(c.status!=='passed'&&c.name!=='fmt-all')throw Error('Failed gate '+c.name);
 }
}
const nativeSummaries=(await readFile(out+'/native-addons.log','utf8')).matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g);
audit.nativeCounts=[...nativeSummaries].map(m=>m.slice(1).map(Number)).reduce((a,b)=>a.map((v,i)=>v+b[i]),[0,0,0,0,0]);
if(JSON.stringify(audit.nativeCounts)!=='[209,0,0,0,0]')throw Error('Native counts changed');
for(const file of ['run.mjs','negative-control.mjs','audit.mjs']){
 const path=out+'/'+file;audit.source.push({path,sha256:hash(await readFile(path))});
}
audit.status='passed';
audit.note='71 directed functions each engine,61 inherited. Typed SIXEL15 each. Full image639/img01..48/raw pixel/private/GC/native OS remain open. O artifact paths checked before cleanup, not permanent caches.';
await writeFile(out+'/audit.json',JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify({status:audit.status,phasePins:audit.source.length,comparator:audit.comparator.length,artifacts:audit.artifacts.length,original:audit.originalCount,copiedOriginal:audit.copiedOriginalCount,native:audit.nativeCounts}));
