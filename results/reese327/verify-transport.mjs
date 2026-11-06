// Verify transport completion separately from remaining failed application assertions.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { testCounts } from '../../scripts/e2e/upstream-config.mjs';
import { sourceIdentity, verifyBuiltPackage } from '../../scripts/e2e/build-identity.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const pins = [];
async function read(file) {
  const bytes = await fs.readFile(path.join(root, file));
  pins.push({ path: file, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
  return JSON.parse(bytes);
}
function setupFailures(raw) {
  const failures = [];
  function visit(suites) {
    for (const suite of suites ?? []) {
      for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
        const result = test.results?.at(-1);
        if (result?.error?.message?.includes('beforeAll')) failures.push({ title: spec.title,
          file: spec.file, project: test.projectName, error: result.error });
        assert((test.results?.length ?? 0) <= 1, 'No retries allowed');
      }
      visit(suite.suites);
    }
  }
  visit(raw.suites); return failures;
}
const progress = await read('results/reese327/transport-browser-progress.json');
assert(progress.finishedAt && progress.active === null);
assert.deepEqual(progress.errors, []);
assert.equal(progress.commands.length, 3);
const source = await sourceIdentity(root);
assert.equal(source.sha256, progress.source.sha256);
assert.equal(source.files, 1866);
const build = await verifyBuiltPackage(root, source);
const packages = await read('results/integration531/package-files.json');
assert.equal(packages.length, 75);
for (const entry of packages) {
  const bytes = await fs.readFile(path.join(root, entry.path));
  assert.equal(bytes.length, entry.bytes);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, entry.path);
}
const initial = await read('results/reese327/browser-summary.json');
const oldCore = initial.rows.find(row => row.name === 'core');
const oldRaw = await read(path.posix.join(path.posix.dirname(oldCore.report), 'playwright.json'));
const oldSetupFailures = setupFailures(oldRaw);
assert.equal(oldSetupFailures.length, 6);
const rows = [];
for (const directory of await fs.readdir(path.join(root, 'results/e2e/upstream'))) {
  const summaryFile = `results/e2e/upstream/${directory}/summary.json`;
  let summary;
  try { summary = JSON.parse(await fs.readFile(path.join(root, summaryFile), 'utf8')); }
  catch (error) { if (error.code === 'ENOENT') continue; throw error; }
  if (summary.source?.sha256 !== source.sha256) continue;
  const rawFile = `results/e2e/upstream/${directory}/playwright.json`;
  const raw = JSON.parse(await fs.readFile(path.join(root, rawFile), 'utf8'));
  const start = Date.parse(raw.stats.startTime);
  if (start < Date.parse(progress.startedAt) || start > Date.parse(progress.finishedAt)) continue;
  await read(summaryFile); await read(rawFile);
  const counts = testCounts(raw);
  assert.deepEqual(counts, summary.counts);
  assert.equal(counts.unrun, 0);
  assert.equal(raw.config.workers, 2);
  assert(raw.config.projects.every(project => project.retries === 0));
  assert.deepEqual(setupFailures(raw), []);
  assert.deepEqual(summary.serverErrors, []);
  const wasmRequests = summary.requests.filter(request => request.url === '/pkg/xterm_web_bg.wasm');
  assert(wasmRequests.length > 0 && wasmRequests.every(request => request.status === 200));
  assert(summary.runtimes.length > 0 && summary.runtimes.every(runtime => runtime.state === 'ready'));
  rows.push({ name: summary.suite, group: 'results/e2e/upstream', report: summaryFile, counts,
    status: summary.status, source: summary.source, wasmRequests: wasmRequests.length,
    missingBindings: [...new Set(summary.runtimes.flatMap(runtime => runtime.missingBindings))] });
}
assert.deepEqual(rows.map(row => row.name).sort(), ['addon-fit', 'addon-image', 'core']);
const updatedRows = initial.rows.filter(row => row.group === 'results/e2e/upstream').map(row => rows.find(next => next.name === row.name) ?? row);
const totals = Object.fromEntries(['discovered', 'executed', 'passed', 'failed', 'skipped', 'unrun'].map(key =>
  [key, updatedRows.reduce((sum, row) => sum + row.counts[key], 0)]));
assert.equal(totals.discovered, 4116); assert.equal(totals.unrun, 0);
const proof = { transportVerification: 'passed', applicationStatus: 'failed; full equivalence NOT PROVEN', source, build,
  unchangedPackageFiles: packages.length, oldCore: { report: oldCore.report, counts: oldCore.counts, setupFailures: oldSetupFailures },
  newCompleteConfigRows: rows, pins,
  scope: 'Three complete unfiltered configs on unchanged Rust package. All application failures remain failures; no original hook/assertion/retry setting changed.' };
await fs.writeFile(new URL('transport-proof.json', import.meta.url), JSON.stringify(proof, null, 2) + '\n');
await fs.writeFile(new URL('transport-updated-summary.json', import.meta.url), JSON.stringify({
  originalConfigCount: 14, originalCounts: totals, rows: updatedRows,
  scope: 'Derived latest complete-config accounting across first14cfg and later3cfg fetch attempts on identical Rust inputs. Not one fresh14cfg run or full parity.'
}, null, 2) + '\n');
console.log(JSON.stringify({ transport: 'passed', newRows: rows.map(row => ({ name: row.name, counts: row.counts })), derivedLatest: totals }));
