import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, symlinkSync, unlinkSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runtimeFiles, stageFiles, nativeArtifacts, releaseProfile } from './measure-rust-size.mjs';

const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
function fixture(run) {
  mkdirSync(join(project, 'tmp'), { recursive: true });
  const root = mkdtempSync(join(project, 'tmp/rust-size-'));
  const put = (file, bytes = 'fixture') => { mkdirSync(dirname(join(root, file)), { recursive: true }); writeFileSync(join(root, file), bytes); };
  for (const file of ['browser/pkg/xterm_web.js', 'browser/pkg/xterm_web_bg.wasm', 'browser/pkg/xterm_web.d.ts',
    'browser/pkg/snippets/test/inline0.js', 'browser/xterm.mjs', 'browser/xterm.css', 'LICENSE', 'licenses/upstream.txt']) put(file);
  try { run({ root, put }); } finally { rmSync(root, { recursive: true }); }
}
function messages(root, patch = {}) {
  const profile = { test: false, opt_level: '3', debug_assertions: false, overflow_checks: false, ...patch };
  return [
    { reason: 'compiler-artifact', profile, target: { name: 'native', kind: ['example'] }, executable: join(root, 'target/release/examples/native.exe'), filenames: [] },
    { reason: 'compiler-artifact', profile, target: { name: 'xterm_core', kind: ['lib'] }, filenames: [join(root, 'target/release/libxterm_core.rlib')] }
  ];
}
test('runtime closure includes WASM, bootstrap, CSS, every generated snippet and shipped notices', () => fixture(({ root }) => {
  const selected = runtimeFiles(root);
  assert.equal(selected.files.length, 7);
  assert.ok(selected.files.some(row => row.path === 'browser/pkg/snippets/test/inline0.js'));
  assert.deepEqual(selected.excluded.map(row => row.path), ['browser/pkg/xterm_web.d.ts']);
  const staged = join(root, 'stage');
  stageFiles(root, staged, selected.files);
  assert.equal(existsSync(join(staged, 'browser/pkg/snippets/test/inline0.js')), true);
  assert.equal(existsSync(join(staged, 'browser/pkg/xterm_web.d.ts')), false);
}));
test('missing mandatory WASM and unreviewed generated assets fail closed', () => fixture(({ root, put }) => {
  rmSync(join(root, 'browser/pkg/xterm_web_bg.wasm'));
  assert.throws(() => runtimeFiles(root), /Missing mandatory/);
  put('browser/pkg/xterm_web_bg.wasm'); put('browser/pkg/new-worker.bin');
  assert.throws(() => runtimeFiles(root), /Unreviewed generated/);
}));
test('linked generated folders and source ancestors cannot be copied as artifacts', () => fixture(({ root, put }) => {
  put('outside/inline.js');
  const link = join(root, 'browser/pkg/linked');
  symlinkSync(join(root, 'outside'), link, process.platform === 'win32' ? 'junction' : 'dir');
  try {
    assert.throws(() => runtimeFiles(root), /Linked runtime/);
    assert.throws(() => stageFiles(root, join(root, 'stage'), [{ path: 'browser/pkg/linked/inline.js' }]), /Linked artifact ancestor/);
  } finally { unlinkSync(link); }
}));
test('staging rejects artifact traversal and preserves unrelated outside files', () => fixture(({ root }) => {
  assert.throws(() => stageFiles(root, join(root, 'stage'), [{ path: '../other/secret' }]), /Escaped artifact/);
}));
test('native selection uses actual release compiler outputs, not guessed filenames', () => fixture(({ root, put }) => {
  put('target/release/examples/native.exe'); put('target/release/libxterm_core.rlib');
  const output = messages(root).map(row => JSON.stringify(row)).join('\n');
  const artifacts = nativeArtifacts(root, output);
  assert.equal(artifacts.length, 2);
  assert.equal(artifacts[0].path, 'target/release/examples/native.exe');
  assert.equal(artifacts[1].path, 'target/release/libxterm_core.rlib');
  assert.equal(releaseProfile.CARGO_BUILD_JOBS, '2');
  assert.equal(releaseProfile.CARGO_PROFILE_RELEASE_OPT_LEVEL, '3');
}));
test('duplicate, absent, wrong-profile and escaped native artifacts fail', () => fixture(({ root, put }) => {
  put('target/release/examples/native.exe'); put('target/release/libxterm_core.rlib');
  const rows = messages(root), encode = rows => rows.map(row => JSON.stringify(row)).join('\n');
  assert.throws(() => nativeArtifacts(root, ''), /uniquely identify/);
  assert.throws(() => nativeArtifacts(root, encode([...rows, rows[0]])), /uniquely identify/);
  assert.throws(() => nativeArtifacts(root, encode(messages(root, { opt_level: '0' }))), /release profile/);
  rows[0].executable = join(root, 'outside.exe');
  assert.throws(() => nativeArtifacts(root, encode(rows)), /escaped private release/);
}));
test('linked private targets cannot masquerade as owned release artifacts', () => fixture(({ root, put }) => {
  put('outside/release/examples/native.exe'); put('outside/release/libxterm_core.rlib');
  const link = join(root, 'target');
  symlinkSync(join(root, 'outside'), link, process.platform === 'win32' ? 'junction' : 'dir');
  try { assert.throws(() => nativeArtifacts(root, messages(root).map(row => JSON.stringify(row)).join('\n')), /Linked artifact ancestor/); }
  finally { unlinkSync(link); }
}));
