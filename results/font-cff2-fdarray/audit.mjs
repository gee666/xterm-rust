// Read-only scoped source, raw execution and retained compiler verification.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const hash=b=>createHash('sha256').update(b).digest('hex');
const pins=JSON.parse(await fs.readFile('results/font-cff2-fdarray/producer-hashes.json'));
for(const [file,expected]of Object.entries(pins.files))assert.equal(hash(await fs.readFile(file)),expected,file);
const fixtureBytes=await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff2-fdarray.json'),fixture=JSON.parse(fixtureBytes);
assert.equal(fixture.cases.length,860);
assert.equal(fixture.cases.filter(c=>!c.presenceOnly).length,852);
assert.equal(fixture.cases.filter(c=>c.presenceOnly).length,8);
assert.equal(fixture.cases.filter(c=>c.outcome.message==='This is a CFF2 font, but FDArray information is missing').length,188);
const source=JSON.parse(await fs.readFile('results/font-cff2-fdarray/source-browser.json'));
assert.equal(source.status,'passed');assert.equal(source.browsers.length,3);assert.equal(source.fixtureSha256,hash(fixtureBytes));
for(const b of source.browsers){assert.equal(b.status,'passed');assert.equal(b.constructors,860);assert.equal(b.addonCases,4);assert.deepEqual(b.addonRaw.debug[1],{name:'Error',message:'This is a CFF2 font, but FDArray information is missing'});}
const raw=JSON.parse(await fs.readFile('results/font-cff2-fdarray/rust-browser.json'));
assert.equal(raw.status,'passed');assert.equal(raw.browsers.length,3);
assert.equal(hash(await fs.readFile(raw.artifacts.compiler.path)),raw.artifacts.compiler.sha256);
assert.equal(hash(await fs.readFile('Cargo.lock')),raw.artifacts.cargoLockSha256);
assert.equal(hash(await fs.readFile('crates/xterm-font-addons/tests/font_cff2_fdarray_browser.rs')),raw.artifacts.testSourceSha256);
for(const b of raw.browsers){assert.equal(b.status,'passed');assert.deepEqual(b.counts,{passed:2,failed:0,ignored:0,filtered:0});assert.equal(b.tests.length,2);assert.deepEqual(b.errors,[]);}
const negative=JSON.parse(await fs.readFile('results/font-cff2-fdarray/negative.json'));
assert.equal(negative.status,'passed');assert.equal(negative.mismatches,188);assert.equal(negative.bypassedSuccesses,128);assert.equal(negative.laterHmtxErrors,60);
assert.equal(negative.originalSha256,negative.restoredSha256);assert.equal(negative.restoredSha256,pins.files['crates/xterm-addons/src/ligatures/font_cff2_dict.rs']);
const native=await fs.readFile('results/font-cff2-fdarray/native.log','utf8'),summaries=[...native.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
assert.equal(summaries.length,48);assert.equal(summaries.reduce((n,m)=>n+Number(m[1]),0),208);assert(summaries.every(m=>m[2]==='0'&&m[3]==='0'));
for(const file of ['clippy-native.log','clippy-wasm.log','dependents-native.log','dependents-wasm.log']){const text=await fs.readFile(`results/font-cff2-fdarray/${file}`,'utf8');assert(text.includes('Finished `dev` profile'),file);assert(!text.includes('warning:'),file);}
for(const p of ['xterm-core','xterm-addons','xterm-web','xterm-wasm-support','xterm-font-addons']){const text=await fs.readFile(`results/font-cff2-fdarray/fmt-${p}.log`,'utf8');assert.equal(text.trim(),'');}
assert((await fs.readFile('results/font-cff2-fdarray/fmt-workspace.log','utf8')).includes('os error 206'));
if(process.argv.includes('--cached')){
 const cached=JSON.parse(await fs.readFile('results/font-cff2-fdarray/cached-test-pins.json'));
 for(const [file,expected]of Object.entries({...cached.files,...cached.sources}))assert.equal(hash(await fs.readFile(file)),expected,file);
}
const manifest=JSON.parse(await fs.readFile('docs/baseline/reference-manifest.json'));
const roots=[process.env.REFERENCE_ROOT??'P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo'];
if(!process.argv.includes('--original-only'))roots.push('source_repo');
for(const root of roots)for(const entry of manifest)assert.equal(hash(await fs.readFile(`${root}/${entry.path}`)),entry.sha256,`${root}/${entry.path}`);
console.log(JSON.stringify({status:'passed',sourcePins:Object.keys(pins.files).length,referenceFilesEachRoot:manifest.length,referenceRoots:roots.length,compilerSha256:raw.artifacts.compiler.sha256,sourceConstructorsEachBrowser:860,completeRustConstructorMatches:852,limitedPresenceObservations:8,sourceAddonLoadsEach:4,actualWasmFunctionsEach:2,nativeAddonFunctions:208,exactFullWorkspaceFmt:'FAILED OS206',overallEquivalence:'NOT PROVEN'}));
