// Scoped headless verification with retained failures and source snapshot checks.
import { readFile,readdir,mkdir,writeFile,statfs,copyFile } from 'node:fs/promises';
import { createHash,randomUUID } from 'node:crypto';
import path from 'node:path';

const root=process.cwd();
const directory=path.join('results/headless/runs',`run-${Date.now()}-${randomUUID().slice(0,8)}`);
await mkdir(directory,{recursive:true});
const tests=['headless_upstream','headless_contract','headless_reentry','headless_queue',
    'headless_queue_edges','headless_queue_priority','native_queue_differential',
    'public_api_validation','headless_options','headless_modes','public_api_integration','public_adapter_terminal'];
const report={scope:'Scoped headless implementation, not full terminal/browser/platform equivalence',
    node:process.version,platform:process.platform,arch:process.arch,tests,commands:[],status:'incomplete'};
async function persist() {
    await writeFile(path.join(directory,'report.json'),JSON.stringify(report,null,2)+'\n');
    await writeFile('results/headless/latest.json',JSON.stringify({directory,status:report.status,
        changedDuringRun:report.changedDuringRun??[],error:report.error??null},null,2)+'\n');
}
// Set the current attempt before loading shared tooling or reading optional
// inputs, so a failed import cannot leave an older pass looking current.
await persist();
const sha=content=>createHash('sha256').update(content).digest('hex');
async function files(directory) {
    const found=[];
    for(const entry of await readdir(directory,{withFileTypes:true})) {
        const file=path.join(directory,entry.name);
        if(entry.isDirectory())found.push(...await files(file));
        else if(entry.isFile())found.push(file);
    }
    return found;
}
async function snapshot() {
    const selected=[...await files('crates/xterm-core/src'),...await files('crates/xterm-core/tests'),
        'Cargo.toml','Cargo.lock','crates/xterm-core/Cargo.toml','.cargo/config.toml'];
    const hashes={};
    for(const file of selected.sort())hashes[file.replaceAll('\\','/')]=sha(await readFile(file));
    return hashes;
}
async function resources() {
    const space=await statfs(root);const available=space.bavail*space.bsize;const minimum=256*1024*1024;
    if(available<minimum)throw new Error(`Only ${available} bytes remain; scoped verifier requires ${minimum}`);
}
try {
    const {runCommand}=await import('./quality-process.mjs');
    await resources();report.before=await snapshot();
    async function execute(command,args,name) {
        const log=path.join(directory,`${name}.log`);
        const result=await runCommand(command,args,{cwd:root,env:process.env,log,timeoutMs:120000,resourceCheck:resources});
        report.commands.push({...result,log});await persist();return result;
    }
    await execute(process.env.RUSTC??'rustc',['--version'],'rustc-version');
    await execute(process.env.CARGO??'cargo',['--version'],'cargo-version');
    const source=await execute(process.execPath,['scripts/headless-oracle.mjs'],'source-upstream');
    if(source.status==='passed')await copyFile('results/headless/upstream.json',path.join(directory,'source-upstream.json'));
    const args=['-p','xterm-core',...tests.flatMap(name=>['--test',name])];
    await execute(process.env.CARGO??'cargo',['test',...args],'native-tests');
    await execute(process.env.CARGO??'cargo',['clippy','--lib',...args,'--','-D','warnings'],'clippy');
    report.after=await snapshot();
    report.changedDuringRun=[...new Set([...Object.keys(report.before),...Object.keys(report.after)])]
        .filter(file=>report.before[file]!==report.after[file]);
    report.status=report.commands.every(result=>result.status==='passed')&&report.changedDuringRun.length===0?'passed':'failed';
} catch(error) {report.error=error.stack??String(error);report.status='failed';}
finally {await persist();}
console.log(JSON.stringify({directory,status:report.status,changes:report.changedDuringRun,error:report.error,
    commands:report.commands.map(({command,status,log})=>({command,status,log}))},null,2));
if(report.status!=='passed')process.exitCode=1;
