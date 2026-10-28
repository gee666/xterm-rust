// Copyright (c) 2026 xterm-rust contributors. MIT license.
// Scoped commands and actual executed counts, never a permanent success marker.
import { closeSync, copyFileSync, mkdirSync, openSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { captureRustInputs, changedInputs } from './public-api-evidence.mjs';

const root = resolve(import.meta.dirname, '..');
process.chdir(root);
const output = 'results/search';
const attempt = `search-${Date.now()}`;
const archive = `${output}/attempts/${attempt}`;
mkdirSync(archive, { recursive: true });
const testNames = ['search_differential', 'search_cache', 'search_integration', 'search_ring'];
const testFiles = testNames.map(name => `crates/xterm-core/tests/${name}.rs`);
const fixtureNames = ['search-engine.tsv', 'search-translation.tsv', 'search-state.tsv'];
const extraInputs = [...testFiles, ...fixtureNames.map(name => `crates/xterm-core/tests/fixtures/${name}`),
  'scripts/search-baseline.mjs', 'scripts/search-oracle.mjs', 'scripts/check-search.mjs', 'scripts/public-api-evidence.mjs'];
const declared = testFiles.reduce((sum, path) => sum + (readFileSync(path, 'utf8').match(/#\[test\]/g)?.length ?? 0), 0);
const inputs = captureRustInputs(extraInputs);
writeFileSync(`${output}/verification.json`, JSON.stringify({ status: 'running', attempt, inputs }, null, 2) + '\n');
const records = [];
function run(name, command, args) {
  const stdoutPath = `${output}/${name}.stdout.log`, stderrPath = `${output}/${name}.stderr.log`;
  const stdout = openSync(stdoutPath, 'w'), stderr = openSync(stderrPath, 'w');
  let result;
  try { result = spawnSync(command, args, { cwd: root, stdio: ['ignore', stdout, stderr], timeout: 120_000 }); }
  finally { closeSync(stdout); closeSync(stderr); }
  copyFileSync(stdoutPath, `${archive}/${name}.stdout.log`);
  copyFileSync(stderrPath, `${archive}/${name}.stderr.log`);
  records.push({ name, command: [command, ...args], status: result.status, error: result.error?.message ?? null,
    stdout: `${archive}/${name}.stdout.log`, stderr: `${archive}/${name}.stderr.log` });
  return result.status === 0 && !result.error;
}
const tests = testNames.flatMap(name => ['--test', name]);
const commands = [
  ['original', process.execPath, ['scripts/search-baseline.mjs']],
  ['oracle', process.execPath, ['scripts/search-oracle.mjs']],
  ['rust-version', 'rustc', ['--version', '--verbose']],
  ['cargo-version', 'cargo', ['--version']],
  ['fmt', 'rustfmt', ['--edition', '2021', '--check', 'crates/xterm-core/src/search/mod.rs', ...testFiles]],
  ['rust', 'cargo', ['test', '-p', 'xterm-core', ...tests]],
  ['clippy', 'cargo', ['clippy', '-p', 'xterm-core', ...tests, '--', '-D', 'warnings']],
];
for (const [name, command, args] of commands) { if (!run(name, command, args)) break; }
const changedDuringRun = changedInputs(inputs, captureRustInputs(extraInputs));
const rustRecord = records.find(record => record.name === 'rust');
const rust = rustRecord ? readFileSync(rustRecord.stdout, 'utf8') : '';
const results = [...rust.matchAll(/^test [^\r\n]+ \.\.\. (ok|FAILED|ignored)$/gm)].map(match => match[1]);
const passed = results.filter(status => status === 'ok').length;
const failed = results.filter(status => status === 'FAILED').length;
const ignored = results.filter(status => status === 'ignored').length;
const fixtureLines = name => readFileSync(`crates/xterm-core/tests/fixtures/${name}`, 'utf8').trimEnd().split('\n').length;
const report = {
  command: 'node scripts/check-search.mjs', node: process.version, platform: process.platform, attempt,
  overallEquivalence: 'NOT PROVEN',
  status: changedDuringRun.length ? 'changed-during-run' : records.length === commands.length && records.every(r => r.status === 0 && !r.error) ? 'passed' : 'failed',
  records, inputs, changedDuringRun,
  rust: { declared, discovered: [...rust.matchAll(/^running (\d+) tests?$/gm)].reduce((sum, row) => sum + Number(row[1]), 0),
    executed: passed + failed, passed, failed, ignored, unrun: declared - passed - failed - ignored },
  comparisons: {
    engine: rust.includes('test unchanged_upstream_calls_and_seeded_engine_corpus ... ok') ? fixtureLines('search-engine.tsv') : 0,
    translation: rust.includes('test actual_cell_wrapped_translation_corpus ... ok') ? fixtureLines('search-translation.tsv') : 0,
    state: rust.includes('test optional_options_and_highlighting_state_corpus ... ok') ? fixtureLines('search-state.tsv') : 0,
  },
  mapping: { originalEngine: { fullyReplayed: 58 }, originalCache: { fullyMapped: 23, cacheDisposalOnlyPartial: 2, terminalEventSubscriptionsUnrun: 3 } },
  unrun: ['Full addon/browser verification is task59, not this core runner', 'Native Linux/macOS runtime', 'Workspace-wide quality gates are tracked separately']
};
for (const path of [`${output}/verification.json`, `${archive}/verification.json`]) writeFileSync(path, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, rust: report.rust, comparisons: report.comparisons }));
if (report.status !== 'passed') process.exitCode = 1;
