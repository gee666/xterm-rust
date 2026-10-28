// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Execute original async InputHandler assertions without extracting/replacing bodies.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { mkdirSync,readFileSync,writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
const root=resolve(import.meta.dirname,'..');
const require=createRequire(resolve(root,'source_repo/package.json'));
const {build}=require('esbuild');const Mocha=require('mocha');
const file='source_repo/src/common/InputHandler.test.ts';
const output=resolve(root,'tmp/input-handler-async/upstream.cjs');
mkdirSync(resolve(root,'tmp/input-handler-async'),{recursive:true});
const bundle=await build({absWorkingDir:root,entryPoints:[file],outfile:output,bundle:true,platform:'node',format:'cjs',target:'node24',tsconfig:'source_repo/src/common/tsconfig.json',metafile:true,logLevel:'silent'});
const hashes=Object.fromEntries(Object.keys(bundle.metafile.inputs).sort().map(path=>[path,createHash('sha256').update(readFileSync(resolve(root,path))).digest('hex')]));
const mocha=new Mocha({timeout:10000,reporter:'spec',grep:'InputHandler - async handlers'});mocha.addFile(output);
const outcomes=[];
const failures=await new Promise(resolve=>{const runner=mocha.run(resolve);runner.on('pass',test=>outcomes.push({name:test.fullTitle(),status:'passed'}));runner.on('fail',(test,error)=>outcomes.push({name:test.fullTitle(),status:'failed',error:String(error.stack)}));runner.on('pending',test=>outcomes.push({name:test.fullTitle(),status:'skipped'}));});
const report={scope:'Only the three original InputHandler - async handlers cases',overallEquivalence:'NOT PROVEN',command:'node scripts/input-handler-async-baseline.mjs',node:process.version,platform:process.platform,esbuild:require('esbuild/package.json').version,mocha:require('mocha/package.json').version,sourceHashes:hashes,counts:{discovered:3,executed:outcomes.filter(t=>t.status!=='skipped').length,passed:outcomes.filter(t=>t.status==='passed').length,failed:failures,skipped:outcomes.filter(t=>t.status==='skipped').length},outcomes,unrun:['Other InputHandler suites are not counted by this focused run','Rust original-case migration','Native Linux/macOS runtime']};
mkdirSync(resolve(root,'results/input-handler-async'),{recursive:true});writeFileSync(resolve(root,'results/input-handler-async/baseline.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report.counts));assert.equal(outcomes.length,3);assert.equal(failures,0);
