// Use the same real command path for native interaction and deterministic checks.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { performance } from 'node:perf_hooks';
import { fileURLToPath } from 'node:url';
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const attempt = createEvidenceRun(path.join(root, 'results/demos/native'), { kind: 'native-demo' });
const relative = value => path.relative(root, value).split(path.sep).join('/');
const report = { scope: 'Native demo only; full terminal equivalence NOT PROVEN',
  environment: { node: process.version, platform: process.platform, architecture: process.arch,
    rustflags: process.env.RUSTFLAGS ?? null, cargoEncodedRustflags: process.env.CARGO_ENCODED_RUSTFLAGS ?? null },
  commands: [], negativeChecks: [], failedChecks: [], status: 'incomplete', gaps: [] };
const plannedChecks = ['rustc', 'metadata', 'target-cfg', 'unit', 'clippy', 'format', 'build',
  'script-plain', 'script-ansi', 'interactive', 'stdin-eof', 'help',
  ...['unknown', 'escape', 'zero-cols', 'zero-rows', 'budget', 'assertion'].map(name => `negative-${name}`)];
const reserveMiB = 1024;
let runtime;
async function checkDisk() {
  runtime.requireHeadroom(await runtime.diskHeadroom(root, reserveMiB));
  try { runtime.requireHeadroom(await runtime.diskHeadroom(path.join(root, 'target'), reserveMiB)); }
  catch (error) { if (error.code !== 'ENOENT') throw error; }
}
async function run(name, command, args, expected = 'success') {
  await checkDisk();
  const log = path.join(attempt.directory, `${name}.log`);
  const result = await runtime.runCommand(command, args, { cwd: root, env: process.env, log,
    timeoutMs: 120000, resourceCheck: checkDisk });
  report.commands.push({ ...result, id: name, expected, log: relative(log) });
  persistEvidence(attempt, report);
  return { result, output: await readFile(log, 'utf8') };
}
async function success(name, command, args) {
  const runResult = await run(name, command, args);
  assert.equal(runResult.result.status, 'passed', `${name}: inspect retained log`);
  return runResult.output;
}
// A lint/test failure must remain a failure, but need not hide independent CLI
// observations. Build/setup/interruption failures still prevent unsafe execution.
async function gate(name, command, args) {
  const checked = await run(name, command, args);
  if (checked.result.status !== 'passed') report.failedChecks.push(name);
  if (checked.result.resourceError || checked.result.timedOut || checked.result.aborted || checked.result.terminationError) {
    throw new Error(`${name}: interrupted; no further commands started`);
  }
}
async function interactive(executable) {
  await checkDisk();
  const input = 'reset\ninput PING\\r\nunsupported-command\nwrite after-error\nassert-line 0 after-error\nassert-output PING\\r\nresize 12 4\nassert-size 12 4\nquit\n';
  const started = performance.now();
  // The native example cannot spawn children. A bounded stdin pipe exercises
  // its interactive loop, which the generic runner's ignored stdin cannot do.
  const result = spawnSync(executable, ['--plain'], { cwd: root, env: process.env,
    encoding: 'utf8', input, timeout: 30000, maxBuffer: 8 * 1024 * 1024 });
  const stdout = result.stdout ?? '';
  const stderr = result.stderr ?? '';
  for (const [name, text] of [['interactive-input.txt', input], ['interactive-stdout.log', stdout], ['interactive-stderr.log', stderr]]) {
    await writeFile(path.join(attempt.directory, name), text);
  }
  report.commands.push({ id: 'interactive', command: [executable, '--plain'], expected: 'success with one rejected interactive command',
    exitCode: result.status, signal: result.signal, error: result.error?.message ?? null,
    timedOut: result.error?.code === 'ETIMEDOUT', elapsedMs: performance.now() - started,
    status: result.status === 0 && !result.error && !result.signal ? 'passed' : 'failed',
    log: relative(path.join(attempt.directory, 'interactive-stdout.log')) });
  persistEvidence(attempt, report);
  assert(!result.error && result.status === 0 && !result.signal, 'Interactive command pipe failed');
  const errorLines = stderr.split(/\r?\n/).filter(line => line.startsWith('error: '));
  assert.equal(errorLines.length, 1, 'Only the deliberately unknown command should fail');
  assert(errorLines[0].includes('unknown command'));
  assert(!stderr.includes('[core:Error]'), 'Valid interactive input must not hide an engine error diagnostic');
  report.interactiveDiagnosticLines = stderr.split(/\r?\n/).filter(line => line.startsWith('[core:')).length;
  for (const name of ['assert-line', 'assert-output', 'assert-size']) assert(stdout.includes(`ok ${name}`));
  assert(stdout.includes('Assertions: 3; disposed: true'), 'Interactive errors must not prevent later checks or cleanup');
  report.interactiveChecks = { recoveredCommandErrors: 1, successfulStateAssertions: 3, disposed: true };
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
  const metadata = JSON.parse(await success('metadata', cargo, ['metadata', '--locked', '--no-deps', '--format-version', '1']));
  runtime.assertSharedTarget(root, metadata.target_directory, process.platform);
  report.environment.targetDirectory = metadata.target_directory;
  const cfg = await success('target-cfg', cargo, ['rustc', '--locked', '-p', 'xterm-core', '--lib', '--', '--print', 'cfg']);
  report.environment.resolvedTargetOs = runtime.assertResolvedTarget(process.platform, cfg);
  await gate('unit', cargo, ['test', '--locked', '-p', 'xterm-core', '--example', 'native']);
  await gate('clippy', cargo, ['clippy', '--locked', '-p', 'xterm-core', '--example', 'native', '--', '-D', 'warnings']);
  await gate('format', 'rustfmt', ['--edition', '2021', '--check',
    'crates/xterm-core/examples/native.rs', 'crates/xterm-core/examples/native/commands.rs',
    'crates/xterm-core/examples/native/checks.rs', 'crates/xterm-core/examples/native/view.rs',
    'crates/xterm-core/examples/native/escape.rs', 'crates/xterm-core/examples/native/arguments.rs',
    'crates/xterm-core/examples/native/diagnostics.rs']);
  const build = await success('build', cargo, ['build', '--locked', '-p', 'xterm-core', '--example', 'native', '--message-format=json-render-diagnostics']);
  const artifacts = build.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line))
    .filter(message => message.reason === 'compiler-artifact' && message.target.name === 'native' &&
      message.target.kind.includes('example') && !message.profile.test && message.executable);
  assert.equal(artifacts.length, 1, 'Use this build artifact, never a stale hardcoded executable');
  const executable = artifacts[0].executable;
  report.executable = relative(executable);
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
  await interactive(executable);
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
    const accepted = Number.isInteger(checked.result.exitCode) && checked.result.exitCode > 0 &&
      !checked.result.signal && !checked.result.error && !checked.result.timedOut &&
      !checked.result.resourceError && !checked.result.terminationError && !checked.result.loggingError &&
      !checked.result.aborted && !checked.result.abortError &&
      checked.output.includes(message) && !checked.output.includes('panicked');
    report.negativeChecks.push({ name, accepted, expectedMessage: message });
    assert(accepted, `Negative case ${name} did not fail with its expected diagnostic`);
  }
  report.assertionsPerScript = expectedAssertions;
  report.sourceAfter = await runtime.snapshot(root);
  report.sourceStable = report.sourceBefore.sha256 === report.sourceAfter.sha256;
  if (!report.sourceStable) report.gaps.push('Shared source changed during execution; repeat on a stable snapshot before final acceptance.');
  report.status = report.failedChecks.length ? 'failed' : report.sourceStable ? 'passed' : 'incomplete';
} catch (error) {
  report.status = 'failed';
  report.error = String(error.stack);
} finally {
  report.unrunChecks = plannedChecks.filter(name => !report.commands.some(command => command.id === name));
  persistEvidence(attempt, report);
}
console.log(JSON.stringify({ status: report.status, runId: attempt.id,
  summary: relative(path.join(attempt.directory, 'summary.json')), failedChecks: report.failedChecks,
  unrunChecks: report.unrunChecks, gaps: report.gaps, error: report.error }, null, 2));
if (report.status !== 'passed') process.exitCode = 1;
