// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import { readFile, writeFile, copyFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, relative, sep } from 'node:path';
import { runCommand } from '../../quality-process.mjs';
import { parseRows, project } from '../../input-handler-core-projection.mjs';

export const caseIds = ['vt-22'];
export async function runCase({ caseId, root, outputDirectory, signal }) {
  if (!caseIds.includes(caseId)) throw new Error(`Unsupported print/control case: ${caseId}`);
  const commands = [];
  const local = path => relative(root, path).split(sep).join('/');
  async function command(name, program, args, timeoutMs = 120_000) {
    const log = join(outputDirectory, `${name}.log`);
    const result = await runCommand(program, args, { cwd: root, env: process.env, log, timeoutMs, signal });
    commands.push({ argv: [program, ...args], exitCode: result.exitCode, signal: result.signal,
      aborted: result.aborted, timedOut: result.timedOut, error: result.error, log: local(log) });
    await writeFile(join(outputDirectory, 'command-receipts.json'), JSON.stringify(commands, null, 2) + '\n');
    if (result.status !== 'passed') throw new Error(`${name} failed: ${result.error ?? result.abortError ?? `exit ${result.exitCode}`}; retained ${local(log)}`);
    return readFile(log, 'utf8');
  }
  await command('original-print-control', process.execPath, ['scripts/input-handler-core-oracle.mjs']);
  const baseline = JSON.parse(await readFile(join(root, 'results/input-handler-core/oracle.json'), 'utf8'));
  if (baseline.baseline.failed || baseline.baseline.passed !== 24 || baseline.baseline.assertions !== 81) throw new Error('Original print/control baseline incomplete or changed');
  await writeFile(join(outputDirectory, 'baseline.json'), JSON.stringify(baseline, null, 2) + '\n');
  const tests = await command('rust-backspace-source-cases', 'cargo', ['test', '--locked', '-p', 'xterm-core', '--test', 'input_handler_core_backspace']);
  if (!/test result:.*8 passed; 0 failed/.test(tests.replace(/\u001b\[[0-9;]*m/g, ''))) throw new Error('Eight original backspace cases were not executed successfully');
  const build = await command('rust-recorder-build', 'cargo', ['build', '--locked', '-p', 'xterm-core', '--example', 'input_handler_core_trace', '--message-format=json']);
  const artifacts = build.split(/\r?\n/).filter(line => line.startsWith('{')).map(line => JSON.parse(line));
  const executable = artifacts.find(item => item.reason === 'compiler-artifact' && item.target?.name === 'input_handler_core_trace' && item.executable)?.executable;
  if (!executable) throw new Error('Cargo did not identify the actual recorder executable');
  // The canonical target may be a verified junction. Pin a regular case-owned
  // file before execution/hash, rather than weakening generic path validation.
  const pinned = join(outputDirectory, `input-handler-core-rust${process.platform === 'win32' ? '.exe' : ''}`);
  await copyFile(executable, pinned, constants.COPYFILE_EXCL);
  const native = await command('rust-recorded-states', pinned, []);
  const runtime = (await command('rust-version', 'rustc', ['--version'])).trim();
  const referenceText = await readFile(join(root, 'crates/xterm-core/tests/fixtures/input-handler-core.tsv'), 'utf8');
  const referenceRows = parseRows(referenceText, 'reverse-');
  const nativeRows = parseRows(native, 'reverse-');
  if (referenceRows.length !== 200 || nativeRows.length !== 200) throw new Error('Reverse-wrap case requires40traces and200independent snapshots, not zero or partial captures');
  const differences = [];
  for (let index = 0; index < referenceRows.length; index++) {
    try { assert.deepEqual(nativeRows[index], referenceRows[index]); }
    catch (error) { differences.push({ index, reference: referenceRows[index], rust: nativeRows[index], error: error.message }); }
  }
  await writeFile(join(outputDirectory, 'counterexamples.json'), JSON.stringify(differences, null, 2) + '\n');
  if (differences.length) {
    await writeFile(join(outputDirectory, 'paired-captures.json'), JSON.stringify({ reference: project(referenceRows), rust: project(nativeRows) }, null, 2) + '\n');
    throw new Error(`${differences.length} independent reverse-wrap snapshots differ; counterexamples and paired captures retained`);
  }
  const sourceAssertions = baseline.baseline.outcomes.filter(item => item.name.includes('BS with reverseWraparound')).reduce((sum, item) => sum + item.assertions, 0);
  if (sourceAssertions !== 36) throw new Error('Original backspace assertion accounting changed');
  return {
    caseId,
    sourceHashes: Object.entries(baseline.sourceHashes).map(([path, sha256]) => ({ path, sha256 })),
    artifactHashes: [{ path: local(pinned), sha256: createHash('sha256').update(await readFile(pinned)).digest('hex') }],
    commands,
    captures: [{ engine: 'native', implementation: 'rust-native', runtimeVersion: `${runtime}; ${process.platform}-${process.arch}`,
      assertions: sourceAssertions + referenceRows.length,
      reference: project(referenceRows), rust: project(nativeRows) }]
  };
}
