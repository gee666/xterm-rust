// Runs the supplied xterm.js build/unit runner without modifying source or tests.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { sha256, referenceIntegrity, unitCounts, createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const source = path.join(root, 'source_repo');
const evidence = createEvidenceRun(path.join(root, 'results/upstream-baseline'), { kind: 'original-unit-baseline' });
const results = evidence.directory;
const relativeArtifact = file => path.relative(root, file).replaceAll('\\', '/');
const manifestPath = path.join(root, 'docs/baseline/reference-manifest.json');
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
fs.mkdirSync(results, { recursive: true });
const portable = value => value.replaceAll('\\', '/');
const records = [];

const checkReference = () => referenceIntegrity(source, manifest);

function requireIntegrity(check, phase) {
  if (check.mismatches.length) throw new Error(`${phase}: reference content differs from manifest`);
}

const MIN_FREE_BYTES = 256 * 1024 * 1024;
function checkDisk() {
  const disk = fs.statfsSync(root);
  const available = disk.bavail * disk.bsize;
  if (available < MIN_FREE_BYTES) throw new Error(`Only ${available} bytes free; baseline requires at least ${MIN_FREE_BYTES}`);
}

async function run(name, args, timeoutMs) {
  const { runCommand } = await import('./quality-process.mjs');
  checkDisk();
  const logFile = path.join(results, `${name}.log`);
  const log = relativeArtifact(logFile);
  const result = await runCommand(process.execPath, args, {
    cwd: source, log: logFile, timeoutMs,
    env: { ...process.env, FORCE_COLOR: '0', XTERM_BASELINE_REGISTRATION_FILE: path.join(results, 'unit-registration.json') }, resourceCheck: checkDisk
  });
  records.push({ ...result, command: ['node', ...args], cwd: 'source_repo', log });
  if (result.status !== 'passed') throw new Error(`${name} failed; inspect retained log and process metadata`);
}

function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const file = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(file) : [file];
  });
}

function unitSources() {
  const dirs = [path.join(source, 'src')];
  for (const entry of fs.readdirSync(path.join(source, 'addons'), { withFileTypes: true })) {
    const directory = path.join(source, 'addons', entry.name, 'src');
    if (entry.isDirectory() && fs.existsSync(directory)) dirs.push(directory);
  }
  return dirs.flatMap(walk).filter(file => file.endsWith('.test.ts')).sort();
}

function sourceForGenerated(file) {
  const relative = portable(path.relative(source, file));
  if (relative.endsWith('.test.ts') && fs.existsSync(file)) return relative;
  if (relative.startsWith('out-esbuild/')) {
    return `src/${relative.slice('out-esbuild/'.length).replace(/\.js$/, '.ts')}`;
  }
  const match = relative.match(/^(addons\/[^/]+)\/out-esbuild\/(.+)\.js$/);
  if (match) return `${match[1]}/src/${match[2]}.ts`;
  throw new Error(`Unexpected unit test output path: ${relative}`);
}

