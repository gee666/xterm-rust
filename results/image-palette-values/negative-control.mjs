// Use when proving the new assertions fail against the exact integrated base.
// Restore only eleven owned production inputs. Keep all old/new assertions.
import {readFile,writeFile,mkdir,copyFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const root=process.cwd(),out='results/image-palette-values',base='11ad71a8';
const files=['crates/xterm-addons/src/image/sixel/canvas.rs',
 'crates/xterm-addons/src/image/sixel/handler.rs','crates/xterm-addons/src/image/sixel/mod.rs',
 'crates/xterm-addons/src/image/sixel/parser.rs','crates/xterm-web/src/image/addon.rs',
 'crates/xterm-web/src/image/addon/bindings.rs','crates/xterm-web/src/image/addon/controls.rs',
 'crates/xterm-web/src/image/addon/option_inspection.rs','crates/xterm-web/src/image/addon/options.rs',
 'crates/xterm-web/src/image/addon/protocols.rs','crates/xterm-web/src/image/sixel_protocol.rs'];
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const saved=new Map(),baseline=new Map(),audit={base,files:[],restored:false};
await mkdir(out,{recursive:true});
for(const file of files){
 const bytes=await readFile(file),old=spawnSync('git',['show',base+':'+file],{timeout:30000,maxBuffer:1024*1024});
 if(old.status!==0||old.error)throw Error('git show '+file);
 saved.set(file,bytes);baseline.set(file,old.stdout);
 audit.files.push({file,fixedSha256:hash(bytes),baselineSha256:hash(old.stdout)});
}
try{
 for(const file of files){await writeFile(file,baseline.get(file));if(hash(await readFile(file))!==hash(baseline.get(file)))throw Error('baseline '+file)}
 const record=await runCommand('node',['scripts/image-canvas-check.mjs'],{
  cwd:root,env:{...process.env,WASM_TEST:'image_addon_browser',BROWSERS:'chromium',
   CARGO_BUILD_JOBS:'2',CARGO_TARGET_DIR:resolve('target'),TEMP:resolve('tmp/palette-temp'),TMP:resolve('tmp/palette-temp')},
  timeoutMs:240000,log:resolve(out,'negative.log')});
 audit.command=record;
 for(const [from,to] of [['results/browser-host/image_addon_browser-results.json','negative-browser.json'],['results/image-addon/verification.json','negative-verification.json']])await copyFile(from,resolve(out,to));
 const report=JSON.parse(await readFile(resolve(out,'negative-browser.json'),'utf8'));
 const browser=report.browsers[0];
 audit.failed=[...browser.output.matchAll(/^test (.*?) \.\.\. FAIL$/gm)].map(m=>m[1]).sort();
 if(report.browsers.length!==1||browser.name!=='chromium'||browser.tests.length!==71||
 browser.counts.passed!==61||browser.counts.failed!==10||browser.counts.ignored||browser.counts.filtered||
 browser.errors.length||audit.failed.some(name=>!name.startsWith('raw_palette'))||
 record.exitCode!==1||record.timedOut||record.error||record.resourceError)
   throw Error('Negative control did not complete61P10F only new palette functions');
 audit.status='passed-negative-control';console.log(audit.status);
}finally{
 for(const[file,bytes]of saved)await writeFile(file,bytes);
 for(const entry of audit.files){
  entry.restoredSha256=hash(await readFile(entry.file));
  if(entry.restoredSha256!==entry.fixedSha256)throw Error('restore '+entry.file);
 }
 audit.restored=true;
 await writeFile(resolve(out,'negative-audit.json'),JSON.stringify(audit,null,2)+'\n');
}
