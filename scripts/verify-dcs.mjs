// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Run from repository root. No dependency installation or source edits.
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { runCommand } from './quality-process.mjs';

const baseDirectory = 'results/dcs';
await mkdir(baseDirectory, { recursive: true });
const existing = await readdir(baseDirectory);
let runNumber = existing.reduce((maximum, name) => {
  const match = name.match(/^run-(\d+)$/);
  return match ? Math.max(maximum, Number(match[1])) : maximum;
}, 0) + 1;
let directory;
// Concurrent independent reviewers may start the same run number. Reserve a
// directory atomically, with bounded retries rather than overwriting evidence.
for (let attempt = 0; attempt < 100; attempt++, runNumber++) {
  const candidate = `${baseDirectory}/run-${String(runNumber).padStart(4, '0')}`;
  try { await mkdir(candidate); directory = candidate; break; }
  catch (error) { if (error.code !== 'EEXIST') throw error; }
}
if (!directory) throw new Error('could not reserve a DCS verification directory');
const inputs = [
  'Cargo.toml', 'Cargo.lock', '.cargo/config.toml',
  'crates/xterm-core/src/parser/dcs.rs', 'crates/xterm-core/src/parser/dcs_handler.rs',
  'crates/xterm-core/src/parser/handler.rs', 'crates/xterm-core/src/parser/handler_registry.rs',
  'crates/xterm-core/src/parser/handler_slots.rs', 'crates/xterm-core/src/parser/dcs_control.rs',
  'crates/xterm-core/tests/handler_registry.rs', 'crates/xterm-core/tests/dcs.rs',
  'crates/xterm-core/tests/dcs_fallback.rs',
  ...['params', 'table', 'vt500_table', 'apc', 'osc', 'osc_handler'].map(name => `crates/xterm-core/src/parser/${name}.rs`),
  'crates/xterm-core/src/input/text_decoder.rs', 'crates/xterm-core/src/lifecycle.rs',
  'crates/xterm-core/src/string_builder.rs', 'crates/xterm-core/src/event.rs',
  'crates/xterm-core/src/event/dispatch.rs',
  'crates/xterm-core/tests/escape_dcs.rs',
  ...['escape', 'escape_handlers', 'escape_parse', 'escape_resume'].map(name => `crates/xterm-core/src/parser/${name}.rs`),
  'crates/xterm-core/tests/dcs_support/mod.rs',
  ...['callback', 'mutation', 'oracle', 'raw_errors', 'disposal'].map(name => `crates/xterm-core/tests/dcs_cases/${name}.rs`),
  'crates/xterm-core/tests/fixtures/dcs-oracle.tsv',
  'scripts/check-dcs.mjs', 'scripts/parser-oracle-common.mjs',
  'scripts/dcs-integration-oracle.mjs', 'scripts/escape-reference.mjs',
  'crates/xterm-core/tests/fixtures/escape-dcs.tsv',
  'scripts/quality-process.mjs', 'scripts/verify-dcs.mjs',
  'scripts/tsv-records.mjs', 'scripts/tsv-records.test.mjs'
];
const report = {
  platform: process.platform, arch: process.arch, node: process.version,
  runDirectory: directory, inputs: [], toolchains: {}, outcomes: [], complete: false, passed: false,
  scope: 'DCS and real escape-parser DCS integration only; no full equivalence or other-platform runtime claim'
};
const save = async () => {
  const json = `${JSON.stringify(report, null, 2)}\n`;
  await writeFile(`${directory}/verification.json`, json);
  await writeFile(`${baseDirectory}/verification.json`, json);
};
// Invalidate earlier success before running commands. If interrupted, this run
// stays incomplete rather than leaving an old green summary beside a new log.
await save();
for (const path of inputs) report.inputs.push({
  path, sha256: createHash('sha256').update(await readFile(path)).digest('hex')
});
for (const [name, command, args] of [
  ['rustc', 'rustc', ['-Vv']], ['cargo', 'cargo', ['-V']]
]) {
  const log = `${directory}/${name}-version.log`;
  const outcome = await runCommand(command, args, { log, timeoutMs: 120_000 });
  report.toolchains[name] = { ...outcome, log, output: await readFile(log, 'utf8') };
  await save();
}
const checks = [
  ['record-endings', process.execPath, ['--test', 'scripts/tsv-records.test.mjs']],
  ['oracle', process.execPath, ['scripts/check-dcs.mjs']],
  ['integration-oracle', process.execPath, ['scripts/dcs-integration-oracle.mjs']],
  ['rustfmt', 'rustfmt', ['--edition', '2021', '--check',
    'crates/xterm-core/src/parser/dcs.rs', 'crates/xterm-core/src/parser/dcs_handler.rs',
    'crates/xterm-core/src/parser/handler_registry.rs', 'crates/xterm-core/tests/handler_registry.rs',
    'crates/xterm-core/tests/dcs.rs', 'crates/xterm-core/tests/dcs_fallback.rs',
    'crates/xterm-core/tests/escape_dcs.rs']],
  ['rust-tests', 'cargo', ['test', '-p', 'xterm-core', '--test', 'dcs', '--test', 'dcs_fallback', '--test', 'escape_dcs', '--test', 'handler_registry']],
  ['clippy', 'cargo', ['clippy', '-p', 'xterm-core', '--test', 'dcs', '--test', 'dcs_fallback', '--test', 'escape_dcs', '--test', 'handler_registry', '--', '-D', 'warnings']]
];
for (const [name, command, args] of checks) {
  const log = `${directory}/${name}.log`;
  const outcome = await runCommand(command, args, { log, timeoutMs: 120_000 });
  report.outcomes.push({ name, ...outcome, log });
  if (name === 'oracle' || name === 'integration-oracle') {
    const source = name === 'oracle' ? `${baseDirectory}/oracle.json` : 'results/dcs-integration/oracle.json';
    await writeFile(`${directory}/${name}.json`, await readFile(source));
  }
  await save();
  console.log(`${name}: ${outcome.status}, ${log}`);
}
report.changedInputs = [];
for (const input of report.inputs) {
  const sha256 = createHash('sha256').update(await readFile(input.path)).digest('hex');
  if (sha256 !== input.sha256) report.changedInputs.push({ path: input.path, before: input.sha256, after: sha256 });
}
report.complete = true;
report.passed = report.changedInputs.length === 0 && [...Object.values(report.toolchains), ...report.outcomes]
  .every(outcome => outcome.status === 'passed');
const tests = await readFile(`${directory}/rust-tests.log`, 'utf8');
report.rustTestSuites = [...tests.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)].map(counts => ({
  passed: Number(counts[1]), failed: Number(counts[2]), ignored: Number(counts[3]),
  measured: Number(counts[4]), filtered: Number(counts[5])
}));
await save();
if (!report.passed) process.exitCode = 1;
