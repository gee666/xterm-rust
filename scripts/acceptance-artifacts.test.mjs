import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { mkdtemp, mkdir, writeFile, readFile, rm, symlink, lstat } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { pinArtifact } from './acceptance-artifacts.mjs';
import { regularFile } from './acceptance-validation.mjs';

const project = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const signature = 'Signature: 8a477f597d28d172789f06886806bc55\n';
const enough = { observeFree: async () => 4n * 1024n * 1024n * 1024n };
async function fixture(t, relocated = false) {
  await mkdir(path.join(project, 'tmp'), { recursive: true });
  const root = await mkdtemp(path.join(project, 'tmp/artifact-pin-test-'));
  t.after(() => rm(root, { recursive: true }));
  const target = path.join(root, relocated ? 'backing-cache' : 'target');
  await mkdir(path.join(target, 'debug'), { recursive: true });
  await writeFile(path.join(target, 'CACHEDIR.TAG'), signature);
  if (relocated) await symlink(target, path.join(root, 'target'), 'junction');
  const executable = path.join(root, 'target/debug/fixture.exe');
  await writeFile(executable, 'synthetic artifact storage test, not Rust execution\n', { mode: 0o755 });
  return { root, target, executable };
}

test('pinning copies immutable bytes and deduplicates complete content', async t => {
  const { root, executable } = await fixture(t);
  const first = await pinArtifact(root, executable, enough);
  const second = await pinArtifact(root, executable, enough);
  assert.deepEqual(first, second);
  const pinned = await regularFile(root, first.path);
  assert.deepEqual(await readFile(pinned), await readFile(executable));
  const info = await lstat(pinned);
  if (process.platform !== 'win32') assert.equal(info.mode & 0o111, 0o111);
  await writeFile(executable, 'later compiler output');
  assert.notDeepEqual(await readFile(pinned), await readFile(executable));
});

test('approved canonical target junction needs no artifact validator exception', async t => {
  const { root, executable } = await fixture(t, true);
  await assert.rejects(regularFile(root, 'target/debug/fixture.exe'), /symbolic/);
  const pinned = await pinArtifact(root, executable, enough);
  assert.ok((await regularFile(root, pinned.path)).endsWith('artifact.exe'));
  assert.deepEqual(await readFile(path.join(root, pinned.path)), await readFile(executable));
});

test('nested target links and paths outside the Cargo cache are rejected', async t => {
  const { root, target } = await fixture(t);
  const outside = path.join(root, 'outside');
  await mkdir(outside);
  await writeFile(path.join(outside, 'other.exe'), 'outside');
  await assert.rejects(pinArtifact(root, path.join(outside, 'other.exe'), enough), /outside/);
  await symlink(outside, path.join(target, 'nested'), 'junction');
  await assert.rejects(pinArtifact(root, path.join(target, 'nested/other.exe'), enough), /symbolic/);
});

test('regular private target needs no Cargo marker; invalid kinds still fail', async t => {
  const { root, executable, target } = await fixture(t);
  await writeFile(path.join(target, 'debug/data.txt'), 'not a compiler artifact');
  await assert.rejects(pinArtifact(root, path.join(target, 'debug/data.txt'), enough), /kind/);
  await assert.rejects(pinArtifact(root, path.join(target, 'debug'), enough), /nonregular/);
  await rm(path.join(target, 'CACHEDIR.TAG'));
  assert.ok((await pinArtifact(root, executable, enough)).path.endsWith('artifact.exe'));
});

test('low space never writes a new retained artifact and corrupt pins are not reused', async t => {
  const { root, executable } = await fixture(t);
  await assert.rejects(pinArtifact(root, executable, { observeFree: async () => 0n }), /headroom/);
  const pinned = await pinArtifact(root, executable, enough);
  await writeFile(path.join(root, pinned.path), 'tampered artifact');
  await assert.rejects(pinArtifact(root, executable, enough), /changed/);
});

test('concurrent pins publish complete bytes without duplicate or partial artifacts', async t => {
  const { root, executable } = await fixture(t);
  const pins = await Promise.all(Array.from({ length: 4 }, () => pinArtifact(root, executable, enough)));
  for (const pin of pins) assert.deepEqual(pin, pins[0]);
  assert.deepEqual(await readFile(path.join(root, pins[0].path)), await readFile(executable));
});
