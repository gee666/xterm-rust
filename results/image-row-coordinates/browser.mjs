// Use when reproducing a fixed or negative-control image row browser matrix.
import {mkdir, readFile, writeFile, copyFile, statfs} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {freemem} from 'node:os';
import {runCommand} from '../../scripts/quality-process.mjs';
const phase=process.argv[2];
if (!['initial','negative','final'].includes(phase)) throw Error('Pass initial, negative or final');
const out=resolve('results/image-row-coordinates');
const scratch=resolve('tmp/browser');
await mkdir(scratch,{recursive:true});
const disk=await statfs(process.cwd());
const headroom={RAMavailable:freemem(),diskFree:Number(disk.bavail)*Number(disk.bsize)};
if(headroom.RAMavailable<8*2**30||headroom.diskFree<10*2**30)throw Error('Insufficient resource headroom');
await writeFile(resolve(out,phase+'-headroom.json'),JSON.stringify(headroom,null,2)+'\n');
const command=await runCommand(process.execPath,['scripts/image-canvas-check.mjs'],{
  cwd:process.cwd(),env:{...process.env,TEMP:scratch,TMP:scratch,CARGO_TARGET_DIR:resolve('target'),CARGO_BUILD_JOBS:'2',WASM_TEST:'image_addon_browser',BROWSERS:phase==='negative'?'chromium':'chromium,firefox,webkit'},
  timeoutMs:300000,log:resolve(out,phase+'.log')
});
await writeFile(resolve(out,phase+'-command.json'),JSON.stringify(command,null,2)+'\n');
for(const [from,to] of [['results/browser-host/image_addon_browser-results.json',phase+'-browser.json'],['results/image-addon/verification.json',phase+'-verification.json']])await copyFile(from,resolve(out,to));
const report=JSON.parse(await readFile(resolve(out,phase+'-browser.json'),'utf8'));
if(report.artifacts) await copyFile(resolve(dirname(dirname(report.artifacts.wasm.path)),'image_addon_browser-build.log'),resolve(out,phase+'-build.log'));
if(phase==='negative') {
  if(command.status!=='failed'||report.status!=='failed'||report.browsers.length!==1)throw Error('Control did not complete as a failing Chromium suite');
  const b=report.browsers[0];
  if(b.tests.length!==64||b.counts.passed!==61||b.counts.failed!==3||b.counts.ignored||b.counts.filtered||b.errors.length)throw Error('Unexpected control counters');
} else {
  if(command.status!=='passed'||report.status!=='passed'||report.browsers.map(b=>b.name).join(',')!=='chromium,firefox,webkit')throw Error('Incomplete fixed matrix');
  for(const b of report.browsers)if(b.tests.length!==64||b.counts.passed!==64||b.counts.failed||b.counts.ignored||b.counts.filtered||b.errors.length)throw Error('Unexpected fixed counters');
}
console.log(JSON.stringify(report.browsers.map(({name,version,counts})=>({name,version,counts}))));
