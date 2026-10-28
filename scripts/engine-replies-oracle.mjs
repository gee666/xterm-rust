// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Selected original assertion bodies execute without changing source_repo.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {createHash,randomUUID} from 'node:crypto';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {snapshotPaths} from './quality-snapshot.mjs';
process.chdir(resolve(import.meta.dirname,'..'));
const require=createRequire(resolve('source_repo/package.json'));
const directory='results/terminal-engine/replies',attempt=`run-${Date.now()}-${randomUUID()}`;
const commandName='node scripts/engine-replies-oracle.mjs';
mkdirSync(directory,{recursive:true});mkdirSync('tmp/terminal-engine/replies',{recursive:true});
const publish=report=>{
    for(const file of [`${directory}/${attempt}.json`,`${directory}/latest.json`])writeFileSync(file,JSON.stringify(report,null,2)+'\n');
};
publish({attempt,command:commandName,status:'running'});
let stopBuild;
try{
    const esbuild=require('esbuild'),Mocha=require('mocha');stopBuild=esbuild.stop;
    const production=await esbuild.build({entryPoints:['source_repo/src/common/InputHandler.test.ts'],outfile:'tmp/terminal-engine/replies/source.cjs',bundle:true,platform:'node',format:'cjs',target:'node24',tsconfig:'source_repo/src/common/tsconfig.json',metafile:true,logLevel:'silent'});
    const hashes=Object.fromEntries(Object.keys(production.metafile.inputs).sort().map(path=>[path,createHash('sha256').update(readFileSync(path)).digest('hex')]));
    const grep=/InputHandler (DECSC\/DECRC - save and restore cursor|setCursorStyle|windowOptions|XTVERSION|DECSTR) /;
    const mocha=new Mocha({timeout:10000,reporter:'spec',grep});
    mocha.addFile(resolve('tmp/terminal-engine/replies/source.cjs'));
    await mocha.loadFilesAsync();
    const selected=[];mocha.suite.eachTest(test=>{if(grep.test(test.fullTitle()))selected.push(test.fullTitle());});
    const outcomes=[];let failures=0;
    await new Promise(done=>{
        const runner=mocha.run(count=>{failures=count;done();});
        runner.on('pass',test=>outcomes.push({name:test.fullTitle(),status:'passed'}));
        runner.on('fail',(test,error)=>outcomes.push({name:test.fullTitle(),status:'failed',error:error.stack??String(error)}));
        runner.on('pending',test=>outcomes.push({name:test.fullTitle(),status:'skipped'}));
    });
    assert.equal(selected.length,20,'review original-suite selection if source changed');
    const cargo=process.env.CARGO??(process.platform==='win32'?`${process.env.USERPROFILE}/.cargo/bin/cargo.exe`:'cargo');
    const rustc=process.env.RUSTC??(process.platform==='win32'?`${process.env.USERPROFILE}/.cargo/bin/rustc.exe`:'rustc');
    const versions={};let versionFailed=false;
    for(const [tool,program,args] of [['cargo',cargo,['--version']],['rustc',rustc,['-Vv']]]){
        const result=spawnSync(program,args,{encoding:'utf8',timeout:15000,maxBuffer:1024*1024});
        versions[tool]={argv:[program,...args],exitCode:result.status,stdout:result.stdout??'',stderr:result.stderr??'',error:result.error?.message??null};
        if(result.status!==0||result.error)versionFailed=true;
    }
    const rustInputs=['Cargo.toml','Cargo.lock','.cargo','crates/xterm-core/Cargo.toml','crates/xterm-core/src','crates/xterm-core/tests/fixtures','scripts/engine-replies-oracle.mjs','scripts/quality-snapshot.mjs'];
    const before=await snapshotPaths(process.cwd(),rustInputs);
    const argv=['test','-p','xterm-core','--lib','terminal::reply_tests'];
    const run=spawnSync(cargo,argv,{encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024});
    writeFileSync(`${directory}/${attempt}.stdout.log`,run.stdout??'');
    writeFileSync(`${directory}/${attempt}.stderr.log`,run.stderr??'');
    const after=await snapshotPaths(process.cwd(),rustInputs);
    const nativeDiscovered=['reply_tests.rs','reply_regressions.rs'].reduce((count,file)=>count+[...readFileSync(`crates/xterm-core/src/terminal/${file}`,'utf8').matchAll(/#\[test\]/g)].length,0);
    const native=(run.stdout??'').match(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;/);
    const counts=native?{discovered:nativeDiscovered,executed:Number(native[1])+Number(native[2]),passed:Number(native[1]),failed:Number(native[2]),skipped:Number(native[3]),unrun:nativeDiscovered-Number(native[1])-Number(native[2])-Number(native[3])}:{discovered:nativeDiscovered,executed:0,passed:0,failed:0,skipped:0,unrun:nativeDiscovered};
    const nativeComplete=run.status===0&&native&&counts.executed>=20&&counts.unrun===0&&!run.error;
    const sourceMismatches=Object.entries(hashes).filter(([path,hash])=>createHash('sha256').update(readFileSync(path)).digest('hex')!==hash).map(([path])=>path);
    const passed=outcomes.filter(row=>row.status==='passed').length,failed=outcomes.filter(row=>row.status==='failed').length,skipped=outcomes.filter(row=>row.status==='skipped').length;
    const unchanged=before.sha256===after.sha256;
    const report={attempt,command:commandName,node:process.version,platform:`${process.platform}-${process.arch}`,versions,sourceHashes:hashes,sourceMismatches,
        original:{discovered:selected.length,executed:passed+failed,passed,failed,skipped,unrun:selected.length-outcomes.length,outcomes},
        rust:{command:[cargo,...argv].join(' '),exitCode:run.status,error:run.error?.message??null,...counts,mappedSourceCases:20,directedCases:nativeDiscovered-20,inputSnapshot:{before,after,unchanged}},
        status:failures===0&&sourceMismatches.length===0&&nativeComplete&&unchanged&&!versionFailed?'passed':'failed',
        scope:'20 selected unchanged original save/restore/style/window/XTVERSION/DECSTR cases and actual CoreTerminal Rust counterparts plus separately counted directed regressions. Scoped input stability is not whole-workspace or whole-port proof.'};
    publish(report);
    console.log(JSON.stringify({original:selected.length,passed,failed,rust:counts,rustExit:run.status,scopedRustInputsUnchanged:unchanged,status:report.status}));
    process.exitCode=report.status==='passed'?0:1;
}catch(error){
    publish({attempt,command:commandName,status:'failed',original:null,rust:null,error:{name:error.name,message:error.message,stack:error.stack},scope:'Attempt did not complete; no previous passing result is reused.'});
    throw error;
}finally{stopBuild?.();}
