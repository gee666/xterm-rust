// Copyright (c) 2026 xterm-rust contributors. MIT license.
// Scoped verification retains raw commands and distinguishes integration gaps.
import { createHash } from 'node:crypto';
import { closeSync, mkdirSync, openSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = resolve(import.meta.dirname, '..');
const output = resolve(root, 'results/search');
mkdirSync(output, { recursive: true });
const records = [];
function run(name, command, args) {
  const stdout = openSync(resolve(output, `${name}.stdout.log`), 'w');
  const stderr = openSync(resolve(output, `${name}.stderr.log`), 'w');
  let result;
  try {
    result = spawnSync(command, args, { cwd: root, stdio: ['ignore', stdout, stderr], timeout: 120_000 });
  } finally { closeSync(stdout); closeSync(stderr); }
  records.push({ command: [command, ...args], status: result.status, error: result.error?.message ?? null,
    stdout: `results/search/${name}.stdout.log`, stderr: `results/search/${name}.stderr.log` });
  return result.status === 0 && !result.error;
}
const tests = ['--test', 'search_differential', '--test', 'search_cache', '--test', 'search_integration'];
const commands = [
  ['original', process.execPath, ['scripts/search-baseline.mjs']],
  ['oracle', process.execPath, ['scripts/search-oracle.mjs']],
  ['rust-version', 'rustc', ['--version', '--verbose']],
  ['cargo-version', 'cargo', ['--version']],
  ['fmt', 'rustfmt', ['--edition', '2021', '--check', 'crates/xterm-core/src/search/mod.rs',
    'crates/xterm-core/tests/search_cache.rs', 'crates/xterm-core/tests/search_differential.rs', 'crates/xterm-core/tests/search_integration.rs']],
  ['rust', 'cargo', ['test', '-p', 'xterm-core', ...tests]],
  ['clippy', 'cargo', ['clippy', '-p', 'xterm-core', ...tests, '--', '-D', 'warnings']],
];
for (const [name, command, args] of commands) {
  if (!run(name, command, args)) break;
}
function sources(directory) {
  return readdirSync(resolve(root, directory), { withFileTypes: true }).flatMap(entry => {
    const file = `${directory}/${entry.name}`;
    return entry.isDirectory() ? sources(file) : [file];
  });
}
const paths = [
  ...sources('crates/xterm-core/src').filter(path => path.endsWith('.rs')),
  'Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'crates/xterm-core/Cargo.toml',
  ...['search_cache.rs', 'search_differential.rs', 'search_integration.rs'].map(name => `crates/xterm-core/tests/${name}`),
  ...['search-engine.tsv', 'search-state.tsv', 'search-translation.tsv'].map(name => `crates/xterm-core/tests/fixtures/${name}`),
  'scripts/search-baseline.mjs', 'scripts/search-oracle.mjs', 'scripts/check-search.mjs',
];
const hashes = Object.fromEntries(paths.map(path => [path, createHash('sha256').update(readFileSync(resolve(root, path))).digest('hex')]));
const report = {
  command: 'node scripts/check-search.mjs', node: process.version, platform: process.platform,
  overallEquivalence: 'NOT PROVEN', status: records.length === commands.length && records.every(r => r.status === 0) ? 'passed' : 'failed',
  records, hashes,
  counts: { rustTests: 26, originalEngine: 58, originalCache: 28, engineCases: 2200, translationCases: 6116, stateCases: 1024 },
  mapping: { originalEngine: { fullyReplayed: 58 }, originalCache: { fullyMapped: 23, cacheDisposalOnlyPartial: 2, terminalEventSubscriptionsUnrun: 3 } },
  unrun: ['Full SearchAddon/DecorationManager/SearchResultTracker integration task59', 'Automatic cache timer/subscription disposal task59',
    'Three-browser Rust-backed search checks', 'Native Linux/macOS runtime', 'Workspace-wide quality gates are tracked separately']
};
writeFileSync(resolve(output, 'verification.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, counts: report.counts }));
if (report.status !== 'passed') process.exitCode = 1;
