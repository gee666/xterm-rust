// Run from the producer root. --retained verifies committed proof after cleanup.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const retained=process.argv.includes('--retained');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=async file=>JSON.parse(await fs.readFile(file,'utf8'));
const pins=await read('results/font-gsub-subtables/producer-hashes.json');
for(const [file,expected]of Object.entries(pins))assert.equal(hash(await fs.readFile(file)),expected,file);
const manifest=await read('docs/baseline/reference-manifest.json');
const source=path.dirname(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
for(const record of manifest){const bytes=await fs.readFile(path.join(source,record.path));assert.equal(bytes.length,record.bytes,record.path);assert.equal(hash(bytes),record.sha256,record.path);}
const fixture=await read('crates/xterm-addons/tests/fixtures/font-gsub-subtables.json');
assert.equal(fixture.cases.length,1224);
assert.equal(fixture.cases.filter(c=>c.outcome.status==='ok').length,508);
assert.equal(fixture.cases.filter(c=>c.loadOutcome.status==='ok').length,392);
const sourceBrowser=await read('results/font-gsub-subtables/source-browser.json');
assert.equal(sourceBrowser.status,'passed');
assert.deepEqual(sourceBrowser.browsers.map(b=>b.name),['chromium','firefox','webkit']);
for(const record of sourceBrowser.browsers){assert.equal(record.status,'passed');assert.equal(record.constructors,1224);assert.equal(record.loads,1224);assert.equal(record.addonCases,1224);}
const native=await read('results/font-gsub-subtables/native-artifact.json');
const browser=await read('results/font-gsub-subtables/rust-browser.json');
assert.equal(browser.status,'passed');
assert.deepEqual(browser.browsers.map(b=>b.name),['chromium','firefox','webkit']);
for(const engine of browser.browsers){assert.equal(engine.status,'passed');assert.deepEqual(engine.counts,{passed:2,failed:0,ignored:0,filtered:0});}
assert.equal(hash(await fs.readFile('Cargo.lock')),browser.artifacts.cargoLockSha256);
assert.equal(hash(await fs.readFile('crates/xterm-font-addons/tests/font_gsub_subtables_browser.rs')),browser.artifacts.testSourceSha256);
if(!retained){
 for(const record of [native,...Object.values(browser.artifacts).filter(v=>v&&typeof v==='object'&&v.path)]){
  const bytes=await fs.readFile(record.path);assert.equal(bytes.length,record.bytes,record.path);assert.equal(hash(bytes),record.sha256,record.path);
 }
}
console.log(JSON.stringify({producerPins:Object.keys(pins).length,referenceFiles:manifest.length,cases:1224,browserEngines:3,mode:retained?'retained proof, generated binaries intentionally absent':'live artifact pins',status:'passed'}));
