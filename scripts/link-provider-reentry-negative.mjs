// Compile either the old controller or a native early-commit mutation against
// the new assertions. Always restore exact working bytes after the child exits.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';

const base = '32bf954467fe3389e9cfe42fdd8c34d425975b62';
assert(process.argv.slice(2).every(arg => arg === '--native'), 'Unknown negative-check argument');
const native = process.argv.includes('--native');
const prefix = native ? 'negative-native' : 'negative';
const files = ['crates/xterm-web/src/links/replies.rs',
  ...(native ? [] : ['crates/xterm-web/src/links/browser_linkifier_events.rs'])];
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const original = new Map();
const report = { base, mode: native ? 'early-commit-mutation' : 'original-two-file-controller', files: {}, status: 'preparing' };
for (const file of files) {
  const bytes = await readFile(file);
  original.set(file, bytes);
  report.files[file] = { finalSha256: hash(bytes) };
}
try {
  for (const file of files) {
    let bytes;
    if (native) {
      const text = original.get(file).toString('utf8');
      const pattern = /(\s*)select\(link.clone\(\)\)\?;\n\s*request.link_provided.set\(true\);/g;
      assert.equal([...text.matchAll(pattern)].length, 2);
      bytes = Buffer.from(text.replace(pattern, '$1request.link_provided.set(true);$1select(link.clone())?;'));
    } else {
      const old = spawnSync(process.env.GIT ?? 'git', ['show', `${base}:${file}`],
        { timeout: 30000, maxBuffer: 1024 * 1024 });
      assert.equal(old.status, 0, old.stderr?.toString());
      bytes = old.stdout;
    }
    report.files[file].baselineSha256 = hash(bytes);
    await writeFile(file, bytes);
  }
  const command = native ? process.env.CARGO ?? 'cargo' : process.execPath;
  const args = native ? ['test', '--locked', '-j2', '-p', 'xterm-web', '--test', 'link_provider_transactions',
    'selection_reentry_observes_unassigned_outer_result', '--', '--exact'] : ['scripts/browser-host-check.mjs'];
  report.command = [command, ...args];
  const run = spawnSync(command, args, {
    env: { ...process.env, BROWSERS: 'chromium', WASM_TEST: 'wasm_link_provider_reentry', CARGO_BUILD_JOBS: '2' },
    encoding: 'utf8', timeout: 240000, maxBuffer: 8 * 1024 * 1024
  });
  const output = (run.stdout ?? '') + (run.stderr ?? '');
  await writeFile(`results/link-provider-reentry/${prefix}-command.log`, output);
  report.exitCode = run.status;
  report.error = run.error?.message;
  assert(!run.error, run.error?.message);
  assert.equal(run.status, native ? 101 : 1, 'Early-commit controller must fail the runtime assertion');
  if (native) {
    assert(output.includes('left: 1') && output.includes('right: 2'));
    assert(output.includes('0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out'));
  } else {
    const browser = JSON.parse(await readFile('results/browser-host/wasm_link_provider_reentry-results.json', 'utf8'));
    assert(browser.commands.every(command => command.status === 'passed'), 'Negative must compile and bind');
    assert.equal(browser.browsers.length, 1);
    assert.equal(browser.browsers[0].counts.failed, 1);
    assert.equal(browser.browsers[0].counts.passed, 0);
    assert(JSON.stringify(browser.browsers[0]).includes('nested undefined: expected 2 hovers, got 1'),
      'Negative must fail at the intended provider reentry assertion');
    await writeFile('results/link-provider-reentry/negative-browser.json', JSON.stringify(browser, null, 2) + '\n');
  }
  report.status = 'expected-runtime-failure';
} finally {
  for (const [file, bytes] of original) {
    await writeFile(file, bytes);
    report.files[file].restoredSha256 = hash(await readFile(file));
    assert.equal(report.files[file].restoredSha256, report.files[file].finalSha256);
  }
  await writeFile(`results/link-provider-reentry/${prefix}.json`, JSON.stringify(report, null, 2) + '\n');
}
console.log(report.status);
