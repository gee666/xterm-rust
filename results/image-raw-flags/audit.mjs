// Use when reviewing485 before owned temporary artifacts are removed.
// No compiler, bundler, server or browser is started by this audit.
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve,relative,isAbsolute} from 'node:path';
const out='results/image-raw-flags', root=process.cwd();
const original=process.env.ORIGINAL_REFERENCE;
if(!original)throw Error('Set ORIGINAL_REFERENCE to untouched original source_repo');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const json=async path=>JSON.parse(await readFile(path,'utf8'));
const audit={status:'checking',source:[],comparator:[],artifacts:[],compilerBuilds:[],originalCount:0,copiedOriginalCount:0};
async function check(path,expected){if(hash(await readFile(path))!==expected)throw Error(`Hash mismatch ${path}`);}
const final=await json(`${out}/final-verification.json`);
for(const entry of final.rustInputs){await check(entry.path,entry.sha256);audit.source.push(entry);}
for(const entry of final.sourceInputs){
  await check(entry.path,entry.sha256);
  const local=relative(resolve(root,'source_repo'),entry.path);
  const referenceFile=!isAbsolute(local)&&local!=='..'&&!local.startsWith('..\\')&&!local.startsWith('../');
  if(referenceFile)await check(resolve(original,local),entry.sha256);
  audit.comparator.push({...entry,checkedAgainstOriginal:referenceFile});
}
for(const entry of await json('docs/baseline/reference-manifest.json')){
  await check(resolve(original,entry.path),entry.sha256);audit.originalCount++;
  await check(resolve(root,'source_repo',entry.path),entry.sha256);audit.copiedOriginalCount++;
}
const control=await json(`${out}/negative-audit.json`);
if(control.status!=='passed-negative-control'||!control.restored||control.files.length!==5)throw Error('Incomplete negative control');
for(const entry of control.files)await check(entry.file,entry.fixedSha256);
for(const name of ['initial','final']){
  const report=await json(`${out}/${name}-browser.json`);
  if(report.status!=='passed'||report.browsers.length!==3)throw Error(`Incomplete ${name} matrix`);
  for(const browser of report.browsers){
    const c=browser.counts;
    if(browser.tests.length!==59||c.passed!==59||c.failed||c.ignored||c.filtered||browser.errors.length)throw Error(`Invalid ${name}/${browser.name}`);
  }
}
for(const name of ['initial','negative','final']){
  const report=await json(`${out}/${name}-browser.json`);
  const buildPath=resolve(report.artifacts.bindings.path,'../../image_addon_browser-build.log');
  const build=await readFile(buildPath,'utf8');
  const compiled=build.split(/\r?\n/).filter(line=>line.startsWith('{')).map(line=>JSON.parse(line)).find(entry=>entry.reason==='compiler-artifact'&&entry.target.name==='image_addon_browser');
  if(!compiled||compiled.fresh!==false)throw Error(`No actual compilation for ${name}`);
  audit.compilerBuilds.push({suite:name,artifact:compiled});
  await check(resolve(root,report.artifacts.compiler.path),report.artifacts.compiler.sha256);
  for(const kind of ['wasm','bindings']){
    const entry=report.artifacts[kind];await check(entry.path,entry.sha256);
    audit.artifacts.push({kind:`${name}-${kind}`,...entry});
  }
}
const finalReport=await json(`${out}/final-browser.json`);
const compiler=finalReport.artifacts.compiler;
await check(compiler.originalPath,compiler.sha256);
audit.artifacts.push({kind:'final-compiler',path:compiler.originalPath,sha256:compiler.sha256});
for(const file of [`${out}/negative-control.mjs`,`${out}/audit.mjs`,'scripts/browser-artifact-server.mjs','scripts/browser-artifact-server.test.mjs'])audit.source.push({path:file,sha256:hash(await readFile(file))});
audit.status='passed';
audit.note='59 test functions per engine,54 inherited unchanged. Raw flags only, not full image parity. Temporary artifact paths checked before cleanup, not permanent installations.';
await writeFile(`${out}/audit.json`,JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify({status:audit.status,source:audit.source.length,comparator:audit.comparator.length,originalComparator:audit.comparator.filter(x=>x.checkedAgainstOriginal).length,original:audit.originalCount,copiedOriginal:audit.copiedOriginalCount,artifacts:audit.artifacts.length}));
