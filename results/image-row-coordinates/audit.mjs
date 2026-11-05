// Use when checking retained active-coordinate runs and their exact inputs.
import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
const out=resolve('results/image-row-coordinates');
const hash=b=>createHash('sha256').update(b).digest('hex');
const json=async n=>JSON.parse(await readFile(resolve(out,n+'.json'),'utf8'));
const audit={status:'running',inputPins:0,artifactPins:0,referenceFiles:0,compilations:[]};
const restoration=await json('negative-audit');
if(!restoration.restored||restoration.failed.length!==3||restoration.failed.some(n=>!n.startsWith('row_coordinates::')))throw Error('Bad exact control');
for(const pin of restoration.files) {
  const bytes=execFileSync(process.env.GIT??'git',['show',restoration.baseline+':'+pin.path],{timeout:30000,maxBuffer:16*1024*1024});
  if(hash(bytes)!==pin.baseline||hash(await readFile(pin.path))!==pin.fixed)throw Error('Bad baseline/restoration pin '+pin.path);
}
let names;
const reports={};
for(const phase of ['initial','negative','final']) {
  const report=reports[phase]=await json(phase+'-browser');
  const verification=await json(phase+'-verification');
  const control=phase==='negative';
  if(report.status!==(control?'failed':'passed')||report.browsers.map(b=>b.name).join(',')!==(control?'chromium':'chromium,firefox,webkit'))throw Error('Bad phase '+phase);
  for(const b of report.browsers) {
    if(b.tests.length!==64||b.counts.passed!==(control?61:64)||b.counts.failed!==(control?3:0)||b.counts.ignored||b.counts.filtered||b.errors.length)throw Error('Bad counters '+phase+'/'+b.name);
    names??=JSON.stringify(b.tests);
    if(names!==JSON.stringify(b.tests))throw Error('Test inventory changed');
    const failed=[...b.output.matchAll(/^test ([^\n]+) \.\.\. FAIL$/gm)].map(m=>m[1]);
    if(control&&JSON.stringify(failed)!==JSON.stringify(restoration.failed))throw Error('Unexpected control failures');
  }
  for(const pin of [...verification.rustInputs,...verification.sourceInputs]) {
    if(control&&restoration.files.some(p=>p.path===pin.path)) {
      if(pin.sha256!==restoration.files.find(p=>p.path===pin.path).baseline)throw Error('Unpinned control input');
    } else if(hash(await readFile(resolve(pin.path)))!==pin.sha256)throw Error('Changed phase input '+phase+'/'+pin.path);
    audit.inputPins++;
  }
  for(const kind of ['wasm','bindings']) {
    const pin=report.artifacts[kind];
    if(hash(await readFile(pin.path))!==pin.sha256)throw Error('Changed generated artifact '+phase+'/'+kind);
    audit.artifactPins++;
  }
  const build=await readFile(resolve(out,phase+'-build.log'),'utf8');
  const messages=build.split('\n').filter(s=>s.startsWith('{')).map(s=>JSON.parse(s));
  const artifact=messages.find(m=>m.reason==='compiler-artifact'&&m.target.name==='image_addon_browser'&&m.profile.test);
  if(!artifact||artifact.fresh!==false)throw Error('No fresh actual test compilation '+phase);
  audit.compilations.push({phase,fresh:artifact.fresh,filenames:artifact.filenames});
}
const finalCompiler=reports.final.artifacts.compiler;
if(hash(await readFile(finalCompiler.path))!==finalCompiler.sha256)throw Error('Final compiler artifact changed');
audit.artifactPins++;
if(reports.negative.artifacts.wasm.sha256===reports.final.artifacts.wasm.sha256)throw Error('Control and final WASM must differ');
const native=await json('native-summary');
const log=await readFile(resolve(out,'workspace-tests.log'),'utf8');
const summaries=[...log.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)];
if(summaries.length!==498||native.summaries!==498||native.passed!==2196||summaries.reduce((n,m)=>n+Number(m[1]),0)!==2196||summaries.some(m=>m.slice(2).some(n=>Number(n))))throw Error('Bad native totals');
for(const command of await json('native-commands'))if(command.status!=='passed')throw Error('Failed native command '+command.name);
const format=await json('format-commands');
if(format[0].name!=='fmt-all'||format[0].status!=='failed'||!(await readFile(resolve(out,'fmt-all.log'),'utf8')).includes('206')||format.slice(1).length!==5||format.slice(1).some(c=>c.status!=='passed'))throw Error('Incorrect formatting disclosure');
const baseline=JSON.parse(await readFile('docs/baseline/reference-manifest.json','utf8'));
for(const root of [resolve('source_repo'),resolve(process.env.ORIGINAL_REFERENCE??'P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo')]) {
  for(const pin of baseline)if(hash(await readFile(resolve(root,pin.path)))!==pin.sha256)throw Error('Reference changed '+pin.path);
  audit.referenceFiles+=baseline.length;
}
audit.status='passed';
await writeFile(resolve(out,'audit.json'),JSON.stringify(audit,null,2)+'\n');
console.log(JSON.stringify(audit));
