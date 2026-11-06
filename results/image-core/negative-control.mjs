// Reproduce the missing old bindings without changing any test assertion.
// Run only in an owned swarm worktree, with normal browser-host environment.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFile,writeFile,mkdir,copyFile,statfs} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {freemem} from 'node:os';
import {resolve} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const root=process.cwd(),output=resolve(root,'results/image-core');
const label=process.env.IMAGE_CORE_CONTROL_LABEL??'19';
assert(/^[a-zA-Z0-9_-]+$/.test(label),'Safe result label');
const git=process.env.GIT??'git';
const command=(...args)=>execFileSync(git,args,{cwd:root,encoding:'utf8',timeout:30000}).trim();
assert(command('branch','--show-current').startsWith('swarm/'),'Control requires an owned swarm branch');
const baseline='224d1b339a43eddf899c2944d9dad734d8c44234';
const paths=['crates/xterm-core/src/public_api/parser.rs','crates/xterm-core/src/terminal/mod.rs',
 'crates/xterm-core/src/terminal/state.rs','crates/xterm-wasm-support/src/public_api/parser_api.rs',
 'crates/xterm-wasm-support/src/public_api/parser_identifier.rs','crates/xterm-web/src/terminal/browser_open.rs',
 'crates/xterm-web/src/terminal/internal_api.rs','crates/xterm-web/src/terminal/internal_buffer.rs',
 'crates/xterm-web/src/terminal/mod.rs'];
assert.equal(command('status','--porcelain','--untracked-files=no','--',...paths),'','Commit owned source first');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const fixed=new Map(await Promise.all(paths.map(async p=>[p,await readFile(resolve(root,p))])));
const report={baseline,fixedHead:command('rev-parse','HEAD'),sourcePins:[...fixed].map(([path,b])=>({path,bytes:b.length,sha256:hash(b)})),restored:false};
const currentFixed=JSON.parse(await readFile(resolve(root,'results/browser-host/wasm_image_core-results.json'),'utf8'));
assert.equal(currentFixed.status,'passed','Run the fixed WASM suite before its control');
assert(currentFixed.browsers.every(b=>b.counts.passed===1&&b.counts.failed===0),'Fixed suite must be green');
await mkdir(resolve(output,'fixed'+label),{recursive:true});
await copyFile(resolve(root,'results/browser-host/wasm_image_core-results.json'),resolve(output,'fixed'+label+'/wasm_image_core-results.json'));
async function guard(){const disk=await statfs(root);assert(freemem()>=8*1024**3&&disk.bavail*disk.bsize>=10*1024**3,'RAM/disk reserve');}
try {
 for(const p of paths)await writeFile(resolve(root,p),execFileSync(git,['show',baseline+':'+p],{cwd:root,timeout:30000}));
 await guard();
 report.run=await runCommand(process.execPath,['scripts/browser-host-check.mjs'],{
  cwd:root,env:{...process.env,BROWSERS:'chromium',WASM_TEST:'wasm_image_core'},
  log:resolve(output,'wasm-old-'+label+'.log'),timeoutMs:150000,resourceCheck:guard});
 report.actual=JSON.parse(await readFile(resolve(root,'results/browser-host/wasm_image_core-results.json'),'utf8'));
 assert.equal(report.run.status,'failed','Old implementation must fail unchanged runtime assertions');
 assert.equal(report.actual.browsers.length,1);
 assert.deepEqual(report.actual.browsers[0].counts,{passed:0,failed:1,ignored:0,filtered:0});
 assert(report.actual.browsers[0].output.includes('_parser'),'Expected missing input handler failure');
 report.expectedFailure=true;
} finally {
 for(const [p,bytes] of fixed)await writeFile(resolve(root,p),bytes);
 for(const [p,bytes] of fixed)assert.equal(hash(await readFile(resolve(root,p))),hash(bytes),'Restoration '+p);
 report.restored=true;
 await writeFile(resolve(output,'negative-'+label+'.json'),JSON.stringify(report,null,2)+'\n');
}
console.log('Old public WASM0P1F at missing _parser; exact fixed source restored. This is a negative control, not a passing old implementation.');
