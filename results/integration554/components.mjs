// Integration-only driver reuses accepted source oracles and the browser-host runner.
import assert from 'node:assert/strict';
import {mkdir,readFile,writeFile,cp,statfs} from 'node:fs/promises';
import {join,dirname} from 'node:path';
import {freemem} from 'node:os';
import {runCommand} from '../../scripts/quality-process.mjs';
import {sourceIdentity} from '../../scripts/e2e/build-identity.mjs';
const root=process.cwd(),out=join(root,'results/integration554');
const source='P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo';
const scratch=join(root,'tmp/554-system');await mkdir(scratch,{recursive:true});
Object.assign(process.env,{TEMP:scratch,TMP:scratch,TMPDIR:scratch,CARGO_TARGET_DIR:join(root,'target'),CARGO_BUILD_JOBS:'2',
 CARGO:'C:/Users/user/.cargo/bin/cargo.exe',RUSTC:'C:/Users/user/.cargo/bin/rustc.exe',
 WASM_BINDGEN:'P:/workshop/Windows_Worshop/oira666/xterm-rust/tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe',
 SOURCE_ROOT:source,XTERM_SOURCE_ROOT:source,SOURCE_REPO:source,SOURCE_PACKAGE_JSON:join(source,'package.json'),
 REFERENCE_MODULES:join(source,'node_modules'),REFERENCE_NODE_MODULES:join(source,'node_modules'),NODE_PATH:join(source,'node_modules'),
 PLAYWRIGHT_MODULE:join(source,'node_modules/playwright/index.mjs')});
const json=async file=>JSON.parse(await readFile(file,'utf8'));
const save=(name,data)=>writeFile(join(out,name),JSON.stringify(data,null,2)+'\n');
const commands=[],completed=[];
async function headroom(){const disk=await statfs(root);const v={ramGiB:freemem()/1024**3,diskGiB:Number(disk.bavail)*Number(disk.bsize)/1024**3};assert(v.ramGiB>=8&&v.diskGiB>=10,JSON.stringify(v));return v;}
async function run(name,args,timeoutMs=600000){
 const before=await headroom();const r=await runCommand(process.execPath,args,{cwd:root,env:process.env,log:join(out,name+'.log'),timeoutMs,resourceCheck:headroom});
 commands.push({name,before,...r});await save('components-commands-'+tag+'.json',commands);assert.equal(r.status,'passed',name);return r;
}
const entries=[
 {test:'image_addon_browser',count:87,command:'scripts/image-canvas-check.mjs'},
 {test:'wasm_public_decoration',count:1,prepare:'scripts/public-decoration-source.mjs'},
 {test:'wasm_attach_removal',count:1,prepare:'scripts/attach-removal-oracle.mjs'},
 {test:'wasm_attach_listener_lifetime',count:1},
 {test:'wasm_search_addon',count:22,prepare:'scripts/search-call-args-source.mjs',env:{SEARCH_CASES:'search-retired-timeout'}},
 {test:'wasm_search_delayed_expiry',count:1,prepare:'scripts/search-call-args-source.mjs',env:{SEARCH_CASES:'search-delayed-expiry'}},
 {test:'wasm_selection_teardown',count:1,prepare:'scripts/selection-teardown-source.mjs'},
 {test:'wasm_accessibility',count:33},
 {test:'wasm_accessibility_terminal',count:8},
 {test:'wasm_webgl_context',count:4,prepare:'scripts/webgl-context-oracle.mjs',prelude:'tmp/webgl-context/source.js'},
 {test:'wasm_webgl_frame',count:11,prepare:'scripts/webgl-frame-oracle.mjs',prelude:'tmp/webgl-frame/source.js'},
 {test:'wasm_progress_math',count:1,prepare:'scripts/progress-math-oracle.mjs'},
 {test:'font_gsub_single_browser',count:2,package:'xterm-font-addons',features:'',prepare:'scripts/font-gsub-header-browser-source.mjs',env:{GSUB_CORPUS:'single'},deadline:360000},
 {test:'font_gsub_noncontext_browser',count:2,package:'xterm-font-addons',features:'',prepare:'scripts/font-gsub-header-browser-source.mjs',env:{GSUB_CORPUS:'noncontext'},deadline:360000},
 {test:'font_cff2_fonts_browser',count:2,package:'xterm-font-addons',features:'',deadline:600000}
];
const from=process.argv[2]??entries[0].test,index=entries.findIndex(e=>e.test===from);assert(index>=0,'Explicit valid starting suite required');
const tag=from,input=await sourceIdentity(root);await save('components-input-'+tag+'.json',input);
for(const e of entries.slice(index)){
 delete process.env.SEARCH_CASES;delete process.env.GSUB_CORPUS;delete process.env.WASM_PRELUDE;
 Object.assign(process.env,e.env??{}, {WASM_TEST:e.test,WASM_PACKAGE:e.package??'xterm-web',CARGO_FEATURES:e.features??'acceptance-hooks',WASM_TEST_TIMEOUT_MS:String(e.deadline??60000)});
 if(e.prepare)await run(e.test+'-source',[e.prepare]);
 if(e.prelude)process.env.WASM_PRELUDE=join(root,e.prelude);
 await run(e.test,[e.command??'scripts/browser-host-check.mjs'],e.deadline?1800000:600000);
 const key=e.package?e.package+'__'+e.test:e.test;
 const report=await json(join(root,'results/browser-host',key+'-results.json'));
 await save(e.test+'-results.json',report);
 assert.equal(report.status,'passed');assert.equal(report.browsers.length,3);
 for(const b of report.browsers){assert.equal(b.status,'passed');assert.equal(b.tests.length,e.count);assert.deepEqual(b.counts,{passed:e.count,failed:0,ignored:0,filtered:0});assert.deepEqual(b.errors,[]);assert.deepEqual(b.console.filter(c=>c.type==='error'),[]);}
 const directory=dirname(dirname(report.artifacts.wasm.path));
 const raw=join(out,'browser-raw',e.test);await mkdir(raw,{recursive:true});
 for(const name of [e.test+'-build.log',e.test+'-bindgen-version.log',e.test+'-bindgen.log','commands.json']) await cp(join(directory,name),join(raw,name));
 for(const b of report.browsers)await writeFile(join(raw,e.test+'-'+b.name+'.log'),b.output);
 completed.push({test:e.test,count:e.count,artifacts:report.artifacts,browsers:report.browsers.map(b=>({name:b.name,version:b.version,counts:b.counts}))});
 await save('components-summary-'+tag+'.json',{sourceSha256:input.sha256,completed});
 assert.equal((await sourceIdentity(root)).sha256,input.sha256);
}
console.log(JSON.stringify({status:'passed',tag,completed:completed.map(e=>({test:e.test,count:e.count}))}));
