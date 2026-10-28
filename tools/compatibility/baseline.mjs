// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const fields = ['discovered', 'executed', 'passed', 'failed', 'pending', 'skipped', 'unrun'];
function validateCounts(counts, label) {
  for (const field of fields) assert(Number.isSafeInteger(counts[field]) && counts[field] >= 0, `Invalid baseline ${label}.${field}`);
  assert.equal(counts.discovered, counts.executed + counts.pending + counts.skipped + counts.unrun, `Baseline discovery mismatch: ${label}`);
  assert.equal(counts.executed, counts.passed + counts.failed, `Baseline execution mismatch: ${label}`);
  assert.equal(counts.failed, 0, `Failed baseline: ${label}`);
  assert.equal(counts.skipped, 0, `Filtered baseline: ${label}`);
  assert.equal(counts.unrun, 0, `Incomplete baseline: ${label}`);
}

/** Validate the current completed baseline, never silently reuse a historic pass. */
export async function loadBaseline(root, pointerPath, referenceManifest) {
  const read = async relative => {
    assert(typeof relative === 'string' && relative.length && !path.isAbsolute(relative) && !relative.split(/[\\/]/).includes('..'), `Unsafe baseline path: ${relative}`);
    return readFile(path.join(root, relative));
  };
  const pointer = JSON.parse(await read(pointerPath));
  assert.equal(pointer.status, 'passed', 'Latest baseline is not passed');
  assert(pointer.runId && pointer.summary && !path.isAbsolute(pointer.summary) && !pointer.summary.split(/[\\/]/).includes('..'), 'Invalid baseline pointer');
  const summaryPath = path.posix.join(path.posix.dirname(pointerPath), pointer.summary);
  const summary = JSON.parse(await read(summaryPath));
  assert.equal(summary.status, 'passed', 'Baseline summary is not passed');
  assert.equal(summary.runId, pointer.runId, 'Baseline pointer runId mismatch');
  assert(summary.commands.length > 0, 'Baseline commands missing');
  const commandLogs = [];
  for (const command of summary.commands) {
    assert.equal(command.status, 'passed', 'Baseline command did not pass');
    assert.equal(command.exitCode, 0, 'Baseline command failed');
    for (const field of ['error', 'signal', 'terminationError', 'resourceError']) assert(!command[field], `Baseline command ${field}`);
    assert.equal(command.timedOut, false, 'Baseline command timed out');
    await read(command.log);
    commandLogs.push(command.log);
  }
  for (const phase of ['before', 'after']) {
    assert.equal(summary.reference[phase].files, referenceManifest.length, `Baseline ${phase} file count mismatch`);
    assert.deepEqual(summary.reference[phase].mismatches, [], `Baseline ${phase} reference mismatch`);
  }
  const retained = await read(summary.reference.manifest);
  assert.equal(digest(retained), summary.reference.manifestSha256, 'Baseline manifest hash mismatch');
  const originalManifest = JSON.parse(retained);
  assert.equal(originalManifest.length, referenceManifest.length, 'Baseline retained manifest incomplete');
  assert.equal(new Set(originalManifest.map(row => row.path)).size, originalManifest.length, 'Duplicate baseline manifest path');
  const live = new Map(referenceManifest.map(row => [row.path.replace(/^source_repo\//, ''), row]));
  for (const row of originalManifest) {
    assert.equal(live.get(row.path)?.sha256, row.sha256, `Baseline source hash mismatch: ${row.path}`);
    assert.equal(live.get(row.path)?.bytes, row.bytes, `Baseline source size mismatch: ${row.path}`);
  }
  assert.equal(digest(await read('source_repo/package-lock.json')), summary.reference.packageLockSha256, 'Baseline package lock mismatch');
  const rawBytes = await read(summary.rawReport);
  assert.equal(digest(rawBytes), summary.rawReportSha256, 'Baseline raw report hash mismatch');
  const raw = JSON.parse(rawBytes);
  validateCounts(summary.totals, 'totals');
  assert.equal(new Set(summary.suites.map(s => s.source)).size, summary.suites.length, 'Duplicate baseline suite');
  const expectedSuites = referenceManifest.map(row => row.path.replace(/^source_repo\//, '')).filter(p => /^(src|addons\/[^/]+\/src)\/.*\.test\.ts$/.test(p)).sort();
  assert.deepEqual(summary.suites.map(s => s.source).sort(), expectedSuites, 'Baseline source suite set incomplete');
  for (const suite of summary.suites) validateCounts(suite, suite.source);
  for (const field of fields) assert.equal(summary.suites.reduce((sum, suite) => sum + suite[field], 0), summary.totals[field], `Baseline suite total mismatch: ${field}`);
  for (const [rawKey, totalKey] of [['tests', 'discovered'], ['passes', 'passed'], ['failures', 'failed'], ['pending', 'pending']]) assert.equal(raw.stats[rawKey], summary.totals[totalKey], `Baseline raw stats mismatch: ${rawKey}`);
  return { summary, summaryPath, evidencePaths: [pointerPath, summaryPath, summary.reference.manifest, summary.rawReport, ...commandLogs] };
}
