// Exact pre-repair production control. Restore every owned byte in finally.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile, unlink, statfs } from 'node:fs/promises';
import { freemem } from 'node:os';
import { resolve, dirname } from 'node:path';
import { spawnSync } from 'node:child_process';
import { runCommand } from './quality-process.mjs';
const root = resolve(import.meta.dirname, '..');
const base = '0442e3578a7f57d35c2ae6ff20d7186820983065';
const resume = process.argv.includes('--resume');
assert(process.argv.slice(2).every(value => value === '--resume'));
const output = resolve(root, 'results/public-decoration/control');
await mkdir(output, { recursive: true });
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
function git(args, optional = false) {
  const value = spawnSync('git', args, { cwd: root, timeout: 30000, maxBuffer: 16 * 1024 * 1024 });
  if (!optional) assert.equal(value.status, 0, value.stderr.toString());
  return value;
}
async function headroom() {
  assert(freemem() >= 8 * 1024 ** 3, 'Need 8 GiB available RAM');
  const disk = await statfs(root, { bigint: true });
  assert(disk.bavail * disk.bsize >= 10n * 1024n ** 3n, 'Need 10 GiB free');
}
const changed = git(['diff', '--name-only', base, 'HEAD', '--', 'crates']).stdout.toString().trim().split(/\r?\n/)
  .filter(file => /^crates\/[^/]+\/src\//.test(file));
assert(changed.length > 0);
const oldNames = new Set(git(['ls-tree', '-r', '--name-only', base, '--', ...changed]).stdout.toString().trim().split(/\r?\n/));
const before = new Map();
const report = resume ? JSON.parse(await readFile(resolve(output, 'control.json'), 'utf8'))
  : { base, revision: git(['rev-parse', 'HEAD']).stdout.toString().trim(), production: [], commands: [], status: 'running' };
const latest = resolve(root, 'results/browser-host/wasm_public_decoration-results.json');
const initial = resume ? report.initial : JSON.parse(await readFile(latest, 'utf8'));
assert.equal(initial.status, 'passed');
assert.equal(initial.browsers.length, 3);
report.initial = initial;
for (const file of changed) {
  const bytes = await readFile(resolve(root, file));
  before.set(file, bytes);
  const old = oldNames.has(file) ? git(['show', `${base}:${file}`]).stdout : null;
  if (resume) {
    const pin = report.production.find(value => value.path === file);
    assert.equal(pin?.initialSha256, sha(bytes), `Resume source ${file}`);
    assert.equal(pin?.oldSha256, old && sha(old), `Resume baseline ${file}`);
  } else report.production.push({ path: file, initialSha256: sha(bytes), oldSha256: old && sha(old) });
}
async function run(label, browsers) {
  await headroom();
  const command = await runCommand(process.execPath, ['scripts/browser-host-check.mjs'], {
    cwd: root, env: { ...process.env, WASM_TEST: 'wasm_public_decoration', BROWSERS: browsers, CARGO_BUILD_JOBS: '2' },
    log: resolve(output, `${label}.log`), timeoutMs: 180000, resourceCheck: headroom,
  });
  report.commands.push({ label, ...command });
  const result = JSON.parse(await readFile(latest, 'utf8'));
  report[label] = result;
  await writeFile(resolve(output, `${label}.json`), JSON.stringify(result, null, 2) + '\n');
  return { command, result };
}
function assertNegative(result) {
  assert.equal(result.browsers.length, 1);
  assert.equal(result.browsers[0].counts.passed, 0);
  assert.equal(result.browsers[0].counts.failed, 1);
  assert.match(result.browsers[0].output + '\n' + result.browsers[0].console.map(value => value.text).join('\n'), /missing public registerDecoration/);
}
if (!resume) try {
  for (const file of changed) {
    if (oldNames.has(file)) await writeFile(resolve(root, file), git(['show', `${base}:${file}`]).stdout);
    else await unlink(resolve(root, file));
  }
  const negative = await run('negative', 'chromium');
  assert.equal(negative.command.status, 'failed');
  assertNegative(negative.result);
  report.negativeMatched = true;
} finally {
  for (const [file, bytes] of before) {
    await mkdir(dirname(resolve(root, file)), { recursive: true });
    await writeFile(resolve(root, file), bytes);
    assert.equal(sha(await readFile(resolve(root, file))), sha(bytes), `Restore ${file}`);
  }
  report.productionRestored = true;
  await writeFile(resolve(output, 'control.json'), JSON.stringify(report, null, 2) + '\n');
}
if (resume) {
  assert.equal(report.productionRestored, true);
  assertNegative(report.negative);
  report.negativeMatched = true;
  for (const receipt of [report.initial, report.negative]) {
    for (const key of ['compiler', 'wasm', 'bindings']) {
      const artifact = receipt.artifacts[key];
      assert.equal(sha(await readFile(resolve(root, artifact.path))), artifact.sha256, `Resume ${key}`);
    }
  }
  report.resumedWithoutNegativeReplay = true;
}
const final = await run('final', 'chromium,firefox,webkit');
assert.equal(final.command.status, 'passed');
assert.equal(final.result.browsers.length, 3);
for (const browser of final.result.browsers) {
  assert.equal(browser.counts.passed, 1);
  assert.equal(browser.counts.failed, 0);
}
for (const key of ['compiler', 'wasm', 'bindings']) {
  assert.equal(initial.artifacts[key].sha256, final.result.artifacts[key].sha256, `Restored ${key}`);
  assert.notEqual(initial.artifacts[key].sha256, report.negative.artifacts[key].sha256, `Distinct old ${key}`);
}
report.status = 'passed';
await writeFile(resolve(output, 'control.json'), JSON.stringify(report, null, 2) + '\n');
console.log(`Old production failed at absent API; restored ${changed.length} files and identical compiler/WASM/bindings, final 1/1 each engine`);
