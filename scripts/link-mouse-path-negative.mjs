// An exact old-production control, not a weakened assertion or a synthetic passing host.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { runCommand } from './quality-process.mjs';
const base = 'e64f48703fc4743bc6c7974a8ebde2d4fd1f47a6';
const files = ['crates/xterm-web/src/links/browser_linkifier_events.rs',
  'crates/xterm-web/src/links/browser_payload.rs'];
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const original = new Map();
const report = { base, mode: 'exact-old-production', files: {}, status: 'preparing' };
for (const file of files) {
  const bytes = await readFile(file);
  original.set(file, bytes);
  report.files[file] = { finalSha256: hash(bytes) };
}
try {
  for (const file of files) {
    const old = spawnSync(process.env.GIT ?? 'git', ['show', `${base}:${file}`],
      { timeout: 30000, maxBuffer: 1024 * 1024 });
    assert.equal(old.status, 0, old.stderr?.toString());
    report.files[file].baselineSha256 = hash(old.stdout);
    await writeFile(file, old.stdout);
  }
  report.command = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
    cwd: process.cwd(), env: { ...process.env, BROWSERS: 'chromium', WASM_TEST: 'wasm_link_mouse_path', CARGO_BUILD_JOBS: '2' },
    log: 'results/link-mouse-path/negative-command.log', timeoutMs: 300000
  });
  assert.equal(report.command.exitCode, 1, 'Old controller must fail at runtime');
  assert.equal(report.command.timedOut, false);
  const browser = JSON.parse(await readFile('results/browser-host/wasm_link_mouse_path-results.json', 'utf8'));
  assert(browser.commands.every(command => command.status === 'passed'), 'Control must compile and bind');
  assert.equal(browser.browsers.length, 1);
  assert.equal(browser.browsers[0].counts.failed, 1);
  assert.equal(browser.browsers[0].counts.passed, 0);
  assert(JSON.stringify(browser.browsers[0]).includes('hover: provided=true, expected false'),
    'Control must fail at the intended raw hover-target assertion');
  await writeFile('results/link-mouse-path/negative-browser.json', JSON.stringify(browser, null, 2) + '\n');
  report.status = 'expected-runtime-failure';
} finally {
  for (const [file, bytes] of original) {
    await writeFile(file, bytes);
    report.files[file].restoredSha256 = hash(await readFile(file));
    assert.equal(report.files[file].restoredSha256, report.files[file].finalSha256);
  }
  await writeFile('results/link-mouse-path/negative.json', JSON.stringify(report, null, 2) + '\n');
}
console.log(report.status);
