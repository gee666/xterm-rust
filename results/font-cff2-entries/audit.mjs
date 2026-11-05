// Read-only source, retained compiler, raw receipts and reference validation.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const hash=b=>createHash('sha256').update(b).digest('hex');
const pins=JSON.parse(await fs.readFile('results/font-cff2-entries/producer-hashes.json'));
for(const [file,expected]of Object.entries(pins.files))assert.equal(hash(await fs.readFile(file)),expected,file);
const raw=JSON.parse(await fs.readFile('results/font-cff2-entries/rust-browser.json'));
assert.equal(raw.status,'passed');assert.equal(raw.browsers.length,3);
assert.equal(hash(await fs.readFile(raw.artifacts.compiler.path)),raw.artifacts.compiler.sha256);
assert.equal(hash(await fs.readFile('Cargo.lock')),raw.artifacts.cargoLockSha256);
assert.equal(hash(await fs.readFile('crates/xterm-font-addons/tests/font_cff2_entries_browser.rs')),raw.artifacts.testSourceSha256);
for(const b of raw.browsers){assert.equal(b.status,'passed');assert.deepEqual(b.counts,{passed:2,failed:0,ignored:0,filtered:0});assert.equal(b.tests.length,2);}
const baseline=JSON.parse(await fs.readFile('results/font-cff2-entries/source-browser.json'));
assert.equal(baseline.status,'passed');assert.equal(baseline.browsers.length,3);
for(const b of baseline.browsers){assert.equal(b.status,'passed');assert.equal(b.constructors,898);assert.equal(b.addonCases,4);assert.deepEqual(b.addonRaw.debug[1],{name:'Error',message:'Object [object Object] already has key 0'});}
const fixture=JSON.parse(await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff2-entries.json'));
assert.equal(fixture.cases.length,898);assert.equal(hash(await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff2-entries.json')),baseline.fixtureSha256);
const negative=JSON.parse(await fs.readFile('results/font-cff2-entries/negative.json'));
assert.equal(negative.status,'passed');assert.equal(negative.mismatches,16);assert.equal(negative.originalSha256,negative.restoredSha256);
assert.equal(negative.restoredSha256,pins.files['crates/xterm-addons/src/ligatures/font_cff2_dict.rs']);
const log=await fs.readFile('results/font-cff2-entries/workspace-native-final.log','utf8'),s=[...log.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
assert.equal(s.length,487);assert.equal(s.reduce((n,m)=>n+Number(m[1]),0),2170);assert(s.every(m=>m[2]==='0'&&m[3]==='0'));
for(const file of ['clippy-workspace-native.log','clippy-wasm-final.log','dependents-native.log','dependents-wasm.log']){const t=await fs.readFile(`results/font-cff2-entries/${file}`,'utf8');assert(t.includes('Finished `dev` profile'),file);assert(!t.includes('warning:'),file);}
const manifest=JSON.parse(await fs.readFile('docs/baseline/reference-manifest.json'));
for(const root of [process.env.REFERENCE_ROOT??'P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo','source_repo'])for(const e of manifest)assert.equal(hash(await fs.readFile(`${root}/${e.path}`)),e.sha256,`${root}/${e.path}`);
console.log(JSON.stringify({status:'passed',sourcePins:Object.keys(pins.files).length,referenceFiles:manifest.length,compilerSha256:raw.artifacts.compiler.sha256,sourceConstructorsEachBrowser:898,sourceAddonLoadsEach:4,actualWasmFunctionsEach:2,nativeWorkspacePasses:2170,overallEquivalence:'NOT PROVEN'}));
