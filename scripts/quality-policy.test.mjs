import { test } from 'node:test';
import { mkdir, mkdtemp, writeFile, readFile, rm, symlink } from 'node:fs/promises';
import assert from 'node:assert/strict';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseOptions, outputPath, reserveOutput, assertNativeHost, assertBuildEnvironment, assertResolvedTarget, assertSharedTarget, testCounts } from './quality-policy.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
test('options preserve explicit WASM intent, destination and finite deadlines', () => {
  const defaults = parseOptions([], 'linux', 'x64');
  assert.match(defaults.destination, /^results\/quality\/linux-x64-\d+$/);
  assert.equal(defaults.timeoutMs, 300_000);
  assert.equal(defaults.minFreeMiB, 1024);
  assert.equal(defaults.wasm, false);
  assert.deepEqual(parseOptions(['--out', 'results/quality/custom', '--timeout-ms', '100', '--wasm-check'], 'win32', 'x64'), {
    destination: 'results/quality/custom', timeoutMs: 100, wasm: true, minFreeMiB: 1024
  });
  for (const value of ['0', '-1', 'NaN', 'Infinity', '0.5', '1800001']) {
    assert.throws(() => parseOptions(['--timeout-ms', value], 'linux', 'x64'));
  }
  for (const value of ['0', '255', 'NaN', '256.5', '1048577']) {
    assert.throws(() => parseOptions(['--min-free-mib', value], 'linux', 'x64'));
  }
  assert.equal(parseOptions(['--min-free-mib', '512'], 'linux', 'x64').minFreeMiB, 512);
  for (const args of [['--out'], ['--timeout-ms'], ['--unknown'], ['--min-free-mib']]) assert.throws(() => parseOptions(args, 'linux', 'x64'));
});
test('output destinations cannot overwrite the root or escape results', () => {
  assert.equal(outputPath(root, 'results/quality/custom'), join(root, 'results', 'quality', 'custom'));
  for (const path of ['.', '..', 'results', 'tmp/out', '../results/out', 'results/../../outside', join(root, '..', 'outside')]) {
    assert.throws(() => outputPath(root, path));
  }
});
test('attempt reservation preserves existing evidence and rejects output junctions', async () => {
  await mkdir(join(root, 'tmp'), { recursive: true });
  const directory = await mkdtemp(join(root, 'tmp', 'quality-output-'));
  try {
    const out = await reserveOutput(directory, 'results/quality/attempt');
    await writeFile(join(out, 'evidence.log'), 'retained failure');
    await assert.rejects(reserveOutput(directory, 'results/quality/attempt'), { code: 'EEXIST' });
    assert.equal(await readFile(join(out, 'evidence.log'), 'utf8'), 'retained failure');
    const external = join(directory, 'outside-results');
    await mkdir(external);
    await symlink(external, join(directory, 'results', 'redirect'), process.platform === 'win32' ? 'junction' : 'dir');
    await assert.rejects(reserveOutput(directory, 'results/redirect/attempt'), /real directory/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
test('native evidence rejects missing, foreign and ambient cross-compilation hosts', () => {
  for (const [platform, host] of [['win32', 'x86_64-pc-windows-msvc'], ['linux', 'x86_64-unknown-linux-gnu'], ['darwin', 'aarch64-apple-darwin']]) {
    assert.doesNotThrow(() => assertNativeHost(platform, host, undefined));
    assert.doesNotThrow(() => assertNativeHost(platform, host, host));
    assert.throws(() => assertNativeHost(platform, host, 'wasm32-unknown-unknown'));
  }
  for (const host of [undefined, '', 'x86_64-apple-darwin', 'wasm32-unknown-unknown']) {
    assert.throws(() => assertNativeHost('linux', host, undefined));
  }
});
test('inherited build policy cannot redirect disk or override shared profiles', () => {
  assert.doesNotThrow(() => assertBuildEnvironment({ PATH: 'toolchain' }));
  assert.doesNotThrow(() => assertBuildEnvironment({ CARGO_INCREMENTAL: '0' }));
  for (const key of ['CARGO_TARGET_DIR', 'CARGO_INCREMENTAL', 'CARGO_PROFILE_DEV_DEBUG', 'CARGO_PROFILE_TEST_INCREMENTAL', 'CARGO_PROFILE_RELEASE_LTO']) {
    assert.throws(() => assertBuildEnvironment({ [key]: 'value' }), /must be unset/);
  }
  assert.throws(() => assertBuildEnvironment({ CARGO_INCREMENTAL: '1' }), /must be unset/);
  assert.doesNotThrow(() => assertSharedTarget(root, join(root, 'target'), process.platform));
  assert.throws(() => assertSharedTarget(root, join(root, 'tmp', 'other-target'), process.platform));
});
test('resolved Cargo cfg catches configuration-file targets, not only environment overrides', () => {
  assert.equal(assertResolvedTarget('win32', 'target_os="windows"\n'), 'windows');
  assert.equal(assertResolvedTarget('linux', 'target_os="linux"\n'), 'linux');
  assert.equal(assertResolvedTarget('darwin', 'target_os="macos"\n'), 'macos');
  assert.throws(() => assertResolvedTarget('linux', 'target_os="windows"\n'));
  assert.throws(() => assertResolvedTarget('win32', 'target_os="unknown"\n'));
  assert.throws(() => assertResolvedTarget('darwin', ''));
});
test('test counts preserve failures, ignored and filtered cases per suite', () => {
  const output = 'test result: ok. 3 passed; 0 failed; 2 ignored; 0 measured; 4 filtered out\n' +
    'test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out';
  assert.deepEqual(testCounts(output), [
    { passed: 3, failed: 0, ignored: 2, measured: 0, filteredOut: 4 },
    { passed: 1, failed: 1, ignored: 0, measured: 0, filteredOut: 0 }
  ]);
  assert.deepEqual(testCounts('compilation failed, no suite executed'), []);
  assert.deepEqual(testCounts('  Running tests/lifecycle.rs (target/test)\n' +
    'test result: \u001b[32mok\u001b[0m. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'), [
    { suite: 'tests/lifecycle.rs (target/test)', passed: 9, failed: 0, ignored: 0, measured: 0, filteredOut: 0 }
  ]);
});
