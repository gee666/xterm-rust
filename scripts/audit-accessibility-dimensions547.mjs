// MIT. Read-only verification of scoped accessibility receipts, not equivalence.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'..');
const out=resolve(root,'results/accessibility-dimensions547');
const runtime=process.argv.includes('--runtime');
assert.ok(process.argv.slice(2).every(arg=>arg==='--runtime'));
const json=async name=>JSON.parse(await readFile(resolve(out,name),'utf8'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
let sourceCount=0,evidenceCount=0,runtimeCount=0;
for(const pin of (await json('source-pins.json')).files) {
  assert.equal(hash(await readFile(resolve(root,pin.path))),pin.sha256,pin.path);
  sourceCount++;
}
for(const line of (await readFile(resolve(out,'SHA256SUMS'),'utf8')).trim().split('\n')) {
  const [sha,path]=line.split('  ');
  assert.equal(hash(await readFile(resolve(out,path))),sha,path);
  evidenceCount++;
}
if(runtime) {
  for(const pin of (await json('runtime-pins.json')).files) {
    const bytes=await readFile(pin.path);
    assert.equal(bytes.length,pin.bytes,pin.path);
    assert.equal(hash(bytes),pin.sha256,pin.path);
    runtimeCount++;
  }
}
const expectedWarnings=['row is invalid. Race condition?','row is invalid. Race condition?',
  'columns is null. Race condition?','columns is null. Race condition?'];
for(const [name,count,added] of [['wasm_accessibility',33,8],['wasm_accessibility_terminal',8,1]]) {
  const initial=await json(`initial-${name}.json`);
  const final=await json(`final-${name}.json`);
  for(const kind of ['compiler','wasm','bindings'])assert.equal(initial.artifacts[kind].sha256,final.artifacts[kind].sha256);
  for(const [phase,report] of [['initial',initial],['final',final],['old',await json(`old-${name}.json`)]]) {
    assert.equal(report.browsers.length,phase==='old'?1:3);
    for(const browser of report.browsers) {
      const expected=phase==='old'?(name==='wasm_accessibility'?[26,7]:[7,1]):[count,0];
      assert.deepEqual(browser.counts,{passed:expected[0],failed:expected[1],ignored:0,filtered:0});
      assert.equal(browser.tests.length,count);
      assert.equal(browser.tests.filter(test=>test.includes('::dimensions::')).length,added);
      assert.deepEqual(browser.errors,[]);
      const errors=browser.console.filter(entry=>entry.type==='error');
      if(phase==='old') {
        assert.equal(errors.length,expected[1]);
        const file=name==='wasm_accessibility'?'dimensions.rs:':'terminal_dimensions.rs:';
        for(const error of errors) {
          assert.match(error.text,/^panicked at crates[\\/]xterm-web[\\/]tests[\\/]accessibility_cases[\\/]/);
          assert.ok(error.text.includes(file));
          assert.match(error.text,/assertion `left == right` failed/);
        }
      } else assert.deepEqual(errors,[]);
      assert.deepEqual(browser.console.filter(entry=>entry.type==='warning').map(entry=>entry.text),
        name==='wasm_accessibility'?expectedWarnings:[]);
      const log=await readFile(resolve(out,`${phase}-${name}-${browser.name}.log`),'utf8');
      const match=log.match(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) filtered out/);
      assert.ok(match,`${phase}/${name}/${browser.name} lacks completed counts`);
      assert.deepEqual(match.slice(1).map(Number),[...expected,0,0]);
    }
  }
}
const restore=await json('negative-restore.json');
assert.equal(restore.restored,true);
for(const pin of restore.files)assert.equal(hash(await readFile(resolve(root,pin.path))),pin.fixedSha256);
const manifest=JSON.parse(await readFile(resolve(root,'docs/baseline/reference-manifest.json'),'utf8'));
for(const pin of manifest)assert.equal(hash(await readFile(resolve(root,'source_repo',pin.path))),pin.sha256,pin.path);
assert.match(await readFile(resolve(out,'full-fmt.log'),'utf8'),/os error 206/);
console.log(JSON.stringify({status:'passed',sourceCount,evidenceCount,runtimeCount,
  runtimeRequested:runtime,referenceFiles:manifest.length,fixedPerEngine:[33,8],oldChromium:[[26,7],[7,1]],
  expectedSelectionWarningsPerComponentRun:4,overallEquivalence:'NOT PROVEN'},null,2));
