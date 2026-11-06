// Require actual paired state equality before starting any measured repetitions.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { prepared, verify } from './matched-benchmark-adapter.mjs';
import { validateManifest, outputHash } from './benchmark-contract.mjs';
import { sample, loadEngine } from './matched-benchmark-js.mjs';
import { runCommand } from './quality-process.mjs';
const [manifestPath, jsPath, rustPath, receiptPath] = process.argv.slice(2);
if (!receiptPath || process.argv.length !== 6) throw new Error('Usage: matched-benchmark-preflight.mjs MANIFEST JS_PROVENANCE RUST_PROVENANCE RECEIPT');
const receipt = { status: 'failed', equivalence: 'NOT PROVEN', cases: [] };
writeFileSync(receiptPath, JSON.stringify(receipt));
let scratch;
try {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const javascript = JSON.parse(readFileSync(jsPath, 'utf8'));
  const rust = JSON.parse(readFileSync(rustPath, 'utf8'));
  receipt.manifestSha256 = validateManifest(manifest);
  receipt.builds = { javascript: verify(javascript), rust: verify(rust) };
  scratch = mkdtempSync(join(process.env.TMPDIR ?? 'tmp', 'paired-capture-'));
  const input = join(scratch, 'prepared.json'), output = join(scratch, 'native.json'), log = join(scratch, 'native.log');
  writeFileSync(input, JSON.stringify(prepared(manifest)));
  receipt.command = await runCommand(rust.artifacts.native.path, ['--capture', input, output],
    { cwd: dirname(rust.artifacts.native.path), env: process.env, log, timeoutMs: 120000 });
  receipt.nativeLog = readFileSync(log, 'utf8');
  assert.equal(receipt.command.status, 'passed', 'Native capture failed');
  const captured = JSON.parse(readFileSync(output, 'utf8'));
  receipt.nativeCaptures = captured;
  assert.equal(captured.length, manifest.cases.length);
  const engine = await loadEngine(javascript.artifacts);
  const mismatches = [];
  for (let i = 0; i < manifest.cases.length; i++) {
    const expected = manifest.cases[i], native = captured[i];
    const original = (await sample(expected, engine, undefined, manifest.environment.writeMode)).output;
    const row = { id: expected.id, expected: expected.expectedOutput, javascript: original, rust: native.output };
    receipt.cases.push(row);
    assert.equal(native.id, expected.id);
    row.sourceMatched = outputHash(original) === expected.expectedOutputSha256;
    row.rustMatched = outputHash(native.output) === expected.expectedOutputSha256;
    if (!row.sourceMatched || !row.rustMatched) mismatches.push(expected.id);
  }
  assert.deepEqual(mismatches, [], 'Paired captures differ; no timing allowed');
  verify(javascript); verify(rust);
  receipt.status = 'passed';
} catch (error) { receipt.error = String(error.stack ?? error); process.exitCode = 1; }
finally {
  try { if (scratch) rmSync(scratch, { recursive: true }); }
  catch (error) { receipt.status = 'failed'; receipt.cleanupError = String(error); process.exitCode = 1; }
  writeFileSync(receiptPath, JSON.stringify(receipt));
}
console.log(`${receipt.status}: ${receipt.error ?? receipt.manifestSha256}`);
