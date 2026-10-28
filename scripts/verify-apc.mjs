// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Bounded commands use the shared process-tree runner and existing Cargo profile.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash, randomUUID } from 'node:crypto';
import { runCommand } from './quality-process.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';
import { assertBuildEnvironment } from './quality-policy.mjs';


const base = 'results/apc';
const directory = `${base}/run-${randomUUID()}`;
await mkdir(directory, { recursive: true });
const report = { platform: process.platform, arch: process.arch, node: process.version,
  runDirectory: directory, complete: false, passed: false, inputs: [], outcomes: [], resources: [],
  scope: 'APC subparser and real escape-parser integration only; no terminal/browser or other-platform runtime claim' };
const save = async () => {
  const json = JSON.stringify(report, null, 2) + '\n';
  await writeFile(`${directory}/verification.json`, json);
  await writeFile(`${base}/verification.json`, json);
};
// Invalidate old success first. Interrupted or disk-full runs stay incomplete.
await save();
try { assertBuildEnvironment(process.env); }
catch (error) { report.error = String(error.stack); await save(); throw error; }
const owned = ['apc', 'apc_edges', 'apc_registration', 'apc_differential', 'apc_escape'];
const paths = [
  'Cargo.toml', 'Cargo.lock', '.cargo/config.toml',
  ...['apc', 'osc', 'osc_handler', 'osc_registry', 'handler', 'handler_registry', 'params', 'table', 'vt500_table', 'escape', 'escape_handlers', 'escape_parse', 'escape_resume']
    .map(name => `crates/xterm-core/src/parser/${name}.rs`),
  ...owned.map(name => `crates/xterm-core/tests/${name}.rs`),
  'crates/xterm-core/tests/parser_support/mod.rs', 'crates/xterm-core/tests/parser_support/future.rs',
  ...['apc-oracle', 'apc-mutation', 'apc-escape', 'apc-escape-control']
    .map(name => `crates/xterm-core/tests/fixtures/${name}.tsv`),
  'scripts/check-apc.mjs', 'scripts/apc-escape-oracle.mjs', 'scripts/parser-oracle-common.mjs',
  'scripts/escape-reference.mjs', 'scripts/verify-apc.mjs', 'scripts/quality-process.mjs',
  'scripts/quality-resources.mjs', 'scripts/quality-policy.mjs',
];
const hashes = async () => Promise.all(paths.map(async path => ({ path,
  sha256: createHash('sha256').update(await readFile(path)).digest('hex') })));
report.inputs = await hashes();
await save();
const checks = [
  ['rustc-version', 'rustc', ['-Vv']], ['cargo-version', 'cargo', ['-V']],
  ['subparser-oracle', process.execPath, ['scripts/check-apc.mjs']],
  ['escape-oracle', process.execPath, ['scripts/apc-escape-oracle.mjs']],
  ['rust-tests', 'cargo', ['test', '-p', 'xterm-core', ...owned.flatMap(name => ['--test', name])]],
  ['rustfmt', 'rustfmt', ['--edition', '2021', '--check',
    'crates/xterm-core/src/parser/apc.rs', ...owned.map(name => `crates/xterm-core/tests/${name}.rs`),
    'crates/xterm-core/tests/parser_support/future.rs']],
  ['clippy', 'cargo', ['clippy', '-p', 'xterm-core', ...owned.flatMap(name => ['--test', name]), '--', '-D', 'warnings']],
];
async function resourceCheck() {
  const observation = await diskHeadroom(process.cwd(), 1024);
  report.resources.push({ time: new Date().toISOString(), ...observation });
  requireHeadroom(observation);
}
for (const [name, command, args] of checks) {
  const log = `${directory}/${name}.log`;
  let outcome;
  try {
    await resourceCheck();
    outcome = await runCommand(command, args, { log, timeoutMs: 120_000, resourceCheck });
  } catch (error) {
    report.error = String(error.stack);
    report.outcomes.push({ name, status: 'failed', error: report.error });
    await save();
    break;
  }
  report.outcomes.push({ name, ...outcome, log });
  if (name === 'subparser-oracle') await writeFile(`${directory}/oracle.json`, await readFile('results/apc/oracle.json'));
  if (name === 'escape-oracle') await writeFile(`${directory}/escape-oracle.json`, await readFile('results/apc-escape/oracle.json'));
  await save();
  console.log(`${name}: ${outcome.status}, ${log}`);
}
const after = await hashes();
report.changedInputs = after.filter((value, index) => value.sha256 !== report.inputs[index].sha256);
const testOutcome = report.outcomes.find(outcome => outcome.name === 'rust-tests');
const testLog = testOutcome?.log ? await readFile(testOutcome.log, 'utf8') : '';
report.rustTests = [...testLog.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)]
  .map(match => ({ passed: +match[1], failed: +match[2], ignored: +match[3], measured: +match[4], filtered: +match[5] }));
report.complete = true;
report.passed = report.changedInputs.length === 0 && report.outcomes.every(outcome => outcome.status === 'passed');
await save();
if (!report.passed) process.exitCode = 1;
