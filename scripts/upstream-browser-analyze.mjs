// Derive exact source/project accounting without rewriting completed attempts.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createEvidenceRun, persistEvidence, sha256 } from './upstream-baseline-utils.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo');
const evidence = createEvidenceRun(path.join(root, 'results/upstream-browser-analysis'), { kind: 'original-browser-accounting' });
const portable = file => file.replaceAll('\\', '/');
const relative = file => portable(path.relative(root, file));
const counts = () => ({ discovered: 0, executed: 0, passed: 0, failed: 0, skipped: 0, unrun: 0 });
const report = { status: 'failed', suites: [], failures: [], totals: counts(), browserVersions: [] };
try {
  const pointer = JSON.parse(fs.readFileSync(path.join(root, 'results/upstream-browser/latest.json'), 'utf8'));
  const inputFile = process.argv[2] ? path.resolve(root, process.argv[2])
    : path.resolve(root, 'results/upstream-browser', pointer.summary);
  const inputBytes = fs.readFileSync(inputFile);
  const input = JSON.parse(inputBytes);
  if (input.mode !== 'runtime') throw new Error('Runtime source accounting requires a runtime receipt, not discovery');
  report.input = { path: relative(inputFile), sha256: sha256(inputBytes), status: input.status };
  report.scope = 'Derived accounting of original JS browser tests, not Rust parity. Failed tests remain failed.';
  const bySource = new Map();
  for (const suite of input.suites) {
    for (const test of suite.tests ?? []) {
      const original = path.resolve(source, path.dirname(suite.generatedConfig), test.source);
      if (!original.startsWith(source + path.sep) || !fs.existsSync(original)) {
        throw new Error(`Unresolved original source: ${suite.name} / ${test.source}`);
      }
      const sourceFile = relative(original);
      const key = `${suite.name}\0${sourceFile}\0${test.project}`;
      let entry = bySource.get(key);
      if (!entry) {
        entry = { source: sourceFile, sha256: sha256(fs.readFileSync(original)),
          suite: suite.name, project: test.project, ...counts() };
        bySource.set(key, entry);
      }
      entry.discovered++;
      const result = test.results.at(-1);
      if (!result) entry.unrun++;
      else if (result.status === 'skipped') entry.skipped++;
      else {
        entry.executed++;
        if (result.status === 'passed') entry.passed++;
        else {
          entry.failed++;
          report.failures.push({ source: sourceFile, suite: suite.name, project: test.project,
            title: test.title, expectedStatus: test.expectedStatus,
            status: result.status, error: result.error ?? result.errors ?? null });
        }
      }
    }
    for (const [project, expected] of Object.entries(suite.projects ?? {})) {
      const actual = counts();
      for (const entry of bySource.values()) {
        if (entry.suite === suite.name && entry.project === project) {
          for (const key of Object.keys(actual)) actual[key] += entry[key];
        }
      }
      for (const key of Object.keys(actual)) {
        if (expected[key] !== actual[key]) throw new Error(`Source totals disagree: ${suite.name}/${project}/${key}`);
      }
    }
  }
  report.suites = [...bySource.values()];
  for (const entry of report.suites) {
    for (const key of Object.keys(report.totals)) report.totals[key] += entry[key];
  }
  report.originalProcessStatus = input.status;
  report.server = input.server;
  report.referenceAfter = input.referenceAfter;
  // These launches identify the installed browser binaries after the baseline.
  // They do not rerun assertions or turn a failed baseline into a pass.
  const playwright = await import(pathToFileURL(path.join(source, 'node_modules/playwright/index.mjs')).href);
  for (const name of ['chromium', 'firefox', 'webkit']) {
    const browser = await playwright[name].launch({ timeout: 15_000 });
    try {
      report.browserVersions.push({ name, version: browser.version(),
        executable: playwright[name].executablePath(), measuredAt: new Date().toISOString() });
    } finally { await browser.close(); }
  }
  report.status = 'passed';
} catch (error) {
  report.error = String(error.stack);
} finally {
  persistEvidence(evidence, report);
}
console.log(JSON.stringify({ accounting: report.status, originalBaseline: report.originalProcessStatus,
  summary: relative(path.join(evidence.directory, 'summary.json')), totals: report.totals,
  versions: report.browserVersions, error: report.error }));
if (report.status !== 'passed') process.exitCode = 1;
