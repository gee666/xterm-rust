// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Demonstrate the differential test rejects optional disconnect in the observe catch.
import {readFile, writeFile, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {runCommand} from './quality-process.mjs';
const root = process.cwd();
const source = path.resolve(root, 'crates/xterm-web/src/webgl/device_pixel_observer.rs');
const evidence = path.resolve(root, 'results/device-pixel-observer');
const reportPath = path.resolve(root, 'results/browser-host/wasm_device_pixel_observer-results.json');
const before = await readFile(source);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const text = before.toString('utf8');
const needle = 'state.disconnect(false)?;';
if (text.split(needle).length !== 2) throw new Error('Expected exactly one required catch disconnect');
const mutant = Buffer.from(text.replace(needle, 'state.disconnect(true)?;'));
const record = {kind: 'production observe-catch required-to-optional disconnect',
  originalSha256: hash(before), mutantSha256: hash(mutant), restored: false};
try {
  await writeFile(source, mutant);
  record.command = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
    cwd: root, env: {...process.env, WASM_TEST: 'wasm_device_pixel_observer', BROWSERS: 'chromium'},
    log: path.join(evidence, 'negative-command.log'), timeoutMs: 180000,
  });
  await copyFile(reportPath, path.join(evidence, 'browser-negative-report.json'));
  const report = JSON.parse(await readFile(reportPath, 'utf8'));
  const browser = report.browsers[0];
  record.counts = browser?.counts;
  const failed = browser?.output?.match(/^test (.+) \.\.\. FAIL$/gm);
  record.failedTests = failed;
  if (record.command.exitCode !== 1 || browser?.name !== 'chromium'
      || browser.counts?.passed !== 2 || browser.counts?.failed !== 1
      || browser.counts?.ignored !== 0 || browser.counts?.filtered !== 0
      || failed?.length !== 1
      || failed[0] !== 'test observer_lifecycle_and_live_getters_match_unchanged_source ... FAIL') {
    throw new Error('Production mutant did not fail exactly the source lifecycle comparison');
  }
  record.expectedFailureObserved = true;
} finally {
  if (hash(await readFile(source)) !== hash(mutant)) {
    throw new Error('Owned source changed during mutant; refusing to overwrite uncertain edits');
  }
  await writeFile(source, before);
  record.restoredSha256 = hash(await readFile(source));
  record.restored = record.restoredSha256 === record.originalSha256;
  await writeFile(path.join(evidence, 'negative.json'), JSON.stringify(record, null, 2) + '\n');
  if (!record.restored) throw new Error('Production source restoration failed');
}
console.log('Production mutant: exactly2PASS/1FAIL at lifecycle comparison; exact source SHA restored');
