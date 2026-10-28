// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runCommand } from './quality-process.mjs';
import { snapshot } from './quality-snapshot.mjs';
import { DEFAULT_MIN_FREE_MIB, diskHeadroom, requireHeadroom } from './quality-resources.mjs';

const root = resolve(import.meta.dirname, '..');
const attempt = new Date().toISOString().replaceAll(':', '-');
const output = resolve(root, 'results/buffer/runs', attempt);
mkdirSync(output, { recursive: true });
writeFileSync(resolve(root, 'results/buffer/latest.json'), JSON.stringify({
  report: `results/buffer/runs/${attempt}/report.json`, status: 'incomplete',
}, null, 2) + '\n');
const rustTests = ['buffer_resize_differential', 'buffer_screen_upstream', 'buffer_screen_edges'];
const testArgs = rustTests.flatMap(name => ['--test', name]);
const directNames = [
  'constructor_capacity_is_rows_plus_scrollback', 'constructor_scroll_bottom_is_rows_minus_one',
  'fill_viewport_has_exact_dimensions_and_blank_cells', 'nonwrapped_first', 'nonwrapped_middle',
  'nonwrapped_last', 'wrapped_first', 'wrapped_middle_up', 'wrapped_middle_down',
  'wrapped_middle_both_ways', 'wrapped_last', 'wrapped_to_first_row', 'wrapped_to_last_row',
];
const direct = new Map(directNames.map((name, index) => [index, name]));
['marker_line_adjusts_when_trimmed', 'marker_disposes_when_trimmed_out', 'marker_calls_on_dispose',
  'translate_ascii_section', 'translate_cutoff_wide_character_includes_it',
  'translate_skips_wide_placeholder', 'translate_single_width_emoji', 'translate_double_width_emoji']
  .forEach((name, index) => direct.set(52 + index, name));
direct.set(60, 'original_buffer_idle_cleanup_preserves_then_realigns_exact_backing_bytes');
const tracked = ['scripts/buffer-baseline.mjs', 'scripts/buffer-resize-oracle.mjs',
  'scripts/check-buffer.mjs', ...rustTests.map(name => `crates/xterm-core/tests/${name}.rs`),
  'crates/xterm-core/tests/support/buffer_snapshot.rs',
  ...readdirSync(resolve(root, 'crates/xterm-core/src/buffer')).filter(name => name.endsWith('.rs'))
    .map(name => `crates/xterm-core/src/buffer/${name}`)];
