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
for(const name of ['initial','final','control','iip','sixel','kitty']){
 let report;try{report=JSON.parse(await readFile(resolve(out,name+'-browser.json'),'utf8'));}catch(e){if(e.code==='ENOENT')continue;throw e;}
 if(!report.artifacts)throw Error('Missing runtime artifact identities '+name);
 for(const key of ['wasm','bindings']){
  const p=report.artifacts[key];if(hash(await readFile(p.path))!==p.sha256)throw Error('Runtime artifact mismatch '+name+' '+key);
 }
 for(const key of ['compiler']){
  const p=report.artifacts[key];if(hash(await readFile(resolve(root,p.path)))!==p.sha256)throw Error('Compiler artifact mismatch '+name);
 }
 browsers.push({name,status:report.status,browsers:report.browsers.map(b=>({name:b.name,version:b.version,status:b.status,counts:b.counts,pageErrors:b.errors.length})),artifacts:report.artifacts});
}
const result={sources:pins.length,references,browsers};
await writeFile(resolve(out,'identity-audit.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({sources:result.sources,references,browserPhases:browsers.length}));
