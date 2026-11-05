import {readFile,writeFile,copyFile,mkdir} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const base='fe9dcd9e';
const files=['crates/xterm-addons/src/image/sixel/handler.rs','crates/xterm-web/src/image/addon.rs','crates/xterm-web/src/image/addon/bindings.rs','crates/xterm-web/src/image/addon/option_inspection.rs','crates/xterm-web/src/image/addon/options.rs','crates/xterm-web/src/image/addon/protocols.rs','crates/xterm-web/src/image/sixel_protocol.rs'];
const hash=value=>createHash('sha256').update(value).digest('hex');
const saved=new Map(),audit={base,files:[],restored:false};
const out='results/image-sixel-size-limit';await mkdir(out,{recursive:true});
for(const file of files){
  const fixed=await readFile(file);const result=spawnSync('git',['show',`${base}:${file}`],{timeout:30000,maxBuffer:1024*1024});
  if(result.status!==0||result.error)throw Error(`git show failed ${file}`);
  saved.set(file,fixed);audit.files.push({file,baselineSha256:hash(result.stdout),fixedSha256:hash(fixed)});
}
try{
  for(const file of files){
    const result=spawnSync('git',['show',`${base}:${file}`],{timeout:30000,maxBuffer:1024*1024});
    if(result.status!==0||result.error)throw Error(`git show failed ${file}`);
    await writeFile(file,result.stdout);
    if(hash(await readFile(file))!==audit.files.find(entry=>entry.file===file).baselineSha256)throw Error(`baseline differs ${file}`);
  }
  const result=spawnSync(process.execPath,['scripts/image-canvas-check.mjs'],{env:{...process.env,WASM_TEST:'image_addon_browser',BROWSERS:'chromium'},timeout:240000,encoding:'utf8',maxBuffer:8*1024*1024});
  audit.command=['node','scripts/image-canvas-check.mjs'];audit.exit=result.status;audit.error=result.error?.message??null;
  await writeFile(`${out}/negative.log`,(result.stdout??'')+(result.stderr??''));
  for(const [from,to] of [['results/browser-host/image_addon_browser-results.json','negative-browser.json'],['results/image-addon/verification.json','negative-verification.json']])await copyFile(from,`${out}/${to}`);
  const report=JSON.parse(await readFile(`${out}/negative-browser.json`,'utf8'));
  audit.browsers=report.browsers.map(({name,counts,status})=>({name,counts,status}));
  if(result.status===0||result.error)throw Error('Expected a completed failing browser control');
  if(report.browsers.length!==1||report.browsers[0].name!=='chromium')throw Error('Unexpected negative browsers');
  const failed = [...report.browsers[0].output.matchAll(/^test (.*?) \.\.\. FAIL$/gm)].map(match=>match[1]);
  audit.failedTests = failed;
  const newFailures = failed.filter(name=>name.startsWith('raw_sixel_limit::'));
  if(newFailures.length!==5)throw Error('Expected all five new test failures');
  if(failed.some(name=>!name.startsWith('raw_sixel_limit::') && name!=='alternate_resize_public_reset_and_disposal_match_source'))throw Error('Unexpected failure outside known baseline visibility mismatch');
}finally{
  for(const [file,bytes]of saved)await writeFile(file,bytes);
  for(const entry of audit.files){entry.restoredSha256=hash(await readFile(entry.file));if(entry.restoredSha256!==entry.fixedSha256)throw Error(`restore failed ${entry.file}`);}
  audit.restored=true;
  await writeFile(`${out}/negative-audit.json`,JSON.stringify(audit,null,2)+'\n');
}
console.log(JSON.stringify(audit.browsers));
