// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Focused source/native integration checks. Run in the agent's worktree.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import { runCommand } from './quality-process.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';

const root = resolve(import.meta.dirname, '..');
const attempt = new Date().toISOString().replaceAll(':', '-');
const reportPath = `results/buffer-terminal/runs/${attempt}/report.json`;
const output = resolve(root, 'results/buffer-terminal/runs', attempt);
mkdirSync(output, { recursive: true });
writeFileSync(resolve(root, 'results/buffer-terminal/latest.json'), JSON.stringify({ status: 'incomplete', report: reportPath }) + '\n');
for (const file of ['oracle.json', 'clear-reentry-source.json']) {
  writeFileSync(resolve(root, 'results/buffer-terminal', file), JSON.stringify({ status: 'incomplete' }) + '\n');
}
mkdirSync(resolve(root, 'tmp/buffer-terminal'), { recursive: true });
for (const file of ['rust-summary.json', 'clear-rust-summary.json']) {
  writeFileSync(resolve(root, 'tmp/buffer-terminal', file), JSON.stringify({ status: 'incomplete' }) + '\n');
}
const checks = [];
async function run(command, args) {
  const log = resolve(output, `command-${checks.length}.log`);
  const resourceCheck = async () => requireHeadroom(await diskHeadroom(root, 1024));
  await resourceCheck();
  const result = await runCommand(command, args, { cwd: root, env: process.env,
    log, timeoutMs: 120_000, resourceCheck });
  checks.push({ command: result.command, status: result.status, exitCode: result.exitCode,
    log: relative(root, log).replaceAll('\\', '/') });
  console.log(`${result.status}: ${command} ${args.join(' ')}`);
}
await run(process.execPath, ['scripts/buffer-terminal-oracle.mjs']);
await run(process.execPath, ['scripts/buffer-clear-reentry-oracle.mjs']);
await run('cargo', ['test', '--no-fail-fast', '-p', 'xterm-core', '--test', 'buffer_terminal_differential', '--test', 'native_buffer_cleanup']);
await run('cargo', ['test', '-p', 'xterm-core', '--lib', 'native_cleanup']);
await run('cargo', ['test', '-p', 'xterm-core', '--lib', 'cleanup_owner']);
await run('cargo', ['clippy', '-p', 'xterm-core', '--test', 'buffer_terminal_differential', '--test', 'native_buffer_cleanup', '--', '-D', 'warnings']);
await run('cargo', ['fmt', '-p', 'xterm-core', '--', '--check']);
const executions = checks.filter(check => check.command[0] === 'cargo' && check.command[1] === 'test').map(check => {
  const text = readFileSync(resolve(root, check.log), 'utf8');
  return { command: check.command, suites: [...text.matchAll(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored/g)]
    .map(match => ({ passed: +match[1], failed: +match[2], ignored: +match[3] })) };
});
const native = JSON.parse(readFileSync(resolve(root, 'tmp/buffer-terminal/rust-summary.json')));
const callbacks = JSON.parse(readFileSync(resolve(root, 'tmp/buffer-terminal/clear-rust-summary.json')));
const testsPassed = executions.every(run => run.suites.length > 0 &&
  run.suites.every(suite => suite.passed > 0 && suite.failed === 0 && suite.ignored === 0));
const report = {
  command: 'node scripts/check-buffer-terminal.mjs',
  status: checks.every(check => check.status === 'passed') && testsPassed &&
    native.status === 'passed' && callbacks.status === 'passed' ? 'passed' : 'failed',
  checks, executions, native, callbacks,
  source: 'results/buffer-terminal/oracle.json',
  sourceCallbacks: 'results/buffer-terminal/clear-reentry-source.json',
  overallEquivalence: 'NOT PROVEN',
};
writeFileSync(resolve(root, reportPath), JSON.stringify(report, null, 2) + '\n');
writeFileSync(resolve(root, 'results/buffer-terminal/latest.json'), JSON.stringify({ status: report.status, report: reportPath }, null, 2) + '\n');
console.log(`Report: ${reportPath}`);
if (report.status !== 'passed') process.exitCode = 1;
