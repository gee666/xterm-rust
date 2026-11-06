// Read-only scoped receipts, immutable compiler and reference verification.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const out='results/font-cff2-index',sha=b=>createHash('sha256').update(b).digest('hex');
const json=async p=>JSON.parse(await fs.readFile(p));
const pins=await json(out+'/producer-hashes.json');
for(const [file,expected]of Object.entries(pins.files))assert.equal(sha(await fs.readFile(file)),expected,file);
const logs=await json(out+'/raw-log-bytes.json');
for(const [file,pin]of Object.entries(logs.files)){const b=Buffer.from(pin.base64,'base64');assert.equal(b.length,pin.bytes);assert.equal(sha(b),pin.sha256);assert.equal(sha(await fs.readFile(out+'/'+file)),pin.normalizedSha256,file);}
const fixtureBytes=await fs.readFile('crates/xterm-addons/tests/fixtures/font-cff2-index.json'),fixture=JSON.parse(fixtureBytes);
assert.equal(fixture.cases.length,464);assert.equal(fixture.cases.filter(c=>c.resourceFailure).length,10);assert.equal(fixture.cases.filter(c=>c.outcome.status==='resourceFailure').length,8);assert.equal(fixture.cases.filter(c=>c.phaseTable).length,450);
for(const generation of ['generation','replay']){const resources=await json(out+'/resource-'+generation+'.json');assert.equal(resources.length,10);for(const r of resources){const b=Buffer.from(r.stderrBase64,'base64');assert.equal(sha(b),r.stderrSha256);assert.match(b.toString(),/FATAL ERROR: (?:Reached heap limit|Ineffective mark-compacts near heap limit)/);assert.equal(r.inputSha256,fixture.cases.find(c=>c.name===r.name).inputSha256);}}
const source=await json(out+'/source-browser.json');assert.equal(source.status,'passed');assert.equal(source.fixtureSha256,sha(fixtureBytes));assert.equal(source.excluded.length,265);assert.equal(source.browsers.length,3);
for(const b of source.browsers){assert.equal(b.status,'passed');assert.equal(b.constructors,199);assert.equal(b.addonCases,4);assert.equal(b.addonConstructorCases.length,4);assert.deepEqual(b.addonRaw.debug[1],{name:b.addonConstructorOutcome.name,message:b.addonConstructorOutcome.message});}
const rust=await json(out+'/rust-browser.json');assert.equal(rust.status,'passed');assert.equal(rust.browsers.length,3);assert.equal(sha(await fs.readFile(rust.artifacts.compiler.path)),rust.artifacts.compiler.sha256);assert.equal(sha(await fs.readFile('Cargo.lock')),rust.artifacts.cargoLockSha256);assert.equal(sha(await fs.readFile('crates/xterm-font-addons/tests/font_cff2_index_browser.rs')),rust.artifacts.testSourceSha256);
for(const b of rust.browsers){assert.equal(b.status,'passed');assert.deepEqual(b.counts,{passed:2,failed:0,ignored:0,filtered:0});assert.equal(b.tests.length,2);assert.deepEqual(b.errors,[]);}
const negative=await json(out+'/negative.json');assert.equal(negative.status,'passed');assert.deepEqual([negative.executedBoundsMismatches,negative.accepted,negative.laterHmtx],[147,119,28]);assert.equal(negative.beforeSha256,negative.restoredSha256);assert.equal(negative.restoredSha256,pins.files['crates/xterm-addons/src/ligatures/font_cff_index.rs']);assert.equal(negative.positiveExitCode,0);
const native=await fs.readFile(out+'/native-final.log','utf8'),summaries=[...native.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];assert.equal(summaries.length,50);assert.equal(summaries.reduce((n,m)=>n+Number(m[1]),0),216);assert(summaries.every(m=>m[2]==='0'&&m[3]==='0'));
for(const name of ['clippy-native-final','clippy-wasm-final','dependents-native','dependents-wasm']){const log=await fs.readFile(out+'/'+name+'.log','utf8');assert(log.includes('Finished `dev` profile'),name);assert(!log.includes('warning:'),name);}
for(const member of ['xterm-core','xterm-addons','xterm-web','xterm-font-addons','xterm-wasm-support'])assert.equal((await fs.readFile(out+'/fmt-'+member+'.log','utf8')).trim(),'');
assert((await fs.readFile(out+'/fmt-workspace.log','utf8')).includes('os error 206'));
if(process.argv.includes('--cached')){const cached=await json(out+'/cached-test-pins.json');for(const [file,expected]of Object.entries(cached.files))assert.equal(sha(await fs.readFile(file)),expected,file);}
if(process.argv.includes('--runtime')){const runtime=await json(out+'/generated-runtime-pins.json');for(const [file,pin]of Object.entries(runtime.files)){const b=await fs.readFile(file);assert.equal(b.length,pin.bytes);assert.equal(sha(b),pin.sha256,file);}}
const reference=await json('docs/baseline/reference-manifest.json'),roots=process.env.REFERENCE_ROOT?[process.env.REFERENCE_ROOT]:[];
if(!process.argv.includes('--original-only')||roots.length===0)roots.push('source_repo');
for(const root of roots){for(const entry of reference)assert.equal(sha(await fs.readFile(path.join(root,entry.path))),entry.sha256,root+'/'+entry.path);}
if(process.env.REFERENCE_ROOT)assert.equal(sha(await fs.readFile(path.join(process.env.REFERENCE_ROOT,'node_modules/opentype.js/dist/opentype.js'))),fixture.sourceSha256);
console.log(JSON.stringify({status:'passed',producerPins:Object.keys(pins.files).length,referenceFilesEachRoot:reference.length,referenceRoots:roots.length,sourceRecipes:464,wholeConstructorsObserved:456,wholeConstructorsUnrun:8,totalSourceHeapFailures:10,exactTablePhaseMatches:450,tableResourceObservations:10,sourceSafeConstructorsEachBrowser:199,sourceExcludedObservations:265,rustExactConstructors:199,additional519StrictClosures:8,limitedRustObservations:265,actualWasmFunctionsEach:2,nativeAddonFunctions:216,compilerSha256:rust.artifacts.compiler.sha256,exactWorkspaceFmt:'FAILED OS206',overallEquivalence:'NOT PROVEN'}));
