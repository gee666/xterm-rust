// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Execute original async InputHandler assertions without extracting/replacing bodies.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync,readFileSync,writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root=resolve(import.meta.dirname,'..');
const large=process.argv.includes('--large'),expected=large?1:3;
const scope=large?'Original bounded parse-buffer assertion':'Three original InputHandler async cases';
const receipt=resolve(root,large?'results/input-handler-async/large-baseline.json':'results/input-handler-async/baseline.json'),startedAt=new Date().toISOString();
mkdirSync(resolve(root,'results/input-handler-async'),{recursive:true});
writeFileSync(receipt,JSON.stringify({status:'incomplete',startedAt,scope,overallEquivalence:'NOT PROVEN'})+'\n');
process.on('uncaughtExceptionMonitor',error=>writeFileSync(receipt,JSON.stringify({status:'failed',startedAt,error:String(error?.stack??error),overallEquivalence:'NOT PROVEN'})+'\n'));
const require=createRequire(resolve(root,'source_repo/package.json'));
const {build}=require('esbuild');const Mocha=require('mocha');
const file='source_repo/src/common/InputHandler.test.ts';
const output=resolve(root,'tmp/input-handler-async/upstream.cjs');
mkdirSync(resolve(root,'tmp/input-handler-async'),{recursive:true});
const bundle=await build({absWorkingDir:root,entryPoints:[file],outfile:output,bundle:true,platform:'node',format:'cjs',target:'node24',tsconfig:'source_repo/src/common/tsconfig.json',metafile:true,logLevel:'silent'});
const hashes=Object.fromEntries(Object.keys(bundle.metafile.inputs).sort().map(path=>[path,createHash('sha256').update(readFileSync(resolve(root,path))).digest('hex')]));
const mocha=new Mocha({timeout:10000,reporter:'spec',grep:large?'should parse big chunks in smaller subchunks':'InputHandler - async handlers'});mocha.addFile(output);
const outcomes=[];
const failures=await new Promise(resolve=>{const runner=mocha.run(resolve);runner.on('pass',test=>outcomes.push({name:test.fullTitle(),status:'passed'}));runner.on('fail',(test,error)=>outcomes.push({name:test.fullTitle(),status:'failed',error:String(error.stack)}));runner.on('pending',test=>outcomes.push({name:test.fullTitle(),status:'skipped'}));});
const report={status:outcomes.length===expected&&outcomes.every(t=>t.status==='passed')?'passed':'failed',startedAt,completedAt:new Date().toISOString(),scope,overallEquivalence:'NOT PROVEN',command:'node scripts/input-handler-async-baseline.mjs'+(large?' --large':''),node:process.version,platform:process.platform,esbuild:require('esbuild/package.json').version,mocha:require('mocha/package.json').version,sourceHashes:hashes,counts:{discovered:expected,executed:outcomes.filter(t=>t.status!=='skipped').length,passed:outcomes.filter(t=>t.status==='passed').length,failed:failures,skipped:outcomes.filter(t=>t.status==='skipped').length},outcomes,unrun:['Other InputHandler suites are not counted by this focused run','Rust original-case migration','Native Linux/macOS runtime']};
mkdirSync(resolve(root,'results/input-handler-async'),{recursive:true});writeFileSync(receipt,JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report.counts));assert.equal(outcomes.length,expected);assert.equal(failures,0);
