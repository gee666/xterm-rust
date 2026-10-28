// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { runCommand } from './quality-process.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';
import { snapshot } from './quality-snapshot.mjs';

const root = resolve(import.meta.dirname, '..');
const attempt = new Date().toISOString().replaceAll(':', '-');
const reportPath = `results/buffer-terminal/runs/${attempt}/report.json`;
const output = resolve(root, 'results/buffer-terminal/runs', attempt);
mkdirSync(output, { recursive: true });
writeFileSync(resolve(root, 'results/buffer-terminal/latest.json'), JSON.stringify({ status: 'incomplete', report: reportPath }) + '\n');
for (const file of ['oracle.json', 'clear-reentry-source.json', 'rust-summary.json', 'clear-rust-summary.json']) {
  writeFileSync(resolve(root, 'results/buffer-terminal', file), JSON.stringify({ status: 'incomplete', attempt }) + '\n');
}
function sourceFiles(directory) {
  const result = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isSymbolicLink()) throw new Error(`Review source symlink before hashing: ${path}`);
    if (entry.isDirectory()) result.push(...sourceFiles(path));
    else if (entry.isFile()) result.push(relative(root, path).replaceAll('\\', '/'));
  }
  return result;
}
const inputs = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'crates/xterm-core/Cargo.toml',
  'scripts/check-buffer-terminal.mjs', 'scripts/buffer-terminal-oracle.mjs',
  'scripts/buffer-clear-reentry-oracle.mjs', 'scripts/terminal-snapshot.mjs',
  'crates/xterm-core/tests/buffer_terminal_differential.rs',
  'crates/xterm-core/tests/support/buffer_terminal.rs', 'crates/xterm-core/tests/support/buffer_clear.rs',
  'crates/xterm-core/tests/support/terminal_snapshot.rs',
  'crates/xterm-core/tests/fixtures/buffer-terminal.tsv', 'crates/xterm-core/tests/fixtures/buffer-clear-reentry.tsv',
  ...sourceFiles(resolve(root, 'crates/xterm-core/src'))];
const hashes = () => Object.fromEntries(inputs.map(file => [file,
  createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex')]));
const before = hashes();
const workspaceBefore = await snapshot(root);
const checks = [];
async function run(command, args) {
  const log = resolve(output, `command-${checks.length}.log`);
  const resourceCheck = async () => requireHeadroom(await diskHeadroom(root, 1024));
  await resourceCheck();
  const result = await runCommand(command, args, { cwd: root, env: process.env,
    log, timeoutMs: 120_000, resourceCheck });
  checks.push({ ...result, log: relative(root, log).replaceAll('\\', '/') });
  console.log(`${result.status}: ${command} ${args.join(' ')}`);
}
await run('rustc', ['--version']);
await run('cargo', ['--version']);
await run(process.execPath, ['scripts/buffer-terminal-oracle.mjs']);
await run(process.execPath, ['scripts/buffer-clear-reentry-oracle.mjs']);
await run('cargo', ['test', '-p', 'xterm-core', '--test', 'buffer_terminal_differential']);
await run('cargo', ['clippy', '-p', 'xterm-core', '--test', 'buffer_terminal_differential', '--', '-D', 'warnings']);
await run('cargo', ['fmt', '--all', '--', '--check']);
const after = hashes();
const workspaceAfter = await snapshot(root);
const changed = Object.keys(before).filter(file => before[file] !== after[file]);
function currentArtifact(file, producer) {
  return { ...JSON.parse(readFileSync(resolve(root, file), 'utf8')), producerStatus: producer.status };
}
const source = currentArtifact('results/buffer-terminal/oracle.json', checks[2]);
const sourceCallbacks = currentArtifact('results/buffer-terminal/clear-reentry-source.json', checks[3]);
const native = currentArtifact('results/buffer-terminal/rust-summary.json', checks[4]);
const nativeCallbacks = currentArtifact('results/buffer-terminal/clear-rust-summary.json', checks[4]);
const semanticStatus = checks.slice(2, 5).every(check => check.status === 'passed') &&
  native.status === 'passed' && nativeCallbacks.status === 'passed' ? 'passed' : 'failed';
const report = { command: 'node scripts/check-buffer-terminal.mjs', attempt, node: process.version,
  platform: process.platform, overallEquivalence: 'NOT PROVEN',
  status: checks.every(check => check.status === 'passed') && !changed.length &&
    workspaceBefore.sha256 === workspaceAfter.sha256 ? 'passed' : 'failed', semanticStatus,
  checks, source, sourceCallbacks, native, nativeCallbacks,
  sourceHashesBefore: before, sourceHashesAfter: after, changedDuringRun: changed,
  workspaceBefore, workspaceAfter, workspaceChanged: workspaceBefore.sha256 !== workspaceAfter.sha256,
  limitations: ['Finite real-engine corpus, not full source equivalence proof.',
    'Actual host cleanup consumers179/180 and native Linux/macOS/browser execution remain separate gates.',
    'SearchAddon DOM/selection lifecycle remains task59. This compares live ring SearchEngine outcomes.',
    'If any input hash changed during execution, the receipt is not a stable source checkpoint.'],
};
writeFileSync(resolve(root, reportPath), JSON.stringify(report, null, 2) + '\n');
writeFileSync(resolve(root, 'results/buffer-terminal/latest.json'), JSON.stringify({ status: report.status, semanticStatus, report: reportPath }, null, 2) + '\n');
console.log(`Report: ${reportPath}`);
if (report.status !== 'passed') process.exitCode = 1;
