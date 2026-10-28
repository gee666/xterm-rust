import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile, readFile, cp, rm, readdir, lstat, utimes, realpath } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cacheManifest, relocateCache } from './target-relocation.mjs';
const project = fileURLToPath(new URL('../', import.meta.url));
async function fixedTimes(path) {
  const info = await lstat(path);
  if (info.isDirectory()) for (const name of await readdir(path)) await fixedTimes(join(path, name));
  await utimes(path, 1700000000, 1700000000);
}
async function copy(source, destination, log) {
  await cp(source, destination, { recursive: true, force: true, preserveTimestamps: true });
  const manifest = await cacheManifest(source);
  for (const dir of [...manifest.directories].reverse()) await utimes(join(destination, dir.path), 1700000000, 1700000000);
  await writeFile(log, 'Synthetic byte/mtime copy boundary, no Cargo or compiler executed\n');
  return { command: ['node:test-copy'], exitCode: 0, log };
}
async function fixture(fn) {
  await mkdir(join(project, 'tmp'), { recursive: true });
  const parent = await mkdtemp(join(project, 'tmp', 'target-relocation-test-'));
  const root = join(parent, 'project');
  const destination = join(parent, 'persistent-cache');
  const target = join(root, 'target');
  await mkdir(join(root, '.cargo'), { recursive: true });
  await mkdir(join(target, '.fingerprint', 'package'), { recursive: true });
  await writeFile(join(root, '.cargo', 'config.toml'), '[profile.dev]\ndebug=0\nincremental=false\n');
  await writeFile(join(target, 'CACHEDIR.TAG'), 'Signature: 8a477f597d28d172789f06886806bc55\n');
  await writeFile(join(target, 'compiled.bin'), Buffer.from([0, 1, 127, 255]));
  await writeFile(join(target, '.fingerprint', 'package', 'invoked.timestamp'), '');
  await fixedTimes(target);
  const before = await cacheManifest(target);
  const options = { root, destination, apply: true, approvedBy: 'unit-test', directory: join(root, 'results', 'operation'), copy, observe: () => [], reserveBytes: 0n };
  try { await fn({ root, target, destination, before, options }); }
  finally { await rm(parent, { recursive: true, force: true }); }
}
test('dry run reads exact cache metadata without creating a destination or link', () => fixture(async ({ target, destination, before, options }) => {
  const result = await relocateCache({ ...options, apply: false });
  assert.equal(result.status, 'dry-run-ready');
  assert.deepEqual(await cacheManifest(target), before);
  await assert.rejects(lstat(destination), { code: 'ENOENT' });
}));
test('promotion keeps one canonical link and preserves bytes and nanosecond times', () => fixture(async ({ root, target, destination, before, options }) => {
  const result = await relocateCache(options);
  assert.equal(result.status, 'completed', JSON.stringify(result));
  assert.equal((await lstat(target)).isSymbolicLink(), true);
  assert.equal(await realpath(target), await realpath(destination));
  assert.deepEqual(await cacheManifest(destination), before);
  for (const lane of ['native', 'wasm']) assert.deepEqual(await readdir(join(root, 'tmp', 'build-slots', lane)), []);
  await assert.rejects(lstat(join(root, 'tmp/build-slots/paused.json')), { code: 'ENOENT' });
}));
test('existing destinations and active processes are refused without source loss', () => fixture(async ({ target, destination, before, options }) => {
  await mkdir(destination);
  await writeFile(join(destination, 'user-file'), 'preserve');
  const existing = await relocateCache(options);
  assert.equal(existing.status, 'failed');
  assert.equal(await readFile(join(destination, 'user-file'), 'utf8'), 'preserve');
  assert.deepEqual(await cacheManifest(target), before);
  await rm(destination, { recursive: true });
  const busy = await relocateCache({ ...options, observe: () => [{ pid: 1, name: 'synthetic-busy-build' }] });
  assert.equal(busy.status, 'failed');
  assert.deepEqual(await cacheManifest(target), before);
}));
test('a corrupt copied byte prevents source removal', () => fixture(async ({ target, before, options }) => {
  const result = await relocateCache({ ...options, copy: async (source, destination, log) => {
    const record = await copy(source, destination, log);
    await writeFile(join(destination, 'compiled.bin'), 'corrupt');
    return record;
  } });
  assert.equal(result.status, 'failed');
  assert.match(result.error, /Copied cache verification/);
  assert.deepEqual(await cacheManifest(target), before);
}));
test('junction creation failure restores the original real cache and keeps failure visible', () => fixture(async ({ target, before, options }) => {
  const result = await relocateCache({ ...options, link: async () => { throw new Error('injected junction failure'); } });
  assert.equal(result.status, 'failed');
  assert.equal(result.rollback, 'verified-source-restored');
  assert.equal((await lstat(target)).isSymbolicLink(), false);
  assert.deepEqual(await cacheManifest(target), before);
}));
test('partial backup cleanup leaves the verified canonical cache intact and fails honestly', () => fixture(async ({ target, destination, before, options }) => {
  const result = await relocateCache({ ...options, removeBackup: async path => {
    await rm(join(path, 'compiled.bin'));
    throw new Error('injected partial backup cleanup failure');
  } });
  assert.equal(result.status, 'failed');
  assert.equal(result.phase, 'promoted-backup-cleanup-incomplete');
  assert.equal((await lstat(target)).isSymbolicLink(), true);
  assert.deepEqual(await cacheManifest(destination), before);
  assert.equal((await lstat(result.backup)).isDirectory(), true);
}));
test('an unknown canonical entry is preserved and maintenance remains paused', () => fixture(async ({ root, target, options }) => {
  const result = await relocateCache({ ...options, link: async (_from, path) => {
    await mkdir(path);
    await writeFile(join(path, 'foreign-file'), 'do not overwrite');
    throw new Error('injected concurrent mutation');
  } });
  assert.equal(result.phase, 'manual-recovery-required-backup-retained');
  assert.equal(await readFile(join(target, 'foreign-file'), 'utf8'), 'do not overwrite');
  assert.equal((await lstat(join(root, 'tmp/build-slots/paused.json'))).isFile(), true);
}));
test('post-promotion probe failure rolls back by rename, not a second cross-volume copy', () => fixture(async ({ target, before, options }) => {
  let copies = 0;
  const result = await relocateCache({ ...options, copy: async (...args) => { copies++; return copy(...args); },
    probe: async (_target, _output, stage) => { if (stage === 'after') throw new Error('injected metadata probe failure'); return { stage }; }
  });
  assert.equal(result.status, 'failed');
  assert.equal(result.rollback, 'verified-source-restored');
  assert.equal(copies, 1);
  assert.deepEqual(await cacheManifest(target), before);
}));
test('changed control files are not overwritten and force a safe rollback', () => fixture(async ({ root, target, before, options }) => {
  const result = await relocateCache({ ...options,
    probe: async (_target, _output, stage) => { if (stage === 'after') await writeFile(join(root, 'Cargo.lock'), 'external change'); return { stage }; }
  });
  assert.equal(result.status, 'failed');
  assert.match(result.error, /Controls changed/);
  assert.equal(await readFile(join(root, 'Cargo.lock'), 'utf8'), 'external change');
  assert.deepEqual(await cacheManifest(target), before);
}));
test('a live foreign lane lease is never overwritten or removed', () => fixture(async ({ root, target, before, options }) => {
  const lane = join(root, 'tmp/build-slots/native/active.lock');
  await mkdir(lane, { recursive: true });
  await writeFile(join(lane, 'owner.json'), JSON.stringify({ id: 'foreign', pid: process.pid }));
  const result = await relocateCache(options);
  assert.equal(result.status, 'failed');
  assert.equal(JSON.parse(await readFile(join(lane, 'owner.json'), 'utf8')).id, 'foreign');
  assert.deepEqual(await cacheManifest(target), before);
}));
