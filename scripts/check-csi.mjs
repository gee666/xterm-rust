// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runCommand } from './quality-process.mjs';
import { DEFAULT_MIN_FREE_MIB, diskHeadroom, requireHeadroom } from './quality-resources.mjs';
const root = resolve(import.meta.dirname, '..');
const attempt = new Date().toISOString().replaceAll(':', '-');
const output = resolve(root, 'results/csi/runs', attempt);
mkdirSync(resolve(root, 'results/csi'), { recursive: true });
writeFileSync(resolve(root, 'results/csi/latest.json'), JSON.stringify({ status: 'incomplete', attempt }) + '\n');
mkdirSync(output, { recursive: true });
function rustFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const file = resolve(directory, entry.name);
    return entry.isDirectory() ? rustFiles(file) : entry.name.endsWith('.rs') ? [file] : [];
  });
}
const tracked = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'scripts/check-csi.mjs', 'scripts/csi-baseline.mjs',
  'scripts/csi-oracle.mjs', 'scripts/terminal-snapshot.mjs', 'scripts/quality-process.mjs', 'scripts/quality-resources.mjs',
  'crates/xterm-core/tests/support/terminal_snapshot.rs', 'crates/xterm-core/tests/fixtures/csi-terminal.tsv',
  ...['cursor','editing','events','differential'].map(name => `crates/xterm-core/tests/terminal_csi_${name}.rs`)]
  .map(path => resolve(root, path)).concat(rustFiles(resolve(root, 'crates/xterm-core/src')));
function hashes() {
  return Object.fromEntries(tracked.map(path => [path.slice(root.length + 1).replaceAll('\\', '/'),
    createHash('sha256').update(readFileSync(path)).digest('hex')]));
}
const before = hashes();
const checks = [];
async function run(command, args) {
  const log = resolve(output, `command-${checks.length}.log`);
  const resourceCheck = async () => requireHeadroom(await diskHeadroom(root, DEFAULT_MIN_FREE_MIB));
  let result;
  try { await resourceCheck(); }
  catch (error) {
    writeFileSync(log, String(error) + '\n');
    result = { command: [command, ...args], status: 'failed', exitCode: null, resourceError: String(error) };
  }
  if (!result) result = await runCommand(command, args, { cwd: root, env: process.env, log, timeoutMs: 120_000, resourceCheck });
  checks.push({ ...result, log: log.slice(root.length + 1).replaceAll('\\', '/') });
  console.log(`${result.status}: ${command} ${args.join(' ')}`);
}
await run(process.execPath, ['scripts/csi-baseline.mjs']);
await run(process.execPath, ['scripts/csi-oracle.mjs']);
const tests = ['terminal_csi_cursor','terminal_csi_editing','terminal_csi_events','terminal_csi_differential'];
const args = tests.flatMap(test => ['--test', test]);
await run('cargo', ['test','-p','xterm-core',...args]);
if (!process.argv.includes('--debug-only')) await run('cargo', ['test','--release','-p','xterm-core',...args]);
await run('cargo', ['clippy','-p','xterm-core',...args,'--','-D','warnings']);
await run('rustfmt', ['--check','--edition','2021','crates/xterm-core/src/terminal/csi.rs',
  'crates/xterm-core/src/terminal/csi_edits.rs',...tests.map(test => `crates/xterm-core/tests/${test}.rs`)]);
const after = hashes();
const changed = Object.keys(before).filter(path => before[path] !== after[path]);
const baseline = checks[0].status === 'passed' ? JSON.parse(readFileSync(resolve(root, 'results/csi/baseline.json'), 'utf8')) : null;
const oracle = checks[1].status === 'passed' ? JSON.parse(readFileSync(resolve(root, 'results/csi/oracle.json'), 'utf8')) : null;
const testCounts = checks.filter(check => check.command[0] === 'cargo' && check.command[1] === 'test').map(check => {
  const matches = [...readFileSync(resolve(root, check.log), 'utf8').matchAll(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored/g)];
  const counts = matches.reduce((total, match) => ({ passed: total.passed + +match[1], failed: total.failed + +match[2], ignored: total.ignored + +match[3] }), { passed: 0, failed: 0, ignored: 0 });
  return { command: check.command, status: check.status, ...counts };
});
const report = { command: `node scripts/check-csi.mjs${process.argv.includes('--debug-only') ? ' --debug-only' : ''}`,
  attempt, node: process.version, platform: process.platform, checks, testCounts,
  status: checks.every(check => check.status === 'passed') && changed.length === 0 ? 'passed' : 'failed',
  before, after, changedDuringRun: changed, baseline, oracle, overallEquivalence: 'NOT PROVEN',
  limits: ['Task152 retains 15 negative direct internal cursor assertions, not represented as completed Rust cases.',
    'This scope checks CSI commands through actual Rust parser/buffers and source terminal, not full browser/addon coverage.',
    'Clippy checks core and selected tests; formatting here is scoped. Full workspace gates are task47.',
    'Native Linux/macOS and browser runtime evidence are not provided by this Windows run.'] };
writeFileSync(resolve(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
writeFileSync(resolve(root, 'results/csi/latest.json'), JSON.stringify({ status: report.status,
  report: `results/csi/runs/${attempt}/report.json` }, null, 2) + '\n');
console.log(`Report: results/csi/runs/${attempt}/report.json`);
if (report.status !== 'passed') process.exitCode = 1;