function hashes() {
  return Object.fromEntries(tracked.map(file => [file,
    createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex')]));
}
const before = hashes();
const workspaceBefore = await snapshot(root);
const checks = [];
async function run(command, args) {
  const log = resolve(output, `command-${checks.length}.log`);
  const resourceCheck = async () => requireHeadroom(await diskHeadroom(root, DEFAULT_MIN_FREE_MIB));
  await resourceCheck();
  const result = await runCommand(command, args, { cwd: root, env: process.env,
    log, timeoutMs: 180_000, resourceCheck });
  checks.push({ ...result, log: log.slice(root.length + 1).replaceAll('\\', '/') });
  console.log(`${result.status}: ${command} ${args.join(' ')}`);
}
await run(process.execPath, ['scripts/buffer-baseline.mjs']);
await run(process.execPath, ['scripts/buffer-resize-oracle.mjs']);
await run('cargo', ['test', '--no-fail-fast', '-p', 'xterm-core', ...testArgs]);
await run('cargo', ['test', '-p', 'xterm-core', '--lib', 'original_buffer_idle_cleanup_preserves_then_realigns_exact_backing_bytes']);
if (!process.argv.includes('--debug-only')) {
  await run('cargo', ['test', '--release', '--no-fail-fast', '-p', 'xterm-core', ...testArgs]);
  await run('cargo', ['test', '--release', '-p', 'xterm-core', '--lib', 'original_buffer_idle_cleanup_preserves_then_realigns_exact_backing_bytes']);
}
await run('cargo', ['clippy', '-p', 'xterm-core', ...testArgs, '--', '-D', 'warnings']);
await run('cargo', ['fmt', '--all', '--', '--check']);
const oracle = JSON.parse(readFileSync(resolve(root, 'results/buffer/resize-oracle.json'), 'utf8'));
assert.equal(oracle.baseline.discovered, 61, 'Audit source test-order mapping after any source change');
const mapping = oracle.baseline.outcomes.map((test, ordinal) => {
  const fixtureCases = oracle.cases.filter(c => c.sourceTestOrdinal === ordinal).map(c => c.id);
  const name = direct.get(ordinal);
  return { ordinal, sourceName: test.name, directTest: name ? {
    path: ordinal === 60 ? 'crates/xterm-core/src/buffer/line_buffer_cleanup_tests.rs' :
      'crates/xterm-core/tests/buffer_screen_upstream.rs', name,
  } : null, resizeFixtureCases: fixtureCases,
    mapped: Boolean(name || fixtureCases.length) };
});
const mapped = mapping.filter(test => test.mapped).length;
assert.equal(mapped, 61, 'Every original case needs direct assertions or full resize snapshots');
const after = hashes();
const workspaceAfter = await snapshot(root);
const changed = Object.keys(before).filter(file => before[file] !== after[file]);
const rustCounts = checks.filter(c => c.command[0] === 'cargo' && c.command[1] === 'test').map(c => {
  const text = readFileSync(resolve(root, c.log), 'utf8');
  const counts = [...text.matchAll(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored/g)]
    .reduce((sum, m) => ({ passed: sum.passed + +m[1], failed: sum.failed + +m[2], ignored: sum.ignored + +m[3] }),
      { passed: 0, failed: 0, ignored: 0 });
  return { command: c.command, ...counts, status: c.status };
});
const report = { command: `node scripts/check-buffer.mjs${process.argv.includes('--debug-only') ? ' --debug-only' : ''}`,
  node: process.version, platform: process.platform, attempt,
  status: checks.every(c => c.status === 'passed') && changed.length === 0 && workspaceBefore.sha256 === workspaceAfter.sha256 ? 'passed' : 'failed',
  overallEquivalence: 'NOT PROVEN', sourceHashes: oracle.sourceHashes,
  rustHashesBefore: before, rustHashesAfter: after, changedDuringRun: changed,
  workspaceBefore, workspaceAfter, workspaceChanged: workspaceBefore.sha256 !== workspaceAfter.sha256,
  originalSuite: { discovered: 61, mapped, baseline: oracle.baseline },
  resizeOracle: { originalCalls: oracle.originalResizeCalls, randomizedCalls: oracle.randomizedResizeCalls,
    directedCalls: oracle.directedResizeCalls, seed: oracle.seed, traces: oracle.randomizedTraces,
    fixture: oracle.fixture, fixtureSha256: oracle.fixtureSha256 },
  mapping, checks, rustCounts,
  limitations: ['Resize hydration compares each transition, not the preceding terminal operation sequence.',
    'Random corpus uses width-one cells including Unicode and malformed UTF-16. Original source cases cover wide cells.',
    'Full terminal/parser/addon and host scheduler integration belong to tasks27/79/99.',
    'Native Linux/macOS and browser runtime are unrun by this scoped Windows command.',
    'Workspace-wide quality checks are independent, and passing scoped checks alone does not satisfy full equivalence.'],
};
writeFileSync(resolve(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
writeFileSync(resolve(root, 'results/buffer/latest.json'), JSON.stringify({ report: `results/buffer/runs/${attempt}/report.json`, status: report.status }, null, 2) + '\n');
console.log(`Report: results/buffer/runs/${attempt}/report.json`);
if (report.status !== 'passed') process.exitCode = 1;
