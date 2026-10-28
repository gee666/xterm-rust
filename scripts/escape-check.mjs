// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Reuse the resource-aware runner for exact scoped parser command receipts.
import { readFile, readdir, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { runCommand } from './quality-process.mjs';
import { diskHeadroom, requireHeadroom } from './quality-resources.mjs';
import { reserveOutput, assertBuildEnvironment, testCounts } from './quality-policy.mjs';

const root = process.cwd();
assertBuildEnvironment(process.env);
const out = await reserveOutput(root, process.argv[2] ?? `results/escape-parser/check-${Date.now()}`);
const sourceFiles = [
  'Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'crates/xterm-core/Cargo.toml',
  'crates/xterm-core/src/lib.rs', 'crates/xterm-core/src/lifecycle.rs',
  'crates/xterm-core/src/input/text_decoder.rs', 'crates/xterm-core/src/string_builder.rs',
  'scripts/escape-parser-oracle.mjs', 'scripts/escape-async-mapping.mjs', 'scripts/escape-check.mjs',
];
for (const folder of ['crates/xterm-core/src/parser', 'crates/xterm-core/tests/escape_async_support']) {
  for (const file of (await readdir(folder)).sort()) if (file.endsWith('.rs')) sourceFiles.push(`${folder}/${file}`);
}
for (const file of ['escape_async.rs', 'escape_edges.rs', 'escape_differential.rs', 'escape_upstream.rs']) sourceFiles.push(`crates/xterm-core/tests/${file}`);
for (const file of (await readdir('crates/xterm-core/tests/escape_upstream_support')).sort()) sourceFiles.push(`crates/xterm-core/tests/escape_upstream_support/${file}`);
sourceFiles.push('crates/xterm-core/tests/fixtures/escape-parser.tsv');
async function snapshot() {
  const files = await Promise.all(sourceFiles.sort().map(async file => ({ file, sha256: createHash('sha256').update(await readFile(file)).digest('hex') })));
  return { sha256: createHash('sha256').update(JSON.stringify(files)).digest('hex'), files };
}
const resources = [];
async function resourceCheck() {
  const observation = await diskHeadroom(root, 1024);
  resources.push({ time: new Date().toISOString(), ...observation });
  requireHeadroom(observation);
}
const rustfmtFiles = ['crates/xterm-core/src/parser/escape.rs', 'crates/xterm-core/tests/escape_async.rs', 'crates/xterm-core/tests/escape_edges.rs', 'crates/xterm-core/tests/escape_differential.rs'];
const targets = ['escape_async', 'escape_edges', 'escape_differential', 'escape_upstream'];
const testArgs = targets.flatMap(target => ['--test', target]);
const commands = [
  ['rustc-version', 'rustc', ['-Vv']],
  ['cargo-version', 'cargo', ['-V']],
  ['oracle', process.execPath, ['scripts/escape-parser-oracle.mjs']],
  ['async-mapping', process.execPath, ['scripts/escape-async-mapping.mjs']],
  ['source-mapping', process.execPath, ['scripts/escape-test-mapping.mjs']],
  ['original', process.execPath, ['source_repo/bin/test_unit.js', 'out-esbuild/common/parser/EscapeSequenceParser.test.js', '--reporter=json']],
  ['fmt', 'rustfmt', ['--check', '--edition', '2021', ...rustfmtFiles]],
  ['tests', 'cargo', ['test', '-p', 'xterm-core', ...testArgs]],
  ['clippy', 'cargo', ['clippy', '-p', 'xterm-core', ...testArgs, '--', '-D', 'warnings']],
];
const before = await snapshot();
const results = [];
let failure;
try {
  for (const [name, command, args] of commands) {
    await resourceCheck();
    const log = path.join(out, `${name}.log`);
    const result = await runCommand(command, args, { cwd: root, env: process.env, log, timeoutMs: 180000, resourceCheck });
    results.push({ name, ...result, log: path.relative(root, log).replaceAll('\\', '/') });
    if (result.status !== 'passed') { failure = `${name} failed`; break; }
  }
} catch (error) { failure = error.message; }
const after = await snapshot();
const tests = results.find(result => result.name === 'tests');
const original = results.find(result => result.name === 'original');
let baseline;
if (original?.status === 'passed') baseline = JSON.parse(await readFile(path.join(out, 'original.log'), 'utf8')).stats;
const receipt = {
  node: process.version, platform: process.platform, architecture: process.arch,
  before, after, sourcesStable: before.sha256 === after.sha256, resources, results,
  rustSuites: tests ? testCounts(await readFile(path.join(out, 'tests.log'), 'utf8')) : [],
  original: baseline, failure: failure ?? null,
  status: failure ? 'failed' : before.sha256 === after.sha256 ? 'passed' : 'source-changed-during-run',
  scope: 'Scoped parser only. Original161 cases, mapped Rust suites and seeded full-state corpus. Not full terminal/browser/platform equivalence.',
};
await writeFile(path.join(out, 'receipt.json'), JSON.stringify(receipt, null, 2) + '\n');
console.log(JSON.stringify({ out: path.relative(root, out), status: receipt.status, sourcesStable: receipt.sourcesStable, failure: receipt.failure }));
if (receipt.status !== 'passed') process.exitCode = 1;
