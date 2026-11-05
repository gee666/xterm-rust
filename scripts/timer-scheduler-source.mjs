// Test-only oracle compiled from the intact upstream file.
import {createRequire} from 'node:module';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'..'),require=createRequire(import.meta.url);
const args=process.argv.slice(2);
if(args.length>1||args.some(arg=>arg!=='--write'))throw new Error('Usage: node scripts/timer-scheduler-source.mjs [--write]');
const esbuild=require(process.env.ESBUILD_MODULE??resolve(root,'source_repo/node_modules/esbuild'));
const path='source_repo/src/browser/TimeBasedDebouncer.ts';
const source=await readFile(resolve(root,path)),hash=createHash('sha256').update(source).digest('hex');
const compiled=esbuild.buildSync({entryPoints:[resolve(root,path)],bundle:true,write:false,format:'iife',globalName:'Source',target:'es2022'}).outputFiles[0].text;
const cases=await readFile(resolve(root,'crates/xterm-web/tests/timer_scheduler_cases.js'),'utf8');
// A with environment supplies the virtual browser global without rewriting
// source expressions. Bare clearTimeout retains its Window receiver and its
// property lookup still precedes evaluation of the pending-ID argument.
const factory='with(host){'+compiled+';return Source.TimeBasedDebouncer;}';
const fixture='// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.\n// Generated test-only intact source oracle. SHA-256: '+hash+'\nconst sourceModel=new Function("host",'+JSON.stringify(factory)+');\n'+cases;
const output=resolve(root,'crates/xterm-web/tests/timer_scheduler_oracle.js');
if(args[0]==='--write')await writeFile(output,fixture);
else if(await readFile(output,'utf8')!==fixture)throw new Error('TimeBasedDebouncer oracle differs from unchanged source');
const oracle=await import('data:text/javascript;base64,'+Buffer.from(fixture).toString('base64'));
console.log(JSON.stringify({source:path,sha256:hash,programs:oracle.runTimerSchedulerCases(oracle.sourceRun),rawErrorPrograms:oracle.runRawTimerSchedulerCases(oracle.sourceRun),scope:'source-only corpus check, not Rust/browser evidence'}));
