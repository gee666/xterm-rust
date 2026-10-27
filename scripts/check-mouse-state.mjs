// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Reproducible scoped checks, not full-terminal proof.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, statfsSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = fileURLToPath(new URL('../', import.meta.url));
const directory = path.join(root, 'results/mouse-state');
mkdirSync(directory, { recursive: true });
const cargo = process.env.CARGO || 'cargo';
const rustc = process.env.RUSTC || 'rustc';
const rustfmt = process.env.RUSTFMT || 'rustfmt';
const files = [
  'crates/xterm-core/src/mouse_state.rs', 'crates/xterm-core/src/mouse_protocol.rs',
  'crates/xterm-core/tests/mouse_state.rs', 'crates/xterm-core/tests/mouse_callbacks.rs',
  'crates/xterm-core/tests/mouse_differential.rs'
];
const commands = [];
function run(name, command, args) {
  const disk = statfsSync(root);
  const freeBytes = disk.bsize * disk.bavail;
  assert(freeBytes >= 512 * 1024 * 1024, `only ${freeBytes} bytes free before ${name}; refusing another build`);
  const result = spawnSync(command, args, {
    cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 8 * 1024 * 1024
  });
  const output = `${result.stdout || ''}${result.stderr || ''}${result.error ? `\n${result.error.stack}\n` : ''}`;
  writeFileSync(path.join(directory, `${name}.log`), output);
  const entry = { name, command, args, status: result.status, signal: result.signal,
    passed: result.status === 0 && !result.error, freeBytesBefore: freeBytes, log: `${name}.log` };
  if (result.error) entry.error = String(result.error);
  commands.push(entry);
  return output;
}
const nodeVersion = run('node-version', process.execPath, ['--version']).trim();
const rustVersion = run('rust-version', rustc, ['--version']).trim();
const cargoVersion = run('cargo-version', cargo, ['--version']).trim();
run('oracle-replay', process.execPath, ['scripts/mouse-state-oracle.mjs']);
run('rustfmt', rustfmt, ['--check', '--edition', '2021', ...files]);
const testArgs = ['-p', 'xterm-core', '--test', 'mouse_state', '--test', 'mouse_callbacks', '--test', 'mouse_differential'];
const debug = run('debug-tests', cargo, ['test', ...testArgs]);
const release = run('release-tests', cargo, ['test', '--release', ...testArgs]);
run('clippy', cargo, ['clippy', ...testArgs, '--all-features', '--', '-D', 'warnings']);
const oracle = JSON.parse(readFileSync(path.join(directory, 'oracle.json'), 'utf8'));
function counts(output) {
  const groups = [...output.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)];
  return { passed: groups.reduce((sum, match) => sum + Number(match[1]), 0),
    failed: groups.reduce((sum, match) => sum + Number(match[2]), 0),
    ignored: groups.reduce((sum, match) => sum + Number(match[3]), 0), suites: groups.length };
}
const report = {
  status: commands.every(command => command.passed) ? 'passed scoped mouse checks' : 'failed scoped mouse checks',
  terminalEquivalence: 'NOT PROVEN', platform: process.platform, node: nodeVersion,
  rustc: rustVersion, cargo: cargoVersion,
  implementationHashes: Object.fromEntries(files.map(file =>
    [file, createHash('sha256').update(readFileSync(path.join(root, file))).digest('hex')])),
  sourceHashes: oracle.sourceHashes, upstream: oracle.upstream,
  differential: { seed: oracle.seed, traces: oracle.traces, corpusSha256: oracle.corpusSha256 },
  rust: { debug: counts(debug), release: counts(release), upstreamMapped: 8, targetedAdditional: 11,
    differentialTests: 1 },
  commands,
  scope: 'MouseStateService only. DOM input, terminal DEC mouse modes, transport and native Linux/macOS are not established by this report.'
};
writeFileSync(path.join(directory, 'verification.json'), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
assert(commands.every(command => command.passed), 'one or more scoped mouse checks failed; inspect retained logs');
assert.equal(report.rust.debug.passed, 20);
assert.equal(report.rust.release.passed, 20);
