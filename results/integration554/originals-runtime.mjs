// Exercise the matching production package with unchanged decoration/serialize suites.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,cp,statfs} from 'node:fs/promises';
import {join,dirname} from 'node:path';
import {freemem} from 'node:os';
import {runCommand} from '../../scripts/quality-process.mjs';
import {sourceIdentity,verifyBuiltPackage} from '../../scripts/e2e/build-identity.mjs';
const root=process.cwd(),out=join(root,'results/integration554'),scratch=join(root,'tmp/554-system');
await mkdir(scratch,{recursive:true});
const source='P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo';
Object.assign(process.env,{TEMP:scratch,TMP:scratch,TMPDIR:scratch,CARGO_TARGET_DIR:join(root,'target'),CARGO_BUILD_JOBS:'2',
 CARGO:'C:/Users/user/.cargo/bin/cargo.exe',RUSTC:'C:/Users/user/.cargo/bin/rustc.exe',
 WASM_BINDGEN:'P:/workshop/Windows_Worshop/oira666/xterm-rust/tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe',
 SOURCE_ROOT:source,XTERM_SOURCE_ROOT:source,SOURCE_REPO:source,NODE_PATH:join(source,'node_modules'),
 PLAYWRIGHT_MODULE:join(source,'node_modules/playwright/index.mjs')});
const json=async f=>JSON.parse(await readFile(f,'utf8'));
const save=(name,value)=>writeFile(join(out,name),JSON.stringify(value,null,2)+'\n');
const input=await sourceIdentity(root),build=await verifyBuiltPackage(root,input),commands=[];
assert.equal(input.sha256,(await json(join(out,'production-input.json'))).sha256);
async function headroom(){const d=await statfs(root);const v={ramGiB:freemem()/1024**3,diskGiB:Number(d.bavail)*Number(d.bsize)/1024**3};assert(v.ramGiB>=8&&v.diskGiB>=10,JSON.stringify(v));return v;}
async function run(name,args,timeoutMs){const before=await headroom();const r=await runCommand(process.execPath,args,{cwd:root,env:process.env,log:join(out,name+'.log'),timeoutMs,resourceCheck:headroom});commands.push({name,before,...r});await save('originals-runtime-commands.json',commands);return r;}
const mode=process.argv[2]??'decoration';assert(['decoration','serialize'].includes(mode));
if(mode==='decoration'){
 const r=await run('original-decoration',['scripts/e2e-upstream.mjs','--suite','core','--grep','registerDecoration|decoration color overrides','--no-build','--timeout-ms','180000'],210000);
 const text=await readFile(join(out,'original-decoration.log'),'utf8');
 const dir=text.match(/Results (.+)/)?.[1]?.trim();assert(dir,'Actual runner output path missing');
 const report=await json(join(dir,'summary.json'));await save('original-decoration.json',report);
 const dest=join(out,'original-decoration');await mkdir(dest,{recursive:true});
 for(const file of ['summary.json','playwright.json','playwright.log'])await cp(join(dir,file),join(dest,file));
 assert.equal(r.status,'passed');assert.equal(report.status,'passed');
 for(const [key,count]of Object.entries({discovered:33,executed:27,passed:27,failed:0,skipped:6,unrun:0}))assert.equal(report.counts[key],count,key);
 assert.deepEqual(report.errors,[]);assert.deepEqual(report.serverErrors,[]);
}else{
 const r=await run('serialize-browser',['scripts/serialize-browser-check.mjs','--suite'],600000);
 const src=join(root,'results/serialize-browser'),dest=join(out,'serialize-browser');await mkdir(dest,{recursive:true});
 for(const file of ['verification.json',...['chromium','firefox','webkit'].flatMap(n=>[n+'-source.json',n+'-rust.json',n+'-terminal.png']),...['source-suite','rust-suite'].flatMap(n=>['configuration.cjs.txt','playwright.json','playwright.log'].map(f=>n+'/'+f))]){
  await mkdir(dirname(join(dest,file)),{recursive:true});await cp(join(src,file),join(dest,file));
 }
 const report=await json(join(src,'verification.json'));assert.equal(r.status,'passed');assert.equal(report.status,'passed');assert.deepEqual(report.cleanupErrors,[]);assert.deepEqual(report.serverErrors,[]);assert.equal(report.browsers.length,3);
 for(const b of report.browsers){assert.equal(b.status,'passed');for(const [k,n]of Object.entries({discovered:93,executed:93,passed:93,failed:0}))assert.equal(b[k],n);assert.deepEqual(b.mismatches,[]);}
 assert.equal(report.suites.length,2);for(const s of report.suites){assert.equal(s.result.status,'passed');assert.equal(s.counts.passed,120);for(const f of ['failed','skipped','unrun'])assert.equal(s.counts[f],0);}
}
assert.equal((await sourceIdentity(root)).sha256,input.sha256);assert.deepEqual(await verifyBuiltPackage(root,input),build);
await save('originals-runtime-'+mode+'-identity.json',{mode,input,build,status:'passed'});
console.log(mode+' matching production original suites passed');
