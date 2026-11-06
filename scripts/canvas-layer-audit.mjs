// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// --record produces pins. The checks below, not pin recording, verify evidence.
import {readFile,writeFile,readdir,lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import assert from 'node:assert/strict';
const dir='results/canvas-layer',original=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const json=async p=>JSON.parse(await readFile(p,'utf8'));
const hash=b=>createHash('sha256').update(b).digest('hex');
const bytes=async p=>{const b=await readFile(p);return p.endsWith('.md')?Buffer.from(b.toString('utf8').replaceAll('\r\n','\n')):b;};
const pin=async p=>({path:p.replaceAll('\\','/'),sha256:hash(await bytes(p))});
const verify=async e=>assert.equal(hash(await bytes(e.path)),e.sha256,e.path);
const manifest=await json('docs/baseline/reference-manifest.json');
for(const e of manifest)assert.equal(hash(await readFile(path.join(original,e.path))),e.sha256,e.path);
const inputs=await json(dir+'/source-inputs.json');
for(const e of inputs.inputs)assert.equal(hash(await readFile(e.reference?path.join(original,e.path):e.path)),e.sha256,e.path);
const final=await json(dir+'/browser-final.json');
assert.equal(final.status,'passed');assert.deepEqual(final.browsers.map(b=>b.name),['chromium','firefox','webkit']);
const names=['link','colors','retained','theme','dpr_reentry','resize','draw_error','reentry','base','font_reentry','transparency_error','constructor_error'];
for(const browser of final.browsers){
 assert.equal(browser.status,'passed');assert.deepEqual(browser.counts,{passed:12,failed:0,ignored:0,filtered:0});
 const found=new Map();
 for(const c of browser.console){
  const m=c.text.match(/^layer-(source|rust) (\S+) (.+)$/);if(!m)continue;
  const key=m[1]+':'+m[2];assert.equal(found.has(key),false,key);found.set(key,JSON.parse(m[3]));
 }
 assert.equal(found.size,24);
 for(const name of names)assert.deepEqual(found.get('source:'+name),found.get('rust:'+name),name);
 const snapshots=found.get('rust:link').filter(row=>row[0]==='snapshot');
 assert.equal(snapshots.length,4);assert.notEqual(snapshots[0].at(-1),snapshots[1].at(-1));
 assert.equal(snapshots[1].at(-1),snapshots[3].at(-1));
 const theme=found.get('rust:theme');
 assert.ok(theme.some(row=>row[0]==='note'&&row[1]==='shared:true'));
 assert.deepEqual(theme.filter(row=>row[0]==='note'&&row[1].startsWith('scheduled:')).map(row=>row[1]),['scheduled:1','scheduled:2','scheduled:2']);
}
const negative=await json(dir+'/negative.json');
assert.equal(negative.expectedFailureObserved,true);assert.equal(negative.restored,true);
assert.equal(negative.originalSha256,negative.restoredSha256);assert.ok(negative.counts.failed>0);
assert.equal(negative.counts.ignored,0);assert.equal(negative.counts.filtered,0);
const phase=await json(dir+'/phase-negative.json');
assert.equal(phase.expectedFailureObserved,true);assert.equal(phase.restored,true);
assert.deepEqual(phase.counts,{passed:10,failed:2,ignored:0,filtered:0});
for(const e of phase.files){assert.equal(e.originalSha256,e.restoredSha256);assert.equal((await pin(e.path)).sha256,e.restoredSha256);}
const phaseReport=await json(dir+'/browser-phase-negative.json');
const phaseRows=new Map(phaseReport.browsers[0].console.map(c=>c.text.match(/^layer-(source|rust) (\S+) (.+)$/)).filter(Boolean).map(m=>[m[1]+':'+m[2],JSON.parse(m[3])]));
for(const name of ['dpr_reentry','font_reentry'])assert.notDeepEqual(phaseRows.get('source:'+name),phaseRows.get('rust:'+name),name);
assert.equal((await pin('crates/xterm-web/src/webgl/link_layer.rs')).sha256,negative.restoredSha256);
assert.equal(final.artifacts.testSourceSha256,(await pin('crates/xterm-web/tests/wasm_webgl_canvas_layer.rs')).sha256);
const index=dir+'/verification.json';
if(process.argv.includes('--record')){
 const source=[
  'crates/xterm-web/src/webgl/canvas_layer.rs','crates/xterm-web/src/webgl/canvas_layer_text.rs',
  'crates/xterm-web/src/webgl/layer_services.rs','crates/xterm-web/src/webgl/link_layer.rs',
  'crates/xterm-web/tests/wasm_webgl_canvas_layer.rs','crates/xterm-web/tests/support/canvas_layer.rs',
  'crates/xterm-web/tests/canvas_layer_probe.js','crates/xterm-web/tests/reference/canvas-layer.mjs',
  'scripts/canvas-layer-oracle.mjs','scripts/canvas-layer-negative.mjs','scripts/canvas-layer-audit.mjs',
  'docs/webgl-canvas-layers.md','docs/webgl-port.md','.pi/swarm/skills/canvas-layer-check/SKILL.md'];
 const evidence=(await readdir(dir)).filter(n=>/\.(json|log|txt)$/.test(n)&&!['verification.json','cleanup.json'].includes(n)).sort().map(n=>dir+'/'+n);
 const runtime=new Set();
 async function tree(root){for(const e of await readdir(root,{withFileTypes:true})){
  const p=path.join(root,e.name);assert.equal((await lstat(p)).isSymbolicLink(),false,p);
  if(e.isDirectory())await tree(p);else if(e.isFile())runtime.add(p);
 }}
 for(const p of evidence.filter(p=>/browser-(initial|negative|phase-negative|final)\.json$/.test(p))){
  const report=await json(p);
  for(const k of ['wasm','bindings']){await verify(report.artifacts[k]);runtime.add(report.artifacts[k].path);}
  await tree(path.dirname(report.artifacts.bindings.path));
 }
 await verify(final.artifacts.compiler);runtime.add(final.artifacts.compiler.path);
 runtime.add('tmp/canvas-layer/source.js');
 assert.equal(hash(await readFile('tmp/canvas-layer/source.js')),inputs.sha256);
 await writeFile(index,JSON.stringify({originalHashes:manifest.length,sourceInputs:inputs.inputs.length,
  durable:await Promise.all([...source,...evidence].map(pin)),runtime:await Promise.all([...runtime].sort().map(pin)),
  scope:'Typed actual canvas base/link layers only. WebglRenderer/Addon/Root/full suites remain306. Full equivalence NOT PROVEN.'},null,2)+'\n');
}
const record=await json(index);
for(const e of record.durable)await verify(e);
if(process.argv.includes('--runtime'))for(const e of record.runtime)await verify(e);
console.log(`PASS ${manifest.length} original hashes, ${inputs.inputs.length} source inputs, ${record.durable.length} durable pins, final12x3 and rejected production mutant${process.argv.includes('--runtime')?', '+record.runtime.length+' runtime pins':''}`);
