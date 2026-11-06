// Compare named case outcomes, not source/Rust output equivalence.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { testCounts } from '../../scripts/e2e/upstream-config.mjs';
const root = fileURLToPath(new URL('../../', import.meta.url));
const args = process.argv.slice(2);
if (args.length > 1 || (args[0] && args[0] !== '--updated')) throw new Error('Use only optional --updated');
const updated = args[0] === '--updated';
const sourceReport = 'results/upstream-browser/runs/2026-11-06T00-57-19-880Z-085e4abf-05e5-4305-8f60-de0b5460177c/summary.json';
const pins = [];
async function read(file) {
  const bytes = await fs.readFile(path.join(root, file));
  pins.push({ path: file, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') });
  return JSON.parse(bytes);
}
function cases(raw, config, report) {
  const records = new Map();
  function visit(suites, parents = []) {
    for (const suite of suites ?? []) {
      // Only the generated top-level file name changes from .ts to .js.
      const title = parents.length ? suite.title : suite.title.replace(/\.test\.js$/, '.test.ts');
      const names = [...parents, title];
      for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
        const key = [config, test.projectName, ...names, spec.title].join('\0');
        assert(!records.has(key), `Duplicate case identity: ${key}`);
        const result = test.results?.at(-1);
        const declaredSkip = test.expectedStatus === 'skipped' || test.annotations?.some(annotation => ['skip', 'fixme'].includes(annotation.type));
        const status = !result || (result.status === 'skipped' && !declaredSkip) ? 'unrun'
          : result.status === 'passed' ? 'passed' : result.status === 'skipped' ? 'skipped' : 'failed';
        records.set(key, { config, project: test.projectName, title: [...names, spec.title].join(' / '),
          status, rawStatus: result?.status ?? null, report, file: spec.file, line: spec.line });
      }
      visit(suite.suites, names);
    }
  }
  visit(raw.suites);
  return records;
}
const source = await read(sourceReport);
assert.equal(source.suites.length, 14);
assert.equal(source.server.status, 'stopped');
assert.deepEqual(source.referenceBefore.mismatches, []);
assert.deepEqual(source.referenceAfter.mismatches, []);
const rust = await read(updated ? 'results/reese327/transport-updated-summary.json' : 'results/reese327/browser-summary.json');
assert.equal(rust.originalConfigCount, 14);
const sourceRows = [], compared = [], transitions = {};
const totals = Object.fromEntries(['discovered', 'executed', 'passed', 'failed', 'skipped', 'unrun'].map(key => [key, 0]));
for (const suite of source.suites) {
  const raw = await read(suite.raw);
  const counts = testCounts(raw);
  for (const key of Object.keys(totals)) totals[key] += counts[key];
  sourceRows.push({ name: suite.name, raw: suite.raw, counts });
  const row = rust.rows.find(row => row.group === 'results/e2e/upstream' && row.name === suite.name);
  assert(row, `Missing Rust config: ${suite.name}`);
  const rustRawPath = path.posix.join(path.posix.dirname(row.report), 'playwright.json');
  const sourceCases = cases(raw, suite.name, suite.raw);
  const rustCases = cases(await read(rustRawPath), suite.name, rustRawPath);
  assert.equal(sourceCases.size, rustCases.size);
  for (const [key, original] of sourceCases) {
    const actual = rustCases.get(key);
    assert(actual, `No corresponding Rust case: ${key}`);
    const transition = `${original.status}->${actual.status}`;
    transitions[transition] = (transitions[transition] ?? 0) + 1;
    compared.push({ ...original, sourceStatus: original.status, rustStatus: actual.status,
      sourceRawStatus: original.rawStatus, rustRawStatus: actual.rawStatus, rustReport: actual.report });
  }
}
assert.equal(totals.discovered, 4116);
assert.equal(compared.length, 4116);
const report = { accounting: 'passed', originalBaselineStatus: source.status,
  rustAccountingMode: updated ? 'Derived first14cfg plus later3 complete-config fetch attempts, not one fresh14cfg matrix' : 'Initial complete14cfg attempt',
  scope: 'One-to-one named browser case outcome mapping, not assertion counts or full output equivalence. No failed/skipped/unrun attempt is erased. Rust WebGL passes can exercise DOM while its addon is absent.',
  sourceReport, sourceCounts: totals, rustInitialCounts: rust.originalCounts,
  mappedCaseProjectCombinations: compared.length, transitions, sourceRows, cases: compared, pins };
await fs.writeFile(new URL(updated ? 'source-comparison-updated.json' : 'source-comparison.json', import.meta.url), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ accounting: report.accounting, source: totals, rust: report.rustInitialCounts,
  mapped: compared.length, transitions }));
