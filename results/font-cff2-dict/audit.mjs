// Read-only source, fixture, retained compiler artifact and reference audit.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const hash=b=>createHash('sha256').update(b).digest('hex');
const pins=JSON.parse(await fs.readFile('results/font-cff2-dict/producer-hashes.json','utf8'));
for(const [file,expected]of Object.entries(pins.files))assert.equal(hash(await fs.readFile(file)),expected,file);
const actual=JSON.parse(await fs.readFile('results/font-cff2-dict/rust-browser.json','utf8'));
assert.equal(actual.status,'passed');
assert.equal(hash(await fs.readFile(actual.artifacts.compiler.path)),actual.artifacts.compiler.sha256);
assert.equal(hash(await fs.readFile('crates/xterm-font-addons/tests/font_cff2_dict_browser.rs')),actual.artifacts.testSourceSha256);
assert.equal(hash(await fs.readFile('Cargo.lock')),actual.artifacts.cargoLockSha256);
for(const b of actual.browsers){assert.equal(b.counts.passed,2);assert.equal(b.counts.failed,0);assert.equal(b.counts.ignored,0);assert.equal(b.status,'passed');}
const baseline=JSON.parse(await fs.readFile('results/font-cff2-dict/source-browser.json','utf8'));
assert.equal(baseline.status,'passed');assert.equal(baseline.browsers.length,3);
for(const b of baseline.browsers){assert.equal(b.constructors,1208);assert.equal(b.addonCases,4);assert.equal(b.status,'passed');assert.deepEqual(b.addonRaw.debug[1],{name:'Error',message:'Invalid b0 31'});}
const original=process.env.REFERENCE_ROOT??'P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo';
const manifest=JSON.parse(await fs.readFile('docs/baseline/reference-manifest.json','utf8'));
for(const root of [original,'source_repo'])for(const entry of manifest)assert.equal(hash(await fs.readFile(`${root}/${entry.path}`)),entry.sha256,`${root}/${entry.path}`);
console.log(JSON.stringify({status:'passed',sourcePins:Object.keys(pins.files).length,referenceFiles:manifest.length,compilerSha256:actual.artifacts.compiler.sha256,sourceConstructorsEachBrowser:1208,actualWasmFunctionsEachBrowser:2,overallEquivalence:'NOT PROVEN'}));
