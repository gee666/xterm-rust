import { mkdir, readFile, writeFile, readdir, lstat, rm, symlink, realpath, statfs, rename } from 'node:fs/promises';
import { createReadStream } from 'node:fs';
import { createHash, randomUUID } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { join, relative, dirname, resolve, sep } from 'node:path';
import { observeBuildProcesses } from './quality-idle.mjs';

const CACHE_SIGNATURE = 'Signature: 8a477f597d28d172789f06886806bc55';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
async function exists(path) {
  try { return await lstat(path); }
  catch (error) { if (error.code === 'ENOENT') return null; throw error; }
}
export async function cacheManifest(root) {
  const files = [];
  const directories = [];
  async function visit(path) {
    const info = await lstat(path, { bigint: true });
    const name = relative(root, path).split(sep).join('/');
    if (info.isSymbolicLink()) throw new Error(`Linked entry in cache: ${name}`);
    const metadata = { path: name, mtimeNs: info.mtimeNs.toString(), mode: info.mode.toString() };
    if (info.isDirectory()) {
      directories.push(metadata);
      for (const entry of (await readdir(path)).sort()) await visit(join(path, entry));
    } else if (info.isFile()) {
      const hash = createHash('sha256');
      for await (const chunk of createReadStream(path)) hash.update(chunk);
      files.push({ ...metadata, bytes: info.size.toString(), sha256: hash.digest('hex') });
    } else throw new Error(`Nonregular entry in cache: ${name}`);
  }
  await visit(root);
  return { directories, files };
}
function same(expected, actual, label) {
  if (JSON.stringify(expected) !== JSON.stringify(actual)) throw new Error(`${label}: bytes, paths, modes or nanosecond modification times differ`);
}
export async function controlManifest(root) {
  const paths = ['Cargo.toml', 'Cargo.lock', '.gitattributes', 'AGENTS.md', '.cargo/config.toml'];
  try {
    for (const entry of await readdir(join(root, 'crates'), { withFileTypes: true })) {
      if (entry.isDirectory()) paths.push(`crates/${entry.name}/Cargo.toml`);
    }
  } catch (error) { if (error.code !== 'ENOENT') throw error; }
  const controls = {};
  for (const path of paths.sort()) {
    const file = join(root, path);
    const info = await exists(file);
    if (!info) { controls[path] = { absent: true }; continue; }
    if (!info.isFile() || info.isSymbolicLink()) throw new Error(`Nonregular control file: ${path}`);
    controls[path] = { bytes: info.size, sha256: digest(await readFile(file)) };
  }
  return controls;
}
export async function robocopyCache(source, destination, log) {
  if (process.platform !== 'win32') throw new Error('Native robocopy adapter requires Windows');
  const command = join(process.env.SystemRoot, 'System32', 'robocopy.exe');
  const args = [source, destination, '/E', '/COPY:DAT', '/DCOPY:DAT', '/R:0', '/W:0', '/XJ', '/NFL', '/NDL', '/NP'];
  const result = spawnSync(command, args, { timeout: 120_000, maxBuffer: 2_000_000, windowsHide: true });
  await writeFile(log, Buffer.concat([result.stdout ?? Buffer.alloc(0), result.stderr ?? Buffer.alloc(0)]));
  if (result.error || result.signal || !Number.isInteger(result.status) || result.status < 0 || result.status > 7) throw new Error(`Robocopy failed with exit ${result.status}: ${result.error?.message ?? 'inspect log'}`);
  return { command: [command, ...args], exitCode: result.status, log };
}

/** Copy and verify first, then use same-volume rename for rollback. The old
 * bytes remain untouched until the junction and post-promotion probe pass. */
