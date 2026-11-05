// Compile exact pre-repair files against current assertions, then restore bytes.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
const baseline = process.argv[2];
assert(baseline && /^[\w/.-]+$/.test(baseline), 'Pass a verified pre-repair Git revision');
const paths = ['crates/xterm-web/src/links/browser_payload.rs', 'crates/xterm-web/src/links/browser_linkifier_events.rs'];
const hash = value => createHash('sha256').update(value).digest('hex');
const originals = await Promise.all(paths.map(path => readFile(path)));
const record = { baseline, files: [], restored: false };
try {
  for (const [index, path] of paths.entries()) {
    const result = spawnSync(process.env.GIT ?? 'git', ['show', `${baseline}:${path}`], { timeout: 30000, maxBuffer: 1024 * 1024 });
    assert.equal(result.status, 0, result.stderr?.toString());
    const bytes = result.stdout;
    assert.notEqual(hash(bytes), hash(originals[index]), `Not a pre-repair file: ${path}`);
    record.files.push({ path, finalSha256: hash(originals[index]), baselineSha256: hash(bytes) });
    await writeFile(path, bytes);
  }
  const result = spawnSync(process.execPath, ['scripts/browser-host-check.mjs'], {
    env: { ...process.env, WASM_TEST: 'wasm_link_underline_geometry', BROWSERS: 'chromium' },
    timeout: 240000, maxBuffer: 16 * 1024 * 1024, encoding: 'utf8', windowsHide: true
  });
  await writeFile('results/link-underline-geometry/negative-command.log', `${result.stdout ?? ''}${result.stderr ?? ''}`);
  record.exitCode = result.status;
  record.error = result.error?.message;
  const report = JSON.parse(await readFile('results/browser-host/wasm_link_underline_geometry-results.json', 'utf8'));
  record.report = report;
  assert.equal(result.status, 1, 'Old production must fail actual assertions');
  assert.equal(report.browsers.length, 1, 'Compiler failure is not a behavioral negative');
  assert.equal(report.browsers[0].tests.length, 2);
  assert.equal(report.browsers[0].status, 'failed');
  assert(report.browsers[0].counts.failed > 0);
} finally {
  for (const [index, path] of paths.entries()) await writeFile(path, originals[index]);
  for (const [index, path] of paths.entries()) assert.equal(hash(await readFile(path)), hash(originals[index]), `Restore failed: ${path}`);
  record.restored = true;
  await writeFile('results/link-underline-geometry/negative.json', JSON.stringify(record, null, 2) + '\n');
}
console.log('Exact pre-repair production failed browser assertions; final source hashes restored');
