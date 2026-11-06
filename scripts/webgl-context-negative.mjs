// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
// Change real production resize notification, then restore exact bytes in finally.
import {readFile, writeFile, copyFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import path from 'node:path';
import {runCommand} from './quality-process.mjs';
const root = process.cwd(), directory = 'results/webgl-context';
const file = 'crates/xterm-web/src/webgl/context_lifecycle.rs';
const original = await readFile(file), text = original.toString();
const hash = value => createHash('sha256').update(value).digest('hex');
const needle = 'self.redraw(true)?;';
assert.equal(text.split(needle).length, 2);
const mutant = text.replace(needle, 'self.redraw(false)?;');
const receipt = {productionFile: file, mutation: 'Physical resize redraw sync true -> false',
  originalSha256: hash(original), mutantSha256: hash(mutant), expectedFailureObserved: false,
  sourceScenariosExecuted: 1, sourceScenariosUnrunAfterExpectedFailure: 28};
try {
  await writeFile(file, mutant);
  const command = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
    cwd: root, env: {...process.env, BROWSERS: 'chromium'},
    log: path.resolve(directory, 'negative-command.log'), timeoutMs: 180000,
  });
  receipt.command = command;
  await copyFile('results/browser-host/wasm_webgl_context-results.json', `${directory}/browser-negative-report.json`);
  const report = JSON.parse(await readFile(`${directory}/browser-negative-report.json`, 'utf8'));
  assert.equal(command.status, 'failed');
  assert.equal(report.browsers.length, 1);
  const browser = report.browsers[0];
  assert.deepEqual(browser.counts, {passed: 3, failed: 1, ignored: 0, filtered: 0});
  assert.deepEqual(browser.errors, []);
  assert.ok(browser.console.some(entry => /context case dimensions/.test(entry.text)));
  receipt.counts = browser.counts;
  receipt.expectedFailureObserved = true;
} catch (error) {
  receipt.error = error.stack ?? String(error);
  throw error;
} finally {
  await writeFile(file, original);
  receipt.restoredSha256 = hash(await readFile(file));
  receipt.restored = receipt.originalSha256 === receipt.restoredSha256;
  await writeFile(`${directory}/negative.json`, JSON.stringify(receipt, null, 2) + '\n');
  assert.equal(receipt.restored, true);
}
console.log('PASS actual production sync mutant rejected3P1F; exact source bytes restored');
