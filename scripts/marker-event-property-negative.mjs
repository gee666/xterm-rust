// Exact old production with current assertions, followed by exact restoration.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { runCommand } from './quality-process.mjs';
const baseline = process.argv[2];
assert(baseline && /^[a-f0-9]{40}$/.test(baseline), 'Pass the full inspected pre-repair commit');
const path = 'crates/xterm-web/src/terminal/marker_api.rs';
const output = 'results/marker-event-property';
const reportPath = 'results/browser-host/wasm_terminal_markers-results.json';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const fixed = await readFile(path);
const readReport = async () => JSON.parse(await readFile(reportPath, 'utf8'));
const initial = await readReport();
assert.equal(initial.status, 'passed');
assert.deepEqual(initial.browsers.map(b => b.counts), Array(3).fill({ passed: 5, failed: 0, ignored: 0, filtered: 0 }));
await writeFile(`${output}/initial.json`, JSON.stringify(initial, null, 2) + '\n');
const record = { baseline, path, fixedSha256: hash(fixed), restored: false };
try {
  const old = spawnSync(process.env.GIT ?? 'git', ['show', `${baseline}:${path}`], {
    timeout: 30000, maxBuffer: 1024 * 1024, windowsHide: true
  });
  assert.equal(old.status, 0, old.stderr?.toString());
  record.oldSha256 = hash(old.stdout);
  assert.notEqual(record.oldSha256, record.fixedSha256);
  await writeFile(path, old.stdout);
  record.negativeCommand = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
    cwd: process.cwd(), env: { ...process.env, WASM_TEST: 'wasm_terminal_markers', BROWSERS: 'chromium' },
    timeoutMs: 240000, log: `${output}/negative-command.log`
  });
  record.negative = await readReport();
  assert.equal(record.negativeCommand.exitCode, 1);
  assert.equal(record.negative.browsers.length, 1, 'Build failure is not a behavioral control');
  const browser = record.negative.browsers[0];
  assert.equal(browser.status, 'failed');
  assert.deepEqual(browser.counts, { passed: 4, failed: 1, ignored: 0, filtered: 0 });
  assert(browser.console.some(e => e.text.includes('onDispose own data descriptor')),
    'Old production must fail on the own event descriptor');
} finally {
  await writeFile(path, fixed);
  assert.equal(hash(await readFile(path)), record.fixedSha256);
  record.restored = true;
  await writeFile(`${output}/control.json`, JSON.stringify(record, null, 2) + '\n');
}
const finalCommand = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
  cwd: process.cwd(), env: { ...process.env, WASM_TEST: 'wasm_terminal_markers', BROWSERS: 'chromium,firefox,webkit' },
  timeoutMs: 240000, log: `${output}/final-command.log`
});
const final = await readReport();
await writeFile(`${output}/final.json`, JSON.stringify({ command: finalCommand, report: final }, null, 2) + '\n');
assert.equal(finalCommand.status, 'passed');
assert.equal(final.status, 'passed');
assert.deepEqual(final.browsers.map(b => b.counts), Array(3).fill({ passed: 5, failed: 0, ignored: 0, filtered: 0 }));
for (const kind of ['compiler', 'wasm', 'bindings']) {
  assert.equal(final.artifacts[kind].sha256, initial.artifacts[kind].sha256, `${kind} differs after restoration`);
  assert.notEqual(final.artifacts[kind].sha256, record.negative.artifacts[kind].sha256, `${kind} did not change for old production`);
}
console.log('Old production fails only the new field assertion; restored three-engine matrix passes with exact initial artifacts');