export async function relocateCache({ root, destination, apply, approvedBy, directory,
  copy = robocopyCache, link = (from, to) => symlink(from, to, process.platform === 'win32' ? 'junction' : 'dir'),
  observe = observeBuildProcesses, reserveBytes = 1024n * 1024n * 1024n,
  probe, removeBackup = path => rm(path, { recursive: true, maxRetries: 0 }) }) {
  const target = join(root, 'target');
  const cache = resolve(destination);
  const canonical = value => process.platform === 'win32' ? value.toLowerCase() : value;
  const sourceName = canonical(resolve(target));
  const cacheName = canonical(cache);
  if (cacheName === sourceName || cacheName.startsWith(sourceName + sep) || sourceName.startsWith(cacheName + sep)) throw new Error('Cache destination overlaps the canonical target');
  await mkdir(directory, { recursive: true });
  const report = { startedAt: new Date().toISOString(), target, destination: cache, apply, approvedBy,
    status: 'failed', phase: 'preflight', commands: [], token: randomUUID() };
  const backup = join(root, 'tmp', `target-backup-${report.token}`);
  report.backup = backup;
  const save = () => writeFile(join(directory, 'receipt.json'), JSON.stringify(report, null, 2) + '\n');
  const pause = join(root, 'tmp', 'build-slots', 'paused.json');
  const held = [];
  let paused = false;
  let renamed = false;
  let committed = false;
  let before;
  async function verifyCache(path, label) {
    const actual = await cacheManifest(path);
    const matches = JSON.stringify(before) === JSON.stringify(actual);
    report.verifications ??= [];
    report.verifications.push({ label, matches, at: new Date().toISOString(), manifestSha256: digest(JSON.stringify(actual)) });
    if (!matches) {
      await writeFile(join(directory, `verification-failure-${report.verifications.length}.json`), JSON.stringify(actual, null, 2) + '\n');
      same(before, actual, label);
    }
  }
  async function idle() {
    const active = observe(target);
    report.processObservations ??= [];
    report.processObservations.push({ at: new Date().toISOString(), active });
    if (active.length) throw new Error('Build/test processes are active; relocation refused without killing them');
  }
  try {
    const source = await lstat(target);
    if (!source.isDirectory() || source.isSymbolicLink()) throw new Error('Expected a real canonical target directory, not a link');
    if (!(await readFile(join(target, 'CACHEDIR.TAG'), 'utf8')).startsWith(CACHE_SIGNATURE)) throw new Error('Invalid Cargo cache marker');
    if (await exists(cache)) throw new Error('Destination already exists; no existing directory may be overwritten');
    if (await exists(backup)) throw new Error('Owned rollback backup path already exists');
    report.controls = await controlManifest(root);
    if (apply) {
      if (!approvedBy) throw new Error('Coordinated approval is required');
      await mkdir(dirname(pause), { recursive: true });
      await writeFile(pause, JSON.stringify({ token: report.token, owner: approvedBy, pid: process.pid, reason: 'verified canonical cache relocation' }), { flag: 'wx' });
      paused = true;
      for (const lane of ['native', 'wasm']) {
        const path = join(root, 'tmp', 'build-slots', lane, 'active.lock');
        await mkdir(dirname(path), { recursive: true });
        await mkdir(path);
        held.push(path);
        await writeFile(join(path, 'owner.json'), JSON.stringify({ id: report.token, owner: 'cache-maintenance', lane, pid: process.pid }));
      }
    }
    await idle();
    if (probe) report.preProbe = await probe(target, directory, 'before');
    report.controlsAfterPreflight = await controlManifest(root);
    same(report.controls, report.controlsAfterPreflight, 'Controls changed during preflight');
    before = await cacheManifest(target);
    if (!before.files.length) throw new Error('Refusing an empty cache relocation');
    report.fileCount = before.files.length;
    report.logicalBytes = before.files.reduce((sum, item) => sum + BigInt(item.bytes), 0n).toString();
    await writeFile(join(directory, 'source-manifest.json'), JSON.stringify(before, null, 2) + '\n');
    report.manifestSha256 = digest(JSON.stringify(before));
    report.phase = apply ? 'copy-ready' : 'dry-run-ready';
    await save();
    if (apply) {
      await mkdir(dirname(cache), { recursive: true });
      const space = await statfs(dirname(cache), { bigint: true });
      if (space.bavail * space.bsize < BigInt(report.logicalBytes) + reserveBytes) throw new Error('Destination lacks cache size plus required reserve');
      await mkdir(cache);
      report.commands.push(await copy(target, cache, join(directory, 'copy.log')));
      await verifyCache(cache, 'Copied cache verification');
      await verifyCache(target, 'Source changed during copy');
      await idle();
      report.controlsAfterCopy = await controlManifest(root);
      same(report.controls, report.controlsAfterCopy, 'Controls changed before promotion');
      report.phase = 'verified-copy-before-rename';
      await save();
      await rename(target, backup);
      renamed = true;
      report.phase = 'rollback-backup-retained-before-junction';
      await save();
      await verifyCache(backup, 'Rollback backup verification');
      await link(cache, target);
      if (canonical(await realpath(target)) !== canonical(await realpath(cache))) throw new Error('Canonical target link resolves to the wrong destination');
      await verifyCache(cache, 'Promoted cache verification');
      if (probe) report.postProbe = await probe(target, directory, 'after');
      report.controlsAfterPromotion = await controlManifest(root);
      same(report.controls, report.controlsAfterPromotion, 'Controls changed during promotion');
      await verifyCache(cache, 'Probe modified preserved cache data');
      await verifyCache(backup, 'Rollback backup changed before cleanup');
      committed = true;
      report.phase = 'promoted-and-probed-before-backup-cleanup';
      await save();
      await removeBackup(backup);
      renamed = false;
      report.phase = 'promoted-single-canonical-cache';
    }
    report.status = apply ? 'completed' : 'dry-run-ready';
  } catch (error) {
    report.error = error.message;
    if (renamed && !committed) {
      try {
        const current = await exists(target);
        if (current) {
          if (!current.isSymbolicLink() || canonical(await realpath(target)) !== canonical(await realpath(cache))) throw new Error('Unknown canonical entry appeared; refusing to remove it');
          await rm(target, { recursive: true });
        }
        await rename(backup, target);
        await verifyCache(target, 'Rollback rename verification');
        report.rollback = 'verified-source-restored';
        report.phase = 'rolled-back';
        renamed = false;
      } catch (rollbackError) { report.rollbackError = rollbackError.message; report.phase = 'manual-recovery-required-backup-retained'; }
    } else if (committed) report.phase = 'promoted-backup-cleanup-incomplete';
  } finally {
    for (const path of held.reverse()) {
      try {
        const owner = JSON.parse(await readFile(join(path, 'owner.json'), 'utf8'));
        if (owner.id !== report.token) throw new Error('Maintenance lease ownership changed');
        await rm(path, { recursive: true });
      } catch (error) { report.maintenanceError = error.message; report.status = 'failed'; }
    }
    if (paused && !report.rollbackError && !report.maintenanceError) {
      try {
        const owner = JSON.parse(await readFile(pause, 'utf8'));
        if (owner.token !== report.token) throw new Error('Pause ownership changed; refusing to clear it');
        await rm(pause);
      } catch (error) { report.maintenanceError = error.message; report.status = 'failed'; }
    }
  }
  report.finishedAt = new Date().toISOString();
  await save();
  return report;
}
