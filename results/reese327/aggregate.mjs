// Recompute counts from raw reports; failed application cases stay failed.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { testCounts } from '../../scripts/e2e/upstream-config.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const progress = JSON.parse(await fs.readFile(new URL('matrix-progress.json', import.meta.url), 'utf8'));
const start = Date.parse(progress.startedAt);
const rows = [], failures = [], pins = [];
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
async function read(relative) {
  const bytes = await fs.readFile(path.join(root, relative));
  pins.push({ path: relative, bytes: bytes.length, sha256: hash(bytes) });
  return JSON.parse(bytes);
}
function failedCases(suites, report, parents = []) {
  for (const suite of suites ?? []) {
    for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
      const result = test.results?.at(-1);
      if (result && !['passed', 'skipped'].includes(result.status)) failures.push({ report,
        file: spec.file, line: spec.line, title: [...parents, suite.title, spec.title].join(' / '),
        project: test.projectName, status: result.status, error: result.error });
    }
    failedCases(suite.suites, report, [...parents, suite.title]);
  }
}
for (const group of ['results/e2e/upstream', 'results/demos/browser']) {
  for (const entry of await fs.readdir(path.join(root, group), { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const report = `${group}/${entry.name}/summary.json`;
    let summary;
    try { summary = JSON.parse(await fs.readFile(path.join(root, report), 'utf8')); }
    catch (error) { if (error.code === 'ENOENT') continue; throw error; }
    if (summary.source?.sha256 !== progress.source.sha256) continue;
    const rawPath = `${group}/${entry.name}/playwright.json`;
    const rawBytes = await fs.readFile(path.join(root, rawPath));
    const raw = JSON.parse(rawBytes);
    const attemptStart = Date.parse(raw.stats.startTime);
    if (attemptStart < start || (progress.finishedAt && attemptStart > Date.parse(progress.finishedAt))) continue;
    await read(report);
    pins.push({ path: rawPath, bytes: rawBytes.length, sha256: hash(rawBytes) });
    const counts = testCounts(raw);
    assert.deepEqual(counts, summary.counts, `Raw/stored counter disagreement: ${report}`);
    const row = { name: summary.suite ?? summary.case, group, report, counts, status: summary.status,
      source: summary.source, commands: summary.commands,
      missingBindings: [...new Set((summary.runtimes ?? []).flatMap(runtime => runtime.missingBindings ?? []))] };
    rows.push(row);
    failedCases(raw.suites, rawPath);
  }
}
const originalRows = rows.filter(row => row.group === 'results/e2e/upstream');
const totals = Object.fromEntries(['discovered', 'executed', 'passed', 'failed', 'skipped', 'unrun'].map(key =>
  [key, originalRows.reduce((sum, row) => sum + row.counts[key], 0)]));
assert.equal(new Set(originalRows.map(row => row.name)).size, originalRows.length, 'Overlapping attempts must not be summed');
const report = { status: totals.failed || totals.unrun ? 'failed application checkpoint' : 'scoped execution passed',
  accounting: 'passed', source: progress.source, matrixStatus: progress.status,
  scope: 'Original assertions against Rust on frozen0442 debug package, not full port/native OS proof. Setup attempts are not body passes. WebGL tests may catch a missing addon and run DOM; missing bindings remain explicit.',
  originalConfigCount: originalRows.length, originalCounts: totals, rows, failures, pins };
await fs.writeFile(new URL('browser-summary.json', import.meta.url), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, accounting: report.accounting, configs: originalRows.length, totals,
  demos: rows.filter(row => row.group === 'results/demos/browser').map(row => ({ name: row.name, counts: row.counts })) }));
