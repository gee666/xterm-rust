import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile, readdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { runCommand } from './quality-process.mjs';

const root=resolve(import.meta.dirname,'..');
const checker=await readFile(resolve(root,'scripts/check-public-adapters.mjs'),'utf8');
let ordinal=0;
async function probe(helper) {
  const directory=resolve(root,'tmp/public-adapter-bootstrap-tests',`${Date.now()}-${ordinal++}`);
  await mkdir(resolve(directory,'scripts'),{recursive:true});
  await mkdir(resolve(directory,'results/public-adapters'),{recursive:true});
  await writeFile(resolve(directory,'scripts/check-public-adapters.mjs'),checker);
  if(helper!==undefined) await writeFile(resolve(directory,'scripts/quality-process.mjs'),helper);
  const latest=resolve(directory,'results/public-adapters/verification.json');
  await writeFile(latest,JSON.stringify({id:'old-pass',status:'passed'}));
  const command=await runCommand(process.execPath,[resolve(directory,'scripts/check-public-adapters.mjs')],{
    cwd:directory,env:process.env,log:resolve(directory,'probe.log'),timeoutMs:10000,
  });
  const report=JSON.parse(await readFile(latest,'utf8'));
  assert.notEqual(report.id,'old-pass');
  assert.notEqual(report.status,'passed');
  assert.equal(command.status,'failed');
  const runs=await readdir(resolve(directory,'results/public-adapters/runs'));
  assert.equal(runs.length,1);
  assert.equal(JSON.parse(await readFile(resolve(directory,'results/public-adapters/runs',runs[0],'report.json'),'utf8')).id,report.id);
  return report;
}

test('missing project helper invalidates old pass before input hashing or commands',async()=>{
  const report=await probe(undefined);
  assert.equal(report.status,'failed');
  assert.match(report.error,/quality-process\.mjs/);
  assert.deepEqual(report.commands,[]);
  assert.equal(report.rust.discovered,null);
  assert.equal(report.rust.unrun,null);
});

test('missing inputs retain a failed receipt without invoking a build',async()=>{
  const report=await probe('export function runCommand(){throw new Error("must not execute a command before input validation");}');
  assert.equal(report.status,'failed');
  assert.match(report.error,/ENOENT/);
  assert.deepEqual(report.commands,[]);
  assert.equal(report.snapshotStable,false);
});

test('unsettled helper bootstrap cannot leave a previous pass as latest',async()=>{
  const report=await probe('await new Promise(()=>{});');
  assert.equal(report.status,'incomplete');
  assert.deepEqual(report.commands,[]);
});
