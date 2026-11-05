import {spawnSync} from 'node:child_process';
import {readFileSync, writeFileSync} from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {testCounts} from '../../../scripts/quality-policy.mjs';
const directory = 'results/quality/ci-42090157228';
const run = JSON.parse(readFileSync(`${directory}/run.json`, 'utf8'));
const artifacts = JSON.parse(readFileSync(`${directory}/artifacts.json`, 'utf8'));
const revision = '3aa04692a409bc147670d7c61a9e2ac37b5ea9f8';
if (run.sha !== revision || run.status !== 'completed' || run.conclusion !== 'failure' || run.jobs.length !== 8 || artifacts.length !== 8) throw new Error('Wrong/incomplete checkpoint');
if (run.jobs.filter(job => job.conclusion === 'success').length !== 7) throw new Error('Expected seven successful jobs');
const unzip = process.env.XTERM_UNZIP;
if (!unzip) throw new Error('Missing unzip executable path');
function zip(args) {
  const result = spawnSync(unzip, args, {encoding: 'utf8', timeout: 30000, maxBuffer: 16 * 1024 * 1024});
  if (result.error || result.status !== 0) throw new Error(result.stderr || String(result.error));
  return result.stdout;
}
const totals = counts => counts.reduce((out, item) => ({passed: out.passed + item.passed, failed: out.failed + item.failed, ignored: out.ignored + item.ignored, filteredOut: out.filteredOut + item.filteredOut, summaries: out.summaries + 1}), {passed: 0, failed: 0, ignored: 0, filteredOut: 0, summaries: 0});
const observations = [];
let entriesVerified = 0;
let crcLog = '';
for (const artifact of artifacts) {
  const filename = path.join(directory, artifact.filename);
  const entries = zip(['-Z', '-1', filename]).trim().split(/\r?\n/);
  if (entries.some(entry => entry.startsWith('/') || entry.split('/').includes('..'))) throw new Error('Unsafe ZIP name');
  crcLog += zip(['-t', filename]) + '\n';
  entriesVerified += entries.length;
  const read = name => {
    if (!entries.includes(name)) throw new Error(`Missing ${name} in ${filename}`);
    return zip(['-p', filename, name]);
  };
  const item = {name: artifact.name, filename: artifact.filename, zipEntries: entries.length};
  if (artifact.name === 'quality-miri-focused') {
    const log = read('miri.log');
    if (!log.includes(revision)) throw new Error('Miri revision mismatch');
    item.counts = totals(testCounts(log));
    item.toolVersions = log.split(/\r?\n/).filter(line => /^rustc |^miri |^cargo-miri |^release: |^host: /.test(line));
  } else if (artifact.name === 'quality-wasm-compile') {
    const log = read('compile.log');
    if (!log.startsWith(revision) || !log.includes('rustc 1.99.1') || !log.includes('cloned_ref_to_slice_refs') || !log.includes('wasm_webgl_atlas_page.rs:217')) throw new Error('Unexpected WASM failure');
    item.compilation = 'passed all-target/all-feature check';
    item.lint = 'failed actual stable1.99.1 cloned_ref_to_slice_refs at wasm_webgl_atlas_page.rs:217';
    item.browserRuntime = 'UNRUN';
  } else {
    const summary = JSON.parse(read('summary.json'));
    if (summary.environment.ciCommit !== revision || read('git-revision.log').trim() !== revision || summary.gateStatus !== 'passed' || summary.failedGates.length || !summary.sourceStable) throw new Error(`Native provenance/gate mismatch ${artifact.name}`);
    const commands = [];
    for (const command of summary.commands) {
      const log = read(path.basename(command.log));
      const counts = testCounts(log);
      if (JSON.stringify(counts) !== JSON.stringify(command.testSummaries)) throw new Error(`Raw count mismatch ${artifact.name} ${command.id}`);
      commands.push({id: command.id, status: command.status, ...totals(counts)});
    }
    Object.assign(item, {platform: summary.environment.platform, architecture: summary.environment.architecture, execution: summary.environment.execution, runnerEnvironment: summary.environment.runnerEnvironment, rustHost: summary.environment.rustHost, resolvedOs: summary.environment.resolvedCargoTargetOs, rustc: read('rustc-version.log').trim(), sourceSha256: summary.sourceBefore.sha256, sourceStable: summary.sourceStable, targetDirectory: summary.environment.cargoTargetDirectory, gitStatus: read('git-status.log').trim(), gateStatus: summary.gateStatus, commands});
  }
  observations.push(item);
}
const writing = process.argv.includes('--write');
if (writing) writeFileSync(`${directory}/zip-crc.log`, crcLog);
else if (readFileSync(`${directory}/zip-crc.log`, 'utf8') !== crcLog) throw new Error('Retained CRC log mismatch');
const recordedAt = writing ? new Date().toISOString() : JSON.parse(readFileSync(`${directory}/observed-artifacts.json`, 'utf8')).recordedAt;
const result = {recordedAt, revision, run: run.id, conclusion: run.conclusion, jobs: run.jobs.map(({id, name, status, conclusion}) => ({id, name, status, conclusion})), artifactCount: artifacts.length, zipEntriesVerified: entriesVerified, observations, note: 'Scoped native-quality checkpoint only. Seven jobs passed; WASM lint failed. Browser runtime unrun. Later producers and final304 remain outside this checkpoint.'};
const serialized = JSON.stringify(result, null, 2) + '\n';
if (writing) writeFileSync(`${directory}/observed-artifacts.json`, serialized);
else if (readFileSync(`${directory}/observed-artifacts.json`, 'utf8') !== serialized) throw new Error('Retained observations mismatch');
const files = [...artifacts.map(artifact => artifact.filename), ...run.jobs.map(job => `job-${job.id}.log`), 'run.json', 'artifacts.json', 'compile.log', 'observed-artifacts.json', 'zip-crc.log', 'audit.mjs'];
const sums = files.sort().map(file => createHash('sha256').update(readFileSync(`${directory}/${file}`)).digest('hex') + '  ' + `${directory}/${file}`).join('\n') + '\n';
if (writing) writeFileSync(`${directory}/SHA256SUMS`, sums);
else if (readFileSync(`${directory}/SHA256SUMS`, 'utf8') !== sums) throw new Error('Retained checksum mismatch');
console.log(JSON.stringify({...result, observations: observations.map(item => ({...item, commands: item.commands?.filter(command => command.summaries || command.status !== 'passed')}))}, null, 2));
