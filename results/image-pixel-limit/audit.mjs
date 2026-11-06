// Use when checking scoped source/reference/runtime identities without rebuilding.
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {resolve,extname} from 'node:path';
const root=resolve('.'),out=resolve('results/image-pixel-limit');
const git=(...args)=>execFileSync(process.env.GIT??'git',args,{cwd:root,timeout:30000,maxBuffer:16*1024**2});
const hash=b=>createHash('sha256').update(b).digest('hex');
const textExt=new Set(['.rs','.js','.mjs','.json','.jsonl','.toml','.css','.html','.md','.txt','.ts','.ndjson']);
const normalized=(p,b)=>textExt.has(extname(p))?Buffer.from(b.toString('utf8').replaceAll('\r\n','\n')):b;
const source=git('ls-files','-z','--','Cargo.toml','Cargo.lock','.cargo','crates','browser','scripts').toString().split('\0').filter(Boolean);
const pins=[];
for(const path of source){const b=await readFile(path);pins.push({path,sha256:hash(b),normalizedSHA:hash(normalized(path,b))});}
const references=[];
const manifest=JSON.parse(await readFile('docs/baseline/reference-manifest.json','utf8'));
for(const base of ['source_repo','P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo']){
 for(const entry of manifest){if(hash(await readFile(resolve(base,entry.path)))!==entry.sha256)throw Error('Reference changed '+base+'/'+entry.path);}
 references.push({base,files:manifest.length,passed:true});
}
const native=[];
for(const file of await readdir('target/debug/deps')){
 if(/^(image_iip_pixel_phases|image_iip|kitty_geometry)-.*\.exe$/.test(file)){
  const path='target/debug/deps/'+file;native.push({path,sha256:hash(await readFile(path))});
 }
}
if(native.length!==3)throw Error('Expected three retained focused native executables');
const path=resolve(out,'input-pins.json');
if(process.argv[2]==='--write'){
 await writeFile(path,JSON.stringify({head:git('rev-parse','HEAD').toString().trim(),sources:pins,references,native},null,2)+'\n');
}else{
 const before=JSON.parse(await readFile(path,'utf8'));
 if(JSON.stringify(before.sources)!==JSON.stringify(pins))throw Error('Source inputs changed since pin');
 if(JSON.stringify(before.native)!==JSON.stringify(native))throw Error('Native executable identities changed since pin');
}
const browsers=[];
for(const name of ['initial','final','control','context-control','iip','sixel','kitty']){
 let report;try{report=JSON.parse(await readFile(resolve(out,name+'-browser.json'),'utf8'));}catch(e){if(e.code==='ENOENT')continue;throw e;}
 if(!report.artifacts)throw Error('Missing runtime artifact identities '+name);
 for(const key of ['wasm','bindings']){
  const p=report.artifacts[key];if(hash(await readFile(p.path))!==p.sha256)throw Error('Runtime artifact mismatch '+name+' '+key);
 }
 for(const key of ['compiler']){
  const p=report.artifacts[key];if(hash(await readFile(resolve(root,p.path)))!==p.sha256)throw Error('Compiler artifact mismatch '+name);
 }
 const verification=JSON.parse(await readFile(resolve(out,name+'-verification.json'),'utf8'));
 const baseline=new Map();
 if(['control','context-control'].includes(name)){
  const restore=JSON.parse(await readFile(resolve(out,name+'-restore.json'),'utf8'));
  for(const item of restore.pins){
   if(hash(git('show',restore.base+':'+item.file))!==item.baseSHA ||
      hash(await readFile(item.file))!==item.fixedSHA || !item.restored)
    throw Error('Invalid immutable control/restore identity '+name+' '+item.file);
   baseline.set(item.file,item.baseSHA);
  }
 }
 for(const pin of verification.rustInputs){
  const expected=baseline.get(pin.path)??hash(await readFile(pin.path));
  if(pin.sha256!==expected)throw Error('Runtime Rust input mismatch '+name+' '+pin.path);
 }
 for(const pin of verification.sourceInputs){
  const path=pin.path===resolve('tmp/image-canvas/image-comparators.ts')?resolve(out,name+'-comparators.ts'):pin.path;
  if(hash(await readFile(path))!==pin.sha256)throw Error('Comparator source input mismatch '+name+' '+path);
 }
 const log=resolve(out,name+'-build.log');
 const messages=(await readFile(log,'utf8')).split(/\r?\n/).filter(l=>l.startsWith('{')).map(l=>JSON.parse(l));
 const compiled=messages.find(m=>m.reason==='compiler-artifact'&&m.target.name===report.testName&&m.profile.test);
 if(!compiled || compiled.fresh!==false)throw Error('Missing actual fresh=false compile '+name);
 const prelude=resolve(out,name+'-source-renderer.js');
 browsers.push({name,status:report.status,rustPins:verification.rustInputs.length,comparatorPins:verification.sourceInputs.length,
  fresh:compiled.fresh,buildLog:{path:log,sha256:hash(await readFile(log))},prelude:{path:prelude,sha256:hash(await readFile(prelude))},
  browsers:report.browsers.map(b=>({name:b.name,version:b.version,status:b.status,counts:b.counts,pageErrors:b.errors.length})),artifacts:report.artifacts});
}
const result={sources:pins.length,references,browsers};
if(['--write','--report'].includes(process.argv[2])) {
 await writeFile(resolve(out,'identity-audit.json'),JSON.stringify(result,null,2)+'\n');
} else {
 const saved=JSON.parse(await readFile(resolve(out,'identity-audit.json'),'utf8'));
 if(JSON.stringify(saved)!==JSON.stringify(result))throw Error('Retained identity report differs from current inputs');
}
console.log(JSON.stringify({sources:result.sources,references,browserPhases:browsers.length}));