const before = checkReference();
const report = {
  schemaVersion: 1,
  reference: {
    provenance: 'Supplied source_repo tree, package version 6.0.0; no public commit asserted.',
    manifest: 'docs/baseline/reference-manifest.json',
    manifestSha256: sha256(fs.readFileSync(manifestPath)),
    packageLockSha256: sha256(fs.readFileSync(path.join(source, 'package-lock.json'))),
    before
  },
  environment: { node: process.version, platform: process.platform, arch: process.arch },
  commands: records,
  countSemantics: {
    pending: 'Original Mocha pending records, including source-level disabled cases; never counted as passed or executed.',
    skipped: 'Cases excluded by this wrapper. Zero means no added filters; original pending cases are reported separately.',
    unrun: 'Cases omitted by this wrapper after successful full original discovery. Missing generated modules fail the run.',
    rustMapped: 'Not counted here. Original JS execution does not prove any Rust assertion migration.'
  },
  status: 'failed'
};
try {
  requireIntegrity(before, 'before');
  const npmPackage = path.join(source, 'node_modules/mocha/package.json');
  if (!fs.existsSync(npmPackage)) {
    throw new Error('Install reference dependencies first: npm --prefix source_repo ci --ignore-scripts --no-audit --no-fund');
  }
  report.environment.mocha = JSON.parse(fs.readFileSync(npmPackage, 'utf8')).version;
  report.environment.esbuild = JSON.parse(fs.readFileSync(path.join(source, 'node_modules/esbuild/package.json'), 'utf8')).version;
  await run('runner-tests', ['--test', '../scripts/upstream-baseline.test.mjs'], 60_000);
  await run('esbuild', ['bin/esbuild_all.mjs'], 240_000);
  const rawFile = path.join(results, 'original-unit.json');
  await run('original-unit', [
    'bin/test_unit.js', '--reporter=json',
    `--reporter-options=output=../${relativeArtifact(rawFile)}`,
    '--require=../scripts/upstream-registration.cjs'
  ], 240_000);
  const raw = JSON.parse(fs.readFileSync(rawFile, 'utf8'));
  const registrations = JSON.parse(fs.readFileSync(path.join(results, 'unit-registration.json'), 'utf8'));
  const sourceByTest = new Map();
  const registrationKey = (file, title) => `${file}\0${title}`;
  for (const entry of registrations) {
    if (!entry.registrationFile) throw new Error(`Missing test callsite: ${entry.title}`);
    const key = registrationKey(entry.reportedFile, entry.title);
    const actualSource = sourceForGenerated(entry.registrationFile);
    if (sourceByTest.has(key) && sourceByTest.get(key) !== actualSource) {
      throw new Error(`Ambiguous registration source: ${entry.title}`);
    }
    sourceByTest.set(key, actualSource);
  }
  const sourceForTest = test => {
    const origin = sourceByTest.get(registrationKey(test.file, test.fullTitle));
    if (!origin) throw new Error(`No source registration metadata: ${test.fullTitle}`);
    return origin;
  };
  report.registrationReport = relativeArtifact(path.join(results, 'unit-registration.json'));
  report.registrationCorrections = registrations.filter(entry => sourceForGenerated(entry.reportedFile) !== sourceForGenerated(entry.registrationFile)).length;
  const suites = new Map();
  for (const file of unitSources()) {
    const relative = portable(path.relative(source, file));
    const generated = relative.startsWith('src/')
      ? `out-esbuild/${relative.slice(4).replace(/\.ts$/, '.js')}`
      : relative.replace('/src/', '/out-esbuild/').replace(/\.ts$/, '.js');
    if (!fs.existsSync(path.join(source, generated))) throw new Error(`Missing compiled test: ${relative}`);
    suites.set(relative, {
      source: relative, generated, discovered: 0, passed: 0, failed: 0, pending: 0
    });
  }
  for (const [key, field] of [['tests', 'discovered'], ['passes', 'passed'], ['failures', 'failed'], ['pending', 'pending']]) {
    for (const test of raw[key]) {
      const file = sourceForTest(test);
      const suite = suites.get(file);
      if (!suite) throw new Error(`Executed test has no inventoried source: ${file}`);
      suite[field]++;
    }
  }
  report.suites = [...suites.values()].map(suite => ({
    ...suite, ...unitCounts(suite.discovered, suite.passed, suite.failed, suite.pending),
    registration: suite.discovered ? 'test-bearing' : 'no registered cases; support-only status requires source review'
  }));
  report.totals = unitCounts(raw.tests.length, raw.passes.length, raw.failures.length, raw.pending.length);
  if (raw.stats.tests !== report.totals.discovered || raw.stats.passes !== report.totals.passed || raw.stats.failures !== report.totals.failed) {
    throw new Error('Mocha stats and raw result arrays disagree');
  }
  report.mochaSuites = raw.stats.suites;
  report.moduleCounts = {
    inventoried: report.suites.length,
    testBearing: report.suites.filter(suite => suite.discovered).length,
    noRegisteredCases: report.suites.filter(suite => !suite.discovered).map(suite => suite.source)
  };
  report.pending = raw.pending.map(test => ({ source: sourceForTest(test), title: test.fullTitle }));
  report.rawReport = relativeArtifact(rawFile);
  report.rawReportSha256 = sha256(fs.readFileSync(rawFile));
  report.browserBaseline = { status: 'unrun', reason: 'Separate Playwright browser discovery/execution required; unit success is not browser evidence.' };
  report.status = 'passed';
} catch (error) {
  report.error = String(error.stack);
} finally {
  report.reference.after = checkReference();
  if (report.reference.after.mismatches.length) report.status = 'failed';
  persistEvidence(evidence, report);
}
console.log(JSON.stringify({ status: report.status, summary: relativeArtifact(path.join(results, 'summary.json')), totals: report.totals, modules: report.moduleCounts, error: report.error }));
if (report.status !== 'passed') process.exitCode = 1;
