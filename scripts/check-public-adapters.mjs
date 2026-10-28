// Copyright (c) 2026 xterm-rust contributors. MIT license.
// Keep command failures and exact focused-suite accounting, without treating a
// compile failure as skipped or passing tests.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
const root = resolve(import.meta.dirname,'..');
const destination = resolve(root,'results/public-adapters');
mkdirSync(destination,{recursive:true});
const cargoHome = process.env.CARGO_HOME || resolve(homedir(),'.cargo');
const candidate = resolve(cargoHome,'bin',process.platform==='win32'?'cargo.exe':'cargo');
const cargo = process.env.CARGO || (existsSync(candidate)?candidate:'cargo');
const suites = ['public_addon_manager','public_addon_reentrancy','public_buffer_api','public_parser_api','public_parser_guard','public_adapter_terminal'];
const commands = [
  [process.execPath,['scripts/public-addon-oracle.mjs']],
  [process.execPath,['scripts/public-buffer-oracle.mjs']],
  [cargo,['test','-p','xterm-core',...suites.flatMap(suite=>['--test',suite])]],
];
const runs = commands.map(([command,args],index)=>{
  const result = spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:120000,maxBuffer:20*1024*1024});
  const log = `${result.stdout||''}${result.stderr||''}`;
  const file = `command-${index+1}.log`;
  writeFileSync(resolve(destination,file),log);
  return {command,args,status:result.status,signal:result.signal,error:result.error?.message,log:file,output:log};
});
const rustLog = runs.at(-1).output;
const cases = [...rustLog.matchAll(/^test (\S+) \.\.\. (ok|FAILED|ignored)$/gm)].map(([,name,status])=>({name,status}));
const discovered = suites.reduce((count,suite)=>count+(readFileSync(resolve(root,`crates/xterm-core/tests/${suite}.rs`),'utf8').match(/#\[test\]/g)||[]).length,0);
const inputs = [
  ...suites.map(suite=>`crates/xterm-core/tests/${suite}.rs`),
  ...['addon_manager.rs','buffer.rs','parser.rs','mod.rs'].map(file=>`crates/xterm-core/src/public_api/${file}`),
  'scripts/check-public-adapters.mjs','scripts/public-addon-oracle.mjs','scripts/public-buffer-oracle.mjs',
];
const toolchain = spawnSync(cargo,['--version'],{cwd:root,encoding:'utf8',timeout:10000});
const summary = {
  scope:'Focused adapters only. Full equivalence NOT PROVEN; terminal/browser binding and native platform gates are separate.',
  timestamp:new Date().toISOString(),platform:process.platform,node:process.version,cargo:toolchain.stdout?.trim(),
  inputs:Object.fromEntries(inputs.map(file=>[file,createHash('sha256').update(readFileSync(resolve(root,file))).digest('hex')])),
  commands:runs.map(({output,...run})=>run),
  rust:{discovered,executed:cases.filter(c=>c.status!=='ignored').length,passed:cases.filter(c=>c.status==='ok').length,failed:cases.filter(c=>c.status==='FAILED').length,skipped:cases.filter(c=>c.status==='ignored').length,unrun:discovered-cases.length,cases},
};
writeFileSync(resolve(destination,'verification.json'),`${JSON.stringify(summary,null,2)}\n`);
console.log(JSON.stringify(summary,null,2));
if(runs.some(run=>run.status!==0)) process.exitCode=1;
