// Reuse referenceIntegrity and the existing per-member fmt fallback evidence.
import fs from 'node:fs';
import {spawnSync,execFileSync} from 'node:child_process';
import {referenceIntegrity} from '../../scripts/upstream-baseline-utils.mjs';
const out='results/integration488';
function check(name,cmd,args) {
 const r=spawnSync(cmd,args,{encoding:'utf8',timeout:120000,maxBuffer:8*1024*1024,env:process.env});
 fs.writeFileSync(`${out}/${name}.log`,(r.stdout??'')+(r.stderr??'')+(r.error?String(r.error):''));
 return {name,command:[cmd,...args],status:r.status,error:r.error?String(r.error):null};
}
const report={fmt:[check('fmt-workspace','cargo',['fmt','--all','--','--check'])]};
for(const pkg of ['xterm-core','xterm-addons','xterm-font-addons','xterm-web','xterm-wasm-support']) report.fmt.push(check(`fmt-${pkg}`,'cargo',['fmt','-p',pkg,'--','--check']));
const changed=execFileSync('git',['diff','--name-only','26c7c42d','HEAD'],{encoding:'utf8',timeout:30000}).trim().split(/\r?\n/);
report.syntax=[];
for(const file of changed.filter(f=>/^(crates|scripts)\//.test(f)&&/\.(mjs|js)$/.test(f))) {
 const r=check(`syntax-${report.syntax.length}`,process.execPath,['--check',file]);
 report.syntax.push({file,...r});
}
const manifest=JSON.parse(fs.readFileSync('docs/baseline/reference-manifest.json'));
report.reference=['source_repo','P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo'].map(root=>({root,...referenceIntegrity(root,manifest)}));
fs.writeFileSync(`${out}/light-checks.json`,JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report));
if(report.fmt.slice(1).some(r=>r.status!==0)||report.syntax.some(r=>r.status!==0)||report.reference.some(r=>r.mismatches.length))process.exitCode=1;
