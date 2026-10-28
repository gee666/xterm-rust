// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// Bundle the read-only implementation with its existing esbuild development dependency.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync, copyFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';
import { runCommand } from './quality-process.mjs';
import { DEFAULT_MIN_FREE_MIB, diskHeadroom, requireHeadroom } from './quality-resources.mjs';
const root = resolve(import.meta.dirname, '..');
const temporary = resolve(root, 'tmp/buffer-reflow-oracle');
const output = resolve(root, 'results/buffer-reflow');
mkdirSync(output, { recursive: true });
writeFileSync(resolve(output, 'report.json'), JSON.stringify({ status: 'incomplete',
  command: 'node scripts/buffer-reflow-oracle.mjs', started: new Date().toISOString(),
  overallEquivalence: 'NOT PROVEN' }, null, 2) + '\n');
mkdirSync(temporary, { recursive: true });
const { build } = await import(pathToFileURL(resolve(root, 'source_repo/node_modules/esbuild/lib/main.js')));
const bundle = resolve(temporary, 'reference.mjs');
const built = await build({
  absWorkingDir: root,
  stdin: { contents: `export * from './source_repo/src/common/buffer/BufferReflow';
export { BufferLine } from './source_repo/src/common/buffer/BufferLine';
export { CellData } from './source_repo/src/common/buffer/CellData';
export { CircularList } from './source_repo/src/common/CircularList';
export { BgFlags } from './source_repo/src/common/buffer/Constants';`, resolveDir: root, loader: 'ts' },
  outfile: bundle, bundle: true, platform: 'node', format: 'esm', metafile: true,
  tsconfig: 'source_repo/src/common/tsconfig.json',
});
const reference = await import(pathToFileURL(bundle));
const { BufferLine, CellData, CircularList, BgFlags } = reference;
const hashes = Object.fromEntries(Object.keys(built.metafile.inputs).filter(p => p.startsWith('source_repo/'))
  .map(file => [file, createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex')]));
const seed = 0xb0ffef10;
let state = seed;
function random() { state ^= state << 13; state ^= state >>> 17; state ^= state << 5; return state >>> 0; }
const hex = text => Array.from({ length: text.length }, (_, i) => text.charCodeAt(i).toString(16)).join('.') || '-';
function snapshot(rows) {
  return rows.map(row => {
    const cells = Array.from({ length: row.length }, (_, i) => {
      const cell = row.getCell(i);
      return [cell.content >>> 0, cell.fg >>> 0, cell.bg >>> 0, cell.extended.ext >>> 0,
        cell.extended.urlId, hex(cell.getChars())].join(',');
    });
    return `${+row.isWrapped},${row.getTrimmedLength()};${cells.join(';')}`;
  }).join('|') || '-';
}
function rowsFromList(list) { return Array.from({ length: list.length }, (_, i) => list.get(i)); }
const nullCell = CellData.fromCharData([0, '', 1, 0]);
const tokens = [['a', 1], [' ', 1], ['汉', 2], ['语', 2], ['e\u0301', 1], ['𝄞', 1], ['\ud800', 1], ['', 1]];
const cases = [];
for (let index = 0; index < 3000; index++) {
  const old = 2 + random() % 13;
  const larger = index % 2 === 0;
  const next = larger ? old + 1 + random() % 10 : 1 + random() % (old - 1);
  const rows = [];
  for (let group = 0, count = 1 + random() % 3; group < count; group++) {
    let row = new BufferLine(old);
    rows.push(row);
    let col = 0;
    for (let t = 0, count = random() % 45; t < count; t++) {
      let [chars, width] = tokens[random() % tokens.length];
      if (next === 1 && width === 2) { chars = 'b'; width = 1; }
      if (col + width > old) { row = new BufferLine(old, undefined, true); rows.push(row); col = 0; }
      const cell = CellData.fromCharData([random() & 0x1ff, chars, width, chars.charCodeAt(0)]);
      if (random() % 3 === 0) {
        cell.bg = BgFlags.HAS_EXTENDED | (random() & 0xffffff);
        cell.extended.ext = random() & 0x1fffffff;
        cell.extended.urlId = random() % 4;
      }
      row.setCell(col++, cell);
      if (width === 2) { row.setCell(col++, CellData.fromCharData([0, '', 0, 0])); }
    }
  }
  const input = snapshot(rows);
  const cursor = random() % (rows.length + 3);
  const allow = random() % 2;
  let expected;
  if (larger) {
    const list = new CircularList(rows.length + 8);
    rows.forEach(row => { row.resize(next, nullCell); list.push(row); });
    const removals = reference.reflowLargerGetLinesToRemove(list, old, next, cursor, nullCell, !!allow);
    const events = [];
    list.onDelete(change => events.push(`${change.index},${change.amount},${list.length}`));
    const layout = reference.reflowLargerCreateNewLayout(list, removals);
    const before = snapshot(rowsFromList(list));
    reference.reflowLargerApplyNewLayout(list, layout.layout);
    const identities = rowsFromList(list).map(row => rows.indexOf(row));
    expected = [removals.join(',') || '-', layout.layout.join(',') || '-', layout.countRemoved,
      events.join(';') || '-', before, snapshot(rowsFromList(list)), identities.join(',') || '-'].join('#');
  } else {
    expected = reference.reflowSmallerGetNewLineLengths(rows, old, next).join(',') || '-';
  }
  cases.push([larger ? 'L' : 'S', old, next, cursor, allow, input, expected].join('\t'));
}
const fixture = resolve(root, 'crates/xterm-core/tests/fixtures/buffer-reflow.tsv');
writeFileSync(fixture, cases.join('\n') + '\n');
let commandIndex = 0;
async function run(command, args) {
  const log = resolve(temporary, `command-${commandIndex++}.log`);
  const resourceCheck = async () => requireHeadroom(await diskHeadroom(root, DEFAULT_MIN_FREE_MIB));
  await resourceCheck();
  const result = await runCommand(command, args, { cwd: root, env: process.env, log,
    timeoutMs: 120_000, resourceCheck });
  return { ...result, command: [command, ...args].join(' '), processStatus: result.status,
    status: result.status === 'passed' ? 0 : result.exitCode ?? -1,
    stdout: readFileSync(log, 'utf8'), stderr: '' };
}
const checks = [await run(process.execPath, ['scripts/buffer-baseline.mjs']),
  await run('cargo', ['test', '-p', 'xterm-core', '--test', 'buffer_reflow_upstream', '--test', 'buffer_reflow_edges', '--test', 'buffer_reflow_differential'])];
for (const [i, check] of checks.entries()) writeFileSync(resolve(output, `check-${i}.log`), JSON.stringify(check, null, 2) + '\n');
const rustMatches = [...(checks[1].stdout || '').matchAll(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored/g)];
const rustCounts = rustMatches.reduce((total, match) => ({ passed: total.passed + Number(match[1]),
  failed: total.failed + Number(match[2]), ignored: total.ignored + Number(match[3]) }), { passed: 0, failed: 0, ignored: 0 });
const baseline = JSON.parse(readFileSync(resolve(root, 'results/buffer/original.json'), 'utf8'));
const sourceSuite = baseline.suites.find(suite => suite.source.endsWith('/BufferReflow.ts'));
const report = { seed: `0x${seed.toString(16)}`, command: 'node scripts/buffer-reflow-oracle.mjs',
  node: process.version, platform: process.platform,
  rustc: spawnSync('rustc', ['--version'], { encoding: 'utf8', timeout: 10_000 }).stdout?.trim(),
  reference: hashes, fixture: { path: 'crates/xterm-core/tests/fixtures/buffer-reflow.tsv',
    sha256: createHash('sha256').update(readFileSync(fixture)).digest('hex') },
  upstream: { discovered: sourceSuite.discovered, mapped: 7, executed: sourceSuite.executed,
    passed: sourceSuite.passed, failed: sourceSuite.failed, skipped: sourceSuite.skipped, unrun: sourceSuite.unrun },
  rustTests: { mappedUpstreamCases: 7, mappedEdgeCases: 10, differentialTests: 1,
    executed: rustCounts.passed + rustCounts.failed, ...rustCounts,
    compilationStatus: checks[1].status, log: 'results/buffer-reflow/check-1.log' },
  rustSources: Object.fromEntries(['src/buffer/reflow.rs', 'tests/buffer_reflow_upstream.rs',
    'tests/buffer_reflow_edges.rs', 'tests/buffer_reflow_differential.rs'].map(path => {
      const file = `crates/xterm-core/${path}`;
      return [file, createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex')];
    })),
  differential: { cases: cases.length, widening: 1500, shrinking: 1500,
    compared: 'All raw cells, UTF-16 text, widths, fg/bg, extended attributes/URL, trims, wraps, removal ranges, layouts, callback-time lengths, and surviving row identity/order.' },
  checks: checks.map(({ stdout, stderr, ...result }) => result),
  status: checks.every(check => check.status === 0 && !check.error) ? 'passed' : 'failed',
  overallEquivalence: 'NOT PROVEN',
  limitations: ['Scoped helpers only; full Buffer resize/parser/addon integration belongs to other tasks.',
    'No native Linux/macOS runtime or browser evidence is provided by this Windows run.',
    'Width-one wide-cell helper input returns an error rather than upstream infinite loop.'] };
writeFileSync(resolve(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
const history = resolve(output, 'runs', new Date().toISOString().replaceAll(':', '-'));
mkdirSync(history, { recursive: true });
for (const name of ['report.json', 'check-0.log', 'check-1.log']) copyFileSync(resolve(output, name), resolve(history, name));
console.log(JSON.stringify(report, null, 2));
assert.equal(report.status, 'passed', 'buffer reflow checks failed; inspect retained fixtures and logs');
