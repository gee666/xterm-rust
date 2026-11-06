// Read-only proof receipts. Text producer pins normalize CRLF only; raw hashes
// retain exactly what the tested producer used. Runtime/binary pins are raw.
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {cff2FontsInput,cff2FontsCases} from '../../crates/xterm-font-addons/tests/support/font_cff2_fonts_input.mjs';
const out='results/font-cff2-fonts',sha=b=>createHash('sha256').update(b).digest('hex');
const json=async p=>JSON.parse(await fs.readFile(p)),lf=b=>Buffer.from(b.toString().replaceAll('\r\n','\n'));
const pins=await json(out+'/producer-hashes.json');
for(const [file,pin] of Object.entries(pins.files)){const bytes=await fs.readFile(file);assert.equal(sha(pin.mode==='text'?lf(bytes):bytes),pin.canonicalSha256,file);}
const raw=await json(out+'/raw-log-bytes.json');
for(const [file,pin] of Object.entries(raw.files)){const b=Buffer.from(pin.base64,'base64');assert.equal(sha(b),pin.sha256);assert.equal(b.length,pin.bytes);assert.equal(sha(lf(await fs.readFile(out+'/'+file))),pin.normalizedSha256,file);}
const fixtureFile='crates/xterm-addons/tests/fixtures/font-cff2-fonts.json',fixture=await json(fixtureFile),recipes=cff2FontsCases();
assert.equal(fixture.cases.length,408);assert.equal(recipes.length,408);
assert.equal(fixture.cases.filter(c=>c.resourceFailure).length,12);
assert.equal(fixture.cases.filter(c=>c.phaseTable).length,392);
assert.equal(fixture.cases.filter(c=>c.outcome.status==='resourceFailure').length,12);
const base=new Uint8Array(await fs.readFile('crates/xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf'));
for(const [i,c] of fixture.cases.entries()){
 assert.equal(c.name,recipes[i].name);assert.deepEqual(c.data,recipes[i].data);
 assert.equal(sha(cff2FontsInput(base,c,{...fixture.woffTemplates.otf,...c.encoded})),c.inputSha256,c.name);
 if(c.stringOrder){assert.equal(c.phaseTable.status,'ok');if(c.woff)assert.equal(c.outcome.status,'ok');}
 if(c.indexOrder)assert.equal(c.phaseTable.message,'Offset is outside the bounds of the DataView');
}
for(const kind of ['generation','replay']){const resources=await json(out+'/resource-'+kind+'.json');assert.equal(resources.length,12);for(const r of resources){const bytes=Buffer.from(r.stderrBase64,'base64');assert.equal(sha(bytes),r.stderrSha256);assert.match(bytes.toString(),/FATAL ERROR: (?:Reached heap limit|Ineffective mark-compacts near heap limit)/);assert.equal(r.inputSha256,fixture.cases.find(c=>c.name===r.name).inputSha256);}}
const source=await json(out+'/source-browser.json'),oracle=await json(out+'/oracle.json');
assert.equal(source.status,'passed');assert.equal(source.fixtureSha256,oracle.fixtureSha256);assert.equal(oracle.fixtureSha256,pins.files[fixtureFile].rawSha256);assert.equal(source.sourceSha256,fixture.sourceSha256);assert.equal(source.excluded.length,12);assert.equal(source.browsers.length,3);
for(const b of source.browsers){assert.equal(b.status,'passed');assert.equal(b.constructors,396);assert.equal(b.constructorsRaw.length,396);assert.equal(b.addonCases,4);assert.equal(b.addonConstructorCases.length,4);assert.deepEqual(b.addonRaw.debug[1],{name:'Error',message:'Object [object Object] already has key 0'});}
const rust=await json(out+'/rust-browser.json');assert.equal(rust.status,'passed');assert.equal(rust.browsers.length,3);assert.equal(sha(await fs.readFile(rust.artifacts.compiler.path)),rust.artifacts.compiler.sha256);assert.equal(rust.artifacts.cargoLockSha256,pins.files['Cargo.lock'].rawSha256);assert.equal(rust.artifacts.testSourceSha256,pins.files['crates/xterm-font-addons/tests/font_cff2_fonts_browser.rs'].rawSha256);
for(const b of rust.browsers){assert.equal(b.status,'passed');assert.deepEqual(b.counts,{passed:2,failed:0,ignored:0,filtered:0});assert.equal(b.tests.length,2);assert.deepEqual(b.errors,[]);}
const negative=await json(out+'/negative.json');assert.equal(negative.status,'passed');assert.equal(negative.executedMismatches.length,184);assert.equal(negative.executedDuplicateCases.length,36);assert.equal(negative.designated.length,4);for(const n of negative.designated)assert(negative.executedDuplicateCases.includes(n));assert.equal(negative.beforeSha256,negative.restoredSha256);assert.equal(negative.restoredSha256,pins.files['crates/xterm-addons/src/ligatures/font_cff_index.rs'].rawSha256);assert.equal(negative.positive.status,'passed');
const native=await fs.readFile(out+'/native-final.log','utf8'),summaries=[...native.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)];assert.equal(summaries.length,52);assert.equal(summaries.reduce((n,m)=>n+Number(m[1]),0),222);assert(summaries.every(m=>m.slice(2).every(n=>n==='0')));assert(!/^warning:/m.test(native));
const commands=await json(out+'/native-commands.json');for(const name of ['source-replay','clippy-native-final','clippy-wasm-font','dependents-native','dependents-wasm','native-final','wasm-no-run'])assert.equal(commands.find(c=>c.name===name).status,'passed',name);
for(const p of ['xterm-core','xterm-addons','xterm-web','xterm-font-addons','xterm-wasm-support'])assert.equal((await fs.readFile(out+'/fmt-'+p+'.log','utf8')).trim(),'');assert((await fs.readFile(out+'/fmt-workspace.log','utf8')).includes('os error 206'));
if(process.argv.includes('--cached'))for(const [file,pin] of Object.entries((await json(out+'/cached-test-pins.json')).files))assert.equal(sha(await fs.readFile(file)),pin.sha256,file);
if(process.argv.includes('--runtime'))for(const [file,pin] of Object.entries((await json(out+'/runtime-pins.json')).files))assert.equal(sha(await fs.readFile(file)),pin.sha256,file);
const refs=await json('docs/baseline/reference-manifest.json'),roots=process.env.REFERENCE_ROOT?[process.env.REFERENCE_ROOT]:[];if(!process.argv.includes('--original-only')||!roots.length)roots.push('source_repo');for(const root of roots)for(const r of refs)assert.equal(sha(await fs.readFile(path.join(root,r.path))),r.sha256,root+'/'+r.path);
if(process.env.REFERENCE_ROOT)assert.equal(sha(await fs.readFile(path.join(process.env.REFERENCE_ROOT,'node_modules/opentype.js/dist/opentype.js'))),fixture.sourceSha256);
console.log(JSON.stringify({status:'passed',producerPins:Object.keys(pins.files).length,referenceRoots:roots.length,referenceFilesEachRoot:refs.length,recipes:408,sourceConstructorsObserved:396,sourceConstructorsUnrun:12,sourceHeapFailures:12,exactTablePhases:392,sourceConstructorsEachBrowser:396,sourceExcluded:12,rustFunctionsEachBrowser:2,nativeFunctions:222,compilerSha256:rust.artifacts.compiler.sha256,workspaceFmt:'FAILED OS206',equivalence:'NOT PROVEN'}));
