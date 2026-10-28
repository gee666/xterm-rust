// Copyright (c) 2026 xterm-rust contributors. MIT license.
// Retain each run, bound child process trees and distinguish stale snapshots
// from passing tests. No compile failure is counted as skipped or passing.
import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync, statfsSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve, relative } from 'node:path';
import { createHash, randomUUID } from 'node:crypto';

const root=resolve(import.meta.dirname,'..');
const id=`${new Date().toISOString().replace(/[:.]/g,'-')}-${randomUUID().slice(0,8)}`;
const destination=resolve(root,'results/public-adapters/runs',id);
mkdirSync(destination,{recursive:true});
const report={id,status:'incomplete',scope:'Focused adapters only. Full equivalence NOT PROVEN; terminal/browser binding and native platform gates are separate.',
  started:new Date().toISOString(),platform:process.platform,node:process.version,commands:[],inputsBefore:null};
function persist() {
  writeFileSync(resolve(destination,'report.json'),JSON.stringify(report,null,2)+'\n');
  writeFileSync(resolve(root,'results/public-adapters/verification.json'),JSON.stringify({...report,run:relative(root,destination).replaceAll('\\','/')},null,2)+'\n');
}
// Invalidate any old latest pass before loading project helpers or reading inputs.
persist();
let runCommand;
const cargoHome=process.env.CARGO_HOME||resolve(homedir(),'.cargo');
function executable(name,override) {
  const candidate=resolve(cargoHome,'bin',process.platform==='win32'?`${name}.exe`:name);
  return override||(existsSync(candidate)?candidate:name);
}
const cargo=executable('cargo',process.env.CARGO);
const rustc=executable('rustc',process.env.RUSTC);
const suites=['public_addon_manager','public_addon_reentrancy','public_buffer_api','public_parser_api','public_parser_guard','public_parser_reentrant','public_adapter_terminal'];
function rustFiles(directory) {
  return readdirSync(resolve(root,directory),{withFileTypes:true}).flatMap(entry=>{
    const path=`${directory}/${entry.name}`;
    return entry.isDirectory()?rustFiles(path):entry.isFile()&&entry.name.endsWith('.rs')?[path]:[];
  });
}
function inputHashes() {
  const files=[...rustFiles('crates/xterm-core/src'),...suites.map(suite=>`crates/xterm-core/tests/${suite}.rs`),
    'Cargo.toml','Cargo.lock','.cargo/config.toml','crates/xterm-core/Cargo.toml',
    'scripts/check-public-adapters.mjs','scripts/public-addon-oracle.mjs','scripts/public-buffer-oracle.mjs',
    'scripts/quality-process.mjs','scripts/quality-tree.mjs','crates/xterm-wasm-support/tests/fixtures/public-cell-methods.json',
    ...['public/AddonManager.ts','public/AddonManager.test.ts','public/BufferApiView.ts','CircularList.ts','buffer/CellData.ts','buffer/AttributeData.ts'].map(path=>`source_repo/src/common/${path}`),
  ].sort();
  return Object.fromEntries(files.map(path=>[path,createHash('sha256').update(readFileSync(resolve(root,path))).digest('hex')]));
}
function headroom() {
  const disk=statfsSync(root);
  if(Number(disk.bavail)*Number(disk.bsize)<512*1024*1024) throw new Error('Stopped below512MiB disk headroom');
}
let discovered;
async function command(kind,command,args,timeoutMs=120000) {
  headroom();
  const log=resolve(destination,`${kind}.log`);
  const result=await runCommand(command,args,{cwd:root,env:process.env,log,timeoutMs,resourceCheck:headroom});
  const entry={kind,...result,log:relative(root,log).replaceAll('\\','/')};
  report.commands.push(entry);
  persist();
  return readFileSync(log,'utf8');
}
let rustLog='';
try {
  ({runCommand}=await import('./quality-process.mjs'));
  report.inputsBefore=inputHashes();
  discovered=suites.reduce((count,suite)=>count+(readFileSync(resolve(root,`crates/xterm-core/tests/${suite}.rs`),'utf8').match(/#\[test\]/g)||[]).length,0);
  persist();
  report.cargo=(await command('cargo-version',cargo,['--version'],10000)).trim();
  report.rustc=(await command('rustc-version',rustc,['--version'],10000)).trim();
  await command('source-addon',process.execPath,['scripts/public-addon-oracle.mjs']);
  await command('source-buffer',process.execPath,['scripts/public-buffer-oracle.mjs']);
  rustLog=await command('rust',cargo,['test','-p','xterm-core',...suites.flatMap(suite=>['--test',suite])]);
} catch(error) {report.error=error.stack??String(error);}
finally {
  try {
    report.inputsAfter=inputHashes();
    report.snapshotStable=report.inputsBefore!==null && JSON.stringify(report.inputsBefore)===JSON.stringify(report.inputsAfter);
  } catch(error) {
    report.snapshotError=error.stack??String(error);
    report.snapshotStable=false;
  }
  const cases=[...rustLog.matchAll(/^test (\S+) \.\.\. (ok|FAILED|ignored)$/gm)].map(([,name,status])=>({name,status}));
  report.rust={discovered:discovered??null,executed:cases.filter(c=>c.status!=='ignored').length,passed:cases.filter(c=>c.status==='ok').length,
    failed:cases.filter(c=>c.status==='FAILED').length,skipped:cases.filter(c=>c.status==='ignored').length,unrun:discovered===undefined?null:Math.max(0,discovered-cases.length),cases};
  report.finished=new Date().toISOString();
  const countsComplete=Number.isInteger(discovered)&&discovered>0&&report.rust.executed===discovered&&report.rust.passed===discovered&&report.rust.failed===0&&report.rust.skipped===0&&report.rust.unrun===0;
  report.status=report.error||report.snapshotError||!countsComplete||report.commands.length!==5||report.commands.some(command=>command.status!=='passed')?'failed':report.snapshotStable?'passed':'source-changed';
  persist();
  console.log(JSON.stringify({id,status:report.status,snapshotStable:report.snapshotStable,rust:report.rust,run:relative(root,destination)},null,2));
  if(report.status!=='passed') process.exitCode=1;
}
