// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Executes the read-only SelectionModel and original assertions.
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { homedir } from 'node:os';
import path from 'node:path';

const sourcePath = 'source_repo/src/browser/selection/SelectionModel.ts';
const testsPath = 'source_repo/src/browser/selection/SelectionModel.test.ts';
const source = await readFile(sourcePath, 'utf8');
const testSource = await readFile(testsPath, 'utf8');
const hash = text => createHash('sha256').update(text).digest('hex');
const moduleUrl = text => `data:text/javascript;base64,${Buffer.from(text).toString('base64')}`;
const modelUrl = moduleUrl(stripTypeScriptTypes(source.replace(/^import .*;\r?\n/gm, ''), { mode: 'transform' }));
const { SelectionModel } = await import(modelUrl);

const outcomes = [];
const suites = [];
let beforeEach = () => {};
globalThis.describe = (name, callback) => {
  suites.push(name);
  try { callback(); } finally { suites.pop(); }
};
globalThis.beforeEach = callback => { beforeEach = callback; };
globalThis.it = (name, callback) => {
  try {
    beforeEach();
    callback();
    outcomes.push({ name: [...suites, name].join(' / '), status: 'passed' });
  } catch (error) {
    outcomes.push({ name: [...suites, name].join(' / '), status: 'failed', error: String(error.stack) });
  }
};
// All assertion bodies are retained. This coordinate-only fixture provides the
// three fields SelectionModel reads, rather than initializing a full JS buffer.
const fixture = 'class MockBufferService { constructor(cols, rows) { this.cols = cols; this.rows = rows; this.buffer = { ybase: 0 }; } }';
const transformedTests = fixture + '\n' + testSource
  .replace("import { assert } from 'chai';", "import { strict as assert } from 'node:assert';")
  .replace("import { SelectionModel } from './SelectionModel';", `import { SelectionModel } from '${modelUrl}';`)
  .replace("import { MockBufferService } from '../../common/TestUtils.test';", '');
try { await import(moduleUrl(stripTypeScriptTypes(transformedTests, { mode: 'transform' }))); }
finally {
  delete globalThis.describe;
  delete globalThis.beforeEach;
  delete globalThis.it;
}
assert.equal(outcomes.length, 18, 'upstream test inventory changed');

const cases = [];
const expected = [];
const encoded = point => point ? `${point[0]},${point[1]}` : '-,-';
function record(columns, rows, scrollback, start, end, length, all, trim, clear = false) {
  const model = new SelectionModel({ cols: columns, rows, buffer: { ybase: scrollback } });
  model.selectionStart = start?.slice();
  model.selectionEnd = end?.slice();
  model.selectionStartLength = length;
  model.isSelectAllActive = all;
  const refresh = trim === undefined ? '-' : +model.handleTrim(trim);
  if (clear) model.clearSelection();
  cases.push([columns, rows, scrollback, ...(start ?? ['-', '-']), ...(end ?? ['-', '-']), length, +all, trim ?? '-', +clear].join('\t'));
  expected.push([
    encoded(model.finalSelectionStart), encoded(model.finalSelectionEnd), +model.areSelectionValuesReversed(),
    refresh, encoded(model.selectionStart), encoded(model.selectionEnd), model.selectionStartLength, +model.isSelectAllActive
  ].join('\t'));
}
for (const columns of [1, 2, 3, 80, 132, 65535, 4294967295]) {
  const points = [undefined, [-2, 0], [-1, 3], [0, 0], [1, 0], [columns - 1, 2], [columns, 3], [0, 5], [columns, -1]];
  const lengths = [...new Set([-3, -1, 0, 1, columns - 1, columns, Math.min(4294967295, columns * 2), 4294967295])];
  for (const start of points) for (const end of points) for (const length of lengths) {
    for (const all of [false, true]) for (const trim of [undefined, 0, 1, 3, 4294967295]) {
      record(columns, 24, 1000, start, end, length, all, trim);
    }
  }
}
const seed = 0x051ec710;
let state = seed;
function random() {
  state ^= state << 13; state ^= state >>> 17; state ^= state << 5;
  return state >>> 0;
}
for (let index = 0; index < 20000; index++) {
  const columns = random() % 512 + 1;
  const rows = random() % 100 + 1;
  const scrollback = random() % 65536;
  const start = random() % 8 ? [random() % (columns + 1), random() % (scrollback + rows)] : undefined;
  const end = random() % 8 ? [random() % (columns + 1), random() % (scrollback + rows)] : undefined;
  record(columns, rows, scrollback, start, end, random() % (columns * 8) - columns, random() % 16 === 0,
    random() % 4 ? random() % (scrollback + rows) : undefined, random() % 32 === 0);
}

