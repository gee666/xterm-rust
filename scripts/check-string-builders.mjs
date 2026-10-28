// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Scoped original/Rust string-builder verification.
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, statfsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const directory = path.join(root, 'results/string-builder');
mkdirSync(directory, { recursive: true });
const attempt = createEvidenceRun(directory, { kind: 'string-builder-check' });
const cargo = process.env.CARGO || 'cargo';
const rustfmt = process.env.RUSTFMT || 'rustfmt';
const rustc = process.env.RUSTC || 'rustc';
const rustFiles = ['crates/xterm-core/src/string_builder.rs',
  ...['upstream', 'differential', 'edges'].map(name => `crates/xterm-core/tests/string_builder_${name}.rs`)];
const checks = [
  { name: 'expanded-oracle', command: process.execPath, args: ['--experimental-transform-types', 'scripts/string-builder-oracle.mjs'] },
  { name: 'expanded-tests', command: cargo, args: ['test', '-p', 'xterm-core', '--test', 'string_builder_upstream', '--test', 'string_builder_differential', '--test', 'string_builder_edges'], expectedTests: 27 },
  { name: 'expanded-rustfmt', command: rustfmt, args: ['--edition', '2021', '--check', ...rustFiles] },
  { name: 'expanded-clippy', command: cargo, args: ['clippy', '-p', 'xterm-core', '--lib', '--test', 'string_builder_upstream', '--test', 'string_builder_differential', '--test', 'string_builder_edges', '--all-features', '--', '-D', 'warnings'] },
  { name: 'parser-integration', command: cargo, args: ['test', '-p', 'xterm-core', '--test', 'dcs', '--test', 'osc', '--test', 'apc'], minimumTests: 48 },
];
function version(command) {
  const run = spawnSync(command, ['--version'], { cwd: root, encoding: 'utf8', timeout: 10000 });
  if (run.error || run.status !== 0) throw run.error || new Error(run.stderr);
  return run.stdout.trim();
}
function space() {
  const filesystem = statfsSync(root);
  return filesystem.bavail * filesystem.bsize;
}
const freeBefore = space();
const outcomes = [];
let failed = false;
for (const check of checks) {
  if (failed) {
    outcomes.push({ ...check, status: 'unrun' });
    continue;
  }
  // Honor the shared low-artifact profiles; never create an alternate target.
  if (space() < 512 * 1024 * 1024) {
    outcomes.push({ ...check, status: 'failed', error: 'Less than 512 MiB free before build; free disk and rerun.' });
    failed = true;
    continue;
  }
  const run = spawnSync(check.command, check.args, { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 * 1024 });
  const output = `${run.stdout || ''}${run.stderr || ''}`;
  const log = path.relative(root, path.join(attempt.directory, `${check.name}.log`)).replaceAll('\\', '/');
  writeFileSync(path.join(root, log), output);
  const outcome = { ...check, exitCode: run.status, signal: run.signal, log,
    status: !run.error && run.status === 0 ? 'passed' : 'failed' };
  if (run.error) outcome.error = String(run.error);
  if (check.expectedTests || check.minimumTests) {
    const summaries = [...output.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
    outcome.passed = summaries.reduce((sum, match) => sum + Number(match[1]), 0);
    outcome.failed = summaries.reduce((sum, match) => sum + Number(match[2]), 0);
    outcome.skipped = summaries.reduce((sum, match) => sum + Number(match[3]), 0);
    if ((check.expectedTests && outcome.passed !== check.expectedTests) ||
        (check.minimumTests && outcome.passed < check.minimumTests) || outcome.failed || outcome.skipped) {
      outcome.status = 'failed';
      outcome.error = 'Test counts or skips do not meet the acceptance inventory';
    }
  }
  outcomes.push(outcome);
  failed = outcome.status !== 'passed';
}
const oracle = JSON.parse(readFileSync(path.join(directory, 'oracle.json'), 'utf8'));
writeFileSync(path.join(attempt.directory, 'oracle.json'), `${JSON.stringify(oracle, null, 2)}\n`);
const hashes = Object.fromEntries(rustFiles.map(file => [file,
  createHash('sha256').update(readFileSync(path.join(root, file))).digest('hex')]));
const report = {
  scope: 'String builders and specified parser-adapter tests only. Full terminal equivalence NOT PROVEN.',
  command: 'node scripts/check-string-builders.mjs', platform: process.platform, arch: process.arch,
  node: process.version, rustc: version(rustc), cargo: version(cargo),
  freeBytesBefore: freeBefore, freeBytesAfter: space(), sourceHashes: hashes,
  oracle: { sourceSha256: oracle.sourceSha256, testsSha256: oracle.testsSha256,
    fixtureSha256: oracle.fixtureSha256, baseline: oracle.baseline, seed: oracle.seed,
    traces: oracle.traces, operations: oracle.operations, exhaustiveSingleUnits: oracle.exhaustiveSingleUnits },
  checks: outcomes, status: failed ? 'failed' : 'passed',
  nativePlatformsUnrun: ['linux', 'darwin'].filter(platform => platform !== process.platform),
};
persistEvidence(attempt, report);
writeFileSync(path.join(directory, 'expanded-verification.json'), `${JSON.stringify({ ...report, runId: attempt.id }, null, 2)}\n`);
console.log(JSON.stringify({ status: report.status, checks: outcomes.map(({name, status}) => ({name, status})),
  traces: oracle.traces, operations: oracle.operations }));
if (failed) process.exitCode = 1;
