#!/usr/bin/env node
// Coordinated generated-artifact recovery only. This never deletes source or evidence.
import { readFile, writeFile, mkdir, lstat, statfs } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
import { observeBuildProcesses } from './quality-idle.mjs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const target = join(root, 'target');
const args = process.argv.slice(2);
let apply = false;
let approvedBy = null;
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--apply' && !apply) apply = true;
  else if (args[i] === '--approved-by' && !approvedBy && args[i + 1] && !args[i + 1].startsWith('--')) approvedBy = args[++i];
  else throw new Error('Unknown, duplicated or incomplete cleanup argument');
}
if (apply && !approvedBy) throw new Error('--apply requires a recorded coordinated --approved-by NAME');
const directory = join(root, 'results', 'quality', `resource-clean-${Date.now()}`);
await mkdir(directory, { recursive: true });
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const report = { startedAt: new Date().toISOString(), apply, approvedBy, target, status: 'failed', commands: [] };
async function free() {
  const value = await statfs(root, { bigint: true });
  return (value.bavail * value.bsize).toString();
}
try {
  report.freeBytesBefore = await free();
  for (const control of ['paused.json', 'native/active.lock', 'wasm/active.lock']) {
    try {
      await lstat(join(root, 'tmp/build-slots', control));
      throw new Error('Build lane or recovery maintenance is active; cleanup refused');
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
  }
  const info = await lstat(target);
  if (!info.isDirectory() || info.isSymbolicLink()) throw new Error('Cleanup only accepts the real repository target directory, not a linked or non-directory path');
  const marker = await readFile(join(target, 'CACHEDIR.TAG'));
  if (!marker.toString('utf8').startsWith('Signature: 8a477f597d28d172789f06886806bc55')) throw new Error('Cargo cache marker missing or invalid; cleanup refused');
  report.markerSha256 = digest(marker);
  const profile = await readFile(join(root, '.cargo', 'config.toml'));
  report.profileSha256 = digest(profile);
  report.activeProcesses = observeBuildProcesses(target);
  if (report.activeProcesses.length) throw new Error('Build/test processes are still active; cleanup refused without terminating them');
  {
    // Only the development profile is selected; do not erase release artifacts
    // merely to obtain a short-lived blanket green resource preflight.
    const argv = ['clean', '--profile', 'dev', '--manifest-path', join(root, 'Cargo.toml'), '--target-dir', target, '--locked', '--offline'];
    if (!apply) argv.push('--dry-run');
    const result = spawnSync('cargo', argv, { cwd: root, env: process.env, encoding: 'utf8', timeout: 120_000, maxBuffer: 2_000_000 });
    await writeFile(join(directory, 'cargo-clean.log'), (result.stdout ?? '') + (result.stderr ?? ''));
    report.commands.push({ argv: ['cargo', ...argv], exitCode: result.status, signal: result.signal, error: result.error?.message ?? null });
    if (result.error || result.status !== 0) throw new Error('Generated target cleanup failed; inspect the retained log');
    if (digest(await readFile(join(root, '.cargo', 'config.toml'))) !== report.profileSha256) throw new Error('Build profile changed during cleanup');
  }
  report.freeBytesAfter = await free();
  report.status = apply ? 'completed' : 'cargo-dry-run-completed';
} catch (error) { report.error = error.message; process.exitCode = 1; }
report.finishedAt = new Date().toISOString();
await writeFile(join(directory, 'receipt.json'), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ ...report, receiptDirectory: directory }, null, 2));
