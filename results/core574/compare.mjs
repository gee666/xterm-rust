// Named-case accounting only, not assertion or terminal-state equivalence.
// Reuse Reese's identity algorithm in this evidence-only script because its
// historical all14 CLI is not exported and would overwrite preserved reports.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { testCounts } from '../../scripts/e2e/upstream-config.mjs';
const archive = '43ffa9b1d88c6d96b794aea3c5883079b22aa238';
const sourceSummary = 'results/upstream-browser/runs/2026-11-06T00-57-19-880Z-085e4abf-05e5-4305-8f60-de0b5460177c/summary.json';
const sourceRaw = sourceSummary.replace('summary.json', 'core-all-runtime.json');
const rustDir = 'results/e2e/upstream/2026-11-06T03-34-52-313Z-b2a537a0-12f7-45bc-993f-dd7631d760df';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const pins = [];
function original(path) {
  const bytes = execFileSync('C:/Program Files/Git/cmd/git.exe', ['show', `${archive}:${path}`], { maxBuffer: 16 * 1024 * 1024, timeout: 30000 });
  pins.push({ archive, path, bytes: bytes.length, sha256: sha(bytes) });
  return JSON.parse(bytes);
}
async function actual(path) {
  const bytes = await readFile(path);
  pins.push({ path, bytes: bytes.length, sha256: sha(bytes) });
  return JSON.parse(bytes);
}
const previous = original('results/reese327/source-comparison-updated.json');
const baseline = original(sourceSummary);
const source = original(sourceRaw);
for (const path of [sourceSummary, sourceRaw]) {
  const prior = previous.pins.find(pin => pin.path === path);
  const current = pins.find(pin => pin.path === path);
  assert(prior);
  assert.equal(current.sha256, prior.sha256);
}
assert.equal(baseline.server.status, 'stopped');
assert.deepEqual(baseline.referenceBefore.mismatches, []);
assert.deepEqual(baseline.referenceAfter.mismatches, []);
const runtime = await actual(`${rustDir}/summary.json`);
const rust = await actual(`${rustDir}/playwright.json`);
assert.equal(runtime.status, 'passed');
assert.equal(runtime.source.sha256, '19ffb731fe479eb0630bf8b01c74464c7be91ae039ab4641ab11e56d18b7cf95');
assert.deepEqual(runtime.errors, []);
assert.deepEqual(runtime.serverErrors, []);
assert(runtime.runtimes.length && runtime.runtimes.every(item => item.state === 'ready'));
function cases(raw) {
  const records = new Map();
  function visit(suites, parents = []) {
    for (const suite of suites ?? []) {
      const title = parents.length ? suite.title : suite.title.replace(/\.test\.js$/, '.test.ts');
      const names = [...parents, title];
      for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) {
        const key = [test.projectName, ...names, spec.title].join('\0');
        assert(!records.has(key), `Duplicate identity ${key}`);
        const result = test.results?.at(-1);
        const declaredSkip = test.expectedStatus === 'skipped' || test.annotations?.some(annotation => ['skip', 'fixme'].includes(annotation.type));
        const status = !result || result.status === 'skipped' && !declaredSkip ? 'unrun'
          : result.status === 'passed' ? 'passed' : result.status === 'skipped' ? 'skipped' : 'failed';
        records.set(key, { project: test.projectName, title: [...names, spec.title].join(' / '), status,
          rawStatus: result?.status ?? null, file: spec.file, line: spec.line });
      }
      visit(suite.suites, names);
    }
  }
  visit(raw.suites);
  return records;
}
const sourceCases = cases(source), rustCases = cases(rust), pairs = [], transitions = {}, projects = {};
assert.equal(sourceCases.size, 1434);
assert.equal(rustCases.size, sourceCases.size);
for (const [key, expected] of sourceCases) {
  const observed = rustCases.get(key);
  assert(observed, `Unmapped Rust case ${key}`);
  assert.equal(observed.status, expected.status, `Outcome differs for ${key}`);
  const transition = `${expected.status}->${observed.status}`;
  transitions[transition] = (transitions[transition] ?? 0) + 1;
  const counts = projects[observed.project] ??= { passed: 0, skipped: 0, failed: 0, unrun: 0 };
  counts[observed.status]++;
  pairs.push({ project: expected.project, title: expected.title, sourceStatus: expected.status,
    rustStatus: observed.status, sourceFile: expected.file, sourceLine: expected.line,
    rustFile: observed.file, rustLine: observed.line });
}
assert.deepEqual(testCounts(rust), { discovered: 1434, executed: 1194, passed: 1194, failed: 0, skipped: 240, unrun: 0 });
assert.deepEqual(testCounts(source), testCounts(rust));
assert.deepEqual(transitions, { 'passed->passed': 1194, 'skipped->skipped': 240 });
const previousCore = previous.cases.filter(row => row.config === 'core');
assert.equal(previousCore.length, 1434);
const currentByName = new Map(pairs.map(row => [[row.project, row.title].join('\0'), row]));
const repaired = previousCore.filter(row => row.rustStatus === 'failed').map(row => {
  assert.equal(currentByName.get([row.project, row.title].join('\0'))?.rustStatus, 'passed');
  return { project: row.project, title: row.title, previousStatus: row.rustStatus, currentStatus: 'passed' };
});
assert.equal(repaired.length, 27);
const report = { status: 'passed named-case accounting',
  scope: 'Current full original core config only. Not every assertion, all14 configs, full state, native OS or complete equivalence. Historical other configs have different Rust input identities and are not totaled here.',
  originalAll14BaselineStatus: baseline.status, sourceArchive: archive, sourceSummary, sourceRaw,
  rustDir, source: runtime.source, sourceCounts: testCounts(source), rustCounts: testCounts(rust),
  mapped: pairs.length, projects, transitions, previousCoreFailuresNowPassed: repaired,
  runtimeReady: runtime.runtimes.length,
  missingBindings: [...new Set(runtime.runtimes.flatMap(item => item.missingBindings))],
  pins, cases: pairs };
await writeFile('results/core574/source-pairing.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ mapped: pairs.length, transitions, projects, counts: testCounts(rust) }));
