// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, statfsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const directory = path.join(root, 'results/browser-mouse');
const attempt = createEvidenceRun(directory, { kind: 'browser-mouse-policy-check' });
const cargo = process.env.CARGO || 'cargo';
const rustfmt = process.env.RUSTFMT || 'rustfmt';
const files = ['mouse_coords', 'mouse_policy', 'mouse_wheel'].map(name => `crates/xterm-web/src/input/${name}.rs`)
  .concat(['mouse_policy', 'mouse_routing'].map(name => `crates/xterm-web/tests/${name}.rs`));
const checks = [
  { name: 'oracle', command: process.execPath, args: ['scripts/browser-mouse-oracle.mjs'] },
  { name: 'native-tests', command: cargo, args: ['test', '-p', 'xterm-web', '--test', 'mouse_policy', '--test', 'mouse_routing'], expectedTests: 13 },
  { name: 'fmt', command: rustfmt, args: ['--edition', '2021', '--check', ...files] },
  { name: 'clippy', command: cargo, args: ['clippy', '-p', 'xterm-web', '--lib', '--test', 'mouse_policy', '--test', 'mouse_routing', '--all-features', '--', '-D', 'warnings'] },
  { name: 'wasm-check', command: cargo, args: ['check', '-p', 'xterm-web', '--target', 'wasm32-unknown-unknown'] },
];
function free() { const fs = statfsSync(root); return fs.bavail * fs.bsize; }
const hashFiles = () => Object.fromEntries([...files,
  'crates/xterm-core/src/mouse_state.rs', 'crates/xterm-core/src/mouse_protocol.rs'].map(file =>
    [file, createHash('sha256').update(readFileSync(path.join(root, file))).digest('hex')]));
const hashesBefore = hashFiles();
const freeBefore = free();
const outcomes = [];
let failed = false;
for (const check of checks) {
  if (failed) { outcomes.push({ ...check, status: 'unrun' }); continue; }
  if (free() < 512 * 1024 * 1024) {
    outcomes.push({ ...check, status: 'failed', error: 'Less than 512 MiB free before build' }); failed = true; continue;
  }
  const run = spawnSync(check.command, check.args, { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 * 1024 });
  const output = `${run.stdout || ''}${run.stderr || ''}`;
  const file = path.join(attempt.directory, `${check.name}.log`);
  writeFileSync(file, output);
  const outcome = { ...check, exitCode: run.status, signal: run.signal,
    log: path.relative(root, file).replaceAll('\\', '/'), status: !run.error && run.status === 0 ? 'passed' : 'failed' };
  if (run.error) outcome.error = String(run.error);
  if (check.expectedTests) {
    const summaries = [...output.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
    outcome.passed = summaries.reduce((sum, match) => sum + Number(match[1]), 0);
    outcome.failed = summaries.reduce((sum, match) => sum + Number(match[2]), 0);
    outcome.skipped = summaries.reduce((sum, match) => sum + Number(match[3]), 0);
    if (outcome.passed !== check.expectedTests || outcome.failed || outcome.skipped) outcome.status = 'failed';
  }
  if (check.name === 'wasm-check' && /^warning:/m.test(output)) {
    outcome.status = 'failed'; outcome.error = 'WASM build emitted warnings; inspect retained log';
  }
  outcomes.push(outcome);
  failed = outcome.status !== 'passed';
}
const oracle = outcomes[0].status === 'passed'
  ? JSON.parse(readFileSync(path.join(directory, 'oracle.json'), 'utf8')) : null;
if (oracle) {
  writeFileSync(path.join(attempt.directory, 'oracle.json'), JSON.stringify(oracle, null, 2) + '\n');
  writeFileSync(path.join(attempt.directory, 'upstream.json'), readFileSync(path.join(directory, 'upstream.json')));
}
const hashes = hashFiles();
const sourceChangedDuringRun = JSON.stringify(hashesBefore) !== JSON.stringify(hashes);
if (sourceChangedDuringRun) failed = true;
const toolVersions = Object.fromEntries([['cargo', cargo], ['rustc', process.env.RUSTC || 'rustc'], ['rustfmt', rustfmt]].map(([name, command]) => {
  const run = spawnSync(command, ['--version'], { cwd: root, encoding: 'utf8', timeout: 10000 });
  if (run.error || run.status !== 0) failed = true;
  return [name, { exitCode: run.status, output: run.stdout?.trim(), error: run.error ? String(run.error) : run.stderr?.trim() }];
}));
const report = {
  scope: 'Pure browser mouse policy, original source assertions, native tests and WASM compile. Actual DOM/browser tests are separate and NOT PROVEN here.',
  command: 'node scripts/check-browser-mouse.mjs', node: process.version,
  platform: process.platform, arch: process.arch, toolVersions,
  sourceHashesBefore: hashesBefore, sourceHashes: hashes, sourceChangedDuringRun,
  freeBytesBefore: freeBefore, freeBytesAfter: free(), oracle, checks: outcomes,
  status: failed ? 'failed' : 'passed', browsersUnrun: ['chromium', 'firefox', 'webkit'],
  nativePlatformsUnrun: ['linux', 'darwin'].filter(platform => platform !== process.platform),
};
persistEvidence(attempt, report);
console.log(JSON.stringify({ status: report.status, runId: attempt.id,
  checks: outcomes.map(({name, status}) => ({name, status})) }));
if (failed) process.exitCode = 1;
