// Use when checking raw-flag test sensitivity against the accepted474 product.
// Only five owned production files change; original assertions stay intact.
import {readFile,writeFile,copyFile,mkdir} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const base='fdd3d621';
const files=['crates/xterm-web/src/image/addon.rs','crates/xterm-web/src/image/addon/bindings.rs','crates/xterm-web/src/image/addon/controls.rs','crates/xterm-web/src/image/addon/option_inspection.rs','crates/xterm-web/src/image/addon/options.rs'];
const hash=value=>createHash('sha256').update(value).digest('hex');
const saved=new Map(), baseline=new Map(), audit={base,files:[],restored:false};
const out='results/image-raw-flags';await mkdir(out,{recursive:true});
for(const file of files){
  const fixed=await readFile(file);
  const result=spawnSync('git',['show',`${base}:${file}`],{timeout:30000,maxBuffer:1024*1024});
  if(result.status!==0||result.error)throw Error(`git show failed ${file}`);
  saved.set(file,fixed);baseline.set(file,result.stdout);
  audit.files.push({file,baselineSha256:hash(result.stdout),fixedSha256:hash(fixed)});
}
try{
  for(const file of files){
    await writeFile(file,baseline.get(file));
    if(hash(await readFile(file))!==hash(baseline.get(file)))throw Error(`baseline differs ${file}`);
  }
  const result=spawnSync(process.execPath,['scripts/image-canvas-check.mjs'],{env:{...process.env,WASM_TEST:'image_addon_browser',BROWSERS:'chromium'},timeout:180000,encoding:'utf8',maxBuffer:8*1024*1024});
  audit.command=['node','scripts/image-canvas-check.mjs'];audit.exit=result.status;audit.error=result.error?.message??null;
  await writeFile(`${out}/negative.log`,(result.stdout??'')+(result.stderr??''));
  for(const [from,to] of [['results/browser-host/image_addon_browser-results.json','negative-browser.json'],['results/image-addon/verification.json','negative-verification.json']])await copyFile(from,`${out}/${to}`);
  const report=JSON.parse(await readFile(`${out}/negative-browser.json`,'utf8'));
  audit.browsers=report.browsers.map(({name,counts,status})=>({name,counts,status}));
  if(result.status===0||result.error)throw Error('Expected a completed failing browser control');
  if(report.browsers.length!==1||report.browsers[0].name!=='chromium')throw Error('Unexpected control browsers');
  const failed=[...report.browsers[0].output.matchAll(/^test (.*?) \.\.\. FAIL$/gm)].map(match=>match[1]);
  audit.failedTests=failed;
  const expected=['raw_flags::image_added_reset_reentry_restores_second_raw_scrolling_copy','raw_flags::raw_flag_getters_copy_twice_but_reset_only_replaces_scrolling','raw_flags::raw_flag_values_retain_identity_without_primitive_conversion'].sort();
  if(JSON.stringify(failed.sort())!==JSON.stringify(expected))throw Error(`Unexpected failures ${failed}`);
  if(report.browsers[0].counts.failed!==3||report.browsers[0].counts.passed!==56||report.browsers[0].tests.length!==59)throw Error('Control did not complete all59 tests');
  audit.status='passed-negative-control';
}finally{
  for(const [file,bytes]of saved)await writeFile(file,bytes);
  for(const entry of audit.files){entry.restoredSha256=hash(await readFile(entry.file));if(entry.restoredSha256!==entry.fixedSha256)throw Error(`restore failed ${entry.file}`);}
  audit.restored=true;
  await writeFile(`${out}/negative-audit.json`,JSON.stringify(audit,null,2)+'\n');
}
console.log(JSON.stringify(audit.browsers));
