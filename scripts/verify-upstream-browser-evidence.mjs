// Independently audit the retained xterm.js 6 browser matrix, not Rust parity.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createEvidenceRun, persistEvidence, referenceIntegrity, sha256 } from './upstream-baseline-utils.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const original = path.join(root, 'source_repo');
const attempt = createEvidenceRun(path.join(root, 'results/upstream-browser-review'), { scope:'Reference browser artifact audit only' });
const report = { status:'failed', scope:'Audit of original JS records; neither an original green baseline nor Rust parity',
  overallEquivalence:'NOT PROVEN', raw:[], attachments:[], counts:{} };
const relative = file => path.relative(root,file).replaceAll('\\','/');
function regular(file) {
  const resolved = path.resolve(root,file);
  const rel = path.relative(root,resolved);
  assert(!path.isAbsolute(rel) && rel !== '..' && !rel.startsWith('..' + path.sep), 'artifact escapes repository');
  let at = root;
  for (const part of rel.split(path.sep)) {
    at = path.join(at,part);
    assert(!fs.lstatSync(at).isSymbolicLink(), 'audit refuses symbolic artifact paths');
  }
  assert(fs.statSync(resolved).isFile(), 'artifact is not a regular file');
  return resolved;
}
function freshCounts() { return { discovered:0, executed:0, passed:0, failed:0, skipped:0, unrun:0 }; }
function collect(suites, parents=[], result=[]) {
  for (const suite of suites) {
    const names = [...parents,suite.title];
    for (const spec of suite.specs ?? []) for (const test of spec.tests ?? []) result.push({
      source:spec.file ?? suite.file, title:[...names,spec.title].join(' / '), project:test.projectName,
      expectedStatus:test.expectedStatus, results:test.results ?? [], outcome:test.status ?? null
    });
    collect(suite.suites ?? [],names,result);
  }
  return result;
}
try {
  assert(process.argv[2], 'Pass the immutable complete runtime summary path');
  const inputFile = regular(process.argv[2]);
  const inputBytes = fs.readFileSync(inputFile);
  const input = JSON.parse(inputBytes);
  report.input = { path:relative(inputFile), sha256:sha256(inputBytes) };
  assert.equal(input.mode,'runtime');
  assert.equal(input.schemaVersion,1);
  assert.match(input.runId,/^[a-zA-Z0-9-]+$/);
  assert.equal(input.selectedProject,'all original projects');
  assert.equal(input.configInventory.length,14);
  assert.equal(input.suites.length,14);
  assert.deepEqual([...new Set(input.suites.map(s=>s.name))].sort(),[...input.configInventory].sort());
  const manifest = JSON.parse(fs.readFileSync(path.join(root,'docs/baseline/reference-manifest.json'),'utf8'));
  report.reference = referenceIntegrity(original,manifest);
  assert.equal(report.reference.files,776); assert.deepEqual(report.reference.mismatches,[]);
  for (const prior of [input.referenceBefore,input.referenceAfter]) {
    assert.equal(prior.files,776); assert.deepEqual(prior.mismatches,[]);
  }
  const total = freshCounts();
  const projects = ['Chromium','FirefoxStable','WebKit'];
  const runRoot = path.join(root,'results/upstream-browser/runs',input.runId);
  for (const suite of input.suites) {
    const rawFile = regular(suite.raw);
    assert(rawFile.startsWith(runRoot + path.sep), 'raw record belongs to another attempt');
    regular(suite.log);
    const bytes = fs.readFileSync(rawFile);
    const raw = JSON.parse(bytes);
    assert.deepEqual(raw.errors ?? [],suite.errors ?? []);
    assert.deepEqual(raw.stats,suite.rawStats);
    const tests = collect(raw.suites ?? []);
    assert.deepEqual(tests,suite.tests,'summary differs from raw Playwright cases/results');
    assert.deepEqual(Object.keys(suite.projects).sort(),projects);
    assert.equal(suite.command.timedOut,false); assert.equal(suite.command.resourceError,null);
    assert.equal(suite.command.terminationError,null);
    const byProject = Object.fromEntries(projects.map(name=>[name,freshCounts()]));
    for (const test of tests) {
      const counts = byProject[test.project]; assert(counts,'unknown original project');
      counts.discovered++;
      const result = test.results.at(-1);
      if (!result) counts.unrun++;
      else if (result.status === 'skipped') counts.skipped++;
      else { counts.executed++; counts[result.status === 'passed'?'passed':'failed']++; }
      const sourceFile = regular(path.resolve(original,path.dirname(suite.generatedConfig),test.source));
      assert(sourceFile.startsWith(original + path.sep),'test provenance escapes original source');
      for (const attempt of test.results) for (const attachment of attempt.attachments ?? []) {
        if (!attachment.path) continue;
        const file = regular(attachment.path);
        report.attachments.push({ path:relative(file), sha256:sha256(fs.readFileSync(file)) });
      }
    }
    assert.deepEqual(byProject,suite.projects,'raw project counters differ');
    for (const counts of Object.values(byProject)) for (const key of Object.keys(total)) total[key]+=counts[key];
    report.raw.push({ suite:suite.name, path:relative(rawFile), sha256:sha256(bytes), cases:tests.length });
  }
  assert.deepEqual(total,{discovered:4116,executed:3824,passed:3752,failed:72,skipped:292,unrun:0});
  assert.equal(input.status,'failed','the known original failures must not be rewritten as a green baseline');
  assert.equal(input.server.status,'stopped');
  assert(!input.shutdownError,'retained server shutdown failed');
  report.counts=total; report.originalRuntimeStatus=input.status;
  report.serverStatus=input.server.status; report.status='passed';
} catch (error) { report.error=String(error.stack); }
persistEvidence(attempt,report);
console.log(JSON.stringify({ audit:report.status, originalRuntimeStatus:report.originalRuntimeStatus,
  counts:report.counts, rawReports:report.raw.length, attachments:report.attachments.length,
  summary:relative(path.join(attempt.directory,'summary.json')), error:report.error }));
if (report.status !== 'passed') process.exitCode=1;
