import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, symlinkSync, readFileSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { compressed, inventory, main } from './distribution-size.mjs';

const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
function fixture(run) {
  const base = resolve(project, 'tmp');
  mkdirSync(base, { recursive: true });
  const folder = mkdtempSync(join(base, 'distribution-size-'));
  const root = join(folder, 'ship');
  mkdirSync(root);
  writeFileSync(join(root, 'core.js'), 'export const text = "終端";\r\n');
  writeFileSync(join(root, 'core.wasm'), Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]));
  writeFileSync(join(root, 'empty.css'), '');
  const manifest = { schemaVersion: 1, distributions: [{ id: 'rust-web', directory: '.', kind: 'web',
    scope: 'fixture only', profile: 'release', addons: [], build: { commands: ['cargo build --release'],
      toolchain: 'fixture rustc', flags: 'fixture flags', sourceSha256: 'a'.repeat(64) },
    files: [{ path: 'core.js', role: 'binding' }, { path: 'core.wasm', role: 'wasm' }, { path: 'empty.css', role: 'css' }] }] };
  try { run({ folder, root, manifest, distribution: manifest.distributions[0] }); }
  finally { rmSync(folder, { recursive: true, force: true }); }
}
test('counts each path once, includes empty files and reconciles all compression totals', () => fixture(({ root, manifest, distribution }) => {
  distribution.files.push({ ...distribution.files[0] });
  const result = inventory(manifest, root).distributions[0];
  assert.equal(result.files.length, 3);
  for (const key of ['raw', 'gzip', 'brotli']) assert.equal(result.totals[key], result.files.reduce((sum, entry) => sum + entry[key], 0));
  assert.equal(result.files.find(file => file.path === 'empty.css').raw, 0);
  assert.equal(result.totals.raw, Buffer.byteLength('export const text = "終端";\r\n') + 8);
}));
test('gzip and Brotli round trips admit WASM, Unicode, CRLF and empty bytes', () => {
  for (const bytes of [Buffer.alloc(0), Buffer.from([0, 97, 115, 109, 255]), Buffer.from('終端\r\n')]) {
    const result = compressed(bytes);
    assert.ok(result.gzip > 0);
    assert.ok(result.brotli > 0);
  }
});
test('recount is byte/hash stable and independent of manifest file order', () => fixture(({ root, manifest, distribution }) => {
  const first = inventory(manifest, root);
  distribution.files.reverse();
  assert.deepEqual(inventory(manifest, root), first);
}));
test('native files have raw sizes only, not download comparisons', () => fixture(({ root, manifest, distribution }) => {
  distribution.kind = 'native';
  const result = inventory(manifest, root).distributions[0];
  assert.deepEqual(Object.keys(result.totals), ['raw']);
  assert.ok(result.files.every(file => !('gzip' in file) && !('brotli' in file)));
}));
test('rejects missing files and undeclared mandatory supporting assets', () => fixture(({ root, manifest, distribution }) => {
  distribution.files.push({ path: 'absent.font', role: 'font' });
  assert.throws(() => inventory(manifest, root), /Missing distribution file/);
  distribution.files.pop();
  writeFileSync(join(root, 'required-worker.js'), 'postMessage(1)');
  assert.throws(() => inventory(manifest, root), /Undeclared distribution asset/);
}));
test('excluded nonruntime files need reasons and must exist', () => fixture(({ root, manifest, distribution }) => {
  writeFileSync(join(root, 'core.js.map'), '{}');
  distribution.excluded = [{ path: 'core.js.map', reason: 'Developer sourcemap, not runtime' }];
  assert.equal(inventory(manifest, root).distributions[0].excluded.length, 1);
  distribution.excluded[0].reason = '';
  assert.throws(() => inventory(manifest, root), /nonempty text/);
  distribution.excluded = [{ path: 'missing.map', reason: 'not present' }];
  assert.throws(() => inventory(manifest, root), /Undeclared distribution asset/);
}));
test('rejects traversal, absolute, Windows drive and nonnormalized paths', () => fixture(({ root, manifest, distribution }) => {
  for (const path of ['../core.js', '/core.js', 'C:/core.js', 'a\\core.js', './core.js', 'a//core.js', 'a/../core.js', '']) {
    distribution.files[0].path = path;
    assert.throws(() => inventory(manifest, root));
  }
}));
test('rejects junction roots and nested junction entries without following them', () => fixture(({ folder, root, manifest, distribution }) => {
  const link = join(folder, 'link');
  symlinkSync(root, link, process.platform === 'win32' ? 'junction' : 'dir');
  try { assert.throws(() => inventory(manifest, link), /Not a real directory/); }
  finally { rmSync(link); }
  const outside = join(folder, 'outside');
  mkdirSync(outside);
  writeFileSync(join(outside, 'secret.js'), 'not shipped');
  symlinkSync(outside, join(root, 'nested'), process.platform === 'win32' ? 'junction' : 'dir');
  try {
    assert.throws(() => inventory(manifest, root), /Linked entry/);
    distribution.directory = 'nested';
    assert.throws(() => inventory(manifest, root), /Linked entry/);
  } finally { rmSync(join(root, 'nested')); }
}));
test('rejects directory file entries and conflicting declarations', () => fixture(({ root, manifest, distribution }) => {
  mkdirSync(join(root, 'directory'));
  distribution.files.push({ path: 'directory', role: 'not-file' });
  assert.throws(() => inventory(manifest, root), /Missing distribution file/);
  distribution.files.pop();
  distribution.files.push({ path: 'core.js', role: 'different-role' });
  assert.throws(() => inventory(manifest, root), /Conflicting repeated file/);
  distribution.files.pop();
  distribution.excluded = [{ path: 'core.js', reason: 'conflict' }];
  assert.throws(() => inventory(manifest, root), /Conflicting excluded file/);
}));
test('rejects absent release provenance, addons and duplicate ids', () => fixture(({ root, manifest, distribution }) => {
  for (const [key, bad] of [['profile', 'debug'], ['addons', undefined], ['kind', 'unknown'], ['scope', ''], ['build', {}]]) {
    const previous = distribution[key];
    distribution[key] = bad;
    assert.throws(() => inventory(manifest, root));
    distribution[key] = previous;
  }
  manifest.distributions.push(structuredClone(distribution));
  assert.throws(() => inventory(manifest, root), /Repeated distribution id/);
}));
test('CLI writes a machine-readable receipt and rejects outputs inside inputs', () => fixture(({ folder, root, manifest }) => {
  const input = join(folder, 'manifest.json'), output = join(folder, 'report.json');
  writeFileSync(input, JSON.stringify(manifest));
  const args = ['--root', root, '--manifest', input, '--output', output];
  const report = main(args);
  assert.deepEqual(JSON.parse(readFileSync(output)), report);
  assert.match(report.manifestSha256, /^[a-f0-9]{64}$/);
  assert.throws(() => main(['--root', root, '--manifest', input, '--output', join(root, 'report.json')]), /outside/);
  assert.throws(() => main(['--root', root]), /Usage/);
  assert.throws(() => main([...args, '--root', root]), /Usage/);
}));
test('CLI exits nonzero for missing assets, malformed JSON and unknown flags', () => fixture(({ folder, root, manifest, distribution }) => {
  const input = join(folder, 'manifest.json'), output = join(folder, 'report.json');
  distribution.files.push({ path: 'missing.wasm', role: 'wasm' });
  writeFileSync(input, JSON.stringify(manifest));
  const cli = resolve(project, 'scripts/distribution-size.mjs');
  const invoke = args => spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8', timeout: 10000 });
  const args = ['--root', root, '--manifest', input, '--output', output];
  assert.equal(invoke(args).status, 1);
  writeFileSync(input, 'invalid JSON');
  assert.equal(invoke(args).status, 1);
  assert.equal(invoke(['--bad', 'value']).status, 1);
}));
