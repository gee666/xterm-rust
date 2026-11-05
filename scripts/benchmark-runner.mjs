import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync, rmSync, lstatSync, openSync, readSync, closeSync } from 'node:fs';
import { join, resolve, dirname, relative, isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runCommand } from './quality-process.mjs';
import { validateManifest, validateResult } from './benchmark-contract.mjs';

const MAX_RESULT_BYTES = 8 * 1024 * 1024;
const within = (parent, child) => { const path = relative(parent, child); return !isAbsolute(path) && path !== '..' && !path.startsWith(`..${process.platform === 'win32' ? '\\' : '/'}`); };
const DIAGNOSTIC_BYTES = 64 * 1024;
function evidence(path) {
  let info;
  try { info = lstatSync(path); }
  catch (error) { if (error.code === 'ENOENT') return null; throw error; }
  assert.ok(info.isFile() && !info.isSymbolicLink(), 'Invalid benchmark output file');
  const truncated = info.size > MAX_RESULT_BYTES;
  const buffer = Buffer.alloc(Math.min(info.size, truncated ? DIAGNOSTIC_BYTES : MAX_RESULT_BYTES));
  const fd = openSync(path, 'r');
  try {
    const count = readSync(fd, buffer, 0, buffer.length, 0);
    return { raw: buffer.subarray(0, count).toString('utf8'), bytes: info.size, truncated };
  } finally { closeSync(fd); }
}
function checkSize(paths) {
  for (const path of paths) {
    try { assert.ok(lstatSync(path).size <= MAX_RESULT_BYTES, 'Benchmark output size limit exceeded'); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
}
/** Each adapter receives one shared manifest path and its own fresh result path.
 * Adapters own awaiting writes/frames and monotonic steady-state measurement.
 * Receipt validation detects missing completion but cannot audit an adapter's code.
 */
export async function runBenchmarks({ manifest, adapters, scratchRoot, output, timeoutMs }) {
  const receipt = { schemaVersion: 1, status: 'failed', equivalence: 'NOT PROVEN', commands: [], results: {}, summary: null };
  const scratch = resolve(scratchRoot), destination = resolve(output);
  assert.ok(!within(scratch, destination), 'Durable receipt cannot be inside disposable scratch');
  mkdirSync(dirname(destination), { recursive: true });
  // Invalidate prior success even when preflight rejects the next run.
  writeFileSync(destination, JSON.stringify(receipt, null, 2) + '\n');
  let owned;
  try {
    receipt.manifestSha256 = validateManifest(manifest);
    receipt.manifest = manifest;
    assert.ok(Number.isSafeInteger(timeoutMs) && timeoutMs > 0, 'Finite positive process deadline required');
    assert.deepEqual(Object.keys(adapters).sort(), ['javascript', 'rust']);
    assert.ok(lstatSync(scratch).isDirectory() && !lstatSync(scratch).isSymbolicLink(), 'Scratch root must be a real owned directory');
    owned = mkdtempSync(join(scratch, 'benchmark-'));
    const input = join(owned, 'manifest.json');
    writeFileSync(input, JSON.stringify(manifest) + '\n');
    const summaries = {};
    for (const engine of ['javascript', 'rust']) {
      const adapter = adapters[engine];
      assert.ok(typeof adapter.program === 'string' && adapter.program.trim());
      assert.ok(Array.isArray(adapter.args) && adapter.args.every(value => typeof value === 'string'));
      const resultPath = join(owned, `${engine}.json`), log = join(owned, `${engine}.log`);
      const command = await runCommand(adapter.program, [...adapter.args, '--benchmark-manifest', input, '--benchmark-output', resultPath],
        { cwd: adapter.cwd, env: process.env, log, timeoutMs, monitorMs: 100,
          resourceCheck: () => checkSize([log, resultPath]) });
      // Record process outcomes before file inspection, which can itself fail.
      const record = { engine, ...command };
      receipt.commands.push(record);
      const logEvidence = evidence(log);
      record.output = logEvidence;
      const captured = evidence(resultPath);
      if (captured) receipt.results[engine] = captured;
      assert.ok(!logEvidence?.truncated && !captured?.truncated, 'Benchmark output size limit exceeded');
      assert.equal(command.status, 'passed', `${engine} adapter failed`);
      assert.ok(captured, 'Missing benchmark output');
      // Partial/malformed results survive even nonzero exit or timeout cleanup.
      const parsed = JSON.parse(captured.raw);
      summaries[engine] = validateResult(manifest, parsed, engine);
    }
    receipt.summary = summaries.javascript.map((row, index) => {
      const rust = summaries.rust[index];
      return { id: row.id, javascript: row, rust,
        javascriptOverRustMedian: rust.medianMs > 0 ? row.medianMs / rust.medianMs : null };
    });
    receipt.status = 'passed';
  } catch (error) {
    receipt.error = String(error.stack ?? error);
  } finally {
    try { if (owned) rmSync(owned, { recursive: true }); }
    catch (error) { receipt.status = 'failed'; receipt.summary = null; receipt.cleanupError = String(error); }
    if (receipt.status !== 'passed') receipt.summary = null;
    writeFileSync(destination, JSON.stringify(receipt, null, 2) + '\n');
  }
  return receipt;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    assert.equal(process.argv.length, 4, 'Usage: node scripts/benchmark-runner.mjs --config CONFIG.json');
    assert.equal(process.argv[2], '--config');
    const result = await runBenchmarks(JSON.parse(readFileSync(process.argv[3], 'utf8')));
    console.log(`${result.status}: ${result.error ?? result.manifestSha256}`);
    if (result.status !== 'passed') process.exitCode = 1;
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
