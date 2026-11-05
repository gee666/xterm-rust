// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// --record captures pins, but never substitutes for the assertions below.
import {readFile,writeFile,readdir,lstat} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import assert from 'node:assert/strict';
const directory='results/browser-atlas-cache';
const original=path.resolve(process.env.SOURCE_ROOT??'source_repo');
const json=async file=>JSON.parse(await readFile(file,'utf8'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const bytes=async file=>{const raw=await readFile(file);return file.endsWith('.md')?Buffer.from(raw.toString('utf8').replaceAll('\r\n','\n')):raw;};
const pin=async file=>({path:file.replaceAll('\\','/'),sha256:hash(await bytes(file))});
const verify=async entry=>assert.equal(hash(await bytes(entry.path)),entry.sha256,entry.path);
const manifest=await json('docs/baseline/reference-manifest.json');
for(const entry of manifest)assert.equal(hash(await readFile(path.join(original,entry.path))),entry.sha256,entry.path);
const inputs=await json(`${directory}/source-inputs.json`);
for(const entry of inputs.inputs)assert.equal(hash(await readFile(path.join(original,entry.path))),entry.sha256,entry.path);
await verify(inputs.wrapper);
const final=await json(`${directory}/browser-final.json`);
assert.equal(final.status,'passed');
assert.deepEqual(final.browsers.map(b=>b.name),['chromium','firefox','webkit']);
const scenarios=['sharing','move','failure','creation_failure','services_failure','reentry','create_reentry','canvas_reentry'];
for(const browser of final.browsers){
  assert.equal(browser.status,'passed');
  assert.deepEqual(browser.counts,{passed:9,failed:0,ignored:0,filtered:0});
  const rows=browser.console.map(c=>c.text).filter(t=>t.startsWith('cache-source ')||t.startsWith('cache-rust '));
  const found=new Map();
  for(const text of rows){
    const match=text.match(/^cache-(source|rust) (\S+) (.+)$/);assert.ok(match,text);
    const key=`${match[1]}:${match[2]}`;assert.equal(found.has(key),false,key);
    found.set(key,JSON.parse(match[3]));
  }
  assert.equal(found.size,16);
  for(const scenario of scenarios)assert.deepEqual(found.get(`rust:${scenario}`),found.get(`source:${scenario}`),scenario);
  const sharing=found.get('rust:sharing');
  assert.notEqual(sharing.pixels[0],sharing.pixels[1]);
  assert.equal(sharing.pixels[1],sharing.pixels[2]);
  assert.deepEqual(found.get('rust:canvas_reentry').versions,[0,-1,0]);
}
const source='crates/xterm-web/src/webgl/browser_cache.rs';
const negative=await json(`${directory}/negative.json`);
assert.equal(negative.expectedFailureObserved,true);
assert.equal(negative.restored,true);
assert.equal(negative.originalSha256,negative.restoredSha256);
assert.equal((await pin(source)).sha256,negative.restoredSha256);
assert.ok(negative.counts.failed>0);
assert.equal(negative.counts.ignored,0);assert.equal(negative.counts.filtered,0);
const test='crates/xterm-web/tests/wasm_webgl_browser_cache.rs';
assert.equal(final.artifacts.testSourceSha256,(await pin(test)).sha256);
const index=`${directory}/verification.json`;
if(process.argv.includes('--record')){
  const sourceFiles=[source,test,'crates/xterm-web/src/webgl/mod.rs','crates/xterm-web/src/terminal/runtime.rs',
    'crates/xterm-web/tests/support/browser_atlas_cache.rs','crates/xterm-web/tests/browser_atlas_cache_probe.js',
    'crates/xterm-web/tests/reference/browser-atlas-cache.mjs','scripts/browser-atlas-cache-oracle.mjs',
    'scripts/browser-atlas-cache-negative.mjs','scripts/browser-atlas-cache-audit.mjs',
    'docs/webgl-browser-cache.md','docs/webgl-port.md','.pi/swarm/skills/browser-atlas-cache-check/SKILL.md'];
  const evidence=(await readdir(directory)).filter(n=>/\.(json|log|txt)$/.test(n)&&n!=='verification.json'&&n!=='cleanup.json').sort().map(n=>`${directory}/${n}`);
  const runtime=new Set();
  async function tree(dir){for(const entry of await readdir(dir,{withFileTypes:true})){
    const file=path.join(dir,entry.name);assert.equal((await lstat(file)).isSymbolicLink(),false,file);
    if(entry.isDirectory())await tree(file);else if(entry.isFile())runtime.add(file);
  }}
  for(const name of evidence.filter(n=>/browser-(initial|negative|final)\.json$/.test(n))){
    const report=await json(name);
    for(const key of ['wasm','bindings']){
      const entry=report.artifacts[key];await verify(entry);runtime.add(entry.path);
    }
    await tree(path.dirname(report.artifacts.bindings.path));
  }
  await verify(final.artifacts.compiler);runtime.add(final.artifacts.compiler.path);
  const native=(await readFile(`${directory}/native.log`,'utf8')).match(/target[\\/]debug[\\/]deps[\\/]webgl-[0-9a-f]+\.exe/);
  assert.ok(native);runtime.add(native[0].replaceAll('\\','/'));
  runtime.add('tmp/browser-atlas-cache/source.js');
  assert.equal(hash(await readFile('tmp/browser-atlas-cache/source.js')),inputs.bundleSha256);
  await writeFile(index,JSON.stringify({originalHashes:manifest.length,sourceInputs:inputs.inputs.length,
    durable:await Promise.all([...sourceFiles,...evidence].map(pin)),runtime:await Promise.all([...runtime].sort().map(pin)),
    scope:'Typed real shared browser atlas cache only. Renderer/Addon/Root/full suites remain306; full equivalence NOT PROVEN.'},null,2)+'\n');
}
const record=await json(index);
for(const entry of record.durable)await verify(entry);
if(process.argv.includes('--runtime'))for(const entry of record.runtime)await verify(entry);
console.log(`PASS ${manifest.length} original hashes, ${inputs.inputs.length} source inputs, ${record.durable.length} durable pins, final9x3 incl8source scenarios and rejected shared-clock mutant${process.argv.includes('--runtime')?`, ${record.runtime.length} runtime pins`:''}`);