await mkdir('results/selection-model', { recursive: true });
const cargo = process.env.CARGO ?? (process.platform === 'win32' ? path.join(homedir(), '.cargo/bin/cargo.exe') : 'cargo');
const command = [cargo, 'run', '--quiet', '-p', 'xterm-web', '--example', 'selection_oracle'];
const input = cases.join('\n') + '\n';
const result = spawnSync(command[0], command.slice(1), {
  input, encoding: 'utf8', timeout: 180000, maxBuffer: 64 * 1024 * 1024
});
const actual = result.stdout?.trimEnd().split(/\r?\n/) ?? [];
const mismatch = expected.findIndex((value, index) => value !== actual[index]);
const passed = result.status === 0 && mismatch === -1 && actual.length === expected.length;
await writeFile('results/selection-model/native-stderr.log', result.stderr ?? '');
const report = {
  reference: { sourcePath, testsPath, sourceSha256: hash(source), testsSha256: hash(testSource) },
  implementation: { path: 'crates/xterm-web/src/selection/model.rs', sha256: hash(await readFile('crates/xterm-web/src/selection/model.rs', 'utf8')) },
  environment: { node: process.version, platform: process.platform, arch: process.arch },
  originalAssertionBodies: {
    discovered: outcomes.length, executed: outcomes.length, passed: outcomes.filter(test => test.status === 'passed').length,
    failed: outcomes.filter(test => test.status === 'failed').length, skipped: 0, unrun: 0, outcomes,
    harness: 'Assertion bodies unchanged; Chai maps to Node strict assertions; MockBufferService replaced by cols/rows/ybase coordinate fixture. This is not the full upstream runner baseline.'
  },
  differential: {
    seed, randomCases: 20000, cases: cases.length, resultFieldsPerCase: 8, command,
    inputSha256: hash(input), expectedSha256: hash(expected.join('\n') + '\n'),
    compared: result.status === 0 ? Math.min(expected.length, actual.length) : 0,
    status: passed ? 'matched' : 'failed', nativeExitCode: result.status, error: result.error?.message,
    mismatchIndex: passed ? null : mismatch
  },
  limitations: ['Coordinate model only; SelectionService text/DOM, terminal integration and browser behavior are not established.',
    'Native execution platform above is not evidence of other operating systems.']
};
await writeFile('results/selection-model/oracle.json', JSON.stringify(report, null, 2) + '\n');
if (!passed) {
  await writeFile('results/selection-model/counterexample.json', JSON.stringify({
    seed, failureKind: result.status === 0 ? 'mismatch' : 'native-execution', index: mismatch, input: cases[mismatch], expected: expected[mismatch], actual: actual[mismatch],
    nativeExitCode: result.status, error: result.error?.message
  }, null, 2) + '\n');
}
console.log(JSON.stringify({ upstreamAssertionBodies: report.originalAssertionBodies.passed, differential: report.differential }, null, 2));
assert.equal(report.originalAssertionBodies.failed, 0, 'unchanged original assertion bodies failed');
assert(passed, 'Rust/source selection mismatch or native execution failed; retained results/selection-model/counterexample.json');
