// Read-only scoped evidence verification. Run from the producer repository root.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const pins=JSON.parse(await fs.readFile('results/font-gsub-refs/producer-hashes.json','utf8'));
for(const [file,expected]of Object.entries(pins))assert.equal(hash(await fs.readFile(file)),expected,file);
const manifest=JSON.parse(await fs.readFile('docs/baseline/reference-manifest.json','utf8'));
const source=path.dirname(path.resolve(process.env.SOURCE_PACKAGE_JSON??'source_repo/package.json'));
for(const record of manifest){const bytes=await fs.readFile(path.join(source,record.path));assert.equal(bytes.length,record.bytes,record.path);assert.equal(hash(bytes),record.sha256,record.path);}
const native=JSON.parse(await fs.readFile('results/font-gsub-refs/native-artifact.json','utf8'));
const bytes=await fs.readFile(native.path);assert.equal(bytes.length,native.bytes);assert.equal(hash(bytes),native.sha256);
const browser=JSON.parse(await fs.readFile('results/font-gsub-refs/rust-browser.json','utf8'));
assert.equal(browser.status,'passed');
for(const record of Object.values(browser.artifacts).filter(value=>value&&typeof value==='object'&&value.path)){
 const bytes=await fs.readFile(record.path);assert.equal(bytes.length,record.bytes,record.path);assert.equal(hash(bytes),record.sha256,record.path);
}
assert.equal(hash(await fs.readFile('Cargo.lock')),browser.artifacts.cargoLockSha256);
assert.equal(hash(await fs.readFile('crates/xterm-font-addons/tests/font_gsub_refs_browser.rs')),browser.artifacts.testSourceSha256);
for(const engine of browser.browsers){assert.equal(engine.status,'passed');assert.deepEqual(engine.counts,{passed:2,failed:0,ignored:0,filtered:0});}
console.log(JSON.stringify({producerPins:Object.keys(pins).length,referenceFiles:manifest.length,nativeBytes:bytes.length,browserEngines:browser.browsers.length,status:'passed'}));
