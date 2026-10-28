// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Scoped evidence, with immutable attempts and no reuse of stale success.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, statfsSync, writeFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { runCommand } from './quality-process.mjs';
import { assertBuildEnvironment } from './quality-policy.mjs';
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const resultRoot = path.join(root, 'results/mouse-state');
const attempt = createEvidenceRun(resultRoot, { kind: 'scoped-mouse-state' });
const portable = file => path.relative(root, file).split(path.sep).join('/');
const files = [
  'crates/xterm-core/src/mouse_state.rs', 'crates/xterm-core/src/mouse_protocol.rs',
  'crates/xterm-core/tests/mouse_state.rs', 'crates/xterm-core/tests/mouse_callbacks.rs',
  'crates/xterm-core/tests/mouse_differential.rs', 'crates/xterm-core/tests/mouse_custom.rs'
];
const report = { status: 'incomplete', terminalEquivalence: 'NOT PROVEN', platform: process.platform,
  node: process.version, commands: [],
  scope: 'Typed MouseStateService only. Dynamic prototype registry task171, DOM routing and whole-terminal/native Linux/macOS equivalence are not established.' };
const current = path.join(resultRoot, 'verification.json');
writeFileSync(current, `${JSON.stringify({ ...report, runId: attempt.id }, null, 2)}\n`);
function hashes() {
  return Object.fromEntries(files.map(file => [file,
    createHash('sha256').update(readFileSync(path.join(root, file))).digest('hex')]));
}
function resources() {
  const disk = statfsSync(root);
  const freeBytes = disk.bsize * disk.bavail;
  assert(freeBytes >= 1024 ** 3, `only ${freeBytes} bytes free; mouse check requires1GiB reserve`);
}
async function run(name, command, args) {
  resources();
  const log = path.join(attempt.directory, `${name}.log`);
  const result = await runCommand(command, args, { cwd: root, env: process.env, log,
    timeoutMs: 120000, resourceCheck: resources });
  report.commands.push({ ...result, name, command: [command, ...args], log: portable(log) });
  persistEvidence(attempt, report);
  return readFile(log, 'utf8');
}
function counts(output) {
  const groups = [...output.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
  return { passed: groups.reduce((sum, match) => sum + Number(match[1]), 0),
    failed: groups.reduce((sum, match) => sum + Number(match[2]), 0),
    ignored: groups.reduce((sum, match) => sum + Number(match[3]), 0), suites: groups.length };
}
try {
  assertBuildEnvironment(process.env);
  report.implementationHashes = hashes();
  const cargo = process.env.CARGO || 'cargo';
  report.rustc = (await run('rust-version', process.env.RUSTC || 'rustc', ['--version'])).trim();
  report.cargo = (await run('cargo-version', cargo, ['--version'])).trim();
  await run('oracle-replay', process.execPath, ['scripts/mouse-state-oracle.mjs']);
  await run('raw-button-oracle', process.execPath, ['scripts/mouse-raw-button-oracle.mjs']);
  await run('rustfmt', process.env.RUSTFMT || 'rustfmt', ['--check', '--edition', '2021', ...files]);
  const targets = ['-p', 'xterm-core', '--test', 'mouse_state', '--test', 'mouse_callbacks',
    '--test', 'mouse_differential', '--test', 'mouse_custom'];
  const debug = await run('debug-tests', cargo, ['test', '--locked', ...targets]);
  const release = await run('release-tests', cargo, ['test', '--locked', '--release', ...targets]);
  await run('clippy', cargo, ['clippy', '--locked', ...targets, '--all-features', '--', '-D', 'warnings']);
  report.rust = { debug: counts(debug), release: counts(release), upstreamMapped: 8,
    targetedAdditional: 16, differentialTests: 2 };
  if (report.commands.find(command => command.name === 'oracle-replay')?.status === 'passed') {
    const oracle = JSON.parse(readFileSync(path.join(resultRoot, 'oracle.json'), 'utf8'));
    report.sourceHashes = oracle.sourceHashes;
    report.upstream = oracle.upstream;
    report.customAssertions = oracle.customAssertions;
    report.differential = { seed: oracle.seed, traces: oracle.traces, corpusSha256: oracle.corpusSha256 };
  }
  if (report.commands.find(command => command.name === 'raw-button-oracle')?.status === 'passed') {
    report.rawButtons = JSON.parse(readFileSync(path.join(resultRoot, 'raw-buttons-oracle.json'), 'utf8'));
  }
  assert(report.commands.every(command => command.status === 'passed'), 'scoped command failed; inspect immutable attempt logs');
  assert.equal(report.rust.debug.passed, 26);
  assert.equal(report.rust.release.passed, 26);
  report.implementationHashesAfter = hashes();
  assert.deepEqual(report.implementationHashesAfter, report.implementationHashes, 'mouse implementation changed during execution');
  report.status = 'passed';
} catch (error) {
  report.status = 'failed'; report.error = String(error.stack); process.exitCode = 1;
} finally {
  persistEvidence(attempt, report);
  writeFileSync(current, `${JSON.stringify({ ...report, runId: attempt.id }, null, 2)}\n`);
}
console.log(JSON.stringify({ status: report.status, runId: attempt.id,
  summary: portable(path.join(attempt.directory, 'summary.json')), error: report.error }, null, 2));
