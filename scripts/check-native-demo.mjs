// Use the same real command path for native interaction and deterministic checks.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const attempt = createEvidenceRun(path.join(root, 'results/demos/native'), { kind: 'native-demo' });
const relative = value => path.relative(root, value).split(path.sep).join('/');
const report = { scope: 'Native demo only; full terminal equivalence NOT PROVEN',
  environment: { node: process.version, platform: process.platform, architecture: process.arch,
    rustflags: process.env.RUSTFLAGS ?? null, cargoEncodedRustflags: process.env.CARGO_ENCODED_RUSTFLAGS ?? null },
  commands: [], negativeChecks: [], status: 'incomplete', gaps: [] };
const reserveMiB = 1024;
let runtime;
async function checkDisk() { runtime.requireHeadroom(await runtime.diskHeadroom(root, reserveMiB)); }
async function run(name, command, args, expected = 'success') {
  await checkDisk();
  const log = path.join(attempt.directory, `${name}.log`);
  const result = await runtime.runCommand(command, args, { cwd: root, env: process.env, log,
    timeoutMs: 120000, resourceCheck: checkDisk });
  report.commands.push({ ...result, expected, log: relative(log) });
  persistEvidence(attempt, report);
  return { result, output: await readFile(log, 'utf8') };
}
async function success(name, command, args) {
  const runResult = await run(name, command, args);
  assert.equal(runResult.result.status, 'passed', `${name}: inspect retained log`);
  return runResult.output;
}
try {
  // Allocate the incomplete attempt before loading shared tools. A broken
  // tooling import must not leave an older successful run looking current.
  runtime = Object.assign({}, ...await Promise.all([
    import('./quality-process.mjs'), import('./quality-snapshot.mjs'),
    import('./quality-resources.mjs'), import('./quality-policy.mjs')
  ]));
  runtime.assertBuildEnvironment(process.env);
  report.sourceBefore = await runtime.snapshot(root);
  const cargo = process.env.CARGO ?? 'cargo';
  report.environment.rustc = (await success('rustc', process.env.RUSTC ?? 'rustc', ['--version'])).trim();
  await success('unit', cargo, ['test', '--locked', '-p', 'xterm-core', '--example', 'native']);
  await success('clippy', cargo, ['clippy', '--locked', '-p', 'xterm-core', '--example', 'native', '--', '-D', 'warnings']);
  await success('format', 'rustfmt', ['--edition', '2021', '--check',
    'crates/xterm-core/examples/native.rs', 'crates/xterm-core/examples/native/commands.rs',
    'crates/xterm-core/examples/native/checks.rs', 'crates/xterm-core/examples/native/view.rs',
    'crates/xterm-core/examples/native/escape.rs']);
  await success('build', cargo, ['build', '--locked', '-p', 'xterm-core', '--example', 'native']);
  const executable = path.join(root, `target/debug/examples/native${process.platform === 'win32' ? '.exe' : ''}`);
  const script = 'crates/xterm-core/tests/fixtures/native-demo.txt';
  const expectedAssertions = (await readFile(path.join(root, script), 'utf8')).split(/\r?\n/).filter(line => line.startsWith('assert-')).length;
  assert(expectedAssertions >= 20, 'Demo script must assert actual state, not startup alone');
  const plain = await success('script-plain', executable, ['--plain', '--script', script]);
  assert(plain.includes(`Assertions: ${expectedAssertions}; disposed: true`));
  assert(!plain.includes('\x1b'), 'Plain inspection output must not execute control sequences');
  const ansi = await success('script-ansi', executable, ['--script', script]);
  assert(ansi.includes('\x1b[38;5;1m'), 'Sample must render real red cell attributes');
  assert(ansi.includes('\x1b[38;5;2m'), 'Sample must render real green cell attributes');
  assert(ansi.includes(`Assertions: ${expectedAssertions}; disposed: true`));
  const eof = await success('stdin-eof', executable, ['--plain']);
  assert(eof.includes('disposed: true'), 'EOF must explicitly dispose the terminal');
  await success('help', executable, ['--help']);
  const temporary = path.join(root, 'tmp/native-demo', attempt.id);
  await mkdir(temporary, { recursive: true });
  for (const [name, text, message] of [
    ['unknown', 'unsupported-command\n', 'unknown command'],
    ['escape', 'write \\xGG\n', 'expected hexadecimal digit'],
    ['zero-cols', 'resize 0 4\n', 'at least 2 columns'],
    ['zero-rows', 'resize 2 0\n', 'at least 2 columns'],
    ['budget', 'resize 1000000 2\n', 'exceeds --max-cells'],
    ['assertion', 'assert-cursor 9 9\n', 'expected']]) {
    const file = path.join(temporary, `${name}.txt`);
    await writeFile(file, text);
    const checked = await run(`negative-${name}`, executable, ['--plain', '--script', file], 'failure');
    const accepted = checked.result.exitCode !== 0 && !checked.result.timedOut &&
      checked.output.includes(message) && !checked.output.includes('panicked');
    report.negativeChecks.push({ name, accepted, expectedMessage: message });
    assert(accepted, `Negative case ${name} did not fail with its expected diagnostic`);
  }
  report.assertionsPerScript = expectedAssertions;
  report.sourceAfter = await runtime.snapshot(root);
  report.sourceStable = report.sourceBefore.sha256 === report.sourceAfter.sha256;
  if (!report.sourceStable) report.gaps.push('Shared source changed during execution; repeat on a stable snapshot before final acceptance.');
  report.status = report.sourceStable ? 'passed' : 'incomplete';
} catch (error) {
  report.status = 'failed';
  report.error = String(error.stack);
} finally {
  persistEvidence(attempt, report);
}
console.log(JSON.stringify({ status: report.status, runId: attempt.id,
  summary: relative(path.join(attempt.directory, 'summary.json')), gaps: report.gaps, error: report.error }, null, 2));
if (report.status !== 'passed') process.exitCode = 1;
