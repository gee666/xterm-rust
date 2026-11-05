// Use when checking the inactive coordinate guard against the main5088 product.
// Original and inherited assertions stay unchanged. Restore only owned files.
import {readFile, writeFile, copyFile, mkdir} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {resolve, dirname} from 'node:path';
import {runCommand} from '../../scripts/quality-process.mjs';
const base = '5088f8fdd475e55b2408e9f2e7bfaf92e83ca656';
const files = ['crates/xterm-web/src/image/addon/bindings.rs', 'crates/xterm-web/src/image/addon/options.rs'];
const out = resolve('results/image-inactive-coordinates');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const fixed = new Map(), baseline = new Map();
const audit = {base, files: [], restored: false};
await mkdir(out, {recursive: true});
for (const file of files) {
  const bytes = await readFile(file);
  const source = spawnSync('git', ['show', `${base}:${file}`], {timeout: 30_000, maxBuffer: 1024 * 1024});
  if (source.status !== 0 || source.error) throw Error(`Cannot inspect baseline ${file}`);
  fixed.set(file, bytes); baseline.set(file, source.stdout);
  audit.files.push({file, fixedSha256: hash(bytes), baselineSha256: hash(source.stdout)});
}
try {
  for (const file of files) {
    await writeFile(file, baseline.get(file));
    if (hash(await readFile(file)) !== hash(baseline.get(file))) throw Error(`Baseline write differs: ${file}`);
  }
  audit.command = await runCommand(process.execPath, ['scripts/image-canvas-check.mjs'], {
    cwd: process.cwd(), env: {...process.env, WASM_TEST: 'image_addon_browser', BROWSERS: 'chromium'},
    timeoutMs: 300_000, log: resolve(out, 'negative.log')
  });
  for (const [from, to] of [
    ['results/browser-host/image_addon_browser-results.json', 'negative-browser.json'],
    ['results/image-addon/verification.json', 'negative-verification.json']
  ]) await copyFile(from, resolve(out, to));
  const report = JSON.parse(await readFile(resolve(out, 'negative-browser.json'), 'utf8'));
  if (report.artifacts) {
    const run = dirname(dirname(report.artifacts.wasm.path));
    await copyFile(resolve(run, 'image_addon_browser-build.log'), resolve(out, 'negative-build.log'));
  }
  if (audit.command.exitCode !== 1 || audit.command.timedOut || audit.command.error || audit.command.terminationError) {
    throw Error('Expected completed failing assertions, not a process/build failure');
  }
  if (report.browsers.length !== 1 || report.browsers[0].name !== 'chromium') throw Error('Unexpected browser matrix');
  const browser = report.browsers[0];
  audit.failedTests = [...browser.output.matchAll(/^test (.*?) \.\.\. FAIL$/gm)].map(match => match[1]).sort();
  const expected = [
    'inactive_coordinates::inactive_cell_methods_skip_all_raw_coordinate_conversions',
    'inactive_coordinates::inactive_reset_and_disposal_never_run_coordinate_side_effects'
  ].sort();
  if (JSON.stringify(audit.failedTests) !== JSON.stringify(expected)) throw Error(`Unexpected failures: ${audit.failedTests}`);
  if (browser.tests.length !== 56 || browser.counts.passed !== 54 || browser.counts.failed !== 2 ||
      browser.counts.ignored !== 0 || browser.counts.filtered !== 0 || browser.errors.length !== 0) {
    throw Error('The control must finish all56 functions with only the two new failures');
  }
  audit.counts = browser.counts;
  audit.status = 'passed-negative-control';
} finally {
  for (const [file, bytes] of fixed) await writeFile(file, bytes);
  for (const item of audit.files) {
    item.restoredSha256 = hash(await readFile(item.file));
    if (item.restoredSha256 !== item.fixedSha256) throw Error(`Restore failed: ${item.file}`);
  }
  audit.restored = true;
  await writeFile(resolve(out, 'negative-audit.json'), JSON.stringify(audit, null, 2) + '\n');
}
console.log(JSON.stringify(audit.counts));
