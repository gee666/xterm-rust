// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// --record is a producer snapshot, not an independent review. Default checks
// durable/reference pins; --runtime also checks generated binaries before cleanup.
import {readFile,writeFile,readdir,lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import assert from 'node:assert/strict';
const dir='results/webgl-frame',root=process.cwd();
if(!process.env.SOURCE_ROOT)throw Error('SOURCE_ROOT must name the unchanged supplied reference');
const source=path.resolve(process.env.SOURCE_ROOT);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const json=async file=>JSON.parse(await readFile(file,'utf8'));
const pin=async file=>({path:file.replaceAll('\\','/'),sha256:hash(await readFile(file))});
const verify=async entry=>assert.equal(hash(await readFile(path.resolve(root,entry.path))),entry.sha256,entry.path);
const reference=await json('docs/baseline/reference-manifest.json');
for(const entry of reference)await verify({path:path.join(source,entry.path),sha256:entry.sha256});
const oracle=await json(`${dir}/oracle.json`);
assert.equal(oracle.inputs.length,126);
for(const entry of oracle.inputs) {
  assert.ok(reference.some(r=>r.path.replaceAll('\\','/')===entry.path&&r.sha256===entry.sha256),entry.path);
  await verify({path:path.join(source,entry.path),sha256:entry.sha256});
}
await verify(oracle.wrapper);
await verify({path:'scripts/webgl-frame-oracle.mjs',sha256:oracle.generatorSha256});
const reports=[];
for(const phase of ['corrected','final']) {
  const report=await json(`${dir}/browser-${phase}.json`);reports.push(report);
  assert.equal(report.status,'passed');
  assert.deepEqual(report.browsers.map(b=>b.name),['chromium','firefox','webkit']);
  for(const b of report.browsers) {
    assert.deepEqual(b.counts,{passed:11,failed:0,ignored:0,filtered:0});
    assert.deepEqual(b.errors,[]);
    assert.equal(b.console.filter(e=>e.type==='error').length,0);
    assert.equal(b.console.filter(e=>/too many active/i.test(e.text)).length,0);
  }
  assert.equal(report.artifacts.testSourceSha256,(await pin('crates/xterm-web/tests/wasm_webgl_frame.rs')).sha256);
}
assert.equal(reports[0].artifacts.wasm.sha256,reports[1].artifacts.wasm.sha256);
assert.equal(reports[0].artifacts.bindings.sha256,reports[1].artifacts.bindings.sha256);
const negative=await json(`${dir}/negative.json`),rejected=await json(`${dir}/browser-negative.json`);
assert.equal(negative.restored,true);assert.notEqual(negative.originalSha256,negative.mutantSha256);
await verify({path:negative.source,sha256:negative.originalSha256});
assert.notEqual(rejected.artifacts.wasm.sha256,reports[1].artifacts.wasm.sha256);
for(const b of rejected.browsers) {
  assert.deepEqual(b.counts,{passed:2,failed:9,ignored:0,filtered:0});assert.deepEqual(b.errors,[]);
}
const quota = await json(`${dir}/browser-quota-final.json`);
assert.equal((await pin(`${dir}/quota-test.rs.txt`)).sha256,quota.artifacts.testSourceSha256);
const quotaOracle = await json(`${dir}/quota-oracle.json`);
assert.equal((await pin(`${dir}/quota-source.ts.txt`)).sha256,quotaOracle.wrapper.sha256);
const initial = await json(`${dir}/browser-initial-failed.json`);
assert.equal((await pin(`${dir}/initial-test.rs.txt`)).sha256,initial.artifacts.testSourceSha256);
const verification=`${dir}/verification.json`;
async function tree(folder,files) {
  for(const entry of await readdir(folder,{withFileTypes:true})) {
    const file=path.join(folder,entry.name);assert.equal((await lstat(file)).isSymbolicLink(),false,file);
    if(entry.isDirectory())await tree(file,files);else if(entry.isFile())files.push(file);
  }
}
if(process.argv.includes('--record')) {
  const inputs=['crates/xterm-web/src/webgl/browser_frame.rs','crates/xterm-web/src/webgl/browser_frame_host.rs',
    'crates/xterm-web/src/webgl/browser_frame_lifecycle.rs','crates/xterm-web/src/webgl/mod.rs',
    'crates/xterm-web/tests/wasm_webgl_frame.rs','crates/xterm-web/tests/support/webgl_frame.rs',
    'scripts/webgl-frame-oracle.mjs','scripts/webgl-frame-negative.mjs','scripts/webgl-frame-audit.mjs',
    'scripts/support/webgl-frame-source.ts','docs/webgl-browser-frame.md','.pi/swarm/skills/webgl-frame-check/SKILL.md'];
  const receipts=[];await tree(dir,receipts);
  const runtime=new Set([oracle.output.path]);
  for(const file of receipts.filter(f=>/browser-.+\.json$/.test(f))) {
    const report=await json(file);if(!report.artifacts)continue;
    for(const k of ['compiler','wasm','bindings']) {await verify(report.artifacts[k]);runtime.add(path.resolve(root,report.artifacts[k].path));}
    const files=[];await tree(path.dirname(report.artifacts.bindings.path),files);
    for(const f of files)runtime.add(f);
  }
  const snapshot={referenceHashes:reference.length,sourceInputs:oracle.inputs.length,
    durable:await Promise.all([...inputs,...receipts.filter(f=>path.resolve(f)!==path.resolve(verification))].sort().map(pin)),
    runtime:await Promise.all([...runtime].sort().map(pin)),
    scope:'Typed actual frame component only. 9 unchanged-source scenarios/16 framebuffer+model comparisons, 2 Rust lifecycle groups, 11 functions each3engines. Rawhost/fullconstructor/Addon/Root/full suites and context540 initial burstGC stay306. Equivalence NOT PROVEN.'};
  await writeFile(verification,JSON.stringify(snapshot,null,2)+'\n');
}
const snapshot=await json(verification);
for(const p of snapshot.durable)await verify(p);
if(process.argv.includes('--runtime'))for(const p of snapshot.runtime)await verify(p);
console.log(`PASS ${reference.length} original/${oracle.inputs.length} source/${snapshot.durable.length} durable${process.argv.includes('--runtime')?`/${snapshot.runtime.length} runtime`:''}; final11x3/negative2P9F each3`);
