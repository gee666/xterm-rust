// Add complete input snapshots to the existing bounded WASM/browser runner.
import { existsSync, mkdirSync, readFileSync, writeFileSync, statfsSync } from 'node:fs';
import { resolve } from 'node:path';
import { captureRustInputs, changedInputs, sha256 } from './public-api-evidence.mjs';
import { runCommand } from './quality-process.mjs';
process.chdir(resolve(import.meta.dirname,'..'));
const started=Date.now();
const id=new Date(started).toISOString().replace(/[:.]/g,'-');
const directory=`results/public-cell-numbers/browser-${id}`;
mkdirSync(directory,{recursive:true});
const pointer='results/public-cell-numbers/browser-summary.json';
const extra=[
  'crates/xterm-addons/Cargo.toml','crates/xterm-web/Cargo.toml',
  'crates/xterm-web/tests/public_buffer_numbers_wasm.rs',
  'crates/xterm-web/tests/cases/public_buffer_number_callbacks.rs',
  'crates/xterm-core/tests/fixtures/public-cell-numbers.tsv',
  'crates/xterm-core/tests/fixtures/public-line-number-translations.tsv',
  'scripts/browser-host-check.mjs','scripts/public-buffer-numbers-browser.mjs',
];
const roots=['crates/xterm-core/src','crates/xterm-addons/src','crates/xterm-web/src'];
const before=captureRustInputs(extra,roots);
const report={status:'running',componentOnly:true,directory,started:new Date(started).toISOString(),
  node:process.version,platform:`${process.platform}-${process.arch}`,rustInputs:before};
writeFileSync(pointer,JSON.stringify(report,null,2)+'\n');
const headroom=()=>{
  const disk=statfsSync('.');
  if(Number(disk.bavail)*Number(disk.bsize)<512*1024*1024)throw Error('Build stopped below512MiB disk headroom');
};
try {
  headroom();
  report.command=await runCommand(process.execPath,['scripts/browser-host-check.mjs'],{
    cwd:process.cwd(),env:{...process.env,WASM_TEST:'public_buffer_numbers_wasm'},
    log:resolve(directory,'runner.log'),timeoutMs:360000,resourceCheck:headroom,
  });
  const resultPath='results/browser-host/public_buffer_numbers_wasm-results.json';
  if(existsSync(resultPath)) {
    const data=readFileSync(resultPath,'utf8');
    const result=JSON.parse(data);
    writeFileSync(`${directory}/browser-report.json`,data);
    report.browserReportSha256=sha256(resultPath);
    report.browsers=result.browsers;
    report.artifacts=result.artifacts;
    report.reportCreatedDuringRun=Date.parse(result.date)>=started;
  }
  report.changedDuringRun=changedInputs(before,captureRustInputs(extra,roots));
  report.status=report.changedDuringRun.length?'changed-during-run':
    report.command.status==='passed'&&report.reportCreatedDuringRun&&
    report.browsers?.length===3&&['chromium','firefox','webkit'].every(name=>
      report.browsers.filter(row=>row.name===name&&row.status==='passed').length===1)?'passed':'failed';
} catch(error) {report.status='failed';report.error=error.stack??String(error);}
writeFileSync(`${directory}/summary.json`,JSON.stringify(report,null,2)+'\n');
writeFileSync(pointer,JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({status:report.status,browsers:report.browsers?.map(row=>({name:row.name,status:row.status,counts:row.counts})),changed:report.changedDuringRun,directory,error:report.error},null,2));
process.exitCode=report.status==='passed'?0:1;
