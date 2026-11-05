// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// The restored full browser matrix is a separate required step after this tool.
import {readFile, writeFile, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {runCommand} from './quality-process.mjs';
const source='crates/xterm-web/src/webgl/browser_cache.rs';
const directory='results/browser-atlas-cache';
const original=await readFile(source);
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const before='                self.runtime.clone(),';
const text=original.toString('utf8');
assert.equal(text.split(before).length,2,'exactly one runtime forwarding site');
const mutant=text.replace(before,'                Rc::new(AtlasRuntime::default()),');
await writeFile(`${directory}/negative-mutant.rs.txt`,mutant);
const proof={mutation:'Use a private AtlasRuntime for each new atlas instead of the binding-shared runtime.',
 originalSha256:hash(original),mutantSha256:hash(mutant),restored:false,expectedFailureObserved:false};
try {
  await writeFile(source,mutant);
  const result=await runCommand(process.execPath,['scripts/browser-host-check.mjs'],{
    cwd:process.cwd(),env:{...process.env,BROWSERS:'chromium'},timeoutMs:240000,
    log:`${directory}/negative.log`});
  const reportPath='results/browser-host/wasm_webgl_browser_cache-results.json';
  const report=JSON.parse(await readFile(reportPath,'utf8'));
  await copyFile(reportPath,`${directory}/browser-negative.json`);
  assert.equal(result.status,'failed');
  assert.equal(report.status,'failed');
  assert.equal(report.browsers.length,1);
  const browser=report.browsers[0];
  assert.equal(browser.name,'chromium');
  assert.equal(browser.status,'failed');
  assert.equal(browser.counts.ignored,0);
  assert.equal(browser.counts.filtered,0);
  assert.ok(browser.counts.failed>0,'actual Rust assertions must detect the production mutation');
  proof.counts=browser.counts;
  proof.expectedFailureObserved=true;
} finally {
  await writeFile(source,original);
  proof.restoredSha256=hash(await readFile(source));
  proof.restored=proof.originalSha256===proof.restoredSha256;
  await writeFile(`${directory}/negative.json`,JSON.stringify(proof,null,2)+'\n');
}
assert.equal(proof.restored,true);
console.log('Production shared-runtime mutant rejected and exact original bytes restored');
