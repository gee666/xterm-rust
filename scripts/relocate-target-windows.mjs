#!/usr/bin/env node
// Windows storage adapter only; the portable Rust engine has no cache-path dependency.
import { lstat, statfs, realpath, writeFile, readFile, mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { createHash, randomUUID } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { resolve, join, dirname, parse } from 'node:path';
import { fileURLToPath } from 'node:url';
import { relocateCache } from './target-relocation.mjs';

async function main() {
  if (process.platform !== 'win32') throw new Error('This operational relocation adapter requires Windows');
  const root = fileURLToPath(new URL('../', import.meta.url));
  const key = createHash('sha256').update(root.toLowerCase()).digest('hex').slice(0, 12);
  const options = { destination: join(homedir(), '.xterm-rust-target', key), apply: false };
  const seen = new Set();
  const args = process.argv.slice(2);
  for (let index = 0; index < args.length; index++) {
    const flag = args[index];
    if (seen.has(flag)) throw new Error('Duplicate relocation option');
    seen.add(flag);
    if (flag === '--apply') options.apply = true;
    else if (['--destination', '--approved-by', '--reviewed-by'].includes(flag) && args[index + 1] && !args[index + 1].startsWith('--')) {
      const name = { '--destination': 'destination', '--approved-by': 'approvedBy', '--reviewed-by': 'reviewedBy' }[flag];
      options[name] = args[++index];
    } else throw new Error('Unknown or incomplete relocation option');
  }
  if (options.apply && (!options.approvedBy || !options.reviewedBy)) throw new Error('Apply requires coordinated --approved-by and independent --reviewed-by records');
  const destination = resolve(options.destination);
  if (!/^[A-Za-z]:\\/.test(destination)) throw new Error('Destination must be a local drive path, not a network share');
  if (parse(destination).root.toLowerCase() === parse(root).root.toLowerCase()) throw new Error('Destination must relieve the project drive, not create a second cache on it');
  let existing = dirname(destination);
  for (;;) {
    try { await lstat(existing); break; }
    catch (error) {
      if (error.code !== 'ENOENT') throw error;
      const parent = dirname(existing);
      if (parent === existing) throw new Error('Destination drive is unavailable');
      existing = parent;
    }
  }
  const physicalParent = await realpath(existing);
  if (parse(physicalParent).root.toLowerCase() !== parse(destination).root.toLowerCase()) throw new Error('Destination parent redirects to a different drive or network location');
  const rootFree = await statfs(root, { bigint: true });
  const destFree = await statfs(existing, { bigint: true });
  const directory = join(root, 'results', 'quality', `target-relocation-${Date.now()}-${process.pid}-${randomUUID().slice(0, 8)}`);
  const toolHashes = {};
  for (const name of ['relocate-target-windows.mjs', 'target-relocation.mjs', 'quality-idle.mjs', 'build-slot-core.mjs']) {
    toolHashes[`scripts/${name}`] = createHash('sha256').update(await readFile(join(root, 'scripts', name))).digest('hex');
  }
  await mkdir(directory, { recursive: true });
  await writeFile(join(directory, 'review-and-tools.json'), JSON.stringify({ approvedBy: options.approvedBy ?? null, reviewedBy: options.reviewedBy ?? null,
    toolHashes, node: process.version, platform: process.platform, architecture: process.arch }, null, 2) + '\n');
  const probe = async (target, output, stage) => {
    const args = ['metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'];
    const result = spawnSync('cargo', args, { cwd: root, env: process.env, timeout: 30_000, maxBuffer: 5_000_000, windowsHide: true });
    const log = join(output, `cargo-metadata-${stage}.log`);
    await writeFile(log, Buffer.concat([result.stdout ?? Buffer.alloc(0), result.stderr ?? Buffer.alloc(0)]));
    if (result.error || result.status !== 0) throw new Error(`Cargo metadata ${stage} failed; no compilation or dependency installation was requested`);
    const metadata = JSON.parse(result.stdout.toString('utf8'));
    const resolvedTarget = await realpath(metadata.target_directory);
    if (resolvedTarget.toLowerCase() !== (await realpath(target)).toLowerCase()) throw new Error('Cargo metadata does not resolve to the canonical shared target');
    if ((await realpath(metadata.workspace_root)).toLowerCase() !== (await realpath(root)).toLowerCase()) throw new Error('Cargo metadata points to a different workspace');
    return { command: ['cargo', ...args], exitCode: result.status, log, targetDirectory: metadata.target_directory, resolvedTarget, workspaceRoot: metadata.workspace_root };
  };
  const report = await relocateCache({ root, destination, apply: options.apply, approvedBy: options.approvedBy, directory, probe });
  const after = await statfs(root, { bigint: true });
  const targetAfter = await statfs(join(root, 'target'), { bigint: true }).catch(error => {
    if (error.code !== 'ENOENT') throw error;
    return null;
  });
  const observation = { reviewedBy: options.reviewedBy ?? null, rootFreeBefore: String(rootFree.bavail * rootFree.bsize),
    destinationFreeBefore: String(destFree.bavail * destFree.bsize), rootFreeAfter: String(after.bavail * after.bsize),
    canonicalTargetFreeAfter: targetAfter ? String(targetAfter.bavail * targetAfter.bsize) : null, receiptDirectory: directory };
  await writeFile(join(directory, 'space-and-review.json'), JSON.stringify(observation, null, 2) + '\n');
  console.log(JSON.stringify({ ...report, ...observation }, null, 2));
  if (report.status !== (options.apply ? 'completed' : 'dry-run-ready')) process.exitCode = 1;
}
await main().catch(error => { console.error(error.message ?? String(error)); process.exitCode = 1; });
