// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { mkdtemp, mkdir, writeFile, readFile, rm, symlink } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { digest, inventory, verifyReference, buildIndex, checkIndex } from './verification-index.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
async function fixture(callback) {
  await mkdir(path.join(root, 'tmp'), { recursive: true });
  const temporary = await mkdtemp(path.join(root, 'tmp/verification-index-'));
  try { await callback(temporary); } finally { await rm(temporary, { recursive: true, force: true }); }
}

test('inventory hashes exact bytes and sorts nested paths', async () => fixture(async directory => {
  await mkdir(path.join(directory, 'a'));
  await writeFile(path.join(directory, 'z.txt'), 'line\r\n');
  await writeFile(path.join(directory, 'a/b.txt'), '😀');
  assert.deepEqual(await inventory(directory), [
    { path: 'a/b.txt', bytes: 4, sha256: digest(Buffer.from('😀')) },
    { path: 'z.txt', bytes: 6, sha256: digest(Buffer.from('line\r\n')) }
  ]);
}));

test('inventory rejects symlinks and Windows junctions instead of following them', async () => fixture(async directory => {
  await mkdir(path.join(directory, 'target'));
  await writeFile(path.join(directory, 'target/file.log'), 'outside evidence');
  await symlink(path.join(directory, 'target'), path.join(directory, 'linked'), process.platform === 'win32' ? 'junction' : 'dir');
  await assert.rejects(inventory(directory), /Evidence must be a regular file or directory: linked/);
}));

test('reference checker retains changed and missing records', async () => fixture(async directory => {
  await writeFile(path.join(directory, 'same'), 'yes');
  await writeFile(path.join(directory, 'changed'), 'new');
  const manifest = ['same', 'changed', 'missing'].map(name => ({ path: name, bytes: 3, sha256: digest(Buffer.from(name === 'same' ? 'yes' : 'old')) }));
  const report = await verifyReference(directory, manifest);
  assert.equal(report.matched, false);
  assert.equal(report.checkedFiles, 2);
  assert.deepEqual(report.missing, ['missing']);
  assert.deepEqual(report.changed.map(record => record.path), ['changed']);
}));

test('reference checker rejects invalid paths and duplicate entries', async () => fixture(async directory => {
  const record = { path: 'file', bytes: 0, sha256: digest(Buffer.alloc(0)) };
  await writeFile(path.join(directory, 'file'), '');
  for (const invalid of ['../outside', '/outside', 'a\\b', './file', 'a//b', '', 'C:/outside', 'C:relative']) {
    await assert.rejects(verifyReference(directory, [{ ...record, path: invalid }]), /Invalid or duplicate/);
  }
  await assert.rejects(verifyReference(directory, [record, record]), /Invalid or duplicate/);
}));

test('CLI retains failed integrity reports and exits nonzero', async () => fixture(async directory => {
  for (const name of ['scripts', 'source_repo', 'docs/baseline', 'results']) await mkdir(path.join(directory, name), { recursive: true });
  await writeFile(path.join(directory, 'scripts/verification-index.mjs'), await readFile(path.join(root, 'scripts/verification-index.mjs')));
  const manifest = [{ path: 'missing.ts', bytes: 3, sha256: digest(Buffer.from('old')) }];
  await writeFile(path.join(directory, 'docs/baseline/source-manifest.json'), JSON.stringify(manifest));
  for (const changed of [false, true]) {
    if (changed) await writeFile(path.join(directory, 'source_repo/missing.ts'), 'new');
    const run = spawnSync(process.execPath, ['scripts/verification-index.mjs'], { cwd: directory, encoding: 'utf8', timeout: 10000 });
    if (run.error) throw run.error;
    assert.equal(run.status, 1, run.stderr);
    const result = JSON.parse(await readFile(path.join(directory, 'results/verification/index.json'), 'utf8'));
    assert.equal(result.reference.matched, false);
    assert.equal(changed ? result.reference.changed.length : result.reference.missing.length, 1);
  }
}));

test('saved snapshots detect changed, missing, added files and changed manifest bytes', async () => fixture(async directory => {
  for (const name of ['source_repo', 'docs/baseline', 'results/verification']) await mkdir(path.join(directory, name), { recursive: true });
  await writeFile(path.join(directory, 'source_repo/source.ts'), 'source');
  const manifest = [{ path: 'source.ts', bytes: 6, sha256: digest(Buffer.from('source')) }];
  await writeFile(path.join(directory, 'docs/baseline/source-manifest.json'), JSON.stringify(manifest));
  await writeFile(path.join(directory, 'results/change.log'), 'old');
  await writeFile(path.join(directory, 'results/remove.log'), 'old');
  const saved = JSON.stringify(await buildIndex(directory));
  const savedPath = path.join(directory, 'results/verification/index.json');
  await writeFile(savedPath, saved);
  assert.equal((await checkIndex(directory)).matched, true);
  await writeFile(path.join(directory, 'results/change.log'), 'new');
  await rm(path.join(directory, 'results/remove.log'));
  await writeFile(path.join(directory, 'results/added.log'), 'new');
  await writeFile(path.join(directory, 'docs/baseline/source-manifest.json'), JSON.stringify(manifest, null, 2));
  const checked = await checkIndex(directory);
  assert.equal(checked.matched, false);
  assert.deepEqual(checked.changed.map(record => record.path), ['results/change.log']);
  assert.deepEqual(checked.missing, ['results/remove.log']);
  assert.deepEqual(checked.added, ['results/added.log']);
  assert.equal(checked.manifestMatched, false);
  assert.equal(await readFile(savedPath, 'utf8'), saved, 'checking must not replace the prior snapshot');
}));

test('index excludes itself and never infers success from retained evidence', async () => fixture(async directory => {
  for (const name of ['source_repo', 'docs/baseline', 'results/verification']) await mkdir(path.join(directory, name), { recursive: true });
  await writeFile(path.join(directory, 'source_repo/source.ts'), 'source');
  const manifest = [{ path: 'source.ts', bytes: 6, sha256: digest(Buffer.from('source')) }];
  await writeFile(path.join(directory, 'docs/baseline/source-manifest.json'), JSON.stringify(manifest));
  await writeFile(path.join(directory, 'results/failure.log'), 'FAILED');
  await writeFile(path.join(directory, 'results/verification/index.json'), 'old index');
  const index = await buildIndex(directory);
  assert.equal(index.reference.matched, true);
  assert.deepEqual(index.evidence.map(record => record.path), ['results/failure.log']);
  assert.match(index.scope, /NOT PROVEN/);
  assert.match(index.interpretation, /not passed checks/);
}));
